#![allow(nonstandard_style)]
#![allow(rustdoc::broken_intra_doc_links)]

use std::ffi::c_void;
use std::marker;
use std::os::raw::{c_char, c_int};

pub const BROTLI_TRUE: BROTLI_BOOL = 1;
pub const BROTLI_FALSE: BROTLI_BOOL = 0;

pub const BROTLI_MIN_QUALITY: u8 = 0;
pub const BROTLI_DEFAULT_QUALITY: u8 = 11;
pub const BROTLI_MAX_QUALITY: u8 = 11;

pub const BROTLI_MIN_WINDOW_BITS: u8 = 10;
pub const BROTLI_DEFAULT_WINDOW: u8 = 22;
pub const BROTLI_MAX_WINDOW_BITS: u8 = 24;
pub const BROTLI_LARGE_MAX_WINDOW_BITS: u8 = 30;

pub const BROTLI_MIN_INPUT_BLOCK_BITS: u8 = 16;
pub const BROTLI_MAX_INPUT_BLOCK_BITS: u8 = 24;

pub type BROTLI_BOOL = c_int;

#[doc = " Allocating function pointer type."]
#[doc = ""]
#[doc = " @param opaque custom memory manager handle provided by client"]
#[doc = " @param size requested memory region size; can not be @c 0"]
#[doc = " @returns @c 0 in the case of failure"]
#[doc = " @returns a valid pointer to a memory region of at least @p size bytes"]
#[doc = "          long otherwise"]
pub type brotli_alloc_func = Option<extern "C" fn(opaque: *mut c_void, size: usize) -> *mut c_void>;

#[doc = " Deallocating function pointer type."]
#[doc = ""]
#[doc = " This function @b SHOULD do nothing if @p address is @c 0."]
#[doc = ""]
#[doc = " @param opaque custom memory manager handle provided by client"]
#[doc = " @param address memory region pointer returned by ::brotli_alloc_func, or @c 0"]
pub type brotli_free_func = Option<extern "C" fn(opaque: *mut c_void, address: *mut c_void)>;

#[doc = " Default compression mode."]
#[doc = ""]
#[doc = " In this mode compressor does not know anything in advance about the"]
#[doc = " properties of the input."]
pub const BrotliEncoderMode_BROTLI_MODE_GENERIC: BrotliEncoderMode = 0;
#[doc = " Compression mode for UTF-8 formatted text input."]
pub const BrotliEncoderMode_BROTLI_MODE_TEXT: BrotliEncoderMode = 1;
#[doc = " Compression mode used in WOFF 2.0."]
pub const BrotliEncoderMode_BROTLI_MODE_FONT: BrotliEncoderMode = 2;
#[doc = " Options for ::BROTLI_PARAM_MODE parameter."]
pub type BrotliEncoderMode = c_int;

#[doc = " Process input."]
#[doc = ""]
#[doc = " Encoder may postpone producing output, until it has processed enough input."]
pub const BrotliEncoderOperation_BROTLI_OPERATION_PROCESS: BrotliEncoderOperation = 0;
#[doc = " Produce output for all processed input."]
#[doc = ""]
#[doc = " Actual flush is performed when input stream is depleted and there is enough"]
#[doc = " space in output stream. This means that client should repeat"]
#[doc = " ::BROTLI_OPERATION_FLUSH operation until @p available_in becomes @c 0, and"]
#[doc = " ::BrotliEncoderHasMoreOutput returns ::BROTLI_FALSE. If output is acquired"]
#[doc = " via ::BrotliEncoderTakeOutput, then operation should be repeated after"]
#[doc = " output buffer is drained."]
#[doc = ""]
#[doc = " @warning Until flush is complete, client @b SHOULD @b NOT swap,"]
#[doc = "          reduce or extend input stream."]
#[doc = ""]
#[doc = " When flush is complete, output data will be sufficient for decoder to"]
#[doc = " reproduce all the given input."]
pub const BrotliEncoderOperation_BROTLI_OPERATION_FLUSH: BrotliEncoderOperation = 1;
#[doc = " Finalize the stream."]
#[doc = ""]
#[doc = " Actual finalization is performed when input stream is depleted and there is"]
#[doc = " enough space in output stream. This means that client should repeat"]
#[doc = " ::BROTLI_OPERATION_FINISH operation until @p available_in becomes @c 0, and"]
#[doc = " ::BrotliEncoderHasMoreOutput returns ::BROTLI_FALSE. If output is acquired"]
#[doc = " via ::BrotliEncoderTakeOutput, then operation should be repeated after"]
#[doc = " output buffer is drained."]
#[doc = ""]
#[doc = " @warning Until finalization is complete, client @b SHOULD @b NOT swap,"]
#[doc = "          reduce or extend input stream."]
#[doc = ""]
#[doc = " Helper function ::BrotliEncoderIsFinished checks if stream is finalized and"]
#[doc = " output fully dumped."]
#[doc = ""]
#[doc = " Adding more input data to finalized stream is impossible."]
pub const BrotliEncoderOperation_BROTLI_OPERATION_FINISH: BrotliEncoderOperation = 2;
#[doc = " Emit metadata block to stream."]
#[doc = ""]
#[doc = " Metadata is opaque to Brotli: neither encoder, nor decoder processes this"]
#[doc = " data or relies on it. It may be used to pass some extra information from"]
#[doc = " encoder client to decoder client without interfering with main data stream."]
#[doc = ""]
#[doc = " @note Encoder may emit empty metadata blocks internally, to pad encoded"]
#[doc = "       stream to byte boundary."]
#[doc = ""]
#[doc = " @warning Until emitting metadata is complete client @b SHOULD @b NOT swap,"]
#[doc = "          reduce or extend input stream."]
#[doc = ""]
#[doc = " @warning The whole content of input buffer is considered to be the content"]
#[doc = "          of metadata block. Do @b NOT @e append metadata to input stream,"]
#[doc = "          before it is depleted with other operations."]
#[doc = ""]
#[doc = " Stream is soft-flushed before metadata block is emitted. Metadata block"]
#[doc = " @b MUST be no longer than than 16MiB."]
pub const BrotliEncoderOperation_BROTLI_OPERATION_EMIT_METADATA: BrotliEncoderOperation = 3;
#[doc = " Operations that can be performed by streaming encoder."]
pub type BrotliEncoderOperation = c_int;

#[doc = " Tune encoder for specific input."]
#[doc = ""]
#[doc = " ::BrotliEncoderMode enumerates all available values."]
pub const BrotliEncoderParameter_BROTLI_PARAM_MODE: BrotliEncoderParameter = 0;
#[doc = " The main compression speed-density lever."]
#[doc = ""]
#[doc = " The higher the quality, the slower the compression. Range is"]
#[doc = " from ::BROTLI_MIN_QUALITY to ::BROTLI_MAX_QUALITY."]
pub const BrotliEncoderParameter_BROTLI_PARAM_QUALITY: BrotliEncoderParameter = 1;
#[doc = " Recommended sliding LZ77 window size."]
#[doc = ""]
#[doc = " Encoder may reduce this value, e.g. if input is much smaller than"]
#[doc = " window size."]
#[doc = ""]
#[doc = " Window size is `(1 << value) - 16`."]
#[doc = ""]
#[doc = " Range is from ::BROTLI_MIN_WINDOW_BITS to ::BROTLI_MAX_WINDOW_BITS."]
pub const BrotliEncoderParameter_BROTLI_PARAM_LGWIN: BrotliEncoderParameter = 2;
#[doc = " Recommended input block size."]
#[doc = ""]
#[doc = " Encoder may reduce this value, e.g. if input is much smaller than input"]
#[doc = " block size."]
#[doc = ""]
#[doc = " Range is from ::BROTLI_MIN_INPUT_BLOCK_BITS to"]
#[doc = " ::BROTLI_MAX_INPUT_BLOCK_BITS."]
#[doc = ""]
#[doc = " @note Bigger input block size allows better compression, but consumes more"]
#[doc = "       memory. \\n The rough formula of memory used for temporary input"]
#[doc = "       storage is `3 << lgBlock`."]
pub const BrotliEncoderParameter_BROTLI_PARAM_LGBLOCK: BrotliEncoderParameter = 3;
#[doc = " Flag that affects usage of \"literal context modeling\" format feature."]
#[doc = ""]
#[doc = " This flag is a \"decoding-speed vs compression ratio\" trade-off."]
pub const BrotliEncoderParameter_BROTLI_PARAM_DISABLE_LITERAL_CONTEXT_MODELING:
    BrotliEncoderParameter = 4;
#[doc = " Estimated total input size for all ::BrotliEncoderCompressStream calls."]
#[doc = ""]
#[doc = " The default value is 0, which means that the total input size is unknown."]
pub const BrotliEncoderParameter_BROTLI_PARAM_SIZE_HINT: BrotliEncoderParameter = 5;
#[doc = " Flag that determines if \"Large Window Brotli\" is used."]
pub const BrotliEncoderParameter_BROTLI_PARAM_LARGE_WINDOW: BrotliEncoderParameter = 6;
#[doc = " Recommended number of postfix bits (NPOSTFIX)."]
#[doc = ""]
#[doc = " Encoder may change this value."]
#[doc = ""]
#[doc = " Range is from 0 to ::BROTLI_MAX_NPOSTFIX."]
pub const BrotliEncoderParameter_BROTLI_PARAM_NPOSTFIX: BrotliEncoderParameter = 7;
#[doc = " Recommended number of direct distance codes (NDIRECT)."]
#[doc = ""]
#[doc = " Encoder may change this value."]
#[doc = ""]
#[doc = " Range is from 0 to (15 << NPOSTFIX) in steps of (1 << NPOSTFIX)."]
pub const BrotliEncoderParameter_BROTLI_PARAM_NDIRECT: BrotliEncoderParameter = 8;
#[doc = " Number of bytes of input stream already processed by a different instance."]
#[doc = ""]
#[doc = " @note It is important to configure all the encoder instances with same"]
#[doc = "       parameters (except this one) in order to allow all the encoded parts"]
#[doc = "       obey the same restrictions implied by header."]
#[doc = ""]
#[doc = " If offset is not 0, then stream header is omitted."]
#[doc = " In any case output start is byte aligned, so for proper streams stitching"]
#[doc = " \"predecessor\" stream must be flushed."]
#[doc = ""]
#[doc = " Range is not artificially limited, but all the values greater or equal to"]
#[doc = " maximal window size have the same effect. Values greater than 2**30 are not"]
#[doc = " allowed."]
pub const BrotliEncoderParameter_BROTLI_PARAM_STREAM_OFFSET: BrotliEncoderParameter = 9;
#[doc = " Options to be used with ::BrotliEncoderSetParameter."]
pub type BrotliEncoderParameter = c_int;

#[doc = " Raw LZ77 prefix dictionary."]
pub const BrotliSharedDictionaryType_BROTLI_SHARED_DICTIONARY_RAW: BrotliSharedDictionaryType = 0;
#[doc = " Serialized shared dictionary."]
pub const BrotliSharedDictionaryType_BROTLI_SHARED_DICTIONARY_SERIALIZED:
    BrotliSharedDictionaryType = 1;
#[doc = " Input data type for ::BrotliSharedDictionaryAttach."]
pub type BrotliSharedDictionaryType = c_int;

#[doc = " Decoding error, e.g. corrupted input or memory allocation problem."]
pub const BrotliDecoderResult_BROTLI_DECODER_RESULT_ERROR: BrotliDecoderResult = 0;
#[doc = " Decoding successfully completed."]
pub const BrotliDecoderResult_BROTLI_DECODER_RESULT_SUCCESS: BrotliDecoderResult = 1;
#[doc = " Partially done; should be called again with more input."]
pub const BrotliDecoderResult_BROTLI_DECODER_RESULT_NEEDS_MORE_INPUT: BrotliDecoderResult = 2;
#[doc = " Partially done; should be called again with more output."]
pub const BrotliDecoderResult_BROTLI_DECODER_RESULT_NEEDS_MORE_OUTPUT: BrotliDecoderResult = 3;
#[doc = " Result type for ::BrotliDecoderDecompress and"]
#[doc = " ::BrotliDecoderDecompressStream functions."]
pub type BrotliDecoderResult = c_int;

pub const BrotliDecoderErrorCode_BROTLI_DECODER_NO_ERROR: BrotliDecoderErrorCode = 0;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_SUCCESS: BrotliDecoderErrorCode = 1;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_NEEDS_MORE_INPUT: BrotliDecoderErrorCode = 2;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_NEEDS_MORE_OUTPUT: BrotliDecoderErrorCode = 3;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_NIBBLE:
    BrotliDecoderErrorCode = -1;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_RESERVED: BrotliDecoderErrorCode = -2;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_META_NIBBLE:
    BrotliDecoderErrorCode = -3;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_ALPHABET:
    BrotliDecoderErrorCode = -4;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_SAME:
    BrotliDecoderErrorCode = -5;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_CL_SPACE: BrotliDecoderErrorCode = -6;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_HUFFMAN_SPACE: BrotliDecoderErrorCode =
    -7;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_CONTEXT_MAP_REPEAT:
    BrotliDecoderErrorCode = -8;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_1:
    BrotliDecoderErrorCode = -9;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_2:
    BrotliDecoderErrorCode = -10;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_TRANSFORM: BrotliDecoderErrorCode =
    -11;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_DICTIONARY: BrotliDecoderErrorCode =
    -12;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_WINDOW_BITS: BrotliDecoderErrorCode =
    -13;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_PADDING_1: BrotliDecoderErrorCode =
    -14;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_PADDING_2: BrotliDecoderErrorCode =
    -15;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_FORMAT_DISTANCE: BrotliDecoderErrorCode = -16;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_COMPOUND_DICTIONARY: BrotliDecoderErrorCode =
    -18;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_DICTIONARY_NOT_SET: BrotliDecoderErrorCode =
    -19;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_INVALID_ARGUMENTS: BrotliDecoderErrorCode =
    -20;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MODES: BrotliDecoderErrorCode =
    -21;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_TREE_GROUPS: BrotliDecoderErrorCode =
    -22;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MAP: BrotliDecoderErrorCode =
    -25;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_1: BrotliDecoderErrorCode =
    -26;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_2: BrotliDecoderErrorCode =
    -27;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_ALLOC_BLOCK_TYPE_TREES:
    BrotliDecoderErrorCode = -30;
pub const BrotliDecoderErrorCode_BROTLI_DECODER_ERROR_UNREACHABLE: BrotliDecoderErrorCode = -31;
#[doc = " Error code for detailed logging / production debugging."]
#[doc = ""]
#[doc = " See ::BrotliDecoderGetErrorCode and ::BROTLI_LAST_ERROR_CODE."]
pub type BrotliDecoderErrorCode = c_int;

#[doc = " Disable \"canny\" ring buffer allocation strategy."]
#[doc = ""]
#[doc = " Ring buffer is allocated according to window size, despite the real size of"]
#[doc = " the content."]
pub const BrotliDecoderParameter_BROTLI_DECODER_PARAM_DISABLE_RING_BUFFER_REALLOCATION:
    BrotliDecoderParameter = 0;
#[doc = " Flag that determines if \"Large Window Brotli\" is used."]
pub const BrotliDecoderParameter_BROTLI_DECODER_PARAM_LARGE_WINDOW: BrotliDecoderParameter = 1;
#[doc = " Options to be used with ::BrotliDecoderSetParameter."]
pub type BrotliDecoderParameter = c_int;

#[doc = " Opaque structure that holds shared dictionary data."]
#[doc = ""]
#[doc = " Allocated and initialized with ::BrotliSharedDictionaryCreateInstance."]
#[doc = " Cleaned up and deallocated with ::BrotliSharedDictionaryDestroyInstance."]
#[repr(C)]
pub struct BrotliSharedDictionary {
    _unused: [u8; 0],
    _marker: marker::PhantomData<(*mut u8, marker::PhantomPinned)>,
}

#[doc = " Opaque type for pointer to different possible internal structures containing"]
#[doc = " dictionary prepared for the encoder"]
#[repr(C)]
pub struct BrotliEncoderPreparedDictionary {
    _unused: [u8; 0],
    _marker: marker::PhantomData<(*mut u8, marker::PhantomPinned)>,
}

#[doc = " Opaque structure that holds encoder state."]
#[doc = ""]
#[doc = " Allocated and initialized with ::BrotliEncoderCreateInstance."]
#[doc = " Cleaned up and deallocated with ::BrotliEncoderDestroyInstance."]
#[repr(C)]
pub struct BrotliEncoderState {
    _unused: [u8; 0],
    _marker: marker::PhantomData<(*mut u8, marker::PhantomPinned)>,
}

#[doc = " Opaque structure that holds decoder state."]
#[doc = ""]
#[doc = " Allocated and initialized with ::BrotliDecoderCreateInstance."]
#[doc = " Cleaned up and deallocated with ::BrotliDecoderDestroyInstance."]
#[repr(C)]
pub struct BrotliDecoderState {
    _unused: [u8; 0],
    _marker: marker::PhantomData<(*mut u8, marker::PhantomPinned)>,
}

extern "C" {
    #[doc = " Prepares a shared dictionary from the given file format for the encoder."]
    #[doc = ""]
    #[doc = " @p alloc_func and @p free_func @b MUST be both zero or both non-zero. In the"]
    #[doc = " case they are both zero, default memory allocators are used. @p opaque is"]
    #[doc = " passed to @p alloc_func and @p free_func when they are called. @p free_func"]
    #[doc = " has to return without doing anything when asked to free a NULL pointer."]
    #[doc = ""]
    #[doc = " @param type type of dictionary stored in data"]
    #[doc = " @param data_size size of @p data buffer"]
    #[doc = " @param data pointer to the dictionary data"]
    #[doc = " @param quality the maximum Brotli quality to prepare the dictionary for,"]
    #[doc = "        use BROTLI_MAX_QUALITY by default"]
    #[doc = " @param alloc_func custom memory allocation function"]
    #[doc = " @param free_func custom memory free function"]
    #[doc = " @param opaque custom memory manager handle"]
    pub fn BrotliEncoderPrepareDictionary(
        type_: BrotliSharedDictionaryType,
        data_size: usize,
        data: *const u8,
        quality: c_int,
        alloc_func: brotli_alloc_func,
        free_func: brotli_free_func,
        opaque: *mut c_void,
    ) -> *mut BrotliEncoderPreparedDictionary;

    pub fn BrotliEncoderDestroyPreparedDictionary(dictionary: *mut BrotliEncoderPreparedDictionary);

    #[doc = " Attaches a prepared dictionary of any type to the encoder. Can be used"]
    #[doc = " multiple times to attach multiple dictionaries. The dictionary type was"]
    #[doc = " determined by BrotliEncoderPrepareDictionary. Multiple raw prefix"]
    #[doc = " dictionaries and/or max 1 serialized dictionary with custom words can be"]
    #[doc = " attached."]
    #[doc = ""]
    #[doc = " @returns ::BROTLI_FALSE in case of error"]
    #[doc = " @returns ::BROTLI_TRUE otherwise"]
    pub fn BrotliEncoderAttachPreparedDictionary(
        state: *mut BrotliEncoderState,
        dictionary: *const BrotliEncoderPreparedDictionary,
    ) -> BROTLI_BOOL;

    #[doc = " Creates an instance of ::BrotliSharedDictionary."]
    #[doc = ""]
    #[doc = " Fresh instance has default word dictionary and transforms"]
    #[doc = " and no LZ77 prefix dictionary."]
    #[doc = ""]
    #[doc = " @p alloc_func and @p free_func @b MUST be both zero or both non-zero. In the"]
    #[doc = " case they are both zero, default memory allocators are used. @p opaque is"]
    #[doc = " passed to @p alloc_func and @p free_func when they are called. @p free_func"]
    #[doc = " has to return without doing anything when asked to free a NULL pointer."]
    #[doc = ""]
    #[doc = " @param alloc_func custom memory allocation function"]
    #[doc = " @param free_func custom memory free function"]
    #[doc = " @param opaque custom memory manager handle"]
    #[doc = " @returns @c 0 if instance can not be allocated or initialized"]
    #[doc = " @returns pointer to initialized ::BrotliSharedDictionary otherwise"]
    pub fn BrotliSharedDictionaryCreateInstance(
        alloc_func: brotli_alloc_func,
        free_func: brotli_free_func,
        opaque: *mut c_void,
    ) -> *mut BrotliSharedDictionary;

    #[doc = " Deinitializes and frees ::BrotliSharedDictionary instance."]
    #[doc = ""]
    #[doc = " @param dict shared dictionary instance to be cleaned up and deallocated"]
    pub fn BrotliSharedDictionaryDestroyInstance(dict: *mut BrotliSharedDictionary);

    #[doc = " Attaches dictionary to a given instance of ::BrotliSharedDictionary."]
    #[doc = ""]
    #[doc = " Dictionary to be attached is represented in a serialized format as a region"]
    #[doc = " of memory."]
    #[doc = ""]
    #[doc = " Provided data it partially referenced by a resulting (compound) dictionary,"]
    #[doc = " and should be kept untouched, while at least one compound dictionary uses it."]
    #[doc = " This way memory overhead is kept minimal by the cost of additional resource"]
    #[doc = " management."]
    #[doc = ""]
    #[doc = " @param dict dictionary to extend"]
    #[doc = " @param type type of dictionary to attach"]
    #[doc = " @param data_size size of @p data"]
    #[doc = " @param data serialized dictionary of type @p type, with at least @p data_size"]
    #[doc = "        addressable bytes"]
    #[doc = " @returns ::BROTLI_TRUE if provided dictionary is successfully attached"]
    #[doc = " @returns ::BROTLI_FALSE otherwise"]
    pub fn BrotliSharedDictionaryAttach(
        dict: *mut BrotliSharedDictionary,
        type_: BrotliSharedDictionaryType,
        data_size: usize,
        data: *const u8,
    ) -> BROTLI_BOOL;

    #[doc = " Sets the specified parameter to the given encoder instance."]
    #[doc = ""]
    #[doc = " @param state encoder instance"]
    #[doc = " @param param parameter to set"]
    #[doc = " @param value new parameter value"]
    #[doc = " @returns ::BROTLI_FALSE if parameter is unrecognized, or value is invalid"]
    #[doc = " @returns ::BROTLI_FALSE if value of parameter can not be changed at current"]
    #[doc = "          encoder state (e.g. when encoding is started, window size might be"]
    #[doc = "          already encoded and therefore it is impossible to change it)"]
    #[doc = " @returns ::BROTLI_TRUE if value is accepted"]
    #[doc = " @warning invalid values might be accepted in case they would not break"]
    #[doc = "          encoding process."]
    pub fn BrotliEncoderSetParameter(
        state: *mut BrotliEncoderState,
        param: BrotliEncoderParameter,
        value: u32,
    ) -> BROTLI_BOOL;

    #[doc = " Creates an instance of ::BrotliEncoderState and initializes it."]
    #[doc = ""]
    #[doc = " @p alloc_func and @p free_func @b MUST be both zero or both non-zero. In the"]
    #[doc = " case they are both zero, default memory allocators are used. @p opaque is"]
    #[doc = " passed to @p alloc_func and @p free_func when they are called. @p free_func"]
    #[doc = " has to return without doing anything when asked to free a NULL pointer."]
    #[doc = ""]
    #[doc = " @param alloc_func custom memory allocation function"]
    #[doc = " @param free_func custom memory free function"]
    #[doc = " @param opaque custom memory manager handle"]
    #[doc = " @returns @c 0 if instance can not be allocated or initialized"]
    #[doc = " @returns pointer to initialized ::BrotliEncoderState otherwise"]
    pub fn BrotliEncoderCreateInstance(
        alloc_func: brotli_alloc_func,
        free_func: brotli_free_func,
        opaque: *mut c_void,
    ) -> *mut BrotliEncoderState;

    #[doc = " Deinitializes and frees ::BrotliEncoderState instance."]
    #[doc = ""]
    #[doc = " @param state decoder instance to be cleaned up and deallocated"]
    pub fn BrotliEncoderDestroyInstance(state: *mut BrotliEncoderState);

    #[doc = " Calculates the output size bound for the given @p input_size."]
    #[doc = ""]
    #[doc = " @warning Result is only valid if quality is at least @c 2 and, in"]
    #[doc = "          case ::BrotliEncoderCompressStream was used, no flushes"]
    #[doc = "          (::BROTLI_OPERATION_FLUSH) were performed."]
    #[doc = ""]
    #[doc = " @param input_size size of projected input"]
    #[doc = " @returns @c 0 if result does not fit @c size_t"]
    pub fn BrotliEncoderMaxCompressedSize(input_size: usize) -> usize;

    #[doc = " Performs one-shot memory-to-memory compression."]
    #[doc = ""]
    #[doc = " Compresses the data in @p input_buffer into @p encoded_buffer, and sets"]
    #[doc = " @p *encoded_size to the compressed length."]
    #[doc = ""]
    #[doc = " @note If ::BrotliEncoderMaxCompressedSize(@p input_size) returns non-zero"]
    #[doc = "       value, then output is guaranteed to be no longer than that."]
    #[doc = ""]
    #[doc = " @note If @p lgwin is greater than ::BROTLI_MAX_WINDOW_BITS then resulting"]
    #[doc = "       stream might be incompatible with RFC 7932; to decode such streams,"]
    #[doc = "       decoder should be configured with"]
    #[doc = "       ::BROTLI_DECODER_PARAM_LARGE_WINDOW = @c 1"]
    #[doc = ""]
    #[doc = " @param quality quality parameter value, e.g. ::BROTLI_DEFAULT_QUALITY"]
    #[doc = " @param lgwin lgwin parameter value, e.g. ::BROTLI_DEFAULT_WINDOW"]
    #[doc = " @param mode mode parameter value, e.g. ::BROTLI_DEFAULT_MODE"]
    #[doc = " @param input_size size of @p input_buffer"]
    #[doc = " @param input_buffer input data buffer with at least @p input_size"]
    #[doc = "        addressable bytes"]
    #[doc = " @param[in, out] encoded_size @b in: size of @p encoded_buffer; \\n"]
    #[doc = "                 @b out: length of compressed data written to"]
    #[doc = "                 @p encoded_buffer, or @c 0 if compression fails"]
    #[doc = " @param encoded_buffer compressed data destination buffer"]
    #[doc = " @returns ::BROTLI_FALSE in case of compression error"]
    #[doc = " @returns ::BROTLI_FALSE if output buffer is too small"]
    #[doc = " @returns ::BROTLI_TRUE otherwise"]
    pub fn BrotliEncoderCompress(
        quality: c_int,
        lgwin: c_int,
        mode: BrotliEncoderMode,
        input_size: usize,
        input_buffer: *const u8,
        encoded_size: *mut usize,
        encoded_buffer: *mut u8,
    ) -> BROTLI_BOOL;

    #[doc = " Compresses input stream to output stream."]
    #[doc = ""]
    #[doc = " The values @p *available_in and @p *available_out must specify the number of"]
    #[doc = " bytes addressable at @p *next_in and @p *next_out respectively."]
    #[doc = " When @p *available_out is @c 0, @p next_out is allowed to be @c NULL."]
    #[doc = ""]
    #[doc = " After each call, @p *available_in will be decremented by the amount of input"]
    #[doc = " bytes consumed, and the @p *next_in pointer will be incremented by that"]
    #[doc = " amount. Similarly, @p *available_out will be decremented by the amount of"]
    #[doc = " output bytes written, and the @p *next_out pointer will be incremented by"]
    #[doc = " that amount."]
    #[doc = ""]
    #[doc = " @p total_out, if it is not a null-pointer, will be set to the number"]
    #[doc = " of bytes compressed since the last @p state initialization."]
    #[doc = ""]
    #[doc = ""]
    #[doc = ""]
    #[doc = " Internally workflow consists of 3 tasks:"]
    #[doc = "  -# (optionally) copy input data to internal buffer"]
    #[doc = "  -# actually compress data and (optionally) store it to internal buffer"]
    #[doc = "  -# (optionally) copy compressed bytes from internal buffer to output stream"]
    #[doc = ""]
    #[doc = " Whenever all 3 tasks can't move forward anymore, or error occurs, this"]
    #[doc = " method returns the control flow to caller."]
    #[doc = ""]
    #[doc = " @p op is used to perform flush, finish the stream, or inject metadata block."]
    #[doc = " See ::BrotliEncoderOperation for more information."]
    #[doc = ""]
    #[doc = " Flushing the stream means forcing encoding of all input passed to encoder and"]
    #[doc = " completing the current output block, so it could be fully decoded by stream"]
    #[doc = " decoder. To perform flush set @p op to ::BROTLI_OPERATION_FLUSH."]
    #[doc = " Under some circumstances (e.g. lack of output stream capacity) this operation"]
    #[doc = " would require several calls to ::BrotliEncoderCompressStream. The method must"]
    #[doc = " be called again until both input stream is depleted and encoder has no more"]
    #[doc = " output (see ::BrotliEncoderHasMoreOutput) after the method is called."]
    #[doc = ""]
    #[doc = " Finishing the stream means encoding of all input passed to encoder and"]
    #[doc = " adding specific \"final\" marks, so stream decoder could determine that stream"]
    #[doc = " is complete. To perform finish set @p op to ::BROTLI_OPERATION_FINISH."]
    #[doc = " Under some circumstances (e.g. lack of output stream capacity) this operation"]
    #[doc = " would require several calls to ::BrotliEncoderCompressStream. The method must"]
    #[doc = " be called again until both input stream is depleted and encoder has no more"]
    #[doc = " output (see ::BrotliEncoderHasMoreOutput) after the method is called."]
    #[doc = ""]
    #[doc = " @warning When flushing and finishing, @p op should not change until operation"]
    #[doc = "          is complete; input stream should not be swapped, reduced or"]
    #[doc = "          extended as well."]
    #[doc = ""]
    #[doc = " @param state encoder instance"]
    #[doc = " @param op requested operation"]
    #[doc = " @param[in, out] available_in @b in: amount of available input; \\n"]
    #[doc = "                 @b out: amount of unused input"]
    #[doc = " @param[in, out] next_in pointer to the next input byte"]
    #[doc = " @param[in, out] available_out @b in: length of output buffer; \\n"]
    #[doc = "                 @b out: remaining size of output buffer"]
    #[doc = " @param[in, out] next_out compressed output buffer cursor;"]
    #[doc = "                 can be @c NULL if @p available_out is @c 0"]
    #[doc = " @param[out] total_out number of bytes produced so far; can be @c NULL"]
    #[doc = " @returns ::BROTLI_FALSE if there was an error"]
    #[doc = " @returns ::BROTLI_TRUE otherwise"]
    pub fn BrotliEncoderCompressStream(
        state: *mut BrotliEncoderState,
        op: BrotliEncoderOperation,
        available_in: *mut usize,
        next_in: *mut *const u8,
        available_out: *mut usize,
        next_out: *mut *mut u8,
        total_out: *mut usize,
    ) -> BROTLI_BOOL;

    #[doc = " Checks if encoder instance reached the final state."]
    #[doc = ""]
    #[doc = " @param state encoder instance"]
    #[doc = " @returns ::BROTLI_TRUE if encoder is in a state where it reached the end of"]
    #[doc = "          the input and produced all of the output"]
    #[doc = " @returns ::BROTLI_FALSE otherwise"]
    pub fn BrotliEncoderIsFinished(state: *mut BrotliEncoderState) -> BROTLI_BOOL;

    #[doc = " Checks if encoder has more output."]
    #[doc = ""]
    #[doc = " @param state encoder instance"]
    #[doc = " @returns ::BROTLI_TRUE, if encoder has some unconsumed output"]
    #[doc = " @returns ::BROTLI_FALSE otherwise"]
    pub fn BrotliEncoderHasMoreOutput(state: *mut BrotliEncoderState) -> BROTLI_BOOL;

    #[doc = " Acquires pointer to internal output buffer."]
    #[doc = ""]
    #[doc = " This method is used to make language bindings easier and more efficient:"]
    #[doc = "  -# push data to ::BrotliEncoderCompressStream,"]
    #[doc = "     until ::BrotliEncoderHasMoreOutput returns BROTL_TRUE"]
    #[doc = "  -# use ::BrotliEncoderTakeOutput to peek bytes and copy to language-specific"]
    #[doc = "     entity"]
    #[doc = ""]
    #[doc = " Also this could be useful if there is an output stream that is able to"]
    #[doc = " consume all the provided data (e.g. when data is saved to file system)."]
    #[doc = ""]
    #[doc = " @attention After every call to ::BrotliEncoderTakeOutput @p *size bytes of"]
    #[doc = "            output are considered consumed for all consecutive calls to the"]
    #[doc = "            instance methods; returned pointer becomes invalidated as well."]
    #[doc = ""]
    #[doc = " @note Encoder output is not guaranteed to be contiguous. This means that"]
    #[doc = "       after the size-unrestricted call to ::BrotliEncoderTakeOutput,"]
    #[doc = "       immediate next call to ::BrotliEncoderTakeOutput may return more data."]
    #[doc = ""]
    #[doc = " @param state encoder instance"]
    #[doc = " @param[in, out] size @b in: number of bytes caller is ready to take, @c 0 if"]
    #[doc = "                 any amount could be handled; \\n"]
    #[doc = "                 @b out: amount of data pointed by returned pointer and"]
    #[doc = "                 considered consumed; \\n"]
    #[doc = "                 out value is never greater than in value, unless it is @c 0"]
    #[doc = " @returns pointer to output data"]
    pub fn BrotliEncoderTakeOutput(state: *const BrotliEncoderState, size: *mut usize)
    -> *const u8;

    pub fn BrotliEncoderEstimatePeakMemoryUsage(
        quality: c_int,
        lgwin: c_int,
        input_size: usize,
    ) -> usize;

    pub fn BrotliEncoderGetPreparedDictionarySize(
        dictionary: *const BrotliEncoderPreparedDictionary,
    ) -> usize;

    #[doc = " Gets an encoder library version."]
    #[doc = ""]
    #[doc = " Look at BROTLI_VERSION for more information."]
    pub fn BrotliEncoderVersion() -> u32;

    #[doc = " Sets the specified parameter to the given decoder instance."]
    #[doc = ""]
    #[doc = " @param state decoder instance"]
    #[doc = " @param param parameter to set"]
    #[doc = " @param value new parameter value"]
    #[doc = " @returns ::BROTLI_FALSE if parameter is unrecognized, or value is invalid"]
    #[doc = " @returns ::BROTLI_TRUE if value is accepted"]
    pub fn BrotliDecoderSetParameter(
        state: *mut BrotliDecoderState,
        param: BrotliDecoderParameter,
        value: u32,
    ) -> BROTLI_BOOL;

    #[doc = " Adds LZ77 prefix dictionary, adds or replaces built-in static dictionary and"]
    #[doc = " transforms."]
    #[doc = ""]
    #[doc = " Attached dictionary ownership is not transferred."]
    #[doc = " Data provided to this method should be kept accessible until"]
    #[doc = " decoding is finished and decoder instance is destroyed."]
    #[doc = ""]
    #[doc = " @note Dictionaries can NOT be attached after actual decoding is started."]
    #[doc = ""]
    #[doc = " @param state decoder instance"]
    #[doc = " @param type dictionary data format"]
    #[doc = " @param data_size length of memory region pointed by @p data"]
    #[doc = " @param data dictionary data in format corresponding to @p type"]
    #[doc = " @returns ::BROTLI_FALSE if dictionary is corrupted,"]
    #[doc = "          or dictionary count limit is reached"]
    #[doc = " @returns ::BROTLI_TRUE if dictionary is accepted / attached"]
    pub fn BrotliDecoderAttachDictionary(
        state: *mut BrotliDecoderState,
        type_: BrotliSharedDictionaryType,
        data_size: usize,
        data: *const u8,
    ) -> BROTLI_BOOL;

    #[doc = " Creates an instance of ::BrotliDecoderState and initializes it."]
    #[doc = ""]
    #[doc = " The instance can be used once for decoding and should then be destroyed with"]
    #[doc = " ::BrotliDecoderDestroyInstance, it cannot be reused for a new decoding"]
    #[doc = " session."]
    #[doc = ""]
    #[doc = " @p alloc_func and @p free_func @b MUST be both zero or both non-zero. In the"]
    #[doc = " case they are both zero, default memory allocators are used. @p opaque is"]
    #[doc = " passed to @p alloc_func and @p free_func when they are called. @p free_func"]
    #[doc = " has to return without doing anything when asked to free a NULL pointer."]
    #[doc = ""]
    #[doc = " @param alloc_func custom memory allocation function"]
    #[doc = " @param free_func custom memory free function"]
    #[doc = " @param opaque custom memory manager handle"]
    #[doc = " @returns @c 0 if instance can not be allocated or initialized"]
    #[doc = " @returns pointer to initialized ::BrotliDecoderState otherwise"]
    pub fn BrotliDecoderCreateInstance(
        alloc_func: brotli_alloc_func,
        free_func: brotli_free_func,
        opaque: *mut ::std::os::raw::c_void,
    ) -> *mut BrotliDecoderState;

    #[doc = " Deinitializes and frees ::BrotliDecoderState instance."]
    #[doc = ""]
    #[doc = " @param state decoder instance to be cleaned up and deallocated"]
    pub fn BrotliDecoderDestroyInstance(state: *mut BrotliDecoderState);

    #[doc = " Performs one-shot memory-to-memory decompression."]
    #[doc = ""]
    #[doc = " Decompresses the data in @p encoded_buffer into @p decoded_buffer, and sets"]
    #[doc = " @p *decoded_size to the decompressed length."]
    #[doc = ""]
    #[doc = " @param encoded_size size of @p encoded_buffer"]
    #[doc = " @param encoded_buffer compressed data buffer with at least @p encoded_size"]
    #[doc = "        addressable bytes"]
    #[doc = " @param[in, out] decoded_size @b in: size of @p decoded_buffer; \\n"]
    #[doc = "                 @b out: length of decompressed data written to"]
    #[doc = "                 @p decoded_buffer"]
    #[doc = " @param decoded_buffer decompressed data destination buffer"]
    #[doc = " @returns ::BROTLI_DECODER_RESULT_ERROR if input is corrupted, memory"]
    #[doc = "          allocation failed, or @p decoded_buffer is not large enough;"]
    #[doc = " @returns ::BROTLI_DECODER_RESULT_SUCCESS otherwise"]
    pub fn BrotliDecoderDecompress(
        encoded_size: usize,
        encoded_buffer: *const u8,
        decoded_size: *mut usize,
        decoded_buffer: *mut u8,
    ) -> BrotliDecoderResult;

    #[doc = " Decompresses the input stream to the output stream."]
    #[doc = ""]
    #[doc = " The values @p *available_in and @p *available_out must specify the number of"]
    #[doc = " bytes addressable at @p *next_in and @p *next_out respectively."]
    #[doc = " When @p *available_out is @c 0, @p next_out is allowed to be @c NULL."]
    #[doc = ""]
    #[doc = " After each call, @p *available_in will be decremented by the amount of input"]
    #[doc = " bytes consumed, and the @p *next_in pointer will be incremented by that"]
    #[doc = " amount. Similarly, @p *available_out will be decremented by the amount of"]
    #[doc = " output bytes written, and the @p *next_out pointer will be incremented by"]
    #[doc = " that amount."]
    #[doc = ""]
    #[doc = " @p total_out, if it is not a null-pointer, will be set to the number"]
    #[doc = " of bytes decompressed since the last @p state initialization."]
    #[doc = ""]
    #[doc = " @note Input is never overconsumed, so @p next_in and @p available_in could be"]
    #[doc = " passed to the next consumer after decoding is complete."]
    #[doc = ""]
    #[doc = " @param state decoder instance"]
    #[doc = " @param[in, out] available_in @b in: amount of available input; \\n"]
    #[doc = "                 @b out: amount of unused input"]
    #[doc = " @param[in, out] next_in pointer to the next compressed byte"]
    #[doc = " @param[in, out] available_out @b in: length of output buffer; \\n"]
    #[doc = "                 @b out: remaining size of output buffer"]
    #[doc = " @param[in, out] next_out output buffer cursor;"]
    #[doc = "                 can be @c NULL if @p available_out is @c 0"]
    #[doc = " @param[out] total_out number of bytes decompressed so far; can be @c NULL"]
    #[doc = " @returns ::BROTLI_DECODER_RESULT_ERROR if input is corrupted, memory"]
    #[doc = "          allocation failed, arguments were invalid, etc.;"]
    #[doc = "          use ::BrotliDecoderGetErrorCode to get detailed error code"]
    #[doc = " @returns ::BROTLI_DECODER_RESULT_NEEDS_MORE_INPUT decoding is blocked until"]
    #[doc = "          more input data is provided"]
    #[doc = " @returns ::BROTLI_DECODER_RESULT_NEEDS_MORE_OUTPUT decoding is blocked until"]
    #[doc = "          more output space is provided"]
    #[doc = " @returns ::BROTLI_DECODER_RESULT_SUCCESS decoding is finished, no more"]
    #[doc = "          input might be consumed and no more output will be produced"]
    pub fn BrotliDecoderDecompressStream(
        state: *mut BrotliDecoderState,
        available_in: *mut usize,
        next_in: *mut *const u8,
        available_out: *mut usize,
        next_out: *mut *mut u8,
        total_out: *mut usize,
    ) -> BrotliDecoderResult;

    #[doc = " Checks if decoder has more output."]
    #[doc = ""]
    #[doc = " @param state decoder instance"]
    #[doc = " @returns ::BROTLI_TRUE, if decoder has some unconsumed output"]
    #[doc = " @returns ::BROTLI_FALSE otherwise"]
    pub fn BrotliDecoderHasMoreOutput(state: *const BrotliDecoderState) -> BROTLI_BOOL;

    #[doc = " Acquires pointer to internal output buffer."]
    #[doc = ""]
    #[doc = " This method is used to make language bindings easier and more efficient:"]
    #[doc = "  -# push data to ::BrotliDecoderDecompressStream,"]
    #[doc = "     until ::BROTLI_DECODER_RESULT_NEEDS_MORE_OUTPUT is reported"]
    #[doc = "  -# use ::BrotliDecoderTakeOutput to peek bytes and copy to language-specific"]
    #[doc = "     entity"]
    #[doc = ""]
    #[doc = " Also this could be useful if there is an output stream that is able to"]
    #[doc = " consume all the provided data (e.g. when data is saved to file system)."]
    #[doc = ""]
    #[doc = " @attention After every call to ::BrotliDecoderTakeOutput @p *size bytes of"]
    #[doc = "            output are considered consumed for all consecutive calls to the"]
    #[doc = "            instance methods; returned pointer becomes invalidated as well."]
    #[doc = ""]
    #[doc = " @note Decoder output is not guaranteed to be contiguous. This means that"]
    #[doc = "       after the size-unrestricted call to ::BrotliDecoderTakeOutput,"]
    #[doc = "       immediate next call to ::BrotliDecoderTakeOutput may return more data."]
    #[doc = ""]
    #[doc = " @param state decoder instance"]
    #[doc = " @param[in, out] size @b in: number of bytes caller is ready to take, @c 0 if"]
    #[doc = "                 any amount could be handled; \\n"]
    #[doc = "                 @b out: amount of data pointed by returned pointer and"]
    #[doc = "                 considered consumed; \\n"]
    #[doc = "                 out value is never greater than in value, unless it is @c 0"]
    #[doc = " @returns pointer to output data"]
    pub fn BrotliDecoderTakeOutput(state: *mut BrotliDecoderState, size: *mut usize) -> *const u8;

    #[doc = " Checks if instance has already consumed input."]
    #[doc = ""]
    #[doc = " Instance that returns ::BROTLI_FALSE is considered \"fresh\" and could be"]
    #[doc = " reused."]
    #[doc = ""]
    #[doc = " @param state decoder instance"]
    #[doc = " @returns ::BROTLI_TRUE if decoder has already used some input bytes"]
    #[doc = " @returns ::BROTLI_FALSE otherwise"]
    pub fn BrotliDecoderIsUsed(state: *const BrotliDecoderState) -> BROTLI_BOOL;

    #[doc = " Checks if decoder instance reached the final state."]
    #[doc = ""]
    #[doc = " @param state decoder instance"]
    #[doc = " @returns ::BROTLI_TRUE if decoder is in a state where it reached the end of"]
    #[doc = "          the input and produced all of the output"]
    #[doc = " @returns ::BROTLI_FALSE otherwise"]
    pub fn BrotliDecoderIsFinished(state: *const BrotliDecoderState) -> BROTLI_BOOL;

    #[doc = " Acquires a detailed error code."]
    #[doc = ""]
    #[doc = " Should be used only after ::BrotliDecoderDecompressStream returns"]
    #[doc = " ::BROTLI_DECODER_RESULT_ERROR."]
    #[doc = ""]
    #[doc = " See also ::BrotliDecoderErrorString"]
    #[doc = ""]
    #[doc = " @param state decoder instance"]
    #[doc = " @returns last saved error code"]
    pub fn BrotliDecoderGetErrorCode(state: *const BrotliDecoderState) -> BrotliDecoderErrorCode;

    #[doc = " Converts error code to a c-string."]
    pub fn BrotliDecoderErrorString(c: BrotliDecoderErrorCode) -> *const c_char;

    #[doc = " Gets a decoder library version."]
    #[doc = ""]
    #[doc = " Look at BROTLI_VERSION for more information."]
    pub fn BrotliDecoderVersion() -> u32;
}
