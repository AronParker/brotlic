use brotlic::{CompressionMode, LargeWindowSize, Quality, WindowSize};

mod common;

fn verify(input: &[u8]) {
    let quality = Quality::best();
    let window_size = WindowSize::best();
    let mode = CompressionMode::Generic;

    let bound = brotlic::compress_bound(input.len(), quality).unwrap();
    let compressed = {
        let mut buf = vec![0; bound];
        let size =
            brotlic::compress(input, buf.as_mut_slice(), quality, window_size, mode).unwrap();

        buf.truncate(size);
        buf
    };

    let decompressed = {
        let mut buf = vec![0; input.len()];
        let size = brotlic::decompress(compressed.as_slice(), buf.as_mut_slice()).unwrap();

        buf.truncate(size);
        buf
    };

    assert_eq!(input, decompressed);
}

#[test]
fn test_min_entropy_small() {
    verify(common::gen_min_entropy(32).as_slice());
}

#[test]
fn test_medium_entropy_small() {
    verify(common::gen_medium_entropy(32).as_slice());
}

#[test]
fn test_max_entropy_small() {
    verify(common::gen_max_entropy(32).as_slice());
}

#[test]
fn test_min_entropy_medium() {
    verify(common::gen_min_entropy(512).as_slice());
}

#[test]
fn test_medium_entropy_medium() {
    verify(common::gen_medium_entropy(512).as_slice());
}

#[test]
fn test_max_entropy_medium() {
    verify(common::gen_max_entropy(512).as_slice());
}

#[test]
fn test_min_entropy_large() {
    verify(common::gen_min_entropy(8192).as_slice());
}

#[test]
fn test_medium_entropy_large() {
    verify(common::gen_medium_entropy(8192).as_slice());
}

#[test]
fn test_max_entropy_large() {
    verify(common::gen_max_entropy(8192).as_slice());
}

#[test]
fn test_encoder_estimate_peak_memory_usage() {
    let usage100 =
        brotlic::compress_estimate_max_mem_usage(100, Quality::best(), WindowSize::best());

    assert!(usage100 > 0);
}

#[test]
fn test_google_brotli_issue_1001() {
    let window_size =
        brotlic::compress_estimate_max_mem_usage(1024 * 1024, Quality::best(), WindowSize::best());
    let large_window_size = brotlic::compress_estimate_max_mem_usage(
        1024 * 1024,
        Quality::best(),
        LargeWindowSize::best(),
    );

    assert!(large_window_size > window_size);
}
