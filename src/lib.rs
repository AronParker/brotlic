pub mod decode;
pub mod encode;

use brotlic_sys::*;
use std::os::raw::c_int;
use std::{error, fmt, io};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Quality(u8);

impl Quality {
    pub fn new(value: u8) -> Result<Quality, QualityError> {
        match value {
            BROTLI_MIN_QUALITY..=BROTLI_MAX_QUALITY => Ok(Quality(value)),
            _ => Err(QualityError),
        }
    }

    pub fn best() -> Quality {
        Quality(BROTLI_MAX_QUALITY)
    }

    pub fn default() -> Quality {
        Quality(BROTLI_DEFAULT_QUALITY)
    }

    pub fn worst() -> Quality {
        Quality(BROTLI_MIN_QUALITY)
    }
}

impl Default for Quality {
    fn default() -> Self {
        Quality::default()
    }
}

#[derive(Debug)]
pub struct QualityError;

impl fmt::Display for QualityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "quality out of range (must be between [{},{}])",
            BROTLI_MIN_QUALITY, BROTLI_MAX_QUALITY
        )
    }
}

impl error::Error for QualityError {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct WindowSize(u8);

impl WindowSize {
    pub fn new(bits: u8) -> Result<WindowSize, WindowSizeError> {
        match bits {
            BROTLI_MIN_WINDOW_BITS..=BROTLI_MAX_WINDOW_BITS => Ok(WindowSize(bits)),
            _ => Err(WindowSizeError),
        }
    }

    pub fn best() -> WindowSize {
        WindowSize(BROTLI_MAX_WINDOW_BITS)
    }

    pub fn default() -> WindowSize {
        WindowSize(BROTLI_DEFAULT_WINDOW)
    }

    pub fn worst() -> WindowSize {
        WindowSize(BROTLI_MIN_WINDOW_BITS)
    }
}

impl Default for WindowSize {
    fn default() -> Self {
        WindowSize::default()
    }
}

impl TryFrom<LargeWindowSize> for WindowSize {
    type Error = WindowSizeError;

    fn try_from(large_window_size: LargeWindowSize) -> Result<Self, Self::Error> {
        WindowSize::new(large_window_size.0)
    }
}

#[derive(Debug)]
pub struct WindowSizeError;

impl fmt::Display for WindowSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "window size out of range (must be between [{},{}])",
            BROTLI_MIN_WINDOW_BITS, BROTLI_MAX_WINDOW_BITS
        )
    }
}

impl error::Error for WindowSizeError {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LargeWindowSize(u8);

impl LargeWindowSize {
    pub fn new(bits: u8) -> Result<LargeWindowSize, LargeWindowSizeError> {
        match bits {
            BROTLI_MIN_WINDOW_BITS..=BROTLI_LARGE_MAX_WINDOW_BITS => Ok(LargeWindowSize(bits)),
            _ => Err(LargeWindowSizeError),
        }
    }

    pub fn best() -> LargeWindowSize {
        LargeWindowSize(BROTLI_LARGE_MAX_WINDOW_BITS)
    }

    pub fn default() -> LargeWindowSize {
        LargeWindowSize(BROTLI_DEFAULT_WINDOW)
    }

    pub fn worst() -> LargeWindowSize {
        LargeWindowSize(BROTLI_MIN_WINDOW_BITS)
    }
}

impl Default for LargeWindowSize {
    fn default() -> Self {
        LargeWindowSize::default()
    }
}

impl From<WindowSize> for LargeWindowSize {
    fn from(window_size: WindowSize) -> Self {
        LargeWindowSize(window_size.0)
    }
}

#[derive(Debug)]
pub struct LargeWindowSizeError;

impl fmt::Display for LargeWindowSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "large window size out of range (must be between [{},{}])",
            BROTLI_MIN_WINDOW_BITS, BROTLI_LARGE_MAX_WINDOW_BITS
        )
    }
}

impl error::Error for LargeWindowSizeError {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BlockBits(u8);

impl BlockBits {
    pub fn new(value: u8) -> Result<BlockBits, BlockBitsError> {
        match value {
            BROTLI_MIN_INPUT_BLOCK_BITS..=BROTLI_MAX_INPUT_BLOCK_BITS => Ok(BlockBits(value)),
            _ => Err(BlockBitsError),
        }
    }

    pub fn worst() -> BlockBits {
        BlockBits(BROTLI_MIN_INPUT_BLOCK_BITS)
    }

    pub fn best() -> BlockBits {
        BlockBits(BROTLI_MAX_INPUT_BLOCK_BITS)
    }
}

#[derive(Debug)]
pub struct BlockBitsError;

impl fmt::Display for BlockBitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "block bits out of range (must be between [{},{}])",
            BROTLI_MIN_INPUT_BLOCK_BITS, BROTLI_MAX_INPUT_BLOCK_BITS
        )
    }
}

impl error::Error for BlockBitsError {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CompressionMode {
    Generic = BrotliEncoderMode_BROTLI_MODE_GENERIC as isize,
    Text = BrotliEncoderMode_BROTLI_MODE_TEXT as isize,
    Font = BrotliEncoderMode_BROTLI_MODE_FONT as isize,
}

impl Default for CompressionMode {
    fn default() -> Self {
        CompressionMode::Generic
    }
}

#[derive(Debug)]
pub struct CompressionError;

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("buffer was too small or compression error occurred")
    }
}

impl error::Error for CompressionError {}

pub type CompressionResult<T> = Result<T, CompressionError>;

#[derive(Debug)]
pub struct DecompressionError;

impl fmt::Display for DecompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("buffer was too small or decompression error occurred")
    }
}

impl error::Error for DecompressionError {}

pub type DecompressionResult<T> = Result<T, DecompressionError>;

#[derive(Debug)]
pub struct ParameterError;

impl fmt::Display for ParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid parameter")
    }
}

impl error::Error for ParameterError {}

pub fn compress(
    input: &[u8],
    output: &mut [u8],
    quality: Quality,
    window_size: WindowSize,
    mode: CompressionMode,
) -> CompressionResult<usize> {
    let mut output_size = output.len();

    let res = unsafe {
        BrotliEncoderCompress(
            quality.0 as c_int,
            window_size.0 as c_int,
            mode as BrotliEncoderMode,
            input.len(),
            input.as_ptr(),
            &mut output_size as *mut usize,
            output.as_mut_ptr(),
        )
    };

    if res != 0 {
        Ok(output_size)
    } else {
        Err(CompressionError)
    }
}

pub fn compress_bound(input_size: usize, quality: Quality) -> Option<usize> {
    // BrotliEncoderMaxCompressedSize documentation:
    // Result is only valid if quality is at least @c 2
    if quality.0 >= 2 {
        Some(unsafe { BrotliEncoderMaxCompressedSize(input_size) })
    } else {
        None
    }
}

pub fn decompress(input: &[u8], output: &mut [u8]) -> DecompressionResult<usize> {
    let mut output_size = output.len();

    let res = unsafe {
        BrotliDecoderDecompress(
            input.len(),
            input.as_ptr(),
            &mut output_size as *mut usize,
            output.as_mut_ptr(),
        )
    };

    if res == BrotliDecoderResult_BROTLI_DECODER_RESULT_SUCCESS {
        Ok(output_size)
    } else {
        Err(DecompressionError)
    }
}

#[derive(Debug)]
pub struct IntoInnerError<W>(W, io::Error);

impl<W> IntoInnerError<W> {
    fn new(writer: W, error: io::Error) -> Self {
        Self(writer, error)
    }

    pub fn error(&self) -> &io::Error {
        &self.1
    }

    pub fn into_inner(self) -> W {
        self.0
    }

    pub fn into_error(self) -> io::Error {
        self.1
    }

    pub fn into_parts(self) -> (io::Error, W) {
        (self.1, self.0)
    }
}

impl<W> From<IntoInnerError<W>> for io::Error {
    fn from(iie: IntoInnerError<W>) -> io::Error {
        iie.1
    }
}

impl<W: fmt::Debug> error::Error for IntoInnerError<W> {}

impl<W> fmt::Display for IntoInnerError<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Write};
    use super::*;
    use super::encode::*;
    use super::decode::*;

    #[test]
    fn roundtrip() {
        let input = vec![0; 8192];

        let encoded = {
            let mut input_stream = Cursor::new(Vec::new());
            let mut compressed_stream = CompressorWriter::new(input_stream);
            compressed_stream.write_all(input.as_slice());

            compressed_stream.into_inner().unwrap().into_inner()
        };

        let decoded = {
            let mut input_stream = Cursor::new(encoded);
            let mut decompressed_stream = DecompressorReader::new(input_stream);
            let mut output = Vec::new();

            decompressed_stream.read_to_end(&mut output).unwrap();
            output
        };

        assert_eq!(input, decoded);
    }
}
