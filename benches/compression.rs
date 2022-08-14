use std::io::Write;
use std::iter;

use brotlic::{BrotliEncoderOptions, Quality, WindowSize};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::{Rng, RngCore, SeedableRng};
use rand_pcg::Pcg32;

fn brotli_compress(input: &[u8]) -> Vec<u8> {
    let mut compressor =
        { brotli::CompressorWriter::new(Vec::with_capacity(input.len()), 4096, 11, 24) };

    compressor.write_all(input).unwrap();
    compressor.into_inner()
}

fn brotlic_compress(input: &[u8]) -> Vec<u8> {
    let encoder = BrotliEncoderOptions::new()
        .quality(Quality::new(11).unwrap())
        .window_size(WindowSize::new(24).unwrap())
        .build()
        .unwrap();

    let mut compressor =
        { brotlic::CompressorWriter::with_encoder(encoder, Vec::with_capacity(input.len())) };

    compressor.write_all(input).unwrap();
    compressor.into_inner().unwrap()
}

pub fn bench(c: &mut Criterion) {
    bench_entropy(c, "min_entropy", gen_min_entropy);
    bench_entropy(c, "low_entropy", gen_low_entropy);
    bench_entropy(c, "medium_entropy", gen_medium_entropy);
    bench_entropy(c, "high_entropy", gen_high_entropy);
    bench_entropy(c, "max_entropy", gen_max_entropy);
}

pub fn bench_entropy(c: &mut Criterion, name: &str, entropy_source: fn(usize) -> Vec<u8>) {
    let input_sizes = { iter::successors(Some(1usize << 5), |x| (*x).checked_shl(5)) };

    let mut group = c.benchmark_group(name);

    for input_size in input_sizes.take(4) {
        let input = entropy_source(input_size);

        {
            let brotli = brotli_compress(&input);
            let brotlic = brotlic_compress(&input);

            assert_eq!(brotli, brotlic);
        }

        group.throughput(Throughput::Bytes(input_size as u64));
        group.bench_with_input(
            BenchmarkId::new("brotli", input_size),
            &input_size,
            |b, &_size| {
                b.iter(|| brotli_compress(&input));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("brotlic", input_size),
            &input_size,
            |b, &_size| {
                b.iter(|| brotlic_compress(&input));
            },
        );
    }
}

fn gen_min_entropy(len: usize) -> Vec<u8> {
    vec![0; len]
}

fn gen_low_entropy(len: usize) -> Vec<u8> {
    let mut res = Vec::with_capacity(len);
    let mut rng = Pcg32::seed_from_u64(len as u64);
    res.resize_with(len, || rng.gen_range(0..64));
    res
}

fn gen_medium_entropy(len: usize) -> Vec<u8> {
    let mut res = Vec::with_capacity(len);
    let mut rng = Pcg32::seed_from_u64(len as u64);
    res.resize_with(len, || rng.gen_range(0..128));
    res
}

fn gen_high_entropy(len: usize) -> Vec<u8> {
    let mut res = Vec::with_capacity(len);
    let mut rng = Pcg32::seed_from_u64(len as u64);
    res.resize_with(len, || rng.gen_range(0..192));
    res
}

fn gen_max_entropy(len: usize) -> Vec<u8> {
    let mut res = vec![0; len];
    let mut rng = Pcg32::seed_from_u64(len as u64);
    rng.fill_bytes(res.as_mut_slice());
    res
}

criterion_group!(benches, bench);
criterion_main!(benches);
