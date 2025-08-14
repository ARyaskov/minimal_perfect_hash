use crate::BuildConfig;

/// CPU feature detection and optimal configuration selection
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub has_avx2: bool,
    pub has_bmi1: bool,
    pub has_bmi2: bool,
    pub has_popcnt: bool,
    pub has_lzcnt: bool,
    pub has_fma: bool,
    pub has_avx512f: bool,
    pub cache_line_size: usize,
    pub estimated_l3_size_mb: usize,
}

impl CpuFeatures {
    /// Detect available CPU features at runtime
    pub fn detect() -> Self {
        Self {
            has_avx2: Self::check_avx2(),
            has_bmi1: Self::check_bmi1(),
            has_bmi2: Self::check_bmi2(),
            has_popcnt: Self::check_popcnt(),
            has_lzcnt: Self::check_lzcnt(),
            has_fma: Self::check_fma(),
            has_avx512f: Self::check_avx512f(),
            cache_line_size: 64, // Standard for x86_64
            estimated_l3_size_mb: estimate_l3_cache_size(),
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn check_avx2() -> bool {
        is_x86_feature_detected!("avx2")
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn check_avx2() -> bool {
        false
    }

    #[cfg(target_arch = "x86_64")]
    fn check_bmi1() -> bool {
        is_x86_feature_detected!("bmi1")
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn check_bmi1() -> bool {
        false
    }

    #[cfg(target_arch = "x86_64")]
    fn check_bmi2() -> bool {
        is_x86_feature_detected!("bmi2")
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn check_bmi2() -> bool {
        false
    }

    #[cfg(target_arch = "x86_64")]
    fn check_popcnt() -> bool {
        is_x86_feature_detected!("popcnt")
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn check_popcnt() -> bool {
        false
    }

    #[cfg(target_arch = "x86_64")]
    fn check_lzcnt() -> bool {
        is_x86_feature_detected!("lzcnt")
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn check_lzcnt() -> bool {
        false
    }

    #[cfg(target_arch = "x86_64")]
    fn check_fma() -> bool {
        is_x86_feature_detected!("fma")
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn check_fma() -> bool {
        false
    }

    #[cfg(target_arch = "x86_64")]
    fn check_avx512f() -> bool {
        is_x86_feature_detected!("avx512f")
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn check_avx512f() -> bool {
        false
    }

    /// Get optimal configuration based on detected CPU features
    pub fn optimal_config(&self) -> BuildConfig {
        let use_simd = self.has_avx2 && cfg!(feature = "simd");
        let use_parallel = cfg!(feature = "parallel") &&
                          std::thread::available_parallelism().map_or(1, |n| n.get()) > 2;

        // Adjust gamma based on cache size
        let gamma = if self.estimated_l3_size_mb > 16 {
            1.25 // More aggressive for larger caches
        } else {
            1.27 // Conservative for smaller caches
        };

        let prefetch_distance = if self.has_avx2 {
            128 // Larger prefetch distance for SIMD
        } else {
            64  // Standard prefetch distance
        };

        BuildConfig {
            gamma,
            use_simd,
            use_parallel,
            prefetch_distance,
            ..Default::default()
        }
    }

    /// Print feature summary
    pub fn print_summary(&self) {
        println!("🖥️  CPU Features Detected:");
        println!("  AVX2:      {}", format_bool(self.has_avx2));
        println!("  BMI1/2:    {}/{}", format_bool(self.has_bmi1), format_bool(self.has_bmi2));
        println!("  POPCNT:    {}", format_bool(self.has_popcnt));
        println!("  LZCNT:     {}", format_bool(self.has_lzcnt));
        println!("  FMA:       {}", format_bool(self.has_fma));
        println!("  AVX-512:   {}", format_bool(self.has_avx512f));
        println!("  L3 Cache:  ~{}MB", self.estimated_l3_size_mb);

        let config = self.optimal_config();
        println!("  Optimal:   SIMD={}, Parallel={}, γ={}",
                 config.use_simd, config.use_parallel, config.gamma);
    }
}

/// Estimate L3 cache size (rough heuristic)
fn estimate_l3_cache_size() -> usize {
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());

    // Rough estimates based on common CPU configurations
    match cores {
        1..=2 => 4,   // 4MB (older/mobile CPUs)
        3..=4 => 8,   // 8MB (mainstream quad-core)
        5..=8 => 12,  // 12MB (mainstream 6-8 core)
        9..=12 => 20, // 20MB (high-end 8-12 core)
        13..=16 => 32, // 32MB (enthusiast 12-16 core)
        _ => 48,      // 48MB+ (HEDT/server)
    }
}

fn format_bool(b: bool) -> &'static str {
    if b { "✓" } else { "✗" }
}

/// Global function for easy access
pub fn detect_features() -> CpuFeatures {
    CpuFeatures::detect()
}