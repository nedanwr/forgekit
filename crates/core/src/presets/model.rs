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
