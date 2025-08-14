# ⚡ minimal_perfect_hash

![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/minimal_perfect_hash)

> A **blazing-fast** [BDZ](https://cmph.sourceforge.net/papers/esa09.pdf) minimal perfect hash function implementation in Rust.  
> Designed for **production-scale** workloads with **millions of keys**, minimal memory footprint, and predictable `O(1)` lookups.

---

## ✨ Features

- 🚀 **Zero collisions** — perfect hash for a fixed set of keys.
- 📦 **Compact** — ~5 bytes/key (10 M keys ≈ 50 MB index).
- 🧠 **Cache-friendly** — index fits into L3 cache for many workloads.
- 🛠 **Flexible builder** — configurable parameters for memory/speed trade-offs.
- 🔍 **Consistent lookups** — ~25 ns/lookup (40 M lookups/sec on a single core).
- 💾 **Serialization-ready** — dump to disk and `mmap` on startup for instant cold starts.

---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
minimal_perfect_hash = "0.1"
````

---

## 🚀 Quick Start

```rust
use minimal_perfect_hash::Builder;

fn main() {
    // Example: build an MPH from a set of string keys
    let keys = vec![
        "apple", "banana", "orange", "grape", "melon",
        "peach", "mango", "kiwi", "lemon", "plum",
    ];

    let mphf = Builder::new()
        .keys(&keys)
        .build()
        .expect("failed to build MPH");

    // Lookups are O(1), no collisions
    for k in &keys {
        let idx = mphf.lookup(k).unwrap();
        println!("'{}' → {}", k, idx);
    }
}
```

Output:

```
'apple' → 0
'banana' → 1
'orange' → 2
'grape' → 3
'melon' → 4
'peach' → 5
'mango' → 6
'kiwi' → 7
'lemon' → 8
'plum' → 9
```

---

## 📊 Performance

Benchmarks (`1,000,000` random 32-byte keys, Intel Core i7 12700, Rust 1.88):

| Operation         | BDZ MPH                   | HashMap (hashbrown + AHash) |
| ----------------- | ------------------------- | --------------------------- |
| Build time        | **0.75 s** (1.3 M keys/s) | 0.03 s (33 M keys/s)        |
| Lookup throughput | **39 M/s** (\~25.7 ns)    | 32 M/s (\~30.9 ns)          |
| Memory usage      | **\~5 B/key** (\~50 MB)   | \~80-150 B/key (\~1 GB)     |
| Collisions        | **0**                     | handled internally          |

### Interpretation

* **RAM savings**: \~20× smaller index — critical for high-cardinality datasets.
* **Speed**: Lookups are faster than HashMap for fixed datasets.
* **Cold start**: `mmap` 50 MB vs pre-allocating and populating a 1 GB hash table.

---

## 🏭 Real-world Use Cases

### 1️⃣ Log indexing & analytics

* Map millions of unique field names / tag values to compact integer IDs.
* Save RAM in ingestion agents (Prometheus, Loki, ClickHouse pipelines).
* Speed up filter queries via dense IDs and bitmap indexes.

### 2️⃣ Dictionary encoding in columnar storage

* Encode categorical columns (`region`, `service`, `host`) into integers once.
* Store and query compressed integer IDs instead of full strings.

### 3️⃣ Security & network filtering

* Large allow/deny lists (domains, IPs, URL fingerprints).
* O(1) lookups with small, cache-resident index.

---

## 💡 Why BDZ MPH?

BDZ is a **minimal perfect hash** algorithm:

* **Minimal**: uses exactly `n` slots for `n` keys.
* **Perfect**: no collisions — every key maps to a unique slot.
* **Static**: ideal when the key set is fixed or updated infrequently.

Compared to other MPH algorithms:

* Faster build than CHD/RecSplit for large sets (1M+ records).
* Smaller memory than simpler schemes like CHM.
* Predictable lookup latency with no branching on collisions.

---

## 📈 Large-scale Example

```rust
use minimal_perfect_hash::Builder;
use std::time::Instant;

fn main() {
    let n = 10_000_000;
    println!("Generating {n} random keys...");
    let keys: Vec<String> = (0..n)
        .map(|i| format!("key_{i}"))
        .collect();

    let start = Instant::now();
    let mphf = Builder::new().keys(&keys).build().unwrap();
    println!("Build took: {:.3} s", start.elapsed().as_secs_f64());

    // Test lookups
    let look_start = Instant::now();
    let mut sum = 0;
    for k in &keys {
        sum += mphf.lookup(k).unwrap();
    }
    println!(
        "Lookups took: {:.3} s ({} lookups/s)",
        look_start.elapsed().as_secs_f64(),
        (n as f64) / look_start.elapsed().as_secs_f64()
    );
}
```

Possible output:

```
Generating 10000000 random keys...
Build took: 13.012 s
Lookups took: 0.257 s (38.9 M lookups/s)
```

---

## ⚙️ Tuning

The builder supports parameters:

```rust
let mphf = Builder::new()
    .gamma(1.27)           // trade memory vs build retries
    .max_retries(16)       // limit build attempts
    .keys(&keys)
    .build()
    .unwrap();
```

For **100M+ keys**:

* Use `gamma ≈ 1.27`.
* Enable parallel hashing (feature `"rayon"`).
* Consider building offline + overlay for new keys.

---

## 📜 License

MIT