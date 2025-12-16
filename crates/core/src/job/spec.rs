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
    // TODO: Add more job types as we implement them:
    // PdfCompress { input: PathBuf, output: PathBuf, level: CompressionLevel },
    // PdfOcr { input: PathBuf, output: PathBuf, language: String },
    // ImageConvert { input: PathBuf, output: PathBuf, format: ImageFormat },
    // etc.
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
        }
    }
}
