//! # ocrmypdf Tool Adapter
//!
//! ocrmypdf is a Python command-line tool that adds OCR text layers to PDFs.
//! It wraps Tesseract OCR and handles PDF-specific complexities like:
//! - Maintaining original PDF structure
//! - Adding invisible text layer over scanned pages
//! - Skipping pages that already have text
//!
//! ## Why ocrmypdf?
//!
//! While we could call Tesseract directly, ocrmypdf handles many edge cases:
//! - PDF page extraction and reassembly
//! - Preserving original quality
//! - Handling encrypted PDFs
//! - Multi-language support
//! - Progress reporting
//!
//! ## Minimum Version
//!
//! ocrmypdf 14.0+ is recommended. Older versions may work but aren't tested.
//!
//! ## Dependencies
//!
//! ocrmypdf requires:
//! - Python 3.8+
//! - Tesseract OCR 4.0+ (with language packs)
//! - Ghostscript (for PDF manipulation)

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tools::{Tool, ToolConfig, ToolInfo};
use crate::utils::error::{ForgeKitError, Result};
use crate::utils::platform::ToolInstallHints;

/// ocrmypdf tool adapter.
///
/// Implements the `Tool` trait for ocrmypdf, handling detection, version checking,
/// and command construction for OCR operations on PDFs.
pub struct OcrmypdfTool;

impl Tool for OcrmypdfTool {
    fn name(&self) -> &'static str {
        "ocrmypdf"
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

        // Probe PATH - try 'ocrmypdf' directly
        let which_output = if cfg!(target_os = "windows") {
            Command::new("where").arg("ocrmypdf").output()
        } else {
            Command::new("which").arg("ocrmypdf").output()
        };

        let path = match which_output {
            Ok(output) if output.status.success() => {
                let path_str = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !path_str.is_empty() {
                    PathBuf::from(path_str)
                } else {
                    PathBuf::from("ocrmypdf")
                }
            }
            _ => PathBuf::from("ocrmypdf"),
        };

        // Verify it works
        let output = Command::new(&path).arg("--version").output().map_err(|_| {
            ForgeKitError::ToolNotFound {
                tool: "ocrmypdf".to_string(),
                hint: ToolInstallHints::for_tool("ocrmypdf"),
            }
        })?;

        if !output.status.success() {
            return Err(ForgeKitError::ToolNotFound {
                tool: "ocrmypdf".to_string(),
                hint: ToolInstallHints::for_tool("ocrmypdf"),
            });
        }

        let version = self.version(&path)?;

        Ok(ToolInfo {
            path,
            version,
            available: true,
        })
    }

    fn version(&self, path: &Path) -> Result<String> {
        let output = Command::new(path)
            .arg("--version")
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run ocrmypdf: {}", e)))?;

        if !output.status.success() {
            return Err(ForgeKitError::Other(anyhow::anyhow!(
                "ocrmypdf --version failed"
            )));
        }

        // ocrmypdf outputs version like "ocrmypdf 14.0.0" or just "14.0.0"
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        Ok(version.trim().to_string())
    }
}

impl OcrmypdfTool {
    /// Get available languages for OCR.
    ///
    /// Returns a list of language codes that Tesseract has installed.
    pub fn get_available_languages(&self, _tool_path: &Path) -> Result<Vec<String>> {
        // ocrmypdf uses tesseract's languages, so we check tesseract directly
        let tesseract_output = Command::new("tesseract")
            .arg("--list-langs")
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run tesseract: {}", e)))?;

        if !tesseract_output.status.success() {
            // Fall back to just returning 'eng' if we can't list languages
            return Ok(vec!["eng".to_string()]);
        }

        let output = String::from_utf8_lossy(&tesseract_output.stdout);
        let languages: Vec<String> = output
            .lines()
            .skip(1) // Skip header line "List of available languages..."
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if languages.is_empty() {
            Ok(vec!["eng".to_string()])
        } else {
            Ok(languages)
        }
    }

    /// Check if a specific language is available.
    pub fn is_language_available(&self, tool_path: &Path, lang: &str) -> bool {
        self.get_available_languages(tool_path)
            .map(|langs| langs.iter().any(|l| l == lang))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocrmypdf_name() {
        let tool = OcrmypdfTool;
        assert_eq!(tool.name(), "ocrmypdf");
    }

    #[test]
    fn test_ocrmypdf_probe() {
        let tool = OcrmypdfTool;
        let config = ToolConfig::default();
        // This will only pass if ocrmypdf is installed
        let result = tool.probe(&config);
        // We don't assert success here since ocrmypdf may not be installed in test environment
        // Just verify it doesn't panic
        match result {
            Ok(info) => {
                assert!(info.available);
                assert!(!info.version.is_empty());
            }
            Err(ForgeKitError::ToolNotFound { .. }) => {
                // Expected if ocrmypdf is not installed
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
