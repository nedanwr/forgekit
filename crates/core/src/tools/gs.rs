//! # Ghostscript (gs) Tool Adapter
//!
//! Ghostscript is a command-line tool for PDF processing. We use it for:
//! - Compressing/recompressing PDFs with actual image downsampling
//! - PDF format conversion
//!
//! ## Why Ghostscript?
//!
//! Unlike tools that only optimize PDF structure, Ghostscript
//! actually recompresses images using direct JPEG Quality Factor (QFactor)
//! control without expensive downsampling. This achieves high speeds (~1s) while
//! maintaining file size differentiation:
//! - Light: High quality (QFactor 0.15)
//! - Standard: Medium quality (QFactor 0.50)
//! - High: Low quality (QFactor 1.50)
//!
//! ## Minimum Version
//!
//! Ghostscript 9.50+ is recommended. Older versions may work but aren't tested.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tools::{Tool, ToolConfig, ToolInfo};
use crate::utils::error::{ForgeKitError, Result};
use crate::utils::platform::ToolInstallHints;

/// Ghostscript tool adapter.
///
/// Implements the `Tool` trait for Ghostscript, handling detection, version checking,
/// and command construction for PDF compression.
pub struct GsTool;

impl Tool for GsTool {
    fn name(&self) -> &'static str {
        "gs"
    }

    fn probe(&self, config: &ToolConfig) -> Result<ToolInfo> {
        // Check override path first
        if let Some(ref path) = config.override_path {
            if path.exists() {
                let version = self.version(path)?;
                return Ok(ToolInfo {
                    path: path.clone(),
                    version,
                    available: true,
                });
            }
        }

        // Probe PATH - try 'gs' first, then 'gswin64c' on Windows
        let binary_names = if cfg!(target_os = "windows") {
            vec!["gswin64c", "gswin32c", "gs"]
        } else {
            vec!["gs"]
        };

        for binary_name in binary_names {
            let which_output = if cfg!(target_os = "windows") {
                Command::new("where").arg(binary_name).output()
            } else {
                Command::new("which").arg(binary_name).output()
            };

            if let Ok(output) = which_output {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    if !path_str.is_empty() {
                        let path = PathBuf::from(&path_str);
                        if let Ok(version) = self.version(&path) {
                            return Ok(ToolInfo {
                                path,
                                version,
                                available: true,
                            });
                        }
                    }
                }
            }
        }

        Err(ForgeKitError::ToolNotFound {
            tool: "gs".to_string(),
            hint: ToolInstallHints::for_tool("gs"),
        })
    }

    fn version(&self, path: &Path) -> Result<String> {
        let output = Command::new(path)
            .arg("--version")
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run gs: {}", e)))?;

        if !output.status.success() {
            return Err(ForgeKitError::Other(anyhow::anyhow!("gs --version failed")));
        }

        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        Ok(version.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gs_name() {
        let tool = GsTool;
        assert_eq!(tool.name(), "gs");
    }

    #[test]
    fn test_gs_probe() {
        let tool = GsTool;
        let config = ToolConfig::default();
        // This will only pass if gs is installed
        let result = tool.probe(&config);
        // We don't assert success here since gs may not be installed in test environment
        // Just verify it doesn't panic
        match result {
            Ok(info) => {
                assert!(info.available);
                assert!(!info.version.is_empty());
            }
            Err(ForgeKitError::ToolNotFound { .. }) => {
                // Expected if gs is not installed
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
