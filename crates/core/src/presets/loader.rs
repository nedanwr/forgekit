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
