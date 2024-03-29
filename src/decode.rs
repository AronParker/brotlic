//! Module that contains the brotli decoder instances
//!
//! Contains decompression abstractions over [`Read`] and [`Write`] and a
//! dedicated low-level encoder.
//!
//! [`Read`]: https://doc.rust-lang.org/stable/std/io/trait.Read.html
//! [`Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html

use std::error::Error;
use std::ffi::CStr;
use std::io::{BufRead, Read, Write};
use std::{fmt, io, ptr, slice};

use brotlic_sys::*;

use crate::{IntoInnerError, SetParameterError};

/// A reference to a brotli decoder.
///
/// This decoder contains internal state of the decoding process. This low-level
/// wrapper intended to be used for people who are familiar with the C API. For
/// higher level abstractions, see [`DecompressorReader`] and
/// [`DecompressorWriter`].
pub struct BrotliDecoder {
    state: *mut BrotliDecoderState,
}

unsafe impl Send for BrotliDecoder {}
unsafe impl Sync for BrotliDecoder {}

impl BrotliDecoder {
    /// Constructs a new brotli decoder instance.
    ///
    /// # Panics
    ///
    /// Panics if the decoder fails to be allocated or initialized
    #[doc(alias = "BrotliDecoderCreateInstance")]
    pub fn new() -> Self {
        let instance = unsafe { BrotliDecoderCreateInstance(None, None, ptr::null_mut()) };

        if !instance.is_null() {
            BrotliDecoder { state: instance }
        } else {
            panic!("BrotliDecoderCreateInstance returned NULL: failed to allocate or initialize");
        }
    }

    /// Checks if the decoder instance reached its final state.
    #[doc(alias = "BrotliDecoderIsFinished")]
    pub fn is_finished(&self) -> bool {
        unsafe { BrotliDecoderIsFinished(self.state) != 0 }
    }

    /// Decompresses the input stream to the output stream.
    ///
    /// This is a low-level API, for higher level abstractions see
    /// [`DecompressorReader`] or [`DecompressorWriter`]. Returns the number of
    /// bytes that were read, written and some additional information. Bytes are
    /// read from `input`, the number of bytes read is returned in the
    /// `bytes_read` field of the result. The `input` is never overconsumed, so
    /// it could be passed to the next consumer after decoding is complete.
    /// Bytes are written to `output`, the number of bytes written is returned
    /// in the `bytes_written` field of the result. The `info` field of the
    /// result communicates the state of the decoding process.
    ///
    /// if `info` is [`DecoderInfo::NeedsMoreInput`], more input is required to
    /// continue decoding. Likewise, if `info` is
    /// [`DecoderInfo::NeedsMoreOutput`], more output is required to continue
    /// the decoding conversion. [`DecoderInfo::Finished`] indicates that the
    /// decoding has finished.
    #[doc(alias = "BrotliDecoderDecompressStream")]
    pub fn decompress(
        &mut self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<DecodeResult, DecodeError> {
        let mut input_ptr = input.as_ptr();
        let mut input_len = input.len();
        let mut output_ptr = output.as_mut_ptr();
        let mut output_len = output.len();

        let result = unsafe {
            BrotliDecoderDecompressStream(
                self.state,
                &mut input_len,
                &mut input_ptr,
                &mut output_len,
                &mut output_ptr,
                ptr::null_mut(),
            )
        };

        let bytes_read = input.len() - input_len;
        let bytes_written = output.len() - output_len;

        #[allow(non_upper_case_globals)]
        let info = match result {
            BrotliDecoderResult_BROTLI_DECODER_RESULT_ERROR => return Err(self.last_error()),
            BrotliDecoderResult_BROTLI_DECODER_RESULT_SUCCESS => DecoderInfo::Finished,
            BrotliDecoderResult_BROTLI_DECODER_RESULT_NEEDS_MORE_INPUT => {
                DecoderInfo::NeedsMoreInput
            }
            BrotliDecoderResult_BROTLI_DECODER_RESULT_NEEDS_MORE_OUTPUT => {
                DecoderInfo::NeedsMoreOutput
            }
            _ => panic!("BrotliDecoderDecompressStream returned an unknown error code"),
        };

        Ok(DecodeResult {
            bytes_read,
            bytes_written,
            info,
        })
    }

    /// Convenience function to call method [`Self::decompress`] with only
    /// input.
    pub fn give_input(&mut self, input: &[u8]) -> Result<(usize, DecoderInfo), DecodeError> {
        let res = self.decompress(input, &mut [])?;

        Ok((res.bytes_read, res.info))
    }

    /// Checks if the decoder has more output.
    #[doc(alias = "BrotliDecoderHasMoreOutput")]
    pub fn has_output(&self) -> bool {
        unsafe { BrotliDecoderHasMoreOutput(self.state) != 0 }
    }

    /// Checks if the decoder has more output and if so, returns a slice to its
    /// internal output buffer. Each byte returned from the slice is considered
    /// "consumed" and must be used as it will not be returned again. Encoder
    /// output is not guaranteed to be contagious, which means that this
    /// function can return `Some(&[u8])` multiple times. Only when the method
    /// returns `None` is when there is no more output available by the decoder.
    ///
    /// # Safety
    ///
    /// For every consecutive call of this function, the previous slice becomes
    /// invalidated.
    #[doc(alias = "BrotliDecoderTakeOutput")]
    pub unsafe fn take_output(&mut self) -> Option<&[u8]> {
        if self.has_output() {
            let mut len: usize = 0;
            let output = BrotliDecoderTakeOutput(self.state, &mut len as _);

            Some(slice::from_raw_parts(output, len))
        } else {
            None
        }
    }

    /// Returns the version of the C brotli decoder library.
    #[doc(alias = "BrotliDecoderVersion")]
    pub fn version() -> u32 {
        unsafe { BrotliDecoderVersion() }
    }

    fn set_param(
        &mut self,
        param: BrotliDecoderParameter,
        value: u32,
    ) -> Result<(), SetParameterError> {
        let r = unsafe { BrotliDecoderSetParameter(self.state, param, value) };

        if r != 0 {
            Ok(())
        } else {
            Err(SetParameterError::Generic)
        }
    }

    fn last_error(&self) -> DecodeError {
        let ec = unsafe { BrotliDecoderGetErrorCode(self.state) };

        #[allow(non_upper_case_globals)]
        match ec {
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_NIBBLE => {
                DecodeError::FormatExuberantNibble
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_RESERVED => {
                DecodeError::FormatReserved
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_META_NIBBLE => {
                DecodeError::FormatExuberantMetaNibble
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_ALPHABET => {
                DecodeError::FormatSimpleHuffmanAlphabet
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_SAME => {
                DecodeError::FormatSimpleHuffmanSame
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_CL_SPACE => {
                DecodeError::FormatClSpace
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_HUFFMAN_SPACE => {
                DecodeError::FormatHuffmanSpace
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_CONTEXT_MAP_REPEAT => {
                DecodeError::FormatContextMapRepeat
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_1 => {
                DecodeError::FormatBlockLength1
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_2 => {
                DecodeError::FormatBlockLength2
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_TRANSFORM => {
                DecodeError::FormatTransform
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_DICTIONARY => {
                DecodeError::FormatDictionary
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_WINDOW_BITS => {
                DecodeError::FormatWindowBits
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_PADDING_1 => {
                DecodeError::FormatPadding1
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_PADDING_2 => {
                DecodeError::FormatPadding2
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_DISTANCE => {
                DecodeError::FormatDistance
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_COMPOUND_DICTIONARY => {
                DecodeError::CompoundDictionary
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_DICTIONARY_NOT_SET => {
                DecodeError::DictionaryNotSet
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_INVALID_ARGUMENTS => {
                DecodeError::InvalidArguments
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MODES => {
                DecodeError::AllocContextModes
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_TREE_GROUPS => {
                DecodeError::AllocTreeGroups
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MAP => {
                DecodeError::AllocContextMap
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_1 => {
                DecodeError::AllocRingBuffer1
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_2 => {
                DecodeError::AllocRingBuffer2
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_BLOCK_TYPE_TREES => {
                DecodeError::AllocBlockTypeTrees
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_UNREACHABLE => DecodeError::Unreachable,
            _ => DecodeError::UnknownError,
        }
    }
}

impl fmt::Debug for BrotliDecoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrotliDecoder")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl Default for BrotliDecoder {
    fn default() -> Self {
        BrotliDecoder::new()
    }
}

impl Drop for BrotliDecoder {
    fn drop(&mut self) {
        unsafe {
            BrotliDecoderDestroyInstance(self.state);
        }
    }
}

/// Decompression options to be used for a [`BrotliDecoder`].
///
/// # Examples
///
/// Building an decoder that supports large window sizes:
/// ```
/// use brotlic::BrotliDecoderOptions;
///
/// let encoder = BrotliDecoderOptions::new().large_window_size(true).build();
/// ```
#[derive(Debug, Clone)]
pub struct BrotliDecoderOptions {
    disable_ring_buffer_reallocation: Option<bool>,
    large_window_size: Option<bool>,
}

impl BrotliDecoderOptions {
    /// Creates a new blank set decoder options.
    ///
    /// initially no modifications are applied to the decoder and everything is
    /// set to its default values.
    pub fn new() -> Self {
        BrotliDecoderOptions {
            disable_ring_buffer_reallocation: None,
            large_window_size: None,
        }
    }

    /// Disable "canny" ring buffer allocation strategy.
    ///
    /// Ring buffer is allocated according to window size, despite the real size
    /// of the content.
    pub fn disable_ring_buffer_reallocation(
        &mut self,
        disable_ring_buffer_reallocation: bool,
    ) -> &mut Self {
        self.disable_ring_buffer_reallocation = Some(disable_ring_buffer_reallocation);
        self
    }

    /// Flag that determines if this decoder supports non standard large window
    /// sizes. By default, this is turned off and window sizes are limited by
    /// RFC7932 (Brotli proper). To support large window sizes outside of the
    /// specification, this flag must be enabled. For more information see
    /// [`LargeWindowSize`].
    ///
    /// [`LargeWindowSize`]: crate::LargeWindowSize
    pub fn large_window_size(&mut self, large_window_size: bool) -> &mut Self {
        self.large_window_size = Some(large_window_size);
        self
    }

    /// Creates a brotli decoder using the specified settings.
    ///
    /// # Errors
    ///
    /// If any of the preconditions of the parameters are violated, an error is
    /// returned.
    #[doc(alias = "BrotliDecoderSetParameter")]
    pub fn build(&self) -> Result<BrotliDecoder, SetParameterError> {
        let mut decoder = BrotliDecoder::new();

        self.configure(&mut decoder)?;

        Ok(decoder)
    }

    fn configure(&self, decoder: &mut BrotliDecoder) -> Result<(), SetParameterError> {
        if let Some(disable_ring_buffer_reallocation) = self.disable_ring_buffer_reallocation {
            let key = BrotliDecoderParameter_BROTLI_DECODER_PARAM_DISABLE_RING_BUFFER_REALLOCATION;
            let value = disable_ring_buffer_reallocation as u32;

            decoder.set_param(key, value)?;
        }

        if let Some(large_window_size) = self.large_window_size {
            let key = BrotliDecoderParameter_BROTLI_DECODER_PARAM_LARGE_WINDOW;
            let value = large_window_size as u32;

            decoder.set_param(key, value)?;
        }

        Ok(())
    }
}

impl Default for BrotliDecoderOptions {
    fn default() -> Self {
        BrotliDecoderOptions::new()
    }
}

/// A struct used by [`BrotliDecoder::decompress`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DecodeResult {
    /// The number of bytes read from `input`.
    pub bytes_read: usize,
    /// The number of bytes written to `output`.
    pub bytes_written: usize,
    /// Information the decoder gave on whether its finished or needs more input
    /// or output.
    pub info: DecoderInfo,
}

/// Additional information provided by the decoder on how decompression should
/// proceed.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DecoderInfo {
    /// The decoder has finished decompressing all input data.
    Finished,
    /// The decoder needs more input to proceed decompression.
    NeedsMoreInput,
    /// The decoder needs more output to proceed decompression.
    NeedsMoreOutput,
}

/// An error returned by [`BrotliDecoder::decompress`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum DecodeError {
    UnknownError = 0,
    FormatExuberantNibble =
        BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_NIBBLE as isize,
    FormatReserved = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_RESERVED as isize,
    FormatExuberantMetaNibble =
        BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_META_NIBBLE as isize,
    FormatSimpleHuffmanAlphabet =
        BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_ALPHABET as isize,
    FormatSimpleHuffmanSame =
        BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_SAME as isize,
    FormatClSpace = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_CL_SPACE as isize,
    FormatHuffmanSpace = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_HUFFMAN_SPACE as isize,
    FormatContextMapRepeat =
        BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_CONTEXT_MAP_REPEAT as isize,
    FormatBlockLength1 = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_1 as isize,
    FormatBlockLength2 = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_2 as isize,
    FormatTransform = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_TRANSFORM as isize,
    FormatDictionary = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_DICTIONARY as isize,
    FormatWindowBits = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_WINDOW_BITS as isize,
    FormatPadding1 = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_PADDING_1 as isize,
    FormatPadding2 = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_PADDING_2 as isize,
    FormatDistance = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_DISTANCE as isize,
    CompoundDictionary = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_COMPOUND_DICTIONARY as isize,
    DictionaryNotSet = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_DICTIONARY_NOT_SET as isize,
    InvalidArguments = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_INVALID_ARGUMENTS as isize,
    AllocContextModes = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MODES as isize,
    AllocTreeGroups = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_TREE_GROUPS as isize,
    AllocContextMap = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MAP as isize,
    AllocRingBuffer1 = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_1 as isize,
    AllocRingBuffer2 = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_2 as isize,
    AllocBlockTypeTrees =
        BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_BLOCK_TYPE_TREES as isize,
    Unreachable = BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_UNREACHABLE as isize,
}

impl Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == DecodeError::UnknownError {
            write!(f, "decode error: unknown error")
        } else {
            let str = unsafe {
                let error_code = *self as BrotliDecoderErrorCode;
                let error_string = BrotliDecoderErrorString(error_code);
                let c_str = CStr::from_ptr(error_string);
                c_str
                    .to_str()
                    .expect("invalid utf-8 returned from BrotliDecoderErrorString")
            };

            write!(f, "brotli decoder error: {}", str)
        }
    }
}

impl From<DecodeError> for io::Error {
    fn from(err: DecodeError) -> Self {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

/// Wraps a reader and decompresses its output.
///
/// # Examples
///
/// Suppose the file `test.brotli` contains brotli compressed data. Let's try to
/// decompress it:
///
/// ```no_run
/// use std::fs::File;
/// use std::io::Read;
///
/// use brotlic::DecompressorWriter;
///
/// let mut input = File::open("test.brotli")?; // test.brotli is brotli compressed
/// let mut output = String::new();
///
/// input.read_to_string(&mut output)?;
///
/// println!("Decompressed text: {}", output);
///
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Debug)]
pub struct DecompressorReader<R: BufRead> {
    inner: R,
    decoder: BrotliDecoder,
}

impl<R: BufRead> DecompressorReader<R> {
    /// Creates a new `DecompressorReader<R>` with a newly created decoder.
    ///
    /// # Panics
    ///
    /// Panics if the decoder fails to be allocated or initialized
    pub fn new(inner: R) -> Self {
        DecompressorReader {
            inner,
            decoder: BrotliDecoder::new(),
        }
    }

    /// Creates a new `DecompressorReader<R>` with a specified decoder.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Read;
    ///
    /// use brotlic::{BrotliDecoderOptions, DecompressorReader};
    ///
    /// let decoder = BrotliDecoderOptions::new()
    ///     .disable_ring_buffer_reallocation(true)
    ///     .build()?;
    ///
    /// let source = [11, 2, 128, 104, 101, 108, 108, 111, 3]; // decompresses to "hello"
    /// let mut decompressor = DecompressorReader::with_decoder(decoder, source.as_slice());
    /// # Ok::<(), brotlic::SetParameterError>(())
    /// ```
    pub fn with_decoder(decoder: BrotliDecoder, inner: R) -> Self {
        DecompressorReader { inner, decoder }
    }

    /// Gets a reference to the underlying reader
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Unwraps this `DecompressorReader<R>`, returning the underlying reader.
    ///
    /// # Errors
    ///
    /// An [`Err`] will be returned if the decompression stream has not been
    /// finished.
    pub fn into_inner(self) -> Result<R, IntoInnerError<DecompressorReader<R>>> {
        if self.decoder.is_finished() {
            Ok(self.inner)
        } else {
            Err(IntoInnerError::new(
                self,
                io::ErrorKind::UnexpectedEof.into(),
            ))
        }
    }

    /// Disassembles this `DecompressorReader<R>`, returning the underlying
    /// reader and decoder.
    ///
    /// `into_parts` makes no attempt to validate that the decompression stream
    /// finished and cannot fail.
    pub fn into_parts(self) -> (R, BrotliDecoder) {
        (self.inner, self.decoder)
    }
}

impl<R: BufRead> Read for DecompressorReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let input = self.inner.fill_buf()?;
            let eof = input.is_empty();
            let DecodeResult {
                bytes_read,
                bytes_written,
                info,
            } = self.decoder.decompress(input, buf)?;
            self.inner.consume(bytes_read);

            match info {
                _ if bytes_written > 0 => return Ok(bytes_written),
                DecoderInfo::Finished => return Ok(0),
                DecoderInfo::NeedsMoreInput if eof => {
                    return Err(io::ErrorKind::UnexpectedEof.into());
                }
                DecoderInfo::NeedsMoreInput => continue,
                DecoderInfo::NeedsMoreOutput if buf.is_empty() => return Ok(0),
                DecoderInfo::NeedsMoreOutput => panic!(
                    "decoder needs output despite not giving any while having the chance to do so"
                ),
            };
        }
    }
}

/// Wraps a writer and decompresses its output.
///
/// `DecompressorWriter<R>` wraps a writer and adds brotli decompression to the
/// output.
///
/// # Examples
///
/// Let's decompress the `test.brotli` file shown in [`CompressorWriter`]:
///
/// ```no_run
/// use std::fs::File;
/// use std::io;
///
/// use brotlic::DecompressorWriter;
///
/// let mut input = File::open("test.brotli")?; // test.brotli is brotli compressed
/// let mut output = File::create("test_reconstructed.txt")?;
/// let mut decompressed_output = DecompressorWriter::new(output);
///
/// io::copy(&mut input, &mut decompressed_output)?;
///
/// # Ok::<(), io::Error>(())
/// ```
///
/// [`CompressorWriter`]: crate::encode::CompressorWriter
#[derive(Debug)]
pub struct DecompressorWriter<W: Write> {
    inner: W,
    decoder: BrotliDecoder,
    panicked: bool,
}

impl<W: Write> DecompressorWriter<W> {
    /// Creates a new `DecompressorWriter<W>` with a newly created decoder.
    ///
    /// # Panics
    ///
    /// Panics if the decoder fails to be allocated or initialized
    pub fn new(inner: W) -> DecompressorWriter<W> {
        DecompressorWriter {
            inner,
            decoder: BrotliDecoder::new(),
            panicked: false,
        }
    }

    /// Creates a new `DecompressorWriter<W>` with a specified decoder.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Write;
    ///
    /// use brotlic::{BrotliDecoderOptions, DecompressorReader, DecompressorWriter};
    ///
    /// let decoder = BrotliDecoderOptions::new()
    ///     .large_window_size(true)
    ///     .build()?;
    ///
    /// let mut writer = DecompressorWriter::with_decoder(decoder, Vec::new());
    /// # Ok::<(), brotlic::SetParameterError>(())
    /// ```
    pub fn with_decoder(decoder: BrotliDecoder, inner: W) -> Self {
        DecompressorWriter {
            inner,
            decoder,
            panicked: false,
        }
    }

    /// Gets a reference to the underlying writer
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// It is inadvisable to directly write to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Unwraps this `DecompressorWriter<W>`, returning the underlying writer.
    ///
    /// If the decompression stream is validated before finishing and will
    /// return an [`Err`] otherwise. The `DecompressorWriter<W>` will not
    /// overcome its input, if an adjacent second compression stream follows it
    /// can be read by another `DecompressorWriter<W>` without length-prefixing.
    ///
    /// # Errors
    ///
    /// An [`Err`] will be returned if the decompression stream has not been
    /// finished.
    pub fn into_inner(self) -> Result<W, IntoInnerError<DecompressorWriter<W>>> {
        if self.decoder.is_finished() {
            Ok(self.into_parts().0)
        } else {
            Err(IntoInnerError::new(
                self,
                io::ErrorKind::UnexpectedEof.into(),
            ))
        }
    }

    /// Disassembles this `DecompressorWriter<W>`, returning the underlying
    /// writer and decoder.
    ///
    /// If the underlying writer panicked, it is not known what portion of the
    /// data was written. In this case, we return `WriterPanicked` to get the
    /// encoder back.
    ///
    /// `into_parts` makes no attempt to validate that the decompression stream
    /// finished and cannot fail.
    pub fn into_parts(self) -> (W, Result<BrotliDecoder, WriterPanicked>) {
        let inner = self.inner;
        let decoder = self.decoder;

        let decoder = if !self.panicked {
            Ok(decoder)
        } else {
            Err(WriterPanicked { decoder })
        };

        (inner, decoder)
    }

    fn flush_decoder_output(&mut self) -> io::Result<()> {
        while let Some(output) = unsafe { self.decoder.take_output() } {
            self.panicked = true;
            let r = self.inner.write_all(output);
            self.panicked = false;
            r?;
        }

        Ok(())
    }
}

impl<W: Write> Write for DecompressorWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let (bytes_read, _decoder_result) = self.decoder.give_input(buf)?;
        self.flush_decoder_output()?;

        Ok(bytes_read)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Error returned from [`DecompressorWriter::into_inner`], when the underlying
/// writer has previously panicked. Contains the decoder that was used for
/// decompression.
#[derive(Debug)]
pub struct WriterPanicked {
    decoder: BrotliDecoder,
}

impl WriterPanicked {
    /// Returns the decoder that was used for decompression. It is unknown what
    /// data was fed to the decoder, so simply using it to finish it is not a
    /// good idea.
    pub fn into_inner(self) -> BrotliDecoder {
        self.decoder
    }
}

impl Error for WriterPanicked {}

impl fmt::Display for WriterPanicked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "DecompressorWriter inner writer panicked, what data remains unwritten is not known",
        )
    }
}
