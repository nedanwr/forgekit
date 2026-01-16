//! # ExifTool Tool Adapter
//!
//! ExifTool is a Perl command-line tool for reading and writing metadata in files.
//! We use it for:
//! - Reading PDF metadata (title, author, subject, keywords, etc.)
//! - Writing/updating PDF metadata
//! - Bulk metadata operations
//!
//! ## Why ExifTool?
//!
//! ExifTool is the most comprehensive metadata tool available:
//! - Supports 400+ file formats including PDF
//! - Can read/write virtually any metadata field
//! - Cross-platform (Windows, macOS, Linux)
//! - Stable CLI interface
//!
//! ## Minimum Version
//!
//! ExifTool 12.0+ is recommended. Older versions may work but aren't tested.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tools::{Tool, ToolConfig, ToolInfo};
use crate::utils::error::{ForgeKitError, Result};
use crate::utils::platform::ToolInstallHints;

/// ExifTool adapter.
///
/// Implements the `Tool` trait for ExifTool, handling detection, version checking,
/// and command construction for metadata operations.
pub struct ExiftoolTool;

impl Tool for ExiftoolTool {
    fn name(&self) -> &'static str {
        "exiftool"
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
            Command::new("where").arg("exiftool").output()
        } else {
            Command::new("which").arg("exiftool").output()
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
                    PathBuf::from("exiftool")
                }
            }
            _ => PathBuf::from("exiftool"),
        };

        // Verify it works
        let output =
            Command::new(&path)
                .arg("-ver")
                .output()
                .map_err(|_| ForgeKitError::ToolNotFound {
                    tool: "exiftool".to_string(),
                    hint: ToolInstallHints::for_tool("exiftool"),
                })?;

        if !output.status.success() {
            return Err(ForgeKitError::ToolNotFound {
                tool: "exiftool".to_string(),
                hint: ToolInstallHints::for_tool("exiftool"),
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
            .arg("-ver")
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run exiftool: {}", e)))?;

        if !output.status.success() {
            return Err(ForgeKitError::Other(anyhow::anyhow!(
                "exiftool -ver failed"
            )));
        }

        // exiftool outputs just the version number like "12.40"
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        Ok(version.trim().to_string())
    }
}

/// PDF metadata fields supported for read/write operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfMetadataField {
    Title,
    Author,
    Subject,
    Keywords,
    Creator,
    Producer,
    CreationDate,
    ModifyDate,
    /// Custom XMP or PDF metadata field
    Custom(String),
}

impl PdfMetadataField {
    /// Convert field name to exiftool tag name
    pub fn to_exiftool_tag(&self) -> String {
        match self {
            PdfMetadataField::Title => "Title".to_string(),
            PdfMetadataField::Author => "Author".to_string(),
            PdfMetadataField::Subject => "Subject".to_string(),
            PdfMetadataField::Keywords => "Keywords".to_string(),
            PdfMetadataField::Creator => "Creator".to_string(),
            PdfMetadataField::Producer => "Producer".to_string(),
            PdfMetadataField::CreationDate => "CreateDate".to_string(),
            PdfMetadataField::ModifyDate => "ModifyDate".to_string(),
            PdfMetadataField::Custom(name) => name.clone(),
        }
    }
}

impl std::str::FromStr for PdfMetadataField {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "title" => PdfMetadataField::Title,
            "author" => PdfMetadataField::Author,
            "subject" => PdfMetadataField::Subject,
            "keywords" => PdfMetadataField::Keywords,
            "creator" => PdfMetadataField::Creator,
            "producer" => PdfMetadataField::Producer,
            "creationdate" | "createdate" | "creation_date" => PdfMetadataField::CreationDate,
            "modifydate" | "moddate" | "modify_date" => PdfMetadataField::ModifyDate,
            _ => PdfMetadataField::Custom(s.to_string()),
        })
    }
}

impl ExiftoolTool {
    /// Read all PDF metadata from a file as JSON.
    ///
    /// Returns raw JSON output from exiftool for maximum flexibility.
    pub fn read_metadata_json(&self, tool_path: &Path, pdf_path: &Path) -> Result<String> {
        let output = Command::new(tool_path)
            .arg("-json")
            .arg("-PDF:all")
            .arg("-XMP:all")
            .arg(pdf_path)
            .output()
            .map_err(|e| ForgeKitError::ProcessingFailed {
                tool: "exiftool".to_string(),
                stderr: format!("Failed to run exiftool: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "exiftool".to_string(),
                stderr: stderr.to_string(),
            });
        }

        let json = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(json)
    }

    /// Read a specific metadata field from a PDF.
    pub fn read_field(&self, tool_path: &Path, pdf_path: &Path, field: &str) -> Result<String> {
        let tag = format!("-{}", field);
        let output = Command::new(tool_path)
            .arg("-s") // Short format
            .arg("-s") // Even shorter (just value)
            .arg("-s") // Shortest (no field name)
            .arg(&tag)
            .arg(pdf_path)
            .output()
            .map_err(|e| ForgeKitError::ProcessingFailed {
                tool: "exiftool".to_string(),
                stderr: format!("Failed to run exiftool: {}", e),
            })?;

        // Note: exiftool returns success even if field doesn't exist
        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(value)
    }

    /// Write metadata to a PDF file.
    ///
    /// Takes a vector of (field, value) pairs.
    /// By default, exiftool modifies the file in place and creates a backup.
    /// Use `overwrite_original` to skip creating a backup.
    pub fn write_metadata(
        &self,
        tool_path: &Path,
        pdf_path: &Path,
        metadata: &[(String, String)],
        overwrite_original: bool,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);

        if overwrite_original {
            cmd.arg("-overwrite_original");
        }

        for (field, value) in metadata {
            let field_obj: PdfMetadataField = field.parse().unwrap();
            let tag = field_obj.to_exiftool_tag();
            cmd.arg(format!("-{}={}", tag, value));
        }

        cmd.arg(pdf_path);

        let output = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
            tool: "exiftool".to_string(),
            stderr: format!("Failed to run exiftool: {}", e),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "exiftool".to_string(),
                stderr: stderr.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exiftool_name() {
        let tool = ExiftoolTool;
        assert_eq!(tool.name(), "exiftool");
    }

    #[test]
    fn test_exiftool_probe() {
        let tool = ExiftoolTool;
        let config = ToolConfig::default();
        // This will only pass if exiftool is installed
        let result = tool.probe(&config);
        // We don't assert success here since exiftool may not be installed in test environment
        // Just verify it doesn't panic
        match result {
            Ok(info) => {
                assert!(info.available);
                assert!(!info.version.is_empty());
            }
            Err(ForgeKitError::ToolNotFound { .. }) => {
                // Expected if exiftool is not installed
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_pdf_metadata_field_parsing() {
        assert_eq!(
            "title".parse::<PdfMetadataField>().unwrap(),
            PdfMetadataField::Title
        );
        assert_eq!(
            "Title".parse::<PdfMetadataField>().unwrap(),
            PdfMetadataField::Title
        );
        assert_eq!(
            "author".parse::<PdfMetadataField>().unwrap(),
            PdfMetadataField::Author
        );
        assert_eq!(
            "CustomField".parse::<PdfMetadataField>().unwrap(),
            PdfMetadataField::Custom("CustomField".to_string())
        );
    }

    #[test]
    fn test_pdf_metadata_field_to_exiftool_tag() {
        assert_eq!(PdfMetadataField::Title.to_exiftool_tag(), "Title");
        assert_eq!(PdfMetadataField::Author.to_exiftool_tag(), "Author");
        assert_eq!(
            PdfMetadataField::CreationDate.to_exiftool_tag(),
            "CreateDate"
        );
        assert_eq!(
            PdfMetadataField::Custom("MyField".to_string()).to_exiftool_tag(),
            "MyField"
        );
    }
}
