use rand::{Rng, SeedableRng};

pub fn gen_min_entropy(len: usize) -> Vec<u8> {
    vec![0; len]
}

pub fn gen_medium_entropy(len: usize) -> Vec<u8> {
    let mut res = Vec::with_capacity(len);
    let mut rng = rand_pcg::Pcg32::seed_from_u64(0);

    res.resize_with(len, || rng.gen_range(0..128));
    res
}

pub fn gen_max_entropy(len: usize) -> Vec<u8> {
    let mut res = vec![0; len];
    let mut rng = rand_pcg::Pcg32::seed_from_u64(0);

    rng.fill(res.as_mut_slice());
    res
}
