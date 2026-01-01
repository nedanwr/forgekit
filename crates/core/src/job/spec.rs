//! # Job Specifications
//!
//! A `JobSpec` describes what you want to do - it's pure data, no execution logic.
//! Think of it as a recipe: "merge these PDFs", "extract these pages", etc.
//!
//! ## Why Separate Specs from Execution?
//!
//! Separating specs from execution gives us:
//! - **Testability**: Easy to test parsing/validation without running tools
//! - **Transparency**: `--plan` flag can show what would happen without doing it
//! - **Serialization**: Could save jobs to disk, queue them, etc. (future)
//! - **Clarity**: The executor code is cleaner when it just focuses on "how", not "what"

use crate::utils::pages::PageSpec;
use std::path::PathBuf;

/// A job specification describing what operation to perform.
///
/// This is pure data - it doesn't execute anything. The executor (`job::executor`)
/// takes a `JobSpec` and actually runs it by calling the appropriate external tools.
///
/// ## Adding a New Job Type
///
/// To add a new operation (e.g., `PdfCompress`):
///
/// 1. Add a variant to this enum with the necessary fields
/// 2. Add a match arm in `executor.rs` to handle it
/// 3. Add a CLI subcommand in `cli/src/commands/`
/// 4. Update `description()` to include it
#[derive(Debug, Clone)]
pub enum JobSpec {
    /// Merge multiple PDFs into a single file.
    ///
    /// `linearize` optimizes the PDF for fast web viewing by reorganizing
    /// the internal structure (useful for large PDFs viewed in browsers).
    PdfMerge {
        /// Input PDF files to merge (at least 2 required).
        inputs: Vec<PathBuf>,
        /// Output PDF file path.
        output: PathBuf,
        /// Whether to linearize the output PDF.
        linearize: bool,
    },
    /// Split a PDF into separate files by page ranges.
    ///
    /// Uses `PageSpec` to define which pages to extract. Can extract multiple
    /// ranges (e.g., pages 1-5 and 10-20) into separate files.
    PdfSplit {
        /// Input PDF file to split.
        input: PathBuf,
        /// Directory where split files will be saved.
        output_dir: PathBuf,
        /// Page specifications defining which pages to extract.
        pages: Vec<PageSpec>,
    },
    /// Compress a PDF using Ghostscript with compression level control.
    ///
    /// Uses Ghostscript with optimized QFactor settings for fast compression.
    PdfCompress {
        /// Input PDF file to compress.
        input: PathBuf,
        /// Output PDF file path.
        output: PathBuf,
        /// Compression level (light, standard, high).
        /// Default is "standard" (balanced compression).
        level: String,
    },
    /// Linearize a PDF for fast web viewing.
    ///
    /// Optimizes the PDF's internal structure for fast web viewing by reorganizing
    /// the internal structure. This is a standalone operation separate from merge.
    PdfLinearize {
        /// Input PDF file to linearize.
        input: PathBuf,
        /// Output PDF file path.
        output: PathBuf,
    },
    /// Reorder pages in a PDF.
    ///
    /// Reorders pages according to the specified order (1-indexed page numbers).
    PdfReorder {
        /// Input PDF file to reorder.
        input: PathBuf,
        /// Output PDF file path.
        output: PathBuf,
        /// Page order (1-indexed). Example: [3, 1, 2] means page 3, then 1, then 2.
        page_order: Vec<u32>,
    },
    /// Extract specific pages from a PDF.
    ///
    /// Extracts pages matching the page specification. Can output to a single PDF
    /// file or separate image files per page.
    PdfExtract {
        /// Input PDF file to extract from.
        input: PathBuf,
        /// Output PDF file path (when format is "pdf").
        output: Option<PathBuf>,
        /// Output directory path (when format is "images").
        output_dir: Option<PathBuf>,
        /// Page specifications defining which pages to extract.
        pages: Vec<PageSpec>,
        /// Output format: "pdf" or "images".
        format: String,
    },
}

impl JobSpec {
    /// Get a human-readable description of what this job does.
    ///
    /// Useful for logging, progress messages, and `--plan` output.
    pub fn description(&self) -> String {
        match self {
            JobSpec::PdfMerge {
                inputs, linearize, ..
            } => {
                let linearize_str = if *linearize { " (linearized)" } else { "" };
                format!("Merge {} PDFs{}", inputs.len(), linearize_str)
            }
            JobSpec::PdfSplit { pages, .. } => {
                format!("Split PDF into {} page spec(s)", pages.len())
            }
            JobSpec::PdfCompress { level, .. } => {
                format!("Compress PDF with {} compression", level)
            }
            JobSpec::PdfLinearize { .. } => "Linearize PDF".to_string(),
            JobSpec::PdfReorder { page_order, .. } => {
                format!("Reorder PDF pages ({} pages)", page_order.len())
            }
            JobSpec::PdfExtract { pages, format, .. } => {
                format!(
                    "Extract {} page spec(s) from PDF as {}",
                    pages.len(),
                    format
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::pages::PageSpec;

    #[test]
    fn test_pdf_compress_description_with_level() {
        let spec = JobSpec::PdfCompress {
            input: PathBuf::from("input.pdf"),
            output: PathBuf::from("output.pdf"),
            level: "high".to_string(),
        };
        assert_eq!(spec.description(), "Compress PDF with high compression");
    }

    #[test]
    fn test_pdf_compress_description_standard() {
        let spec = JobSpec::PdfCompress {
            input: PathBuf::from("input.pdf"),
            output: PathBuf::from("output.pdf"),
            level: "standard".to_string(),
        };
        assert_eq!(spec.description(), "Compress PDF with standard compression");
    }

    #[test]
    fn test_pdf_linearize_description() {
        let spec = JobSpec::PdfLinearize {
            input: PathBuf::from("input.pdf"),
            output: PathBuf::from("output.pdf"),
        };
        assert_eq!(spec.description(), "Linearize PDF");
    }

    #[test]
    fn test_pdf_reorder_description() {
        let spec = JobSpec::PdfReorder {
            input: PathBuf::from("input.pdf"),
            output: PathBuf::from("output.pdf"),
            page_order: vec![3, 1, 2],
        };
        assert_eq!(spec.description(), "Reorder PDF pages (3 pages)");
    }

    #[test]
    fn test_pdf_extract_description() {
        let pages = vec![
            PageSpec::Range {
                start: 1,
                end: Some(5),
            },
            PageSpec::Page(10),
        ];
        let spec = JobSpec::PdfExtract {
            input: PathBuf::from("input.pdf"),
            output: Some(PathBuf::from("output.pdf")),
            output_dir: None,
            pages,
            format: "pdf".to_string(),
        };
        assert_eq!(spec.description(), "Extract 2 page spec(s) from PDF as pdf");
    }
}
