//! # qpdf Tool Adapter
//!
//! qpdf is a command-line tool for manipulating PDF files. We use it for:
//! - Merging PDFs
//! - Splitting/extracting pages
//! - Linearizing PDFs (optimizing for web viewing)
//!
//! ## Why qpdf?
//!
//! qpdf is fast, reliable, and handles most PDF operations we need. It's widely
//! available via package managers (brew, apt, pacman, etc.) and has a stable CLI.
//!
//! ## Minimum Version
//!
//! qpdf 10.0+ is required. Older versions may work but aren't tested.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tools::{Tool, ToolConfig, ToolInfo};
use crate::utils::error::{ForgeKitError, Result};
use crate::utils::platform::ToolInstallHints;

/// qpdf tool adapter.
///
/// Implements the `Tool` trait for qpdf, handling detection, version checking,
/// and (in the executor) command construction and execution.
pub struct QpdfTool;

impl Tool for QpdfTool {
    fn name(&self) -> &'static str {
        "qpdf"
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

        // Probe PATH
        let which_output = if cfg!(target_os = "windows") {
            Command::new("where").arg("qpdf").output()
        } else {
            Command::new("which").arg("qpdf").output()
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
                    PathBuf::from("qpdf")
                }
            }
            _ => PathBuf::from("qpdf"),
        };

        // Verify it works
        let output = Command::new(&path).arg("--version").output().map_err(|_| {
            ForgeKitError::ToolNotFound {
                tool: "qpdf".to_string(),
                hint: ToolInstallHints::for_tool("qpdf"),
            }
        })?;

        if !output.status.success() {
            return Err(ForgeKitError::ToolNotFound {
                tool: "qpdf".to_string(),
                hint: ToolInstallHints::for_tool("qpdf"),
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
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run qpdf: {}", e)))?;

        if !output.status.success() {
            return Err(ForgeKitError::Other(anyhow::anyhow!(
                "qpdf --version failed"
            )));
        }

        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        Ok(version.trim().to_string())
    }
}

impl QpdfTool {
    /// Get the number of pages in a PDF file.
    pub fn get_page_count(&self, tool_path: &Path, pdf_path: &Path) -> Result<u32> {
        let output = Command::new(tool_path)
            .arg("--show-npages")
            .arg(pdf_path)
            .output()
            .map_err(|e| ForgeKitError::ProcessingFailed {
                tool: "qpdf".to_string(),
                stderr: format!("Failed to run qpdf --show-npages: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "qpdf".to_string(),
                stderr: stderr.to_string(),
            });
        }

        let count_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        count_str.parse::<u32>().map_err(|_| {
            ForgeKitError::Other(anyhow::anyhow!(
                "Failed to parse page count from qpdf output: '{}'",
                count_str
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qpdf_name() {
        let tool = QpdfTool;
        assert_eq!(tool.name(), "qpdf");
    }

    #[test]
    fn test_qpdf_probe() {
        let tool = QpdfTool;
        let config = ToolConfig::default();
        // This will only pass if qpdf is installed
        let result = tool.probe(&config);
        // We don't assert success here since qpdf may not be installed in test environment
        // Just verify it doesn't panic
        match result {
            Ok(info) => {
                assert!(info.available);
                assert!(!info.version.is_empty());
            }
            Err(ForgeKitError::ToolNotFound { .. }) => {
                // Expected if qpdf is not installed
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_get_page_count_with_empty_pdf() {
        let tool = QpdfTool;
        let config = ToolConfig::default();

        // Skip test if qpdf is not installed
        let tool_info = match tool.probe(&config) {
            Ok(info) => info,
            Err(ForgeKitError::ToolNotFound { .. }) => {
                println!("Skipping test_get_page_count_with_empty_pdf: qpdf not installed");
                return;
            }
            Err(e) => panic!("Unexpected error probing qpdf: {:?}", e),
        };

        // Use qpdf to create a valid empty PDF (0 pages)
        let temp_dir = std::env::temp_dir();
        let temp_pdf = temp_dir.join("forgekit_test_page_count_empty.pdf");

        // Create a valid 0-page PDF using qpdf --empty
        let create_result = Command::new(&tool_info.path)
            .arg("--empty")
            .arg(&temp_pdf)
            .output();

        match create_result {
            Ok(output) if output.status.success() => {
                // PDF created successfully, now test page count
                let result = tool.get_page_count(&tool_info.path, &temp_pdf);

                // Cleanup
                let _ = std::fs::remove_file(&temp_pdf);

                // Verify result - qpdf --empty creates a PDF with 0 pages
                match result {
                    Ok(count) => assert_eq!(count, 0, "Expected 0 pages in empty PDF"),
                    Err(e) => panic!("Failed to get page count: {:?}", e),
                }
            }
            _ => {
                // Cleanup and skip if we can't create the test PDF
                let _ = std::fs::remove_file(&temp_pdf);
                println!("Skipping test: couldn't create test PDF with qpdf --empty");
            }
        }
    }

    #[test]
    fn test_get_page_count_nonexistent_file() {
        let tool = QpdfTool;
        let config = ToolConfig::default();

        // Skip test if qpdf is not installed
        let tool_info = match tool.probe(&config) {
            Ok(info) => info,
            Err(ForgeKitError::ToolNotFound { .. }) => {
                println!("Skipping test_get_page_count_nonexistent_file: qpdf not installed");
                return;
            }
            Err(e) => panic!("Unexpected error probing qpdf: {:?}", e),
        };

        let nonexistent = PathBuf::from("/nonexistent/file.pdf");
        let result = tool.get_page_count(&tool_info.path, &nonexistent);

        // Should fail with ProcessingFailed error
        assert!(result.is_err());
        match result {
            Err(ForgeKitError::ProcessingFailed { tool, .. }) => {
                assert_eq!(tool, "qpdf");
            }
            Err(e) => panic!("Expected ProcessingFailed error, got: {:?}", e),
            Ok(_) => panic!("Expected error for nonexistent file"),
        }
    }
}
