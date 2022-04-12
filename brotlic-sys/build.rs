use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let include_dir = manifest_dir.join("brotli/c/include");

    cc::Build::new()
        .files(&[
            "brotli/c/common/constants.c",
            "brotli/c/common/context.c",
            "brotli/c/common/dictionary.c",
            "brotli/c/common/platform.c",
            "brotli/c/common/shared_dictionary.c",
            "brotli/c/common/transform.c",
            "brotli/c/dec/bit_reader.c",
            "brotli/c/dec/decode.c",
            "brotli/c/dec/huffman.c",
            "brotli/c/dec/state.c",
            "brotli/c/enc/backward_references.c",
            "brotli/c/enc/backward_references_hq.c",
            "brotli/c/enc/bit_cost.c",
            "brotli/c/enc/block_splitter.c",
            "brotli/c/enc/brotli_bit_stream.c",
            "brotli/c/enc/cluster.c",
            "brotli/c/enc/command.c",
            "brotli/c/enc/compound_dictionary.c",
            "brotli/c/enc/compress_fragment.c",
            "brotli/c/enc/compress_fragment_two_pass.c",
            "brotli/c/enc/dictionary_hash.c",
            "brotli/c/enc/encode.c",
            "brotli/c/enc/encoder_dict.c",
            "brotli/c/enc/entropy_encode.c",
            "brotli/c/enc/fast_log.c",
            "brotli/c/enc/histogram.c",
            "brotli/c/enc/literal_cost.c",
            "brotli/c/enc/memory.c",
            "brotli/c/enc/metablock.c",
            "brotli/c/enc/static_dict.c",
            "brotli/c/enc/utf8_util.c",
        ])
        .include("brotli/c/include")
        .define("BROTLI_BUILD_ENC_EXTRA_API", None)
        .warnings(false)
        .compile("brotli");

    println!("cargo:include={}", include_dir.display());
    println!("cargo:rerun-if-changed=brotli/c");
}
