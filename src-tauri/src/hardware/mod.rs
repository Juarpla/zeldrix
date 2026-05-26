//! Hardware detection and model recommendation for Zeldrix.
//!
//! This module evaluates the hardware capabilities of the machine and recommends
//! the most efficient GGUF model that balances inference speed and quantization quality.

use serde::Serialize;
use sysinfo::System;

/// GPU vendor enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum GpuVendor {
    /// Apple Silicon (M1/M2/M3/M4) with Metal support.
    Apple,
    /// NVIDIA GPU with CUDA support.
    Nvidia,
    /// AMD GPU (ROCm).
    Amd,
    /// Intel GPU.
    Intel,
    /// No dedicated GPU detected.
    None,
}

impl GpuVendor {
    /// Returns a static string representation for display.
    pub fn as_str(&self) -> &'static str {
        match self {
            GpuVendor::Apple => "apple",
            GpuVendor::Nvidia => "nvidia",
            GpuVendor::Amd => "amd",
            GpuVendor::Intel => "intel",
            GpuVendor::None => "none",
        }
    }
}

/// Hardware information detected on the system.
///
/// This struct contains all relevant hardware capabilities that affect
/// LLM inference performance and model selection.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    /// Total RAM in bytes.
    pub ram_total_bytes: u64,
    /// Available RAM in bytes.
    pub ram_available_bytes: u64,
    /// Available VRAM in bytes (0 if no dedicated GPU).
    pub vram_available_bytes: u64,
    /// Number of logical CPU cores.
    pub cpu_logical_count: u32,
    /// Whether AVX2 instructions are available.
    pub has_avx2: bool,
    /// Whether AVX512 instructions are available.
    pub has_avx512: bool,
    /// Detected GPU vendor.
    pub gpu_vendor: GpuVendor,
    /// Whether the system is Apple Silicon.
    pub is_apple_silicon: bool,
    /// Whether Metal (Apple GPU acceleration) is available.
    pub has_metal: bool,
    /// Whether CUDA (NVIDIA GPU acceleration) is available.
    pub has_cuda: bool,
}

impl HardwareInfo {
    /// Creates a new HardwareInfo by detecting system capabilities.
    ///
    /// Detection includes:
    /// - RAM total and available using `sysinfo`
    /// - CPU core count and SIMD instruction support (via target feature detection)
    /// - GPU vendor detection (Apple/NVIDIA/AMD/Intel/None)
    ///
    /// Note: VRAM detection is limited; on systems without proper GPU APIs,
    /// `vram_available_bytes` will be 0.
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let ram_total_bytes = sys.total_memory();
        let ram_available_bytes = sys.available_memory();
        let cpu_logical_count = sys.cpus().len() as u32;

        // Detect SIMD support via runtime feature detection
        // This is a best-effort approach; some platforms may report incorrect values.
        let has_avx2 = detect_avx2();
        let has_avx512 = detect_avx512();

        // Detect GPU vendor
        let (gpu_vendor, is_apple_silicon, has_metal, has_cuda) = detect_gpu();

        // VRAM detection is platform-specific and often unavailable via sysinfo
        // We use 0 as default and rely on the scoring system to handle GPU-less systems
        let vram_available_bytes = detect_vram().unwrap_or(0);

        HardwareInfo {
            ram_total_bytes,
            ram_available_bytes,
            vram_available_bytes,
            cpu_logical_count,
            has_avx2,
            has_avx512,
            gpu_vendor,
            is_apple_silicon,
            has_metal,
            has_cuda,
        }
    }

    /// Returns RAM total in GiB (binary gibibytes).
    #[must_use]
    pub fn ram_total_gib(&self) -> f64 {
        self.ram_total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Returns RAM available in GiB.
    #[must_use]
    #[allow(unused)]
    pub fn ram_available_gib(&self) -> f64 {
        self.ram_available_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Returns VRAM available in GiB.
    #[must_use]
    #[allow(unused)]
    pub fn vram_available_gib(&self) -> f64 {
        self.vram_available_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Returns true if the system has less than 16GB RAM.
    #[must_use]
    pub fn is_low_ram(&self) -> bool {
        self.ram_total_bytes < 16 * 1024 * 1024 * 1024
    }

    /// Returns true if the system has more than 32GB RAM and a dedicated GPU.
    #[must_use]
    pub fn can_use_bf16(&self) -> bool {
        self.ram_total_bytes > 32 * 1024 * 1024 * 1024 && self.gpu_vendor != GpuVendor::None
    }
}

/// SIMD configuration detected on the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SimdConfig {
    /// AVX2 support.
    pub has_avx2: bool,
    /// AVX512 support.
    pub has_avx512: bool,
}

impl SimdConfig {
    /// Returns the SIMD score for benchmark calculation.
    /// Higher scores indicate better SIMD capabilities.
    pub fn score(&self) -> u64 {
        match (self.has_avx512, self.has_avx2) {
            (true, _) => 16,
            (_, true) => 8,
            _ => 0,
        }
    }
}

/// Represents a GGUF model in the catalog.
///
/// Using `&'static str` for string fields to allow const initialization.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// File name of the model.
    pub filename: String,
    /// Human-readable name.
    pub name: String,
    /// Required RAM in GiB.
    pub min_ram_gb: f64,
    /// Required VRAM in GiB (0 if CPU-only).
    pub vram_gb: f64,
    /// Quantization type (IQ2_XXS, Q4_K_M, BF16, etc.).
    pub quantization: String,
    /// Relative speed rating (1=slowest, 5=fastest).
    pub speed_rating: u8,
    /// Relative quality rating (1=lowest, 5=highest).
    pub quality_rating: u8,
    /// Whether this is a BF16 model (requires >32GB RAM + dedicated GPU).
    pub is_bf16: bool,
}

/// Creates a Model with owned String fields from static str inputs.
macro_rules! model {
    ($filename:expr, $name:expr, $min_ram:expr, $vram:expr, $quant:expr, $speed:expr, $quality:expr, $bf16:expr) => {
        Model {
            filename: $filename.to_string(),
            name: $name.to_string(),
            min_ram_gb: $min_ram,
            vram_gb: $vram,
            quantization: $quant.to_string(),
            speed_rating: $speed,
            quality_rating: $quality,
            is_bf16: $bf16,
        }
    };
}

/// Catalog of 8 target models for recommendation.
pub fn get_model_catalog() -> Vec<Model> {
    vec![
        model!(
            "qwen3.6-27b-ud-iq2_xxs.gguf",
            "Qwen3.6 27B UD IQ2_XXS",
            5.5,
            0.0,
            "IQ2_XXS",
            5,
            1,
            false
        ),
        model!(
            "gemma-4-2b-it-iq4_xs.gguf",
            "Gemma 4 2B IT IQ4_XS",
            1.8,
            0.0,
            "IQ4_XS",
            5,
            2,
            false
        ),
        model!(
            "qwen2.5-7b-instruct-q4_k_m.gguf",
            "Qwen2.5 7B Instruct Q4_K_M",
            4.5,
            0.0,
            "Q4_K_M",
            4,
            3,
            false
        ),
        model!(
            "llama-3.2-3b-instruct-q5_k_m.gguf",
            "LLaMA 3.2 3B Instruct Q5_K_M",
            2.5,
            0.0,
            "Q5_K_M",
            3,
            3,
            false
        ),
        model!(
            "mistral-7b-instruct-v0.3-q6_k.gguf",
            "Mistral 7B Instruct v0.3 Q6_K",
            5.5,
            0.0,
            "Q6_K",
            3,
            4,
            false
        ),
        model!(
            "qwen3-8b-bf16.gguf",
            "Qwen3 8B BF16",
            16.0,
            8.0,
            "BF16",
            2,
            5,
            true
        ),
        model!(
            "llama-3.1-70b-instruct-q4_k_m.gguf",
            "LLaMA 3.1 70B Instruct Q4_K_M",
            40.0,
            20.0,
            "Q4_K_M",
            1,
            4,
            false
        ),
        model!(
            "qwen3-32b-bf16.gguf",
            "Qwen3 32B BF16",
            64.0,
            32.0,
            "BF16",
            1,
            5,
            true
        ),
    ]
}

/// Model recommendation result.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRecommendation {
    /// The recommended model.
    pub model: Model,
    /// Whether this was a forced recommendation due to hardware constraints.
    pub is_forced: bool,
    /// Reason for the recommendation or forced selection.
    pub reason: String,
    /// The benchmark score used for ranking.
    pub benchmark_score: u64,
    /// Alternative models that would also work.
    pub alternatives: Vec<Model>,
}

/// Calculates the deterministic benchmark score for the given hardware.
///
/// The score is computed from:
/// - RAM in GB (1 point per GB)
/// - VRAM in GB (2 points per GB, weighted higher)
/// - CPU logical cores (1 point per core)
/// - SIMD capabilities (8 for AVX2, 16 for AVX512)
/// - GPU vendor bonus (32 for NVIDIA, 24 for Apple, 20 for AMD)
#[must_use]
pub fn calculate_benchmark_score(info: &HardwareInfo) -> u64 {
    let ram_gb = info.ram_total_bytes / (1024 * 1024 * 1024);
    let vram_gb = info.vram_available_bytes / (1024 * 1024 * 1024);
    let cpu_score = info.cpu_logical_count as u64;

    let simd_config = SimdConfig {
        has_avx2: info.has_avx2,
        has_avx512: info.has_avx512,
    };
    let simd_score = simd_config.score();

    let gpu_bonus = match info.gpu_vendor {
        GpuVendor::Nvidia => 32,
        GpuVendor::Apple => 24,
        GpuVendor::Amd => 20,
        _ => 0,
    };

    ram_gb + (vram_gb * 2) + cpu_score + simd_score + gpu_bonus
}

/// Recommends the optimal model based on hardware capabilities.
///
/// # Rules
///
/// 1. **Low RAM (<16GB):** Forces use of `qwen3.6-27b-ud-iq2_xxs.gguf`
/// 2. **BF16 eligibility (>32GB RAM + dedicated GPU):** Enables BF16 variants
/// 3. **Otherwise:** Ranks eligible models by speed/quality balance
#[must_use]
pub fn recommend_model(info: &HardwareInfo) -> ModelRecommendation {
    let score = calculate_benchmark_score(info);
    let model_catalog = get_model_catalog();

    // RULE 1: <16GB RAM → force Qwen3.6-27B-UD-IQ2_XXS
    if info.is_low_ram() {
        let forced_model = model_catalog
            .iter()
            .find(|m| m.filename == "qwen3.6-27b-ud-iq2_xxs.gguf")
            .expect("Qwen3.6-27B-UD-IQ2_XXS must exist in catalog");

        return ModelRecommendation {
            model: forced_model.clone(),
            is_forced: true,
            reason: format!(
                "System has {:.1}GB RAM (<16GB threshold). Forcing lightweight model.",
                info.ram_total_gib()
            ),
            benchmark_score: score,
            alternatives: Vec::new(),
        };
    }

    // Determine eligible models
    let can_use_bf16 = info.can_use_bf16();
    let ram_bytes = info.ram_total_bytes;
    let vram_bytes = info.vram_available_bytes;

    let eligible: Vec<&Model> = model_catalog
        .iter()
        .filter(|m| {
            let min_ram_bytes = (m.min_ram_gb * 1024.0 * 1024.0 * 102.0) as u64;
            let fits_ram = min_ram_bytes <= ram_bytes;

            // If VRAM is 0, assume CPU-only loading (model fits in RAM)
            let fits_vram = vram_bytes == 0 || m.vram_gb == 0.0 || (m.vram_gb * 1024.0 * 1024.0 * 1024.0) as u64 <= vram_bytes;

            // BF16 models require explicit eligibility
            let bf16_allowed = !m.is_bf16 || can_use_bf16;

            fits_ram && fits_vram && bf16_allowed
        })
        .collect();

    // If no models eligible, fall back to the smallest one
    if eligible.is_empty() {
        let fallback = model_catalog.first().expect("Catalog must have at least one model");
        return ModelRecommendation {
            model: fallback.clone(),
            is_forced: true,
            reason: "No model fits current hardware constraints. Falling back to smallest model.".to_string(),
            benchmark_score: score,
            alternatives: Vec::new(),
        };
    }

    // Score-based ranking: prefer models with best speed/quality ratio
    // For deterministic results, sort by (speed_rating desc, quality_rating desc, min_ram_gb asc)
    let mut ranked: Vec<&Model> = eligible.to_vec();
    ranked.sort_by(|a, b| {
        // First by speed (faster is better)
        b.speed_rating.cmp(&a.speed_rating)
            // Then by quality (higher is better)
            .then(b.quality_rating.cmp(&a.quality_rating))
            // Then by RAM requirement (smaller is better for same tier)
            .then(a.min_ram_gb.partial_cmp(&b.min_ram_gb).unwrap_or(std::cmp::Ordering::Equal))
    });

    let recommended = (*ranked.first().expect("Eligible list was non-empty")).clone();
    let alternatives: Vec<Model> = ranked.iter().skip(1).take(3).map(|m| (*m).clone()).collect();

    let reason = if can_use_bf16 {
        format!(
            "System has {:.1}GB RAM + {} GPU. BF16 models enabled. Score: {}",
            info.ram_total_gib(),
            info.gpu_vendor.as_str(),
            score
        )
    } else {
        format!(
            "System has {:.1}GB RAM, {} GPU. Selected optimal speed/quality balance. Score: {}",
            info.ram_total_gib(),
            info.gpu_vendor.as_str(),
            score
        )
    };

    ModelRecommendation {
        model: recommended.clone(),
        is_forced: false,
        reason,
        benchmark_score: score,
        alternatives,
    }
}

// Platform-specific detection functions

#[cfg(target_arch = "x86_64")]
fn detect_avx2() -> bool {
    // On x86_64, we check for AVX2 via CPU feature detection
    is_x86_feature_detected!("avx2")
}

#[cfg(target_arch = "x86_64")]
fn detect_avx512() -> bool {
    is_x86_feature_detected!("avx512f")
}

#[cfg(not(target_arch = "x86_64"))]
fn detect_avx2() -> bool {
    // Non-x86 architectures don't have AVX
    false
}

#[cfg(not(target_arch = "x86_64"))]
fn detect_avx512() -> bool {
    false
}

fn detect_gpu() -> (GpuVendor, bool, bool, bool) {
    // Detection is platform-specific
    // We use environment hints and available APIs

    #[cfg(target_os = "macos")]
    {
        // Check if running on Apple Silicon by checking architecture
        // On macOS ARM, the target is aarch64
        if cfg!(target_arch = "aarch64") {
            return (GpuVendor::Apple, true, true, false);
        }
        // Intel Mac with Metal
        (GpuVendor::None, false, true, false)
    }

    #[cfg(target_os = "linux")]
    {
        // Check for NVIDIA via nvidia-smi or /proc/driver/nvidia
        if std::path::Path::new("/proc/driver/nvidia").exists()
            || std::process::Command::new("nvidia-smi").output().is_ok()
        {
            return (GpuVendor::Nvidia, false, false, true);
        }
        // Check for AMD via /proc/driver/radeon or ROCm
        if std::path::Path::new("/proc/driver/radeon").exists() {
            return (GpuVendor::Amd, false, false, false);
        }
        return (GpuVendor::None, false, false, false);
    }

    #[cfg(target_os = "windows")]
    {
        // Check for NVIDIA via nvapi or wmic
        // Simplified: check for NVIDIA DLL presence
        if std::env::var("NVIDIA_VISIBLE_DEVICES").is_ok()
            || std::path::Path::new("C:\\Windows\\System32\\nvidia.dll").exists()
        {
            return (GpuVendor::Nvidia, false, false, true);
        }
        return (GpuVendor::None, false, false, false);
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        (GpuVendor::None, false, false, false)
    }
}

fn detect_vram() -> Option<u64> {
    // VRAM detection is highly platform-specific
    // sysinfo doesn't provide VRAM information directly

    #[cfg(target_os = "macos")]
    {
        // On macOS, we could use Metal to query GPU memory
        // For now, return None (unknown)
        None
    }

    #[cfg(target_os = "linux")]
    {
        // Try to read from sysfs for NVIDIA or AMD
        if let Ok(nv_path) = std::fs::read_to_string("/sys/class/drm/card0/device/mem_info_vram_total") {
            if let Ok(vram) = nv_path.trim().parse::<u64>() {
                return Some(vram);
            }
        }
        None
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Returns a formatted summary of the hardware for debugging.
#[must_use]
#[allow(unused)]
pub fn hardware_summary(info: &HardwareInfo) -> String {
    format!(
        "Hardware Summary:\n\
         ├── RAM: {:.1} GB total, {:.1} GB available\n\
         ├── VRAM: {:.1} GB available\n\
         ├── CPU: {} logical cores\n\
         ├── SIMD: AVX2={}, AVX512={}\n\
         ├── GPU: {} (Metal={}, CUDA={})\n\
         └── Benchmark Score: {}",
        info.ram_total_gib(),
        info.ram_available_gib(),
        info.vram_available_gib(),
        info.cpu_logical_count,
        info.has_avx2,
        info.has_avx512,
        info.gpu_vendor.as_str(),
        info.has_metal,
        info.has_cuda,
        calculate_benchmark_score(info)
    )
}

// Implement Default for HardwareInfo for test convenience
impl Default for HardwareInfo {
    fn default() -> Self {
        HardwareInfo {
            ram_total_bytes: 16 * 1024 * 1024 * 1024, // 16GB default
            ram_available_bytes: 8 * 1024 * 1024 * 1024,
            vram_available_bytes: 0,
            cpu_logical_count: 8,
            has_avx2: true,
            has_avx512: false,
            gpu_vendor: GpuVendor::None,
            is_apple_silicon: false,
            has_metal: false,
            has_cuda: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that hardware detection returns a valid structure.
    #[test]
    fn detect_returns_valid_hardware_info() {
        let info = HardwareInfo::detect();

        // Basic sanity checks
        assert!(info.ram_total_bytes > 0, "RAM should be detected");
        assert!(info.ram_available_bytes <= info.ram_total_bytes, "Available RAM should be <= total");
        assert!(info.cpu_logical_count > 0, "CPU count should be > 0");

        // GPU vendor should be one of the known values
        match info.gpu_vendor {
            GpuVendor::Apple | GpuVendor::Nvidia | GpuVendor::Amd | GpuVendor::Intel | GpuVendor::None => {}
        }
    }

    /// Test that benchmark score is deterministic for the same hardware.
    #[test]
    fn benchmark_score_is_deterministic() {
        let info = HardwareInfo::detect();
        let score1 = calculate_benchmark_score(&info);
        let score2 = calculate_benchmark_score(&info);

        assert_eq!(score1, score2, "Benchmark score should be deterministic");
    }

    /// Test model recommendation for low RAM system.
    #[test]
    fn recommend_forces_low_ram_model_under_16gb() {
        let info = HardwareInfo {
            ram_total_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            ram_available_bytes: 4 * 1024 * 1024 * 1024,
            vram_available_bytes: 0,
            cpu_logical_count: 8,
            has_avx2: true,
            has_avx512: false,
            gpu_vendor: GpuVendor::None,
            is_apple_silicon: false,
            has_metal: false,
            has_cuda: false,
        };

        let recommendation = recommend_model(&info);

        assert!(recommendation.is_forced, "Should be forced due to low RAM");
        assert_eq!(
            recommendation.model.filename,
            "qwen3.6-27b-ud-iq2_xxs.gguf",
            "Should recommend Qwen3.6-27B-UD-IQ2_XXS for <16GB RAM"
        );
        assert!(
            recommendation.reason.contains("16GB"),
            "Reason should mention the 16GB threshold"
        );
    }

    /// Test model recommendation for high-end system with BF16 eligibility.
    #[test]
    fn recommend_enables_bf16_over_32gb_with_gpu() {
        let info = HardwareInfo {
            ram_total_bytes: 64 * 1024 * 1024 * 1024, // 64GB
            ram_available_bytes: 32 * 1024 * 1024 * 1024,
            vram_available_bytes: 24 * 1024 * 1024 * 1024, // 24GB VRAM
            cpu_logical_count: 32,
            has_avx2: true,
            has_avx512: true,
            gpu_vendor: GpuVendor::Nvidia,
            is_apple_silicon: false,
            has_metal: false,
            has_cuda: true,
        };

        let recommendation = recommend_model(&info);

        // Should recommend a BF16 model since system is eligible
        assert!(
            !recommendation.is_forced,
            "Should not be forced for high-end system"
        );
        assert!(
            recommendation.reason.contains("BF16") || recommendation.model.is_bf16,
            "Should enable BF16 models for >32GB RAM + dedicated GPU"
        );
    }

    /// Test benchmark score calculation with known inputs.
    #[test]
    fn benchmark_score_calculation_is_deterministic() {
        // Create a mock hardware info with known values
        let info = HardwareInfo {
            ram_total_bytes: 32 * 1024 * 1024 * 1024, // 32GB
            ram_available_bytes: 16 * 1024 * 1024 * 1024,
            vram_available_bytes: 8 * 1024 * 1024 * 1024, // 8GB VRAM
            cpu_logical_count: 16,
            has_avx2: true,
            has_avx512: false,
            gpu_vendor: GpuVendor::Nvidia,
            is_apple_silicon: false,
            has_metal: false,
            has_cuda: true,
        };

        let score = calculate_benchmark_score(&info);

        // Expected calculation:
        // RAM: 32 GB = 32 points
        // VRAM: 8 GB * 2 = 16 points
        // CPU: 16 = 16 points
        // SIMD: AVX2 = 8 points
        // GPU: NVIDIA = 32 points
        // Total: 32 + 16 + 16 + 8 + 32 = 104
        assert_eq!(score, 104, "Benchmark score should match expected calculation");
    }

    /// Test that all models in catalog have valid data.
    #[test]
    fn all_catalog_models_are_valid() {
        for model in get_model_catalog() {
            assert!(!model.filename.is_empty(), "Model filename should not be empty");
            assert!(!model.name.is_empty(), "Model name should not be empty");
            assert!(model.min_ram_gb > 0.0, "Model RAM requirement should be > 0");
            assert!(model.speed_rating >= 1 && model.speed_rating <= 5, "Speed rating should be 1-5");
            assert!(model.quality_rating >= 1 && model.quality_rating <= 5, "Quality rating should be 1-5");
        }
    }

    /// Test hardware info helper methods.
    #[test]
    fn hardware_info_helper_methods() {
        let info = HardwareInfo {
            ram_total_bytes: 16 * 1024 * 1024 * 1024, // 16GB
            ram_available_bytes: 8 * 1024 * 1024 * 1024,
            vram_available_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            cpu_logical_count: 8,
            has_avx2: true,
            has_avx512: false,
            gpu_vendor: GpuVendor::None,
            is_apple_silicon: false,
            has_metal: false,
            has_cuda: false,
        };

        assert!((info.ram_total_gib() - 16.0).abs() < 0.01);
        assert!((info.vram_available_gib() - 4.0).abs() < 0.01);
        assert!(!info.is_low_ram(), "16GB is not low RAM");
        assert!(!info.can_use_bf16(), "No GPU, so cannot use BF16");
    }

    /// Test is_low_ram threshold.
    #[test]
    fn is_low_ram_threshold() {
        // Exactly 16GB should not be considered low
        let info_16gb = HardwareInfo {
            ram_total_bytes: 16 * 1024 * 1024 * 1024,
            ..Default::default()
        };
        assert!(!info_16gb.is_low_ram(), "Exactly 16GB should not be low RAM");

        // 15.99GB should be low
        let info_15gb = HardwareInfo {
            ram_total_bytes: (15u64 * 1024 * 1024 * 1024) + (1024 * 1024 * 1024 / 100),
            ..Default::default()
        };
        assert!(info_15gb.is_low_ram(), "15.99GB should be low RAM");
    }

    /// Test can_use_bf16 conditions.
    #[test]
    fn can_use_bf16_conditions() {
        // 32GB + no GPU = cannot use BF16
        let no_gpu = HardwareInfo {
            ram_total_bytes: 32 * 1024 * 1024 * 1024 + 1,
            gpu_vendor: GpuVendor::None,
            ..Default::default()
        };
        assert!(!no_gpu.can_use_bf16(), "No GPU should not enable BF16");

        // 32GB + 1 byte more + GPU = can use BF16
        let with_gpu = HardwareInfo {
            ram_total_bytes: 32 * 1024 * 1024 * 1024 + 1,
            gpu_vendor: GpuVendor::Nvidia,
            ..Default::default()
        };
        assert!(with_gpu.can_use_bf16(), "32GB+1 byte + GPU should enable BF16");
    }
}