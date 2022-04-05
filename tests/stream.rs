use std::io::{Read, Write};
use brotlic::{CompressorReader, CompressorWriter, DecompressorReader, DecompressorWriter};

mod common;

fn write_comp_read_decomp_verify(input: &[u8]) {
    let compressed = {
        let mut compressor = CompressorWriter::new(Vec::new());
        compressor.write_all(input).unwrap();
        compressor.into_inner().unwrap()
    };

    let decompressed = {
        let mut decompressor = DecompressorReader::new(compressed.as_slice());
        let mut decompressed = Vec::new();
        decompressor.read_to_end(&mut decompressed).unwrap();
        decompressed
    };

    assert_eq!(input, decompressed);
}

fn read_comp_write_decomp_verify(input: &[u8]) {
    let compressed = {
        let mut compressor = CompressorReader::new(input);
        let mut compressed = Vec::new();
        compressor.read_to_end(&mut compressed).unwrap();
        compressed
    };

    let decompressed = {
        let mut decompressor = DecompressorWriter::new(Vec::new());
        decompressor.write_all(compressed.as_slice()).unwrap();
        decompressor.into_inner().unwrap()
    };

    assert_eq!(input, decompressed);
}

#[test]
fn test_write_comp_min_entropy_small() {
    write_comp_read_decomp_verify(common::gen_min_entropy(32).as_slice());
}

#[test]
fn test_write_comp_medium_entropy_small() {
    write_comp_read_decomp_verify(common::gen_medium_entropy(32).as_slice());
}

#[test]
fn test_write_comp_max_entropy_small() {
    write_comp_read_decomp_verify(common::gen_max_entropy(32).as_slice());
}

#[test]
fn test_write_comp_min_entropy_medium() {
    write_comp_read_decomp_verify(common::gen_min_entropy(512).as_slice());
}

#[test]
fn test_write_comp_medium_entropy_medium() {
    write_comp_read_decomp_verify(common::gen_medium_entropy(512).as_slice());
}

#[test]
fn test_write_comp_max_entropy_medium() {
    write_comp_read_decomp_verify(common::gen_max_entropy(512).as_slice());
}

#[test]
fn test_write_comp_min_entropy_large() {
    write_comp_read_decomp_verify(common::gen_min_entropy(8192).as_slice());
}

#[test]
fn test_write_comp_medium_entropy_large() {
    write_comp_read_decomp_verify(common::gen_medium_entropy(8192).as_slice());
}

#[test]
fn test_write_comp_max_entropy_large() {
    write_comp_read_decomp_verify(common::gen_max_entropy(8192).as_slice());
}

#[test]
fn test_read_comp_min_entropy_small() {
    read_comp_write_decomp_verify(common::gen_min_entropy(32).as_slice());
}

#[test]
fn test_read_comp_medium_entropy_small() {
    read_comp_write_decomp_verify(common::gen_medium_entropy(32).as_slice());
}

#[test]
fn test_read_comp_max_entropy_small() {
    read_comp_write_decomp_verify(common::gen_max_entropy(32).as_slice());
}

#[test]
fn test_read_comp_min_entropy_medium() {
    read_comp_write_decomp_verify(common::gen_min_entropy(512).as_slice());
}

#[test]
fn test_read_comp_medium_entropy_medium() {
    read_comp_write_decomp_verify(common::gen_medium_entropy(512).as_slice());
}

#[test]
fn test_read_comp_max_entropy_medium() {
    read_comp_write_decomp_verify(common::gen_max_entropy(512).as_slice());
}

#[test]
fn test_read_comp_min_entropy_large() {
    read_comp_write_decomp_verify(common::gen_min_entropy(8192).as_slice());
}

#[test]
fn test_read_comp_medium_entropy_large() {
    read_comp_write_decomp_verify(common::gen_medium_entropy(8192).as_slice());
}

#[test]
fn test_read_comp_max_entropy_large() {
    read_comp_write_decomp_verify(common::gen_max_entropy(8192).as_slice());
}
