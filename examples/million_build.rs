use minimal_perfect_hash::{BuildConfig, Builder, MphError};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use std::collections::HashSet;
use std::time::Instant;

const N_KEYS: usize = 1_000_000;
const GEN_SEED: u64 = 42;

fn main() -> Result<(), MphError> {
    println!("--- minimal_perfect_hash test ---");
    println!("n = {N_KEYS}");

    // 1) Generate unique keys
    let t0 = Instant::now();
    let keys = gen_unique_keys(N_KEYS, GEN_SEED);
    let gen_s = t0.elapsed().as_secs_f64();
    println!(
        "gen:    {:>8.3} s   ({:.1} M keys/s)",
        gen_s,
        N_KEYS as f64 / gen_s / 1e6
    );

    // 2) Pre-hashing (BDZ vertices) — time measurement
    //    Note: the builder will still perform hashing itself
    // Use the same config as for build: salt matters for vertex derivation
    let cfg = BuildConfig {
        // For stable build on 1M keys:
        // gamma can be varied between 1.23..1.30
        gamma: 1.25,
        rehash_limit: 32,
        ..Default::default()
    };
    let eff_salt = cfg.salt; // only for measurement; builder may shift salt during rehash

    let t1 = Instant::now();
    let _prehashed = prehash_vertices(&keys, eff_salt, (cfg.gamma * N_KEYS as f64).ceil() as u64);
    let hash_s = t1.elapsed().as_secs_f64();
    println!(
        "hash:   {:>8.3} s   ({:.1} M keys/s)",
        hash_s,
        N_KEYS as f64 / hash_s / 1e6
    );

    // 3) Build MPH
    let t2 = Instant::now();
    let mph = Builder::new()
        .with_config(cfg)
        .build(keys.iter().map(|v| v.as_slice()))?;
    let build_s = t2.elapsed().as_secs_f64();
    println!(
        "build:  {:>8.3} s   ({:.1} M keys/s)",
        build_s,
        N_KEYS as f64 / build_s / 1e6
    );

    // 4) Lookup all keys
    let t3 = Instant::now();
    // Split into chunks to avoid compiler removing the loop and to avoid cache overheating
    let mut acc: u64 = 0;
    for chunk in keys.chunks(32_768) {
        for k in chunk {
            acc ^= mph.index(k);
        }
    }
    let lookup_s = t3.elapsed().as_secs_f64();
    println!(
        "lookup: {:>8.3} s   ({:.1} M lookups/s)   (acc={acc})",
        lookup_s,
        N_KEYS as f64 / lookup_s / 1e6
    );

    println!("----------------------------------------------");
    println!(
        "Total (gen + hash + build + lookup): {:.3} s",
        gen_s + hash_s + build_s + lookup_s
    );

    Ok(())
}

/// Generate N unique 16-byte keys (raw bytes), deterministically.
fn gen_unique_keys(n: usize, seed: u64) -> Vec<Vec<u8>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut set = HashSet::with_capacity(n * 2);
    let mut keys = Vec::with_capacity(n);
    while keys.len() < n {
        let mut buf = [0u8; 16];
        rng.fill_bytes(&mut buf);
        if set.insert(buf) {
            keys.push(buf.to_vec());
        }
    }
    keys
}

/// Precompute vertices (only for profiling hashing time).
/// This reproduces exactly the same formula as inside BDZ:
/// three independent XXH3 with different seeds, then `% m`.
fn prehash_vertices(keys: &[Vec<u8>], salt: u64, m: u64) -> Vec<(u32, u32, u32)> {
    use xxhash_rust::xxh3::xxh3_64_with_seed;

    // Seeds must match those used by the library
    #[inline]
    fn verts(key: &[u8], salt: u64, m: u64) -> (u32, u32, u32) {
        let s1 = salt ^ 0x9E37_79B9_7F4A_7C15;
        let s2 = salt.wrapping_mul(0xA24B_1F6F);
        let s3 = salt ^ 0x853C_49E6_0A6C_9D39;
        let a = xxh3_64_with_seed(key, s1) % m;
        let b = xxh3_64_with_seed(key, s2) % m;
        let c = xxh3_64_with_seed(key, s3) % m;
        (a as u32, b as u32, c as u32)
    }

    let mut out = Vec::with_capacity(keys.len());
    for k in keys {
        out.push(verts(k, salt, m));
    }
    out
}
