//! # Job Execution
//!
//! This module takes `JobSpec`s and actually runs them by calling external tools.
//! It handles tool detection, command construction, execution, progress reporting,
//! and error handling.
//!
//! ## Execution Flow
//!
//! 1. **Validate inputs** - Check files exist, paths are valid, etc.
//! 2. **Probe for tools** - Find the required external tool (qpdf, ffmpeg, etc.)
//! 3. **Build command** - Construct the command-line invocation
//! 4. **Report progress** - Emit progress events as work happens
//! 5. **Handle results** - Return success message or detailed error
//!
//! ## Plan Mode (`--plan` flag)
//!
//! When `plan_only` is true, we skip execution and just return the command that
//! would be run. This is great for transparency and debugging.

use std::path::PathBuf;
use std::process::Command;

use crate::job::progress::{
    new_job_id, ErrorInfo, JobResult, ProgressEvent, ProgressInfo, ProgressReporter,
};
use crate::job::JobSpec;
use crate::presets::get_compression_strategy;
use crate::tools::gs::GsTool;
use crate::tools::qpdf::QpdfTool;
use crate::tools::{Tool, ToolConfig};
use crate::utils::error::{ForgeKitError, Result};
use crate::utils::pages::PageSpec;
use std::time::Instant;

/// Execute a job specification without progress reporting.
///
/// Convenience wrapper that uses `NoOpProgressReporter`. Use this when you
/// don't need JSON output (normal CLI usage).
pub fn execute_job(spec: &JobSpec, plan_only: bool) -> Result<String> {
    execute_job_with_progress(spec, plan_only, &crate::job::progress::NoOpProgressReporter)
}

/// Execute a job specification with progress reporting.
///
/// This is the main entry point for running jobs. It:
/// - Validates the job spec
/// - Finds the required external tools
/// - Executes the job (or shows plan if `plan_only` is true)
/// - Reports progress via the `ProgressReporter`
/// - Returns a success message or detailed error
///
/// # Arguments
///
/// * `spec` - The job to execute (merge PDFs, split pages, etc.)
/// * `plan_only` - If true, show what would be done without actually doing it
/// * `reporter` - Progress reporter (JSON, no-op, or future GUI reporter)
///
/// # Returns
///
/// Success message string, or an error with details about what went wrong.
pub fn execute_job_with_progress(
    spec: &JobSpec,
    plan_only: bool,
    reporter: &dyn ProgressReporter,
) -> Result<String> {
    match spec {
        JobSpec::PdfMerge {
            inputs,
            output,
            linearize,
        } => execute_pdf_merge_with_progress(inputs, output, *linearize, plan_only, reporter),
        JobSpec::PdfSplit {
            input,
            output_dir,
            pages,
        } => execute_pdf_split(input, output_dir, pages, plan_only),
        JobSpec::PdfCompress {
            input,
            output,
            level,
        } => execute_pdf_compress(input, output, &level, plan_only),
        JobSpec::PdfLinearize { input, output } => execute_pdf_linearize(input, output, plan_only),
        JobSpec::PdfReorder {
            input,
            output,
            page_order,
        } => execute_pdf_reorder(input, output, page_order, plan_only),
        JobSpec::PdfExtract {
            input,
            output,
            output_dir,
            pages,
            format,
        } => execute_pdf_extract(
            input,
            output.as_ref(),
            output_dir.as_ref(),
            pages,
            &format,
            plan_only,
        ),
    }
}

fn execute_pdf_merge_with_progress(
    inputs: &[PathBuf],
    output: &PathBuf,
    linearize: bool,
    plan_only: bool,
    reporter: &dyn ProgressReporter,
) -> Result<String> {
    let job_id = new_job_id();
    let start_time = Instant::now();

    // Emit start progress
    reporter.report(&ProgressEvent::Progress {
        version: 1,
        job_id: job_id.clone(),
        progress: ProgressInfo {
            current: 0,
            total: inputs.len() as u32,
            percent: 0,
            stage: Some("starting".to_string()),
        },
        message: format!("Starting merge of {} PDFs", inputs.len()),
    });
    if inputs.len() < 2 {
        return Err(ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "At least 2 input PDFs required for merge".to_string(),
        });
    }

    // Probe for qpdf
    let tool = QpdfTool;
    let config = ToolConfig::default();
    let tool_info = tool.probe(&config)?;

    if plan_only {
        // Generate plan showing the command that would be run
        let mut cmd_parts = vec!["qpdf".to_string()];

        if linearize {
            cmd_parts.push("--linearize".to_string());
        }

        cmd_parts.push("--empty".to_string());
        cmd_parts.push("--pages".to_string());

        for input in inputs {
            cmd_parts.push(format!("{} 1-z", input.display()));
        }

        cmd_parts.push("--".to_string());
        cmd_parts.push(output.display().to_string());

        return Ok(cmd_parts.join(" "));
    }

    // Build qpdf command
    let mut cmd = Command::new(&tool_info.path);

    if linearize {
        cmd.arg("--linearize");
    }

    cmd.arg("--empty");
    cmd.arg("--pages");

    for input in inputs {
        if !input.exists() {
            return Err(ForgeKitError::InvalidInput {
                path: input.clone(),
                reason: "Input file does not exist".to_string(),
            });
        }
        cmd.arg(format!("{} 1-z", input.display()));
    }

    cmd.arg("--");
    cmd.arg(output);

    // Execute
    let output_result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
        tool: "qpdf".to_string(),
        stderr: format!("Failed to execute: {}", e),
    })?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        let error = ForgeKitError::ProcessingFailed {
            tool: "qpdf".to_string(),
            stderr: stderr.to_string(),
        };

        // Emit error event
        reporter.report(&ProgressEvent::Error {
            version: 1,
            job_id: job_id.clone(),
            error: ErrorInfo {
                code: "PROCESSING_FAILED".to_string(),
                message: error.to_string(),
                hint: "Check that input files are valid PDFs".to_string(),
            },
        });

        return Err(error);
    }

    // Emit progress update
    reporter.report(&ProgressEvent::Progress {
        version: 1,
        job_id: job_id.clone(),
        progress: ProgressInfo {
            current: inputs.len() as u32,
            total: inputs.len() as u32,
            percent: 100,
            stage: Some("merging".to_string()),
        },
        message: format!("Merged {} PDFs", inputs.len()),
    });

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let size_bytes = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);

    // Emit complete event
    reporter.report(&ProgressEvent::Complete {
        version: 1,
        job_id: job_id.clone(),
        result: JobResult {
            output: output.display().to_string(),
            size_bytes,
            duration_ms,
        },
    });

    Ok(format!(
        "Successfully merged {} PDFs to {}",
        inputs.len(),
        output.display()
    ))
}

fn execute_pdf_split(
    input: &PathBuf,
    output_dir: &PathBuf,
    pages: &[PageSpec],
    plan_only: bool,
) -> Result<String> {
    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.clone(),
            reason: "Input file does not exist".to_string(),
        });
    }

    // Probe for qpdf
    let tool = QpdfTool;
    let config = ToolConfig::default();
    let tool_info = tool.probe(&config)?;

    // For now, we'll assume total_pages = 100 (in real implementation, we'd query this)
    // This is a simplified version - in production we'd need to get actual page count
    let total_pages = 100; // TODO: Get actual page count from PDF

    if plan_only {
        // Generate plan showing the command that would be run
        let qpdf_pages = PageSpec::to_qpdf_pages(pages, total_pages)?;
        let output_file = output_dir.join("split_output.pdf");
        return Ok(format!(
            "qpdf {} --pages {} -- {}",
            input.display(),
            qpdf_pages,
            output_file.display()
        ));
    }

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir).map_err(ForgeKitError::Io)?;

    // Build qpdf command
    let qpdf_pages = PageSpec::to_qpdf_pages(pages, total_pages)?;
    let output_file = output_dir.join("split_output.pdf");

    let mut cmd = Command::new(&tool_info.path);
    cmd.arg(input);
    cmd.arg("--pages");
    cmd.arg(&qpdf_pages);
    cmd.arg("--");
    cmd.arg(&output_file);

    // Execute
    let output_result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
        tool: "qpdf".to_string(),
        stderr: format!("Failed to execute: {}", e),
    })?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        return Err(ForgeKitError::ProcessingFailed {
            tool: "qpdf".to_string(),
            stderr: stderr.to_string(),
        });
    }

    Ok(format!(
        "Successfully split PDF pages {} to {}",
        qpdf_pages,
        output_file.display()
    ))
}

fn execute_pdf_compress(
    input: &PathBuf,
    output: &PathBuf,
    level: &str,
    plan_only: bool,
) -> Result<String> {
    let strategy = get_compression_strategy(level);

    if plan_only {
        // Generate plan showing the command that would be run
        let mut cmd_parts = vec![strategy.tool.clone()];
        match strategy.tool.as_str() {
            "gs" => {
                // Ghostscript: gs [flags] -sOutputFile=output.pdf input.pdf
                cmd_parts.extend(strategy.flags.iter().cloned());
                cmd_parts.push(format!("-sOutputFile={}", output.display()));
                cmd_parts.push(input.display().to_string());
            }
            "qpdf" => {
                cmd_parts.extend(strategy.flags.iter().cloned());
                cmd_parts.push(input.display().to_string());
                cmd_parts.push(output.display().to_string());
            }
            _ => {
                return Err(ForgeKitError::Other(anyhow::anyhow!(
                    "Unknown compression tool: {}",
                    strategy.tool
                )));
            }
        }
        return Ok(cmd_parts.join(" "));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.clone(),
            reason: "Input file does not exist".to_string(),
        });
    }

    match strategy.tool.as_str() {
        "gs" => {
            // Probe for Ghostscript
            let tool = GsTool;
            let config = ToolConfig::default();
            let tool_info = tool.probe(&config)?;

            // Build Ghostscript command
            // gs [flags] -sOutputFile=output.pdf input.pdf
            // Note: input comes LAST for Ghostscript
            let mut cmd = Command::new(&tool_info.path);
            // Input comes LAST for Ghostscript
            // Output file must come BEFORE any -c (PostScript) flags
            cmd.arg(format!("-sOutputFile={}", output.display()));
            cmd.args(&strategy.flags);
            // Must use -f to separate input file from preceding -c arguments
            cmd.arg("-f");
            cmd.arg(input);

            // Execute
            let output_result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
                tool: "gs".to_string(),
                stderr: format!("Failed to execute: {}", e),
            })?;

            if !output_result.status.success() {
                let stderr = String::from_utf8_lossy(&output_result.stderr);
                return Err(ForgeKitError::ProcessingFailed {
                    tool: "gs".to_string(),
                    stderr: stderr.to_string(),
                });
            }
        }
        "qpdf" => {
            // Probe for qpdf (kept for future use, e.g., page operations)
            let tool = QpdfTool;
            let config = ToolConfig::default();
            let tool_info = tool.probe(&config)?;

            // Build qpdf command
            let mut cmd = Command::new(&tool_info.path);
            cmd.args(&strategy.flags);
            cmd.arg(input);
            cmd.arg(output);

            // Execute
            let output_result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
                tool: "qpdf".to_string(),
                stderr: format!("Failed to execute: {}", e),
            })?;

            if !output_result.status.success() {
                let stderr = String::from_utf8_lossy(&output_result.stderr);
                return Err(ForgeKitError::ProcessingFailed {
                    tool: "qpdf".to_string(),
                    stderr: stderr.to_string(),
                });
            }
        }
        _ => {
            return Err(ForgeKitError::Other(anyhow::anyhow!(
                "Unknown compression tool: {}",
                strategy.tool
            )));
        }
    }

    Ok(format!(
        "Successfully compressed PDF to {}",
        output.display()
    ))
}

fn execute_pdf_linearize(input: &PathBuf, output: &PathBuf, plan_only: bool) -> Result<String> {
    if plan_only {
        return Ok(format!(
            "qpdf --linearize {} {}",
            input.display(),
            output.display()
        ));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.clone(),
            reason: "Input file does not exist".to_string(),
        });
    }

    // Probe for qpdf
    let tool = QpdfTool;
    let config = ToolConfig::default();
    let tool_info = tool.probe(&config)?;

    // Build qpdf command
    let mut cmd = Command::new(&tool_info.path);
    cmd.arg("--linearize");
    cmd.arg(input);
    cmd.arg(output);

    // Execute
    let output_result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
        tool: "qpdf".to_string(),
        stderr: format!("Failed to execute: {}", e),
    })?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        return Err(ForgeKitError::ProcessingFailed {
            tool: "qpdf".to_string(),
            stderr: stderr.to_string(),
        });
    }

    Ok(format!(
        "Successfully linearized PDF to {}",
        output.display()
    ))
}

fn execute_pdf_reorder(
    input: &PathBuf,
    output: &PathBuf,
    page_order: &[u32],
    plan_only: bool,
) -> Result<String> {
    if page_order.is_empty() {
        return Err(ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "Page order cannot be empty".to_string(),
        });
    }

    // Convert page_order (1-indexed) to qpdf pages format
    let pages_str = page_order
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    if plan_only {
        return Ok(format!(
            "qpdf {} --pages {} -- {}",
            input.display(),
            pages_str,
            output.display()
        ));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.clone(),
            reason: "Input file does not exist".to_string(),
        });
    }

    // Probe for qpdf
    let tool = QpdfTool;
    let config = ToolConfig::default();
    let tool_info = tool.probe(&config)?;

    // Build qpdf command
    let mut cmd = Command::new(&tool_info.path);
    cmd.arg(input);
    cmd.arg("--pages");
    cmd.arg(&pages_str);
    cmd.arg("--");
    cmd.arg(output);

    // Execute
    let output_result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
        tool: "qpdf".to_string(),
        stderr: format!("Failed to execute: {}", e),
    })?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        return Err(ForgeKitError::ProcessingFailed {
            tool: "qpdf".to_string(),
            stderr: stderr.to_string(),
        });
    }

    Ok(format!(
        "Successfully reordered PDF pages to {}",
        output.display()
    ))
}

fn execute_pdf_extract(
    input: &PathBuf,
    output: Option<&PathBuf>,
    output_dir: Option<&PathBuf>,
    pages: &[PageSpec],
    format: &str,
    plan_only: bool,
) -> Result<String> {
    if pages.is_empty() {
        return Err(ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "At least one page specification required".to_string(),
        });
    }

    match format {
        "pdf" => {
            let output_path = output.ok_or_else(|| ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: "Output file path required when format is 'pdf'".to_string(),
            })?;

            // For now, we'll assume total_pages = 100 (in real implementation, we'd query this)
            let total_pages = 100; // TODO: Get actual page count from PDF

            if plan_only {
                let qpdf_pages = PageSpec::to_qpdf_pages(pages, total_pages)?;
                return Ok(format!(
                    "qpdf {} --pages {} -- {}",
                    input.display(),
                    qpdf_pages,
                    output_path.display()
                ));
            }

            if !input.exists() {
                return Err(ForgeKitError::InvalidInput {
                    path: input.clone(),
                    reason: "Input file does not exist".to_string(),
                });
            }

            // Probe for qpdf
            let tool = QpdfTool;
            let config = ToolConfig::default();
            let tool_info = tool.probe(&config)?;

            // Build qpdf command
            let qpdf_pages = PageSpec::to_qpdf_pages(pages, total_pages)?;

            let mut cmd = Command::new(&tool_info.path);
            cmd.arg(input);
            cmd.arg("--pages");
            cmd.arg(&qpdf_pages);
            cmd.arg("--");
            cmd.arg(output_path);

            // Execute
            let output_result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
                tool: "qpdf".to_string(),
                stderr: format!("Failed to execute: {}", e),
            })?;

            if !output_result.status.success() {
                let stderr = String::from_utf8_lossy(&output_result.stderr);
                return Err(ForgeKitError::ProcessingFailed {
                    tool: "qpdf".to_string(),
                    stderr: stderr.to_string(),
                });
            }

            Ok(format!(
                "Successfully extracted PDF pages to {}",
                output_path.display()
            ))
        }
        "images" => {
            let output_dir_path = output_dir.ok_or_else(|| ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: "Output directory path required when format is 'images'".to_string(),
            })?;

            if plan_only {
                // For images, we'd use pdf2image or similar tool
                // For now, show a placeholder plan
                return Ok(format!(
                    "pdf2image {} --output-dir {} --pages {}",
                    input.display(),
                    output_dir_path.display(),
                    "1-5" // Simplified for plan
                ));
            }

            // TODO: Implement image extraction using pdf2image or similar tool
            // For now, return an error indicating this feature is not yet implemented
            Err(ForgeKitError::Other(anyhow::anyhow!(
                "Image extraction is not yet implemented. Use format 'pdf' to extract pages to a PDF file."
            )))
        }
        _ => Err(ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: format!(
                "Unknown format '{}'. Supported formats: pdf, images",
                format
            ),
        }),
    }
}
