use serde::Deserialize;
use std::collections::HashMap;

/// Root configuration file structure.
#[derive(Debug, Deserialize)]
pub struct PresetsConfig {
    /// Version of the config schema (e.g., 1).
    pub version: u32,
    /// Map of preset names to their definitions.
    pub presets: HashMap<String, PresetDefinition>,
}

/// Definition of a single preset.
#[derive(Debug, Deserialize, Clone)]
pub struct PresetDefinition {
    /// The tool to use for this preset (e.g., "gs", "qpdf").
    pub tool: String,
    /// List of arguments/flags to pass to the tool.
    /// Can include template placeholders in future, but for now exact flags.
    pub args: Vec<String>,
    /// Optional description of what this preset does.
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_config_deserialize_minimal() {
        let yaml = r#"
version: 1
presets: {}
"#;
        let config: PresetsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.version, 1);
        assert!(config.presets.is_empty());
    }

    #[test]
    fn test_presets_config_deserialize_with_preset() {
        let yaml = r#"
version: 2
presets:
  my-preset:
    tool: qpdf
    args:
      - "--linearize"
      - "--compress-streams=y"
    description: "Test preset"
"#;
        let config: PresetsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.version, 2);
        assert_eq!(config.presets.len(), 1);

        let preset = config.presets.get("my-preset").unwrap();
        assert_eq!(preset.tool, "qpdf");
        assert_eq!(preset.args.len(), 2);
        assert_eq!(preset.args[0], "--linearize");
        assert_eq!(preset.description, Some("Test preset".to_string()));
    }

    #[test]
    fn test_preset_definition_deserialize_without_description() {
        let yaml = r#"
tool: gs
args:
  - "-dNOPAUSE"
"#;
        let preset: PresetDefinition = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(preset.tool, "gs");
        assert_eq!(preset.args, vec!["-dNOPAUSE"]);
        assert!(preset.description.is_none());
    }

    #[test]
    fn test_preset_definition_clone() {
        let preset = PresetDefinition {
            tool: "gs".to_string(),
            args: vec!["-dBATCH".to_string()],
            description: Some("Cloneable".to_string()),
        };

        let cloned = preset.clone();
        assert_eq!(cloned.tool, preset.tool);
        assert_eq!(cloned.args, preset.args);
        assert_eq!(cloned.description, preset.description);
    }

    #[test]
    fn test_presets_config_multiple_presets() {
        let yaml = r#"
version: 1
presets:
  light:
    tool: gs
    args: ["-q"]
  heavy:
    tool: gs
    args: ["-q", "-dCompressFonts=true"]
    description: "Heavy compression"
"#;
        let config: PresetsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.presets.len(), 2);
        assert!(config.presets.contains_key("light"));
        assert!(config.presets.contains_key("heavy"));

        let heavy = config.presets.get("heavy").unwrap();
        assert_eq!(heavy.args.len(), 2);
    }
}
