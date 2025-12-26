//! # Preset System
//!
//! The preset system allows users to apply predefined quality/compression settings
//! to operations without specifying all the tool flags manually.
//!
//! ## Future Implementation
//!
//! Presets will be loaded from YAML files:
//! - Built-in presets: `presets/presets.yaml` (included in package)
//! - User overrides: `~/.config/forgekit/presets.yaml`
//!
//! For now, presets are hardcoded in the executor. This module provides the
//! foundation for future YAML-based preset loading.

/// Compression strategy configuration for different compression levels.
#[derive(Debug, Clone)]
pub struct CompressionStrategy {
    /// Tool to use: "gs" (Ghostscript) or "qpdf"
    pub tool: String,
    /// Additional flags/arguments for the tool
    pub flags: Vec<String>,
}

/// Get compression strategy for a given compression level.
///
/// Maps our compression levels (light, standard, high) to custom Ghostscript flags
/// that achieve fast compression by disabling image downsampling and controlling
/// JPEG quality directly.
///
/// # Arguments
///
/// * `level` - Compression level: "light", "standard", or "high"
///
/// # Returns
///
/// CompressionStrategy with tool name and flags to use
///
/// # Fast Compression Strategy
///
/// We disable expensive image downsampling (`-dDownsample*Images=false`) and rely on
/// direct JPEG Quality Factor (`/QFactor`) control via Distiller params:
///
/// - **Light** (High Quality): QFactor 0.15 (~4.2MB)
/// - **Standard** (Med Quality): QFactor 0.50 (~3.4MB)
/// - **High** (Low Quality): QFactor 1.50 (~2.7MB)
///
/// All levels run extremely fast (~1s) compared to downsampling methods (~8s+).
pub fn get_compression_strategy(level: &str) -> CompressionStrategy {
    // Base flags for all Ghostscript operations
    let mut flags = vec![
        "-sDEVICE=pdfwrite".to_string(),
        "-dCompatibilityLevel=1.4".to_string(),
        "-dNOPAUSE".to_string(),
        "-dBATCH".to_string(),
        "-dQUIET".to_string(),
        // Disable downsampling for speed
        "-dDownsampleColorImages=false".to_string(),
        "-dDownsampleGrayImages=false".to_string(),
        "-dDownsampleMonoImages=false".to_string(),
    ];

    let q_factor = match level {
        // Lower QFactor = Higher Quality
        "light" => "0.15",
        "standard" => "0.5",
        "high" => "1.5",
        _ => "0.5", // Default to standard
    };

    // Inject Distiller params for JPEG quality
    flags.push("-c".to_string());
    flags.push(format!(
        "<< /ColorACSImageDict << /QFactor {} /Blend 1 /ColorTransform 1 /HSamples [1 1 1 1] /VSamples [1 1 1 1] >> >> setdistillerparams",
        q_factor
    ));

    CompressionStrategy {
        tool: "gs".to_string(),
        flags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_compression_strategy_light() {
        let strategy = get_compression_strategy("light");
        assert_eq!(strategy.tool, "gs");
        assert!(strategy.flags.contains(&"-sDEVICE=pdfwrite".to_string()));

        // Assert no downsampling
        assert!(strategy
            .flags
            .contains(&"-dDownsampleColorImages=false".to_string()));

        // Assert QFactor 0.15 (High Quality)
        let param_str = strategy.flags.last().unwrap();
        assert!(param_str.contains("/QFactor 0.15"));
    }

    #[test]
    fn test_get_compression_strategy_standard() {
        let strategy = get_compression_strategy("standard");
        assert_eq!(strategy.tool, "gs");

        // Assert QFactor 0.5 (Standard)
        let param_str = strategy.flags.last().unwrap();
        assert!(param_str.contains("/QFactor 0.5"));
    }

    #[test]
    fn test_get_compression_strategy_high() {
        let strategy = get_compression_strategy("high");

        // Assert QFactor 1.5 (Low Quality)
        let param_str = strategy.flags.last().unwrap();
        assert!(param_str.contains("/QFactor 1.5"));
    }

    #[test]
    fn test_get_compression_strategy_unknown() {
        let strategy = get_compression_strategy("unknown");
        // Unknown defaults to standard (0.5)
        let param_str = strategy.flags.last().unwrap();
        assert!(param_str.contains("/QFactor 0.5"));
    }

    #[test]
    fn test_compression_strategy_base_flags() {
        // All strategies should have the base Ghostscript flags
        for level in &["light", "standard", "high"] {
            let strategy = get_compression_strategy(level);
            assert!(strategy.flags.contains(&"-sDEVICE=pdfwrite".to_string()));
            assert!(strategy.flags.contains(&"-dQUIET".to_string()));
            assert!(strategy
                .flags
                .contains(&"-dDownsampleColorImages=false".to_string()));
            // Ensure -c and params are separate args
            assert!(strategy.flags.contains(&"-c".to_string()));
        }
    }
}
