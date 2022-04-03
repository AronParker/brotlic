use std::ffi::CStr;
use std::io::{BufRead, Read, Write};
use std::{error, fmt, io, ptr, slice};

use brotlic_sys::*;

use crate::{IntoInnerError, ParameterError};

#[derive(Debug)]
pub struct BrotliDecoder {
    state: *mut BrotliDecoderState,
}

impl BrotliDecoder {
    pub fn new() -> Self {
        unsafe {
            let instance = BrotliDecoderCreateInstance(None, None, ptr::null_mut());

            if !instance.is_null() {
                BrotliDecoder { state: instance }
            } else {
                panic!(
                    "BrotliDecoderCreateInstance returned NULL: failed to allocate or initialize"
                );
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        unsafe { BrotliDecoderIsFinished(self.state) != 0 }
    }

    pub fn decompress(
        &mut self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<(usize, usize, DecoderResult), DecoderError> {
        let mut input_ptr = input.as_ptr();
        let mut input_len = input.len();
        let mut output_ptr = output.as_mut_ptr();
        let mut output_len = output.len();

        let result = unsafe {
            BrotliDecoderDecompressStream(
                self.state,
                &mut input_len as _,
                &mut input_ptr as _,
                &mut output_len as _,
                &mut output_ptr as _,
                ptr::null_mut(),
            )
        };

        let bytes_read = input.len() - input_len;
        let bytes_written = output.len() - output_len;

        #[allow(non_upper_case_globals)]
        let decoder_result = match result {
            BrotliDecoderResult_BROTLI_DECODER_RESULT_ERROR => return Err(self.last_error()),
            BrotliDecoderResult_BROTLI_DECODER_RESULT_SUCCESS => DecoderResult::Success,
            BrotliDecoderResult_BROTLI_DECODER_RESULT_NEEDS_MORE_INPUT => {
                DecoderResult::NeedsMoreInput
            }
            BrotliDecoderResult_BROTLI_DECODER_RESULT_NEEDS_MORE_OUTPUT => {
                DecoderResult::NeedsMoreOutput
            }
            _ => panic!("BrotliDecoderDecompressStream returned an unknown error code"),
        };

        Ok((bytes_read, bytes_written, decoder_result))
    }

    pub fn give_input(&mut self, input: &[u8]) -> Result<(usize, DecoderResult), DecoderError> {
        let res = self.decompress(input, &mut [])?;

        Ok((res.0, res.2))
    }

    pub unsafe fn take_output(&mut self) -> Option<&[u8]> {
        if BrotliDecoderHasMoreOutput(self.state) != 0 {
            let mut len: usize = 0;
            let output = BrotliDecoderTakeOutput(self.state, &mut len as _);

            Some(slice::from_raw_parts(output, len))
        } else {
            None
        }
    }

    pub fn version() -> u32 {
        unsafe { BrotliDecoderVersion() }
    }

    fn set_param(
        &mut self,
        param: BrotliDecoderParameter,
        value: u32,
    ) -> Result<(), ParameterError> {
        let r = unsafe { BrotliDecoderSetParameter(self.state, param, value) };

        if r != 0 { Ok(()) } else { Err(ParameterError) }
    }

    fn last_error(&self) -> DecoderError {
        let c = unsafe { BrotliDecoderGetErrorCode(self.state) };

        #[allow(non_upper_case_globals)]
        match c {
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_NIBBLE => {
                DecoderError::FormatExuberantNibble
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_RESERVED => {
                DecoderError::FormatReserved
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_META_NIBBLE => {
                DecoderError::FormatExuberantMetaNibble
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_ALPHABET => {
                DecoderError::FormatSimpleHuffmanAlphabet
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_SAME => {
                DecoderError::FormatSimpleHuffmanSame
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_CL_SPACE => {
                DecoderError::FormatClSpace
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_HUFFMAN_SPACE => {
                DecoderError::FormatHuffmanSpace
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_CONTEXT_MAP_REPEAT => {
                DecoderError::FormatContextMapRepeat
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_1 => {
                DecoderError::FormatBlockLength1
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_2 => {
                DecoderError::FormatBlockLength2
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_TRANSFORM => {
                DecoderError::FormatTransform
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_DICTIONARY => {
                DecoderError::FormatDictionary
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_WINDOW_BITS => {
                DecoderError::FormatWindowBits
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_PADDING_1 => {
                DecoderError::FormatPadding1
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_PADDING_2 => {
                DecoderError::FormatPadding2
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_DISTANCE => {
                DecoderError::FormatDistance
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_COMPOUND_DICTIONARY => {
                DecoderError::CompoundDictionary
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_DICTIONARY_NOT_SET => {
                DecoderError::DictionaryNotSet
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_INVALID_ARGUMENTS => {
                DecoderError::InvalidArguments
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MODES => {
                DecoderError::AllocContextModes
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_TREE_GROUPS => {
                DecoderError::AllocTreeGroups
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MAP => {
                DecoderError::AllocContextMap
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_1 => {
                DecoderError::AllocRingBuffer1
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_2 => {
                DecoderError::AllocRingBuffer2
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_BLOCK_TYPE_TREES => {
                DecoderError::AllocBlockTypeTrees
            }
            BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_UNREACHABLE => DecoderError::Unreachable,
            _ => DecoderError::UnknownError,
        }
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

pub struct BrotliDecoderOptions {
    disable_ring_buffer_reallocation: Option<bool>,
    non_std_window_size_support: Option<bool>,
}

impl BrotliDecoderOptions {
    pub fn new() -> Self {
        BrotliDecoderOptions {
            disable_ring_buffer_reallocation: None,
            non_std_window_size_support: None,
        }
    }

    pub fn disable_ring_buffer_reallocation(
        &mut self,
        disable_ring_buffer_reallocation: bool,
    ) -> &mut Self {
        self.disable_ring_buffer_reallocation = Some(disable_ring_buffer_reallocation);
        self
    }

    pub fn non_std_window_size_support(&mut self, non_std_window_size_support: bool) -> &mut Self {
        self.non_std_window_size_support = Some(non_std_window_size_support);
        self
    }

    pub fn build(&self) -> Result<BrotliDecoder, ParameterError> {
        let mut decoder = BrotliDecoder::new();

        if let Some(disable_ring_buffer_reallocation) = self.disable_ring_buffer_reallocation {
            let key = BrotliDecoderParameter_BROTLI_DECODER_PARAM_DISABLE_RING_BUFFER_REALLOCATION;
            let value = disable_ring_buffer_reallocation as u32;

            decoder.set_param(key, value)?;
        }

        if let Some(non_std_window_size_support) = self.non_std_window_size_support {
            let key = BrotliDecoderParameter_BROTLI_DECODER_PARAM_LARGE_WINDOW;
            let value = non_std_window_size_support as u32;

            decoder.set_param(key, value)?;
        }

        Ok(decoder)
    }
}

impl Default for BrotliDecoderOptions {
    fn default() -> Self {
        BrotliDecoderOptions::new()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DecoderResult {
    Success,
    NeedsMoreInput,
    NeedsMoreOutput,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DecoderError {
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

impl error::Error for DecoderError {}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == DecoderError::UnknownError {
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

impl From<DecoderError> for io::Error {
    fn from(err: DecoderError) -> Self {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

#[derive(Debug)]
pub struct DecompressorReader<R: BufRead> {
    inner: R,
    decoder: BrotliDecoder,
}

impl<R: BufRead> DecompressorReader<R> {
    pub fn new(inner: R) -> Self {
        DecompressorReader {
            inner,
            decoder: BrotliDecoder::new(),
        }
    }

    pub fn with_decoder(decoder: BrotliDecoder, inner: R) -> Self {
        DecompressorReader { inner, decoder }
    }

    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn into_inner(self) -> Result<R, IntoInnerError<DecompressorReader<R>>> {
        if self.decoder.is_finished() {
            Ok(self.inner)
        } else {
            Err(IntoInnerError::new(
                self,
                io::Error::from(io::ErrorKind::UnexpectedEof),
            ))
        }
    }

    pub fn into_parts(self) -> (R, BrotliDecoder) {
        (self.inner, self.decoder)
    }
}

impl<R: BufRead> Read for DecompressorReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let input = self.inner.fill_buf()?;
            let eof = input.is_empty();
            let (read, written, res) = self.decoder.decompress(input, buf)?;
            self.inner.consume(read);

            match res {
                _ if written > 0 => return Ok(written),
                DecoderResult::Success => return Ok(0),
                DecoderResult::NeedsMoreInput if eof => {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                }
                DecoderResult::NeedsMoreInput => continue,
                DecoderResult::NeedsMoreOutput if buf.is_empty() => return Ok(0),
                DecoderResult::NeedsMoreOutput => panic!(
                    "decoder needs output despite not giving any while having the chance to do so"
                ),
            };
        }
    }
}

#[derive(Debug)]
pub struct DecompressorWriter<W: Write> {
    inner: W,
    decoder: BrotliDecoder,
    panicked: bool,
}

impl<W: Write> DecompressorWriter<W> {
    pub fn new(inner: W) -> DecompressorWriter<W> {
        DecompressorWriter {
            inner,
            decoder: BrotliDecoder::new(),
            panicked: false,
        }
    }

    pub fn with_decoder(decoder: BrotliDecoder, inner: W) -> Self {
        DecompressorWriter {
            inner,
            decoder,
            panicked: false,
        }
    }

    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    pub fn into_inner(self) -> Result<W, IntoInnerError<DecompressorWriter<W>>> {
        if self.decoder.is_finished() {
            Ok(self.into_parts().0)
        } else {
            Err(IntoInnerError::new(
                self,
                io::Error::from(io::ErrorKind::UnexpectedEof),
            ))
        }
    }

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

#[derive(Debug)]
pub struct WriterPanicked {
    decoder: BrotliDecoder,
}

impl WriterPanicked {
    pub fn into_inner(self) -> BrotliDecoder {
        self.decoder
    }
}

impl error::Error for WriterPanicked {}

impl fmt::Display for WriterPanicked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "DecompressorWriter inner writer panicked, what data remains unwritten is not known",
        )
    }
}
