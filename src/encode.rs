use crate::{
    BlockBits, CompressionMode, IntoInnerError, LargeWindowSize, ParameterError, Quality,
    WindowSize,
};
use brotlic_sys::*;
use std::io::{BufRead, Read, Write};
use std::{error, fmt, io, mem, ptr, slice};

#[derive(Debug)]
pub struct BrotliEncoder {
    state: *mut BrotliEncoderState,
}

impl BrotliEncoder {
    pub fn new() -> Self {
        unsafe {
            let instance = BrotliEncoderCreateInstance(None, None, ptr::null_mut());

            if !instance.is_null() {
                BrotliEncoder { state: instance }
            } else {
                panic!(
                    "BrotliEncoderCreateInstance returned NULL: failed to allocate or initialize"
                );
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        unsafe { BrotliEncoderIsFinished(self.state) != 0 }
    }

    pub fn compress(
        &mut self,
        input: &[u8],
        output: &mut [u8],
        op: BrotliOperation,
    ) -> Result<(usize, usize), EncoderError> {
        let mut input_ptr = input.as_ptr();
        let mut input_len = input.len();
        let mut output_ptr = output.as_mut_ptr();
        let mut output_len = output.len();

        let result = unsafe {
            BrotliEncoderCompressStream(
                self.state,
                op as BrotliEncoderOperation,
                &mut input_len as _,
                &mut input_ptr as _,
                &mut output_len as _,
                &mut output_ptr as _,
                ptr::null_mut(),
            )
        };

        if result != 0 {
            let bytes_read = input.len() - input_len;
            let bytes_written = output.len() - output_len;

            Ok((bytes_read, bytes_written))
        } else {
            Err(EncoderError)
        }
    }

    pub fn give_input(&mut self, input: &[u8], op: BrotliOperation) -> Result<usize, EncoderError> {
        Ok(self.compress(input, &mut [], op)?.0)
    }

    pub fn flush(&mut self) -> Result<(), EncoderError> {
        self.give_op(BrotliOperation::Flush)
    }

    pub fn finish(&mut self) -> Result<(), EncoderError> {
        self.give_op(BrotliOperation::Finish)
    }

    pub unsafe fn take_output(&mut self) -> Option<&[u8]> {
        if BrotliEncoderHasMoreOutput(self.state) != 0 {
            let mut len: usize = 0;
            let output = BrotliEncoderTakeOutput(self.state, &mut len as _);

            Some(slice::from_raw_parts(output, len))
        } else {
            None
        }
    }

    pub fn version() -> u32 {
        unsafe { BrotliEncoderVersion() }
    }

    fn set_param(
        &mut self,
        param: BrotliEncoderParameter,
        value: u32,
    ) -> Result<(), ParameterError> {
        let r = unsafe { BrotliEncoderSetParameter(self.state, param, value) };

        if r != 0 { Ok(()) } else { Err(ParameterError) }
    }

    fn give_op(&mut self, op: BrotliOperation) -> Result<(), EncoderError> {
        self.give_input(&[], op)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BrotliOperation {
    Process = BrotliEncoderOperation_BROTLI_OPERATION_PROCESS as isize,
    Flush = BrotliEncoderOperation_BROTLI_OPERATION_FLUSH as isize,
    Finish = BrotliEncoderOperation_BROTLI_OPERATION_FINISH as isize,
}

impl Default for BrotliEncoder {
    fn default() -> Self {
        BrotliEncoder::new()
    }
}

impl Drop for BrotliEncoder {
    fn drop(&mut self) {
        unsafe {
            BrotliEncoderDestroyInstance(self.state);
        }
    }
}

pub struct BrotliEncoderOptions {
    mode: Option<CompressionMode>,
    quality: Option<Quality>,
    window_size: Option<LargeWindowSize>,
    block_bits: Option<BlockBits>,
    disable_context_modeling: Option<bool>,
    size_hint: Option<u32>,
    postfix_bits: Option<u32>,
    direct_distance_codes: Option<u32>,
    stream_offset: Option<u32>,
}

impl BrotliEncoderOptions {
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

    pub fn mode(&mut self, mode: CompressionMode) -> &mut Self {
        self.mode = Some(mode);
        self
    }

    pub fn quality(&mut self, quality: Quality) -> &mut Self {
        self.quality = Some(quality);
        self
    }

    pub fn window_size(&mut self, window_size: WindowSize) -> &mut Self {
        self.window_size = Some(window_size.into());
        self
    }

    pub fn non_std_window_size(&mut self, large_window_size: LargeWindowSize) -> &mut Self {
        self.window_size = Some(large_window_size);
        self
    }

    pub fn block_bits(&mut self, block_bits: BlockBits) -> &mut Self {
        self.block_bits = Some(block_bits);
        self
    }

    pub fn disable_context_modeling(&mut self, disable_context_modeling: bool) -> &mut Self {
        self.disable_context_modeling = Some(disable_context_modeling);
        self
    }

    pub fn size_hint(&mut self, size_hint: u32) -> &mut Self {
        self.size_hint = Some(size_hint);
        self
    }

    pub fn postfix_bits(&mut self, postfix_bits: u32) -> &mut Self {
        self.postfix_bits = Some(postfix_bits);
        self
    }

    pub fn direct_distance_codes(&mut self, direct_distance_codes: u32) -> &mut Self {
        self.direct_distance_codes = Some(direct_distance_codes);
        self
    }

    pub fn stream_offset(&mut self, stream_offset: u32) -> &mut Self {
        self.stream_offset = Some(stream_offset);
        self
    }

    pub fn build(&self) -> Result<BrotliEncoder, ParameterError> {
        let mut encoder = BrotliEncoder::new();

        if let Some(mode) = self.mode {
            encoder.set_param(BrotliEncoderParameter_BROTLI_PARAM_MODE, mode as u32)?;
        }

        if let Some(quality) = self.quality {
            encoder.set_param(
                BrotliEncoderParameter_BROTLI_PARAM_QUALITY,
                quality.0 as u32,
            )?;
        }

        if let Some(window_size) = self.window_size {
            encoder.set_param(
                BrotliEncoderParameter_BROTLI_PARAM_LGWIN,
                window_size.0 as u32,
            )?;

            let large_window = WindowSize::try_from(window_size).is_err();

            encoder.set_param(
                BrotliEncoderParameter_BROTLI_PARAM_LARGE_WINDOW,
                large_window as u32,
            )?;
        }

        if let Some(block_bits) = self.block_bits {
            encoder.set_param(
                BrotliEncoderParameter_BROTLI_PARAM_LGBLOCK,
                block_bits.0 as u32,
            )?;
        }

        if let Some(disable_context_modeling) = self.disable_context_modeling {
            encoder.set_param(
                BrotliEncoderParameter_BROTLI_PARAM_DISABLE_LITERAL_CONTEXT_MODELING,
                disable_context_modeling as u32,
            )?;
        }

        if let Some(size_hint) = self.size_hint {
            encoder.set_param(BrotliEncoderParameter_BROTLI_PARAM_SIZE_HINT, size_hint)?;
        }

        if let Some(postfix_bits) = self.postfix_bits {
            encoder.set_param(BrotliEncoderParameter_BROTLI_PARAM_NPOSTFIX, postfix_bits)?;
        }

        if let Some(direct_distance_codes) = self.direct_distance_codes {
            encoder.set_param(
                BrotliEncoderParameter_BROTLI_PARAM_NDIRECT,
                direct_distance_codes,
            )?;
        }

        if let Some(stream_offset) = self.stream_offset {
            encoder.set_param(
                BrotliEncoderParameter_BROTLI_PARAM_STREAM_OFFSET,
                stream_offset,
            )?;
        }

        Ok(encoder)
    }
}

impl Default for BrotliEncoderOptions {
    fn default() -> Self {
        BrotliEncoderOptions::new()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DecoderResult {
    Success,
    NeedsMoreInput,
    NeedsMoreOutput,
}

#[derive(Debug)]
pub struct EncoderError;

impl error::Error for EncoderError {}

impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("brotli encoder error")
    }
}

impl From<EncoderError> for io::Error {
    fn from(err: EncoderError) -> Self {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

#[derive(Debug)]
pub struct CompressorReader<R: BufRead> {
    inner: R,
    encoder: BrotliEncoder,
    op: BrotliOperation,
}

impl<R: BufRead> CompressorReader<R> {
    pub fn new(inner: R) -> Self {
        CompressorReader {
            inner,
            encoder: BrotliEncoder::new(),
            op: BrotliOperation::Process,
        }
    }

    pub fn with_encoder(encoder: BrotliEncoder, inner: R) -> Self {
        CompressorReader {
            inner,
            encoder,
            op: BrotliOperation::Process,
        }
    }

    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn into_inner(self) -> Result<R, IntoInnerError<CompressorReader<R>>> {
        if self.encoder.is_finished() {
            Ok(self.inner)
        } else {
            Err(IntoInnerError::new(
                self,
                io::Error::from(io::ErrorKind::UnexpectedEof),
            ))
        }
    }

    pub fn into_parts(self) -> (R, BrotliEncoder) {
        (self.inner, self.encoder)
    }
}

impl<R: BufRead> Read for CompressorReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let input = self.inner.fill_buf()?;
            let eof = input.is_empty();
            let (read, written) = self.encoder.compress(input, buf, self.op)?;
            self.inner.consume(read);

            match self.op {
                _ if written > 0 => return Ok(written),
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

#[derive(Debug)]
pub struct CompressorWriter<W: Write> {
    inner: W,
    encoder: BrotliEncoder,
    panicked: bool,
}

impl<W: Write> CompressorWriter<W> {
    pub fn new(inner: W) -> CompressorWriter<W> {
        CompressorWriter {
            inner,
            encoder: BrotliEncoder::new(),
            panicked: false,
        }
    }

    pub fn with_encoder(encoder: BrotliEncoder, inner: W) -> Self {
        CompressorWriter {
            inner,
            encoder,
            panicked: false,
        }
    }

    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    pub fn into_inner(mut self) -> Result<W, IntoInnerError<CompressorWriter<W>>> {
        match self.finish() {
            Err(e) => Err(IntoInnerError::new(self, e)),
            Ok(()) => Ok(self.into_parts().0),
        }
    }

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

#[derive(Debug)]
pub struct WriterPanicked {
    encoder: BrotliEncoder,
}

impl WriterPanicked {
    pub fn into_inner(self) -> BrotliEncoder {
        self.encoder
    }
}

impl error::Error for WriterPanicked {}

impl fmt::Display for WriterPanicked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "CompressorWriter inner writer panicked, what data remains unwritten is not known",
        )
    }
}
