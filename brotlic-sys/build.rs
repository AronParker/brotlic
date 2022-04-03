use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mut build = cc::Build::new();

    build
        .files(&[
            "brotli/common/constants.c",
            "brotli/common/context.c",
            "brotli/common/dictionary.c",
            "brotli/common/platform.c",
            "brotli/common/shared_dictionary.c",
            "brotli/common/transform.c",
            "brotli/dec/bit_reader.c",
            "brotli/dec/decode.c",
            "brotli/dec/huffman.c",
            "brotli/dec/state.c",
            "brotli/enc/backward_references.c",
            "brotli/enc/backward_references_hq.c",
            "brotli/enc/bit_cost.c",
            "brotli/enc/block_splitter.c",
            "brotli/enc/brotli_bit_stream.c",
            "brotli/enc/cluster.c",
            "brotli/enc/command.c",
            "brotli/enc/compound_dictionary.c",
            "brotli/enc/compress_fragment.c",
            "brotli/enc/compress_fragment_two_pass.c",
            "brotli/enc/dictionary_hash.c",
            "brotli/enc/encode.c",
            "brotli/enc/encoder_dict.c",
            "brotli/enc/entropy_encode.c",
            "brotli/enc/fast_log.c",
            "brotli/enc/histogram.c",
            "brotli/enc/literal_cost.c",
            "brotli/enc/memory.c",
            "brotli/enc/metablock.c",
            "brotli/enc/static_dict.c",
            "brotli/enc/utf8_util.c",
        ])
        .include("brotli/include")
        .define("BROTLI_BUILD_ENC_EXTRA_API", None)
        .warnings(false)
        .out_dir(out_dir.join("lib"));

    build.compile("brotli");

    let src_include = Path::new("brotli/include/brotli");
    let dst_include = out_dir.join("include");

    fs::create_dir_all(&dst_include).unwrap();
    fs::copy(src_include.join("decode.h"), dst_include.join("decode.h")).unwrap();
    fs::copy(src_include.join("encode.h"), dst_include.join("encode.h")).unwrap();
    fs::copy(src_include.join("port.h"), dst_include.join("port.h")).unwrap();
    fs::copy(
        src_include.join("shared_dictionary.h"),
        dst_include.join("shared_dictionary.h"),
    )
    .unwrap();
    fs::copy(src_include.join("types.h"), dst_include.join("types.h")).unwrap();

    println!("cargo:root={}", out_dir.display());
    println!("cargo:include={}", dst_include.display());
}
