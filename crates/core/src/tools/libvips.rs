//! # libvips Tool Adapter
//!
//! libvips is a fast image processing library. We use the `vips` CLI for:
//! - Format conversion (vips copy)
//! - Resizing (vipsthumbnail)
//! - Metadata stripping (vips copy with [strip] suffix)
//!
//! ## Why libvips?
//!
//! - 10-100x faster than ImageMagick for large images
//! - Streaming architecture uses constant memory
//! - Excellent quality with proper resampling
//!
//! ## Minimum Version: 8.12+

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tools::{Tool, ToolConfig, ToolInfo};
use crate::utils::error::{ForgeKitError, Result};
use crate::utils::image::ImageFormat;
use crate::utils::platform::ToolInstallHints;

/// libvips tool adapter.
pub struct LibvipsTool;

impl Tool for LibvipsTool {
    fn name(&self) -> &'static str {
        "vips"
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

        // Probe PATH for 'vips' command
        let which_output = if cfg!(target_os = "windows") {
            Command::new("where").arg("vips").output()
        } else {
            Command::new("which").arg("vips").output()
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
                    PathBuf::from("vips")
                }
            }
            _ => PathBuf::from("vips"),
        };

        // Verify it works
        let output = Command::new(&path).arg("--version").output().map_err(|_| {
            ForgeKitError::ToolNotFound {
                tool: "vips".to_string(),
                hint: ToolInstallHints::for_tool("libvips"),
            }
        })?;

        if !output.status.success() {
            return Err(ForgeKitError::ToolNotFound {
                tool: "vips".to_string(),
                hint: ToolInstallHints::for_tool("libvips"),
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
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run vips: {}", e)))?;

        if !output.status.success() {
            return Err(ForgeKitError::Other(anyhow::anyhow!(
                "vips --version failed"
            )));
        }

        // vips --version outputs something like: "vips-8.15.0"
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        Ok(version.trim().to_string())
    }
}

impl LibvipsTool {
    /// Convert image format with optional quality and strip options.
    ///
    /// Uses `vips copy input output[Q=quality,strip]` syntax.
    pub fn convert(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        _format: &ImageFormat,
        quality: Option<u8>,
        strip: bool,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("copy");
        cmd.arg(input);

        // Build output with options suffix
        let mut options = Vec::new();
        if let Some(q) = quality {
            options.push(format!("Q={}", q));
        }
        if strip {
            options.push("strip".to_string());
        }

        let output_spec = if options.is_empty() {
            output.display().to_string()
        } else {
            format!("{}[{}]", output.display(), options.join(","))
        };
        cmd.arg(&output_spec);

        let result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
            tool: "vips".to_string(),
            stderr: format!("Failed to execute vips: {}", e),
        })?;

        if !result.status.success() {
            return Err(ForgeKitError::ProcessingFailed {
                tool: "vips".to_string(),
                stderr: String::from_utf8_lossy(&result.stderr).to_string(),
            });
        }

        Ok(())
    }

    /// Resize image using vipsthumbnail.
    ///
    /// Preserves aspect ratio by default.
    pub fn resize(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<()> {
        // vipsthumbnail is in the same directory as vips
        let thumbnail_path = tool_path.with_file_name(if cfg!(target_os = "windows") {
            "vipsthumbnail.exe"
        } else {
            "vipsthumbnail"
        });

        let size = match (width, height) {
            (Some(w), Some(h)) => format!("{}x{}", w, h),
            (Some(w), None) => format!("{}x", w),
            (None, Some(h)) => format!("x{}", h),
            (None, None) => {
                return Err(ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: "Width or height required for resize".to_string(),
                });
            }
        };

        let mut cmd = Command::new(&thumbnail_path);
        cmd.arg(input);
        cmd.arg("-s").arg(&size);
        cmd.arg("-o").arg(output);

        let result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
            tool: "vipsthumbnail".to_string(),
            stderr: format!("Failed to execute vipsthumbnail: {}", e),
        })?;

        if !result.status.success() {
            return Err(ForgeKitError::ProcessingFailed {
                tool: "vipsthumbnail".to_string(),
                stderr: String::from_utf8_lossy(&result.stderr).to_string(),
            });
        }

        Ok(())
    }

    /// Strip metadata from image.
    pub fn strip_metadata(&self, tool_path: &Path, input: &Path, output: &Path) -> Result<()> {
        let output_spec = format!("{}[strip]", output.display());

        let mut cmd = Command::new(tool_path);
        cmd.arg("copy").arg(input).arg(&output_spec);

        let result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
            tool: "vips".to_string(),
            stderr: format!("Failed to execute vips: {}", e),
        })?;

        if !result.status.success() {
            return Err(ForgeKitError::ProcessingFailed {
                tool: "vips".to_string(),
                stderr: String::from_utf8_lossy(&result.stderr).to_string(),
            });
        }

        Ok(())
    }

    /// Build plan command string for convert operation.
    pub fn plan_convert(input: &Path, output: &Path, quality: Option<u8>, strip: bool) -> String {
        let mut options = Vec::new();
        if let Some(q) = quality {
            options.push(format!("Q={}", q));
        }
        if strip {
            options.push("strip".to_string());
        }

        let output_spec = if options.is_empty() {
            output.display().to_string()
        } else {
            format!("{}[{}]", output.display(), options.join(","))
        };

        format!("vips copy {} {}", input.display(), output_spec)
    }

    /// Build plan command string for resize operation.
    pub fn plan_resize(
        input: &Path,
        output: &Path,
        width: Option<u32>,
        height: Option<u32>,
    ) -> String {
        let size = match (width, height) {
            (Some(w), Some(h)) => format!("{}x{}", w, h),
            (Some(w), None) => format!("{}x", w),
            (None, Some(h)) => format!("x{}", h),
            (None, None) => "?".to_string(),
        };

        format!(
            "vipsthumbnail {} -s {} -o {}",
            input.display(),
            size,
            output.display()
        )
    }

    /// Build plan command string for strip operation.
    pub fn plan_strip(input: &Path, output: &Path) -> String {
        format!("vips copy {} {}[strip]", input.display(), output.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_libvips_name() {
        let tool = LibvipsTool;
        assert_eq!(tool.name(), "vips");
    }

    #[test]
    fn test_libvips_probe() {
        let tool = LibvipsTool;
        let config = ToolConfig::default();
        match tool.probe(&config) {
            Ok(info) => {
                assert!(info.available);
                assert!(!info.version.is_empty());
            }
            Err(ForgeKitError::ToolNotFound { .. }) => {
                // Expected if libvips is not installed
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_plan_convert() {
        let plan = LibvipsTool::plan_convert(
            Path::new("input.jpg"),
            Path::new("output.webp"),
            Some(80),
            true,
        );
        assert!(plan.contains("vips copy"));
        assert!(plan.contains("input.jpg"));
        assert!(plan.contains("Q=80"));
        assert!(plan.contains("strip"));
    }

    #[test]
    fn test_plan_resize() {
        let plan = LibvipsTool::plan_resize(
            Path::new("input.jpg"),
            Path::new("output.jpg"),
            Some(800),
            None,
        );
        assert!(plan.contains("vipsthumbnail"));
        assert!(plan.contains("-s 800x"));
    }
}
