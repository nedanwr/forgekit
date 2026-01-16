//! # ImageMagick Tool Adapter
//!
//! ImageMagick is the fallback image processing tool. We use:
//! - `magick convert` (ImageMagick 7) or `convert` (ImageMagick 6)
//!
//! ## Why as Fallback?
//!
//! - More widely available, especially on Windows
//! - Full feature parity with libvips for our use cases
//! - Slower than libvips for large images
//!
//! ## Minimum Version: 7.1+

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tools::{Tool, ToolConfig, ToolInfo};
use crate::utils::error::{ForgeKitError, Result};
use crate::utils::image::ImageFormat;
use crate::utils::platform::ToolInstallHints;

/// ImageMagick tool adapter.
pub struct ImageMagickTool;

impl Tool for ImageMagickTool {
    fn name(&self) -> &'static str {
        "magick"
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

        // Try 'magick' first (ImageMagick 7), then 'convert' (ImageMagick 6)
        // Same binaries on all platforms
        let binaries = vec!["magick", "convert"];

        for binary in binaries {
            let which_output = if cfg!(target_os = "windows") {
                Command::new("where").arg(binary).output()
            } else {
                Command::new("which").arg(binary).output()
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

                        // Verify it's ImageMagick (not some other 'convert')
                        if let Ok(version) = self.version(&path) {
                            if version.to_lowercase().contains("imagemagick") {
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
        }

        Err(ForgeKitError::ToolNotFound {
            tool: "imagemagick".to_string(),
            hint: ToolInstallHints::for_tool("imagemagick"),
        })
    }

    fn version(&self, path: &Path) -> Result<String> {
        // For 'magick', use 'magick --version'
        // For 'convert', use 'convert --version'
        let output = Command::new(path).arg("--version").output().map_err(|e| {
            ForgeKitError::Other(anyhow::anyhow!("Failed to run imagemagick: {}", e))
        })?;

        if !output.status.success() {
            return Err(ForgeKitError::Other(anyhow::anyhow!(
                "imagemagick --version failed"
            )));
        }

        // Output looks like: "Version: ImageMagick 7.1.1-..."
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        Ok(version.trim().to_string())
    }
}

impl ImageMagickTool {
    /// Check if the tool path is 'magick' (v7) requiring subcommand.
    fn is_magick_v7(tool_path: &Path) -> bool {
        tool_path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with("magick"))
            .unwrap_or(false)
    }

    /// Convert image format with optional quality and strip options.
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

        // ImageMagick 7 uses 'magick convert', v6 uses 'convert' directly
        if Self::is_magick_v7(tool_path) {
            cmd.arg("convert");
        }

        cmd.arg(input);

        if let Some(q) = quality {
            cmd.arg("-quality").arg(q.to_string());
        }

        if strip {
            cmd.arg("-strip");
        }

        cmd.arg(output);

        let result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
            tool: "imagemagick".to_string(),
            stderr: format!("Failed to execute imagemagick: {}", e),
        })?;

        if !result.status.success() {
            return Err(ForgeKitError::ProcessingFailed {
                tool: "imagemagick".to_string(),
                stderr: String::from_utf8_lossy(&result.stderr).to_string(),
            });
        }

        Ok(())
    }

    /// Resize image preserving aspect ratio.
    pub fn resize(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<()> {
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

        let mut cmd = Command::new(tool_path);

        if Self::is_magick_v7(tool_path) {
            cmd.arg("convert");
        }

        cmd.arg(input);
        cmd.arg("-resize").arg(&size);
        cmd.arg(output);

        let result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
            tool: "imagemagick".to_string(),
            stderr: format!("Failed to execute imagemagick: {}", e),
        })?;

        if !result.status.success() {
            return Err(ForgeKitError::ProcessingFailed {
                tool: "imagemagick".to_string(),
                stderr: String::from_utf8_lossy(&result.stderr).to_string(),
            });
        }

        Ok(())
    }

    /// Strip metadata from image.
    pub fn strip_metadata(&self, tool_path: &Path, input: &Path, output: &Path) -> Result<()> {
        let mut cmd = Command::new(tool_path);

        if Self::is_magick_v7(tool_path) {
            cmd.arg("convert");
        }

        cmd.arg(input);
        cmd.arg("-strip");
        cmd.arg(output);

        let result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
            tool: "imagemagick".to_string(),
            stderr: format!("Failed to execute imagemagick: {}", e),
        })?;

        if !result.status.success() {
            return Err(ForgeKitError::ProcessingFailed {
                tool: "imagemagick".to_string(),
                stderr: String::from_utf8_lossy(&result.stderr).to_string(),
            });
        }

        Ok(())
    }

    /// Build plan command string for convert operation.
    pub fn plan_convert(input: &Path, output: &Path, quality: Option<u8>, strip: bool) -> String {
        let mut parts = vec!["magick", "convert"];
        parts.push(&*Box::leak(input.display().to_string().into_boxed_str()));

        let quality_str;
        if let Some(q) = quality {
            parts.push("-quality");
            quality_str = q.to_string();
            parts.push(&quality_str);
        }

        if strip {
            parts.push("-strip");
        }

        // Build the string manually to avoid lifetime issues
        let mut result = String::from("magick convert ");
        result.push_str(&input.display().to_string());

        if let Some(q) = quality {
            result.push_str(&format!(" -quality {}", q));
        }

        if strip {
            result.push_str(" -strip");
        }

        result.push(' ');
        result.push_str(&output.display().to_string());

        result
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
            "magick convert {} -resize {} {}",
            input.display(),
            size,
            output.display()
        )
    }

    /// Build plan command string for strip operation.
    pub fn plan_strip(input: &Path, output: &Path) -> String {
        format!(
            "magick convert {} -strip {}",
            input.display(),
            output.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imagemagick_name() {
        let tool = ImageMagickTool;
        assert_eq!(tool.name(), "magick");
    }

    #[test]
    fn test_imagemagick_probe() {
        let tool = ImageMagickTool;
        let config = ToolConfig::default();
        match tool.probe(&config) {
            Ok(info) => {
                assert!(info.available);
                assert!(!info.version.is_empty());
                // Verify it's ImageMagick
                assert!(info.version.to_lowercase().contains("imagemagick"));
            }
            Err(ForgeKitError::ToolNotFound { .. }) => {
                // Expected if ImageMagick is not installed
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_is_magick_v7() {
        assert!(ImageMagickTool::is_magick_v7(Path::new("/usr/bin/magick")));
        assert!(ImageMagickTool::is_magick_v7(Path::new("magick.exe")));
        assert!(!ImageMagickTool::is_magick_v7(Path::new(
            "/usr/bin/convert"
        )));
    }

    #[test]
    fn test_plan_convert() {
        let plan = ImageMagickTool::plan_convert(
            Path::new("input.jpg"),
            Path::new("output.webp"),
            Some(80),
            true,
        );
        assert!(plan.contains("magick convert"));
        assert!(plan.contains("input.jpg"));
        assert!(plan.contains("-quality 80"));
        assert!(plan.contains("-strip"));
    }

    #[test]
    fn test_plan_resize() {
        let plan = ImageMagickTool::plan_resize(
            Path::new("input.jpg"),
            Path::new("output.jpg"),
            Some(800),
            None,
        );
        assert!(plan.contains("magick convert"));
        assert!(plan.contains("-resize 800x"));
    }
}
