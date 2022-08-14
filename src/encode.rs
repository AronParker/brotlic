//! Module that contains the brotli encoder instances
//!
//! Contains compression abstractions over [`Read`] and [`Write`] and a
//! dedicated low-level encoder.
//!
//! [`Read`]: https://doc.rust-lang.org/stable/std/io/trait.Read.html
//! [`Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html

use std::alloc::GlobalAlloc;
use std::error::Error;
use std::io::{BufRead, Read, Write};
use std::{fmt, io, mem, ptr, slice};

use brotlic_sys::*;

use crate::{
    BlockSize, CompressionMode, IntoInnerError, LargeWindowSize, Quality, SetParameterError,
    WindowSize,
};

/// A reference to a brotli encoder.
///
/// This encoder contains internal state of the encoding process. This low-level
/// wrapper intended to be used for people who are familiar with the C API. For
/// higher level abstractions, see [`CompressorReader`] and
/// [`CompressorWriter`].
pub struct BrotliEncoder {
    state: *mut BrotliEncoderState,

    // this field is read read across FFI boundaries
    #[allow(dead_code)]
    alloc: Option<Box<Box<dyn GlobalAlloc>>>,
}

unsafe impl Send for BrotliEncoder {}
unsafe impl Sync for BrotliEncoder {}

impl BrotliEncoder {
    /// Constructs a new brotli encoder instance.
    ///
    /// # Panics
    ///
    /// Panics if the encoder fails to be allocated or initialized
    #[doc(alias = "BrotliEncoderCreateInstance")]
    pub fn new() -> Self {
        let instance = unsafe { BrotliEncoderCreateInstance(None, None, ptr::null_mut()) };

        if !instance.is_null() {
            BrotliEncoder {
                state: instance,
                alloc: None,
            }
        } else {
            panic!("BrotliEncoderCreateInstance returned NULL: failed to allocate or initialize");
        }
    }

    /// Constructs a new brotli encoder instance using allocator `alloc`.
    ///
    /// # Panics
    ///
    /// Panics if the encoder fails to be allocated or initialized
    #[doc(alias = "BrotliEncoderCreateInstance")]
    pub fn new_in<A>(alloc: A) -> Self
    where
        A: GlobalAlloc + 'static,
    {
        let alloc: Box<Box<dyn GlobalAlloc>> = Box::new(Box::new(alloc));
        let alloc_ptr: *const Box<dyn GlobalAlloc> = alloc.as_ref();
        let instance = unsafe {
            BrotliEncoderCreateInstance(Some(crate::malloc), Some(crate::free), alloc_ptr as _)
        };

        if !instance.is_null() {
            BrotliEncoder {
                state: instance,
                alloc: Some(alloc),
            }
        } else {
            panic!("BrotliEncoderCreateInstance returned NULL: failed to allocate or initialize");
        }
    }

    /// Checks if the encoder instance reached its final state.
    #[doc(alias = "BrotliEncoderIsFinished")]
    pub fn is_finished(&self) -> bool {
        unsafe { BrotliEncoderIsFinished(self.state) != 0 }
    }

    /// Compresses input stream to output stream.
    ///
    /// This is a low-level API, for higher level abstractions see
    /// [`CompressorReader`] or [`CompressorWriter`]. Returns the number of
    /// bytes that were read and written. Bytes are read from `input`, the
    /// number of bytes read is returned in the `bytes_read` field of the
    /// result. Bytes are written to `output`, the number of bytes written is
    /// returned in the `bytes_written` field of the result. The operation `op`
    /// specifies the intention behind this call, whether it is to simply
    /// process input, flush the input or finish the input. Care must be taken
    /// to not swap, reduce or extend the input stream while flushing or
    /// finishing. Additionally the operation should not change until all the
    /// input has been processed and all the output has been read from the
    /// internal buffer.
    ///
    /// The internal workflow consists of three steps:
    ///
    /// 1. read from input into the internal buffer
    /// 2. compress input
    /// 3. write into output from the internal buffer
    ///
    /// Whenever any of these tasks can't move forward, control flow is returned
    /// to the caller. This is a wrapper around the
    /// `BrotliEncoderCompressStream` function of the C brotli API. For more
    /// information consult its documentation.
    #[doc(alias = "BrotliEncoderCompressStream")]
    pub fn compress(
        &mut self,
        input: &[u8],
        output: &mut [u8],
        op: BrotliOperation,
    ) -> Result<EncodeResult, EncodeError> {
        let mut input_ptr = input.as_ptr();
        let mut input_len = input.len();
        let mut output_ptr = output.as_mut_ptr();
        let mut output_len = output.len();

        let result = unsafe {
            BrotliEncoderCompressStream(
                self.state,
                op as BrotliEncoderOperation,
                &mut input_len,
                &mut input_ptr,
                &mut output_len,
                &mut output_ptr,
                ptr::null_mut(),
            )
        };

        if result != 0 {
            let bytes_read = input.len() - input_len;
            let bytes_written = output.len() - output_len;

            Ok(EncodeResult {
                bytes_read,
                bytes_written,
            })
        } else {
            Err(EncodeError)
        }
    }

    /// Convenience function to call method [`Self::compress`] with only input
    /// and no output.
    pub fn give_input(&mut self, input: &[u8], op: BrotliOperation) -> Result<usize, EncodeError> {
        Ok(self.compress(input, &mut [], op)?.bytes_read)
    }

    /// Attempts the flush the encoding stream.
    ///
    /// Actual flush is performed when all output has been successfully read.
    /// Use [`Self::has_output`] to verify that flushing completedNo other
    /// modifying operation should be queried before flushing has been
    /// finalized. When flush is complete, output data will be sufficient for a
    /// decoder to reproduce all given input. Calling this function might
    /// resulting in a worse compression ratio, because the encoder is forced to
    /// emit all output immediately.
    pub fn flush(&mut self) -> Result<(), EncodeError> {
        self.give_op(BrotliOperation::Flush)
    }

    /// Finalizes the encoding stream.
    ///
    /// Actual finalization is performed when all output from the encoder has
    /// been successfully read. Use [`Self::is_finished`] to verify that the
    /// encoder is finished. Once this method has been called, no further input
    /// should be processed.
    ///
    /// For more information, see
    /// `BrotliEncoderOperation::BROTLI_OPERATION_FINISH`
    pub fn finish(&mut self) -> Result<(), EncodeError> {
        self.give_op(BrotliOperation::Finish)
    }

    /// Checks if the encoder has more output.
    #[doc(alias = "BrotliEncoderHasMoreOutput")]
    pub fn has_output(&self) -> bool {
        unsafe { BrotliEncoderHasMoreOutput(self.state) != 0 }
    }

    /// Checks if the encoder has more output and if so, returns a slice to its
    /// internal output buffer.
    ///
    /// Each byte returned from the slice is considered "consumed" and must be
    /// used as it will not be returned again. Encoder output is not guaranteed
    /// to be contagious, which means that this function can return
    /// `Some(&[u8])` multiple times. Only when the method returns `None` is
    /// when there is no more output available by the encoder.
    ///
    /// # Safety
    ///
    /// For every consecutive call of this function, the previous slice becomes
    /// invalidated.
    #[doc(alias = "BrotliEncoderTakeOutput")]
    pub unsafe fn take_output(&mut self) -> Option<&[u8]> {
        if self.has_output() {
            let mut len: usize = 0;
            let output = BrotliEncoderTakeOutput(self.state, &mut len as _);

            Some(slice::from_raw_parts(output, len))
        } else {
            None
        }
    }

    /// Returns the version of the C brotli encoder library.
    #[doc(alias = "BrotliEncoderVersion")]
    pub fn version() -> u32 {
        unsafe { BrotliEncoderVersion() }
    }

    fn set_param(
        &mut self,
        param: BrotliEncoderParameter,
        value: u32,
    ) -> Result<(), SetParameterError> {
        let r = unsafe { BrotliEncoderSetParameter(self.state, param, value) };

        if r != 0 {
            Ok(())
        } else {
            Err(SetParameterError::Generic)
        }
    }

    fn give_op(&mut self, op: BrotliOperation) -> Result<(), EncodeError> {
        self.give_input(&[], op)?;
        Ok(())
    }
}

impl fmt::Debug for BrotliEncoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrotliEncoder")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl Default for BrotliEncoder {
    fn default() -> Self {
        BrotliEncoder::new()
    }
}

impl Drop for BrotliEncoder {
    #[doc(alias = "BrotliEncoderDestroyInstance")]
    fn drop(&mut self) {
        unsafe {
            BrotliEncoderDestroyInstance(self.state);
        }
    }
}

/// The operation for the encoder to process.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BrotliOperation {
    /// Instructs the encoder to keep processing input data.
    Process = BrotliEncoderOperation_BROTLI_OPERATION_PROCESS as isize,

    /// Instructs the encoder to commit a flushing operation. Care must be taken
    /// once a flush is initiated, to keep submitting flush operations till the
    /// encoder has no more output available. Additionally, the input stream
    /// should not be swapped, reduced or extended.
    Flush = BrotliEncoderOperation_BROTLI_OPERATION_FLUSH as isize,

    /// Instructs the encoder to commit a finish operation. Care must be taken
    /// once a finishing operation is initiated, to keep submitting flush
    /// operations till the encoder has no more output available. Additionally,
    /// the input stream should not be swapped, reduced or extended.
    Finish = BrotliEncoderOperation_BROTLI_OPERATION_FINISH as isize,
}

/// Compression options to be used for a [`BrotliEncoder`].
///
/// # Examples
///
/// Building an encoder using text mode and use a custom quality:
/// ```
/// use brotlic::{BrotliEncoderOptions, CompressionMode, Quality};
///
/// let encoder = BrotliEncoderOptions::new()
///     .mode(CompressionMode::Text)
///     .quality(Quality::new(5)?)
///     .build()?;
///
/// # Ok::<(), brotlic::SetParameterError>(())
/// ```
#[derive(Debug, Clone)]
pub struct BrotliEncoderOptions {
    mode: Option<CompressionMode>,
    quality: Option<Quality>,
    window_size: Option<LargeWindowSize>,
    block_bits: Option<BlockSize>,
    disable_context_modeling: Option<bool>,
    size_hint: Option<u32>,
    postfix_bits: Option<u32>,
    direct_distance_codes: Option<u32>,
    stream_offset: Option<u32>,
}

impl BrotliEncoderOptions {
    /// Creates a new blank set encoder options.
    ///
    /// initially no modifications are applied to the encoder and everything is
    /// set to its default values.
    pub fn new() -> Self {
        BrotliEncoderOptions {
            mode: None,
            quality: None,
            window_size: None,
            block_bits: None,
            disable_context_modeling: None,
            size_hint: None,
            postfix_bits: None,
            direct_distance_codes: None,
            stream_offset: None,
        }
    }

    /// Allows to tune a brotli compressor for a specific type of input.
    pub fn mode(&mut self, mode: CompressionMode) -> &mut Self {
        self.mode = Some(mode);
        self
    }

    /// The main compression speed-desnity lever. Higher quality means better
    /// compression ratios at the expense of slower compression times. For more
    /// information see [`Quality`]
    ///
    /// [`Quality`]: crate::Quality
    pub fn quality(&mut self, quality: Quality) -> &mut Self {
        self.quality = Some(quality);
        self
    }

    /// Recommended sliding LZ77 window size according to RFC7932 (Brotli
    /// proper). For more information see [`WindowSize`].
    ///
    /// [`WindowSize`]: crate::WindowSize
    pub fn window_size(&mut self, window_size: WindowSize) -> &mut Self {
        self.window_size = Some(window_size.into());
        self
    }

    /// The non-standard large window size to use. For more information see
    /// [`LargeWindowSize`].
    ///
    /// Warning: The decompressor needs explicit support in order to use this
    /// feature. This is not supported by the convenience [`decompress`]
    /// function. A matching [`BrotliDecoder`] must set [`large_window_size`] to
    /// true to decode non standard window sizes properly.
    ///
    /// [`LargeWindowSize`]: crate::LargeWindowSize
    /// [`decompress`]: crate::decompress
    /// [`BrotliDecoder`]: crate::decode::BrotliDecoder
    /// [`large_window_size`]: crate::decode::BrotliDecoderOptions::large_window_size
    pub fn large_window_size(&mut self, large_window_size: LargeWindowSize) -> &mut Self {
        self.window_size = Some(large_window_size);
        self
    }

    /// The recommended input block size to use.
    ///
    /// The encoder may reduce this value, e.g. when the input is much smaller
    /// than the input block size.
    pub fn block_size(&mut self, block_size: BlockSize) -> &mut Self {
        self.block_bits = Some(block_size);
        self
    }

    /// Disable "literal context modeling" format feature.
    ///
    /// Disabling literal context modeling decreases compression ratio in favor
    /// of decompression speed.
    pub fn disable_context_modeling(&mut self, disable_context_modeling: bool) -> &mut Self {
        self.disable_context_modeling = Some(disable_context_modeling);
        self
    }

    /// Estimated total input size.
    ///
    /// This is 0 by default, which corresponds to the size being unknown.
    pub fn size_hint(&mut self, size_hint: u32) -> &mut Self {
        self.size_hint = Some(size_hint);
        self
    }

    /// The number of postfix bits to use
    ///
    /// The encoder may change this value on the fly.
    ///
    /// Valid ranges are from `0` to `3` (`BROTLI_MAX_NPOSTFIX`) inclusive.
    pub fn postfix_bits(&mut self, postfix_bits: u32) -> &mut Self {
        self.postfix_bits = Some(postfix_bits);
        self
    }

    /// Recommended number of direct distance codes.
    ///
    /// The encoder may change this value on the fly.
    ///
    /// Valid range is from 0 to (15 << postfix) inclusive in steps of (1 <<
    /// postfix), where postfix is the number of postfix bits.
    pub fn direct_distance_codes(&mut self, direct_distance_codes: u32) -> &mut Self {
        self.direct_distance_codes = Some(direct_distance_codes);
        self
    }

    /// Number of bytes already processed by a different instance.
    ///
    /// It is worth noting that when using this parameter, all other encoders
    /// must share the same parameters, so that all encoded parts obey the same
    /// restrictions as implied by the header of the compression stream.
    ///
    /// If the offset is non-zero, the stream header is omitted. Values greater
    /// than 2**30 are not allowed.
    pub fn stream_offset(&mut self, stream_offset: u32) -> &mut Self {
        self.stream_offset = Some(stream_offset);
        self
    }

    /// Creates a brotli encoder with the specified settings using allocator
    /// `alloc`.
    ///
    /// # Errors
    ///
    /// If any of the preconditions of the parameters are violated, an error is
    /// returned.
    #[doc(alias = "BrotliEncoderSetParameter")]
    pub fn build(&self) -> Result<BrotliEncoder, SetParameterError> {
        let mut encoder = BrotliEncoder::new();

        self.configure(&mut encoder)?;

        Ok(encoder)
    }

    /// Creates a brotli encoder using the specified settings.
    ///
    /// # Errors
    ///
    /// If any of the preconditions of the parameters are violated, an error is
    /// returned.
    #[doc(alias = "BrotliEncoderSetParameter")]
    pub fn build_in<A>(&self, alloc: A) -> Result<BrotliEncoder, SetParameterError>
    where
        A: GlobalAlloc + 'static,
    {
        let mut encoder = BrotliEncoder::new_in(alloc);

        self.configure(&mut encoder)?;

        Ok(encoder)
    }

    fn configure(&self, encoder: &mut BrotliEncoder) -> Result<(), SetParameterError> {
        if let Some(mode) = self.mode {
            let key = BrotliEncoderParameter_BROTLI_PARAM_MODE;
            let value = mode as u32;

            encoder.set_param(key, value)?;
        }

        if let Some(quality) = self.quality {
            let key = BrotliEncoderParameter_BROTLI_PARAM_QUALITY;
            let value = quality.0 as u32;

            encoder.set_param(key, value)?;
        }

        if let Some(window_size) = self.window_size {
            let key = BrotliEncoderParameter_BROTLI_PARAM_LGWIN;
            let value = window_size.0 as u32;

            encoder.set_param(key, value)?;

            let large_window = WindowSize::try_from(window_size).is_err();

            let key = BrotliEncoderParameter_BROTLI_PARAM_LARGE_WINDOW;
            let value = large_window as u32;

            encoder.set_param(key, value)?;
        }

        if let Some(block_bits) = self.block_bits {
            let key = BrotliEncoderParameter_BROTLI_PARAM_LGBLOCK;
            let value = block_bits.0 as u32;

            encoder.set_param(key, value)?;
        }

        if let Some(disable_context_modeling) = self.disable_context_modeling {
            let key = BrotliEncoderParameter_BROTLI_PARAM_DISABLE_LITERAL_CONTEXT_MODELING;
            let value = disable_context_modeling as u32;

            encoder.set_param(key, value)?;
        }

        if let Some(size_hint) = self.size_hint {
            let key = BrotliEncoderParameter_BROTLI_PARAM_SIZE_HINT;
            let value = size_hint;

            encoder.set_param(key, value)?;
        }

        if let Some(postfix_bits) = self.postfix_bits {
            if postfix_bits > 3 {
                return Err(SetParameterError::InvalidPostfix);
            }

            let key = BrotliEncoderParameter_BROTLI_PARAM_NPOSTFIX;
            let value = postfix_bits;

            encoder.set_param(key, value)?;
        }

        if let Some(direct_distance_codes) = self.direct_distance_codes {
            let postfix = self.postfix_bits.unwrap_or(0);

            if (direct_distance_codes > (15 << postfix))
                || (direct_distance_codes & ((1 << postfix) - 1)) != 0
            {
                return Err(SetParameterError::InvalidDirectDistanceCodes);
            }

            let key = BrotliEncoderParameter_BROTLI_PARAM_NDIRECT;
            let value = direct_distance_codes;

            encoder.set_param(key, value)?;
        }

        if let Some(stream_offset) = self.stream_offset {
            if stream_offset > (1 << 30) {
                return Err(SetParameterError::InvalidStreamOffset);
            }

            let key = BrotliEncoderParameter_BROTLI_PARAM_STREAM_OFFSET;
            let value = stream_offset;

            encoder.set_param(key, value)?;
        }

        Ok(())
    }
}

impl Default for BrotliEncoderOptions {
    fn default() -> Self {
        BrotliEncoderOptions::new()
    }
}

/// A struct used by [`BrotliEncoder::compress`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct EncodeResult {
    /// the number of bytes read from `input`.
    pub bytes_read: usize,
    /// the number of bytes written to `output`.
    pub bytes_written: usize,
}

/// An error returned by [`BrotliEncoder::compress`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct EncodeError;

impl Error for EncodeError {}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("brotli encoder error")
    }
}

impl From<EncodeError> for io::Error {
    fn from(err: EncodeError) -> Self {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

/// Wraps a reader and compresses its output.
///
/// The compression stream produced by brotli must be finished in order to be
/// decompressed. This is done when the underlying reader reaches EOF.
/// Therefore, `CompressorReader<R>` does not work with `BufRead`s that return
/// an infinite amount of data. When [`read`] returns zero on a non-zero buffer,
/// the compression is considered finished.
///
/// # Examples
///
/// Suppose the file `test.txt` contains uncompressed text. Let's try to
/// compress it:
///
/// ```no_run
/// use std::fs::File;
/// use std::io::Read;
///
/// use brotlic::DecompressorWriter;
///
/// let mut input = File::open("test.txt")?; // test.brotli is brotli compressed
/// let mut output = Vec::new();
///
/// input.read_to_end(&mut output)?;
///
/// println!("Compressed length: {}", output.len());
///
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// [`read`]: CompressorReader::read
#[derive(Debug)]
pub struct CompressorReader<R: BufRead> {
    inner: R,
    encoder: BrotliEncoder,
    op: BrotliOperation,
}

impl<R: BufRead> CompressorReader<R> {
    /// Creates a new `CompressorReader<R>` with a newly created encoder.
    ///
    /// # Panics
    ///
    /// Panics if the encoder fails to be allocated or initialized
    pub fn new(inner: R) -> Self {
        CompressorReader {
            inner,
            encoder: BrotliEncoder::new(),
            op: BrotliOperation::Process,
        }
    }

    /// Creates a new `CompressorReader<R>` with a newly created encoder using
    /// allocator `alloc`.
    ///
    /// # Panics
    ///
    /// Panics if the encoder fails to be allocated or initialized
    pub fn new_in<A>(inner: R, alloc: A) -> Self
    where
        A: GlobalAlloc + 'static,
    {
        CompressorReader {
            inner,
            encoder: BrotliEncoder::new_in(alloc),
            op: BrotliOperation::Process,
        }
    }

    /// Creates a new `CompressorReader<R>` with a specified encoder.
    ///
    /// # Examples
    ///
    /// ```
    /// use brotlic::{BrotliEncoderOptions, CompressorReader, Quality, WindowSize};
    ///
    /// let encoder = BrotliEncoderOptions::new()
    ///     .quality(Quality::new(6)?)
    ///     .window_size(WindowSize::new(18)?)
    ///     .build()?;
    ///
    /// let underlying_source = [1, 2, 3, 4, 5];
    /// let writer = CompressorReader::with_encoder(encoder, underlying_source.as_slice());
    /// # Ok::<(), brotlic::SetParameterError>(())
    /// ```
    pub fn with_encoder(encoder: BrotliEncoder, inner: R) -> Self {
        CompressorReader {
            inner,
            encoder,
            op: BrotliOperation::Process,
        }
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

    /// Unwraps this `CompressorReader<R>`, returning the underlying reader.
    ///
    /// # Errors
    ///
    /// An [`Err`] will be returned if the compression stream has not been
    /// finished.
    pub fn into_inner(self) -> Result<R, IntoInnerError<CompressorReader<R>>> {
        if self.encoder.is_finished() {
            Ok(self.inner)
        } else {
            Err(IntoInnerError::new(
                self,
                io::ErrorKind::UnexpectedEof.into(),
            ))
        }
    }

    /// Disassembles this `CompressorReader<R>`, returning the underlying reader
    /// and encoder.
    ///
    /// `into_parts` makes no attempt to validate that the compression stream
    /// finished and cannot fail.
    pub fn into_parts(self) -> (R, BrotliEncoder) {
        (self.inner, self.encoder)
    }
}

impl<R: BufRead> Read for CompressorReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let input = self.inner.fill_buf()?;
            let eof = input.is_empty();
            let EncodeResult {
                bytes_read,
                bytes_written,
            } = self.encoder.compress(input, buf, self.op)?;
            self.inner.consume(bytes_read);

            match self.op {
                _ if bytes_written > 0 => return Ok(bytes_written),
                _ if buf.is_empty() => return Ok(0),
                _ if !eof => continue,
                BrotliOperation::Process => {
                    self.op = BrotliOperation::Finish;
                    continue;
                }
                BrotliOperation::Finish => return Ok(0),
                _ => unreachable!(),
            }
        }
    }
}

/// Wraps a writer and compresses its output.
///
/// `CompressorWriter<W>` wraps a writer and adds brotli compression to the
/// output. It is critical to finish the compression stream, otherwise
/// decompression will not be successful. Dropping will attempt to finish the
/// compression stream, any errors that might arise however will be ignored.
/// Calling [`into_inner`] ensures that the compression stream is finished.
///
/// Calling [`flush`] will not only flush the underlying writer, but also flush
/// all of its compression stream. This will lead to a slight decrease of
/// compression quality, as output will be forced to be flushed as is and not
/// compressed till the block is finished.
///
/// # Examples
///
/// Let's compress some text file named `text.txt` and write the output to
/// `test.brotli`:
///
/// ```no_run
/// use std::fs::File;
/// use std::io;
///
/// use brotlic::CompressorWriter;
///
/// let mut input = File::open("test.txt")?; // test.txt is uncompressed
/// let mut output = File::create("test.brotli")?;
/// let mut compressed_output = CompressorWriter::new(output);
///
/// io::copy(&mut input, &mut compressed_output)?;
///
/// # Ok::<(), io::Error>(())
/// ```
///
/// To decompress it again, use [`DecompressorWriter`].
///
/// [`into_inner`]: CompressorWriter::into_inner
/// [`flush`]: CompressorWriter::flush
/// [`DecompressorWriter`]: crate::decode::DecompressorWriter
#[derive(Debug)]
pub struct CompressorWriter<W: Write> {
    inner: W,
    encoder: BrotliEncoder,
    panicked: bool,
}

impl<W: Write> CompressorWriter<W> {
    /// Creates a new `CompressorWriter<W>` with a newly created encoder.
    ///
    /// # Panics
    ///
    /// Panics if the encoder fails to be allocated or initialized
    pub fn new(inner: W) -> Self {
        CompressorWriter {
            inner,
            encoder: BrotliEncoder::new(),
            panicked: false,
        }
    }

    /// Creates a new `CompressorWriter<W>` with a newly created encoder using
    /// allocator `alloc`.
    ///
    /// # Panics
    ///
    /// Panics if the encoder fails to be allocated or initialized
    pub fn new_in<A>(inner: W, alloc: A) -> Self
    where
        A: GlobalAlloc + 'static,
    {
        CompressorWriter {
            inner,
            encoder: BrotliEncoder::new_in(alloc),
            panicked: false,
        }
    }

    /// Creates a new `CompressorWriter<W>` with a specified encoder.
    ///
    /// # Examples
    ///
    /// ```
    /// use brotlic::{BrotliEncoderOptions, CompressorWriter, Quality, WindowSize};
    ///
    /// let encoder = BrotliEncoderOptions::new()
    ///     .quality(Quality::new(4)?)
    ///     .window_size(WindowSize::new(16)?)
    ///     .build()?;
    ///
    /// let underlying_storage = Vec::new();
    /// let writer = CompressorWriter::with_encoder(encoder, underlying_storage);
    /// # Ok::<(), brotlic::SetParameterError>(())
    /// ```
    pub fn with_encoder(encoder: BrotliEncoder, inner: W) -> Self {
        CompressorWriter {
            inner,
            encoder,
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

    /// Unwraps this `CompressorWriter<W>`, returning the underlying writer.
    ///
    /// The compression stream is finished before returning the writer.
    ///
    /// # Errors
    ///
    /// An [`Err`] will be returned if an error occurs while finishing the
    /// compression stream.
    pub fn into_inner(mut self) -> Result<W, IntoInnerError<CompressorWriter<W>>> {
        match self.finish() {
            Err(e) => Err(IntoInnerError::new(self, e)),
            Ok(()) => Ok(self.into_parts().0),
        }
    }

    /// Disassembles this `CompressorWriter<W>`, returning the underlying writer
    /// and encoder.
    ///
    /// If the underlying writer panicked, it is not known what portion of the
    /// data was written. In this case, we return `WriterPanicked` to get the
    /// encoder back. It is worth noting that the compression stream is not
    /// finished and hence cannot be successfully decompressed. To obtain the
    /// writer once the compression stream is finished, use [`into_inner`].
    ///
    /// `into_parts` makes no attempt to finish the compression stream and
    /// cannot fail.
    ///
    /// [`into_inner`]: Self::into_inner
    pub fn into_parts(self) -> (W, Result<BrotliEncoder, WriterPanicked>) {
        let inner = unsafe { ptr::read(&self.inner) };
        let encoder = unsafe { ptr::read(&self.encoder) };
        let panicked = self.panicked;
        mem::forget(self);

        let encoder = if !panicked {
            Ok(encoder)
        } else {
            Err(WriterPanicked { encoder })
        };

        (inner, encoder)
    }

    fn finish(&mut self) -> io::Result<()> {
        self.encoder.finish()?;
        self.flush_encoder_output()
    }

    fn flush_encoder_output(&mut self) -> io::Result<()> {
        while let Some(output) = unsafe { self.encoder.take_output() } {
            self.panicked = true;
            let r = self.inner.write_all(output);
            self.panicked = false;
            r?;
        }

        Ok(())
    }
}

impl<W: Write> Write for CompressorWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_read = self.encoder.give_input(buf, BrotliOperation::Process)?;
        self.flush_encoder_output()?;

        Ok(bytes_read)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.encoder.flush()?;
        self.flush_encoder_output()?;

        self.inner.flush()
    }
}

impl<W: Write> Drop for CompressorWriter<W> {
    fn drop(&mut self) {
        if !self.panicked {
            let _r = self.finish();
        }
    }
}

/// Error returned from [`CompressorWriter::into_inner`], when the underlying
/// writer has previously panicked. Contains the encoder that was used for
/// compression.
#[derive(Debug)]
pub struct WriterPanicked {
    encoder: BrotliEncoder,
}

impl WriterPanicked {
    /// Returns the encoder that was used for compression. It is unknown what
    /// data was fed to the encoder, so simply using it to finish it is not a
    /// good idea.
    pub fn into_inner(self) -> BrotliEncoder {
        self.encoder
    }
}

impl Error for WriterPanicked {}

impl fmt::Display for WriterPanicked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "CompressorWriter inner writer panicked, what data remains unwritten is not known",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_quality() {
        let invalid = Quality::new(12);

        assert_eq!(invalid.unwrap_err(), SetParameterError::InvalidQuality);
    }

    #[test]
    fn invalid_window_size() {
        let invalid = WindowSize::new(25);

        assert_eq!(invalid.unwrap_err(), SetParameterError::InvalidWindowSize);
    }

    #[test]
    fn invalid_large_window_size() {
        let invalid = LargeWindowSize::new(31);

        assert_eq!(invalid.unwrap_err(), SetParameterError::InvalidWindowSize);
    }

    #[test]
    fn invalid_block_size() {
        let invalid = BlockSize::new(25);

        assert_eq!(invalid.unwrap_err(), SetParameterError::InvalidBlockSize);
    }

    #[test]
    fn valid_stream_offset() {
        let res = BrotliEncoderOptions::new().stream_offset(1 << 30).build();

        assert!(res.is_ok());
    }

    #[test]
    fn invalid_stream_offset() {
        let res = BrotliEncoderOptions::new()
            .stream_offset((1 << 30) + 2)
            .build();

        assert_eq!(res.unwrap_err(), SetParameterError::InvalidStreamOffset);
    }

    #[test]
    fn valid_postfix_bits() {
        let res = BrotliEncoderOptions::new().postfix_bits(3).build();

        assert!(res.is_ok());
    }

    #[test]
    fn invalid_postfix_bits() {
        let res = BrotliEncoderOptions::new().postfix_bits(7).build();

        assert_eq!(res.unwrap_err(), SetParameterError::InvalidPostfix);
    }

    #[test]
    fn valid_direct_distance_codes() {
        let res = BrotliEncoderOptions::new()
            .postfix_bits(3)
            .direct_distance_codes(120)
            .build();

        assert!(res.is_ok());
    }

    #[test]
    fn invalid_direct_distance_codes() {
        let res = BrotliEncoderOptions::new()
            .postfix_bits(2)
            .direct_distance_codes(120)
            .build();

        assert_eq!(
            res.unwrap_err(),
            SetParameterError::InvalidDirectDistanceCodes
        );
    }
}
