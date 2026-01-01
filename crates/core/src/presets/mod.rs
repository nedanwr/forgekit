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

pub mod loader;
pub mod model;

use crate::presets::loader::load_presets;

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
/// Maps compression levels to presets defined in the YAML configuration.
/// Defaults to "standard" if the requested level is not found.
pub fn get_compression_strategy(level: &str) -> CompressionStrategy {
    // Load presets (cached)
    let config = match load_presets() {
        Ok(c) => c,
        Err(e) => {
            // This should effectively never happen with embedded defaults
            eprintln!("Failed to load presets: {}", e);
            // Fallback emergency hardcoded standard
            return CompressionStrategy {
                tool: "gs".to_string(),
                flags: vec!["-sDEVICE=pdfwrite".to_string(), "-dQUIET".to_string()], // Minimal fallback
            };
        }
    };

    // Look up preset, default to "standard"
    let preset = config
        .presets
        .get(level)
        .or_else(|| config.presets.get("standard"))
        .expect("Standard preset missing from defaults!");

    CompressionStrategy {
        tool: preset.tool.clone(),
        flags: preset.args.clone(),
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
