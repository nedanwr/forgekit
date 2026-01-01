use crate::presets::model::PresetsConfig;
use crate::utils::error::Result;
use std::sync::OnceLock;

// Embedded default presets
// These exactly match the specific QFactor logic we developed for fast compression.
const DEFAULT_PRESETS_YAML: &str = r#"
version: 1
presets:
  light:
    tool: gs
    description: "High quality (~300 dpi equivalent)"
    args:
      - "-sDEVICE=pdfwrite"
      - "-dCompatibilityLevel=1.4"
      - "-dNOPAUSE"
      - "-dBATCH"
      - "-dQUIET"
      - "-dDownsampleColorImages=false"
      - "-dDownsampleGrayImages=false"
      - "-dDownsampleMonoImages=false"
      - "-c"
      - "<< /ColorACSImageDict << /QFactor 0.15 /Blend 1 /ColorTransform 1 /HSamples [1 1 1 1] /VSamples [1 1 1 1] >> >> setdistillerparams"

  standard:
    tool: gs
    description: "Medium quality (balanced defaults)"
    args:
      - "-sDEVICE=pdfwrite"
      - "-dCompatibilityLevel=1.4"
      - "-dNOPAUSE"
      - "-dBATCH"
      - "-dQUIET"
      - "-dDownsampleColorImages=false"
      - "-dDownsampleGrayImages=false"
      - "-dDownsampleMonoImages=false"
      - "-c"
      - "<< /ColorACSImageDict << /QFactor 0.5 /Blend 1 /ColorTransform 1 /HSamples [1 1 1 1] /VSamples [1 1 1 1] >> >> setdistillerparams"

  high:
    tool: gs
    description: "Low quality (smallest size)"
    args:
      - "-sDEVICE=pdfwrite"
      - "-dCompatibilityLevel=1.4"
      - "-dNOPAUSE"
      - "-dBATCH"
      - "-dQUIET"
      - "-dDownsampleColorImages=false"
      - "-dDownsampleGrayImages=false"
      - "-dDownsampleMonoImages=false"
      - "-c"
      - "<< /ColorACSImageDict << /QFactor 1.5 /Blend 1 /ColorTransform 1 /HSamples [1 1 1 1] /VSamples [1 1 1 1] >> >> setdistillerparams"
"#;

// Global cache for loaded presets
static PRESETS_CACHE: OnceLock<PresetsConfig> = OnceLock::new();

/// Load presets from defaults (and future file overrides).
///
/// Currently just returns the built-in defaults.
pub fn load_presets() -> Result<&'static PresetsConfig> {
    Ok(PRESETS_CACHE.get_or_init(|| {
        // Parse embedded defaults
        serde_yaml::from_str(DEFAULT_PRESETS_YAML)
            .expect("Failed to parse default presets! This is a bug in the embedded YAML.")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_presets_returns_valid_config() {
        let config = load_presets().expect("Failed to load presets");

        // Verify version
        assert_eq!(config.version, 1);

        // Verify all expected presets exist
        assert!(config.presets.contains_key("light"));
        assert!(config.presets.contains_key("standard"));
        assert!(config.presets.contains_key("high"));
    }

    #[test]
    fn test_load_presets_light_preset_correct() {
        let config = load_presets().expect("Failed to load presets");
        let light = config.presets.get("light").expect("light preset missing");

        assert_eq!(light.tool, "gs");
        assert!(light.description.is_some());
        assert!(light.args.iter().any(|a| a.contains("QFactor 0.15")));
    }

    #[test]
    fn test_load_presets_standard_preset_correct() {
        let config = load_presets().expect("Failed to load presets");
        let standard = config
            .presets
            .get("standard")
            .expect("standard preset missing");

        assert_eq!(standard.tool, "gs");
        assert!(standard.args.iter().any(|a| a.contains("QFactor 0.5")));
    }

    #[test]
    fn test_load_presets_high_preset_correct() {
        let config = load_presets().expect("Failed to load presets");
        let high = config.presets.get("high").expect("high preset missing");

        assert_eq!(high.tool, "gs");
        assert!(high.args.iter().any(|a| a.contains("QFactor 1.5")));
    }

    #[test]
    fn test_load_presets_all_have_required_gs_flags() {
        let config = load_presets().expect("Failed to load presets");

        for (name, preset) in &config.presets {
            assert!(
                preset.args.iter().any(|a| a == "-sDEVICE=pdfwrite"),
                "Preset '{}' missing -sDEVICE=pdfwrite",
                name
            );
            assert!(
                preset.args.iter().any(|a| a == "-dNOPAUSE"),
                "Preset '{}' missing -dNOPAUSE",
                name
            );
            assert!(
                preset.args.iter().any(|a| a == "-dBATCH"),
                "Preset '{}' missing -dBATCH",
                name
            );
        }
    }

    #[test]
    fn test_load_presets_is_cached() {
        // Call twice - should return same reference (OnceLock behavior)
        let config1 = load_presets().expect("First load failed");
        let config2 = load_presets().expect("Second load failed");

        // Both should point to the same static reference
        assert!(std::ptr::eq(config1, config2));
    }
}
