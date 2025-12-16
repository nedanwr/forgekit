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
