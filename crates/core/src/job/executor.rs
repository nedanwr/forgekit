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
use crate::utils::temp::create_temp_file;
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
        } => execute_pdf_compress(input, output, level, plan_only),
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
            format,
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

    // Get actual page count
    let total_pages = if plan_only {
        // In plan mode, we might not have the file, or just want to show example
        if input.exists() {
            tool.get_page_count(&tool_info.path, input).unwrap_or(100)
        } else {
            100 // Placeholder for plan
        }
    } else {
        tool.get_page_count(&tool_info.path, input)?
    };

    if plan_only {
        // Generate plan showing the command that would be run
        let qpdf_pages = PageSpec::to_qpdf_pages(pages, total_pages as usize)?;
        let output_file = output_dir.join("split_output.pdf");
        return Ok(format!(
            "qpdf {} --pages . {} -- {}",
            input.display(),
            qpdf_pages,
            output_file.display()
        ));
    }

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir).map_err(ForgeKitError::Io)?;

    // Build qpdf command
    let qpdf_pages = PageSpec::to_qpdf_pages(pages, total_pages as usize)?;
    let output_file = output_dir.join("split_output.pdf");

    let mut cmd = Command::new(&tool_info.path);
    cmd.arg(input);
    cmd.arg("--pages");
    cmd.arg(".");
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
                // Ghostscript: gs -sOutputFile=output.pdf [flags] -f input.pdf
                // -sOutputFile must come before -c (PostScript) flags
                // -f separates the input file from preceding -c arguments
                cmd_parts.push(format!("-sOutputFile={}", output.display()));
                cmd_parts.extend(strategy.flags.iter().cloned());
                cmd_parts.push("-f".to_string());
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
            "qpdf {} --pages . {} -- {}",
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
    cmd.arg(".");
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

fn extract_pages_to_pdf_helper(
    input: &PathBuf,
    output_path: &PathBuf,
    pages: &[PageSpec],
    plan_only: bool,
) -> Result<Option<String>> {
    // Get actual page count
    let total_pages = if plan_only {
        if input.exists() {
            let tool = QpdfTool;
            let config = ToolConfig::default();
            if let Ok(info) = tool.probe(&config) {
                tool.get_page_count(&info.path, input).unwrap_or(100)
            } else {
                100
            }
        } else {
            100
        }
    } else {
        // We need to probe for tool info first to get path
        let tool = QpdfTool;
        let config = ToolConfig::default();
        let tool_info = tool.probe(&config)?;
        tool.get_page_count(&tool_info.path, input)?
    };

    if plan_only {
        let qpdf_pages = PageSpec::to_qpdf_pages(pages, total_pages as usize)?;
        return Ok(Some(format!(
            "qpdf {} --pages . {} -- {}",
            input.display(),
            qpdf_pages,
            output_path.display()
        )));
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
    let qpdf_pages = PageSpec::to_qpdf_pages(pages, total_pages as usize)?;

    let mut cmd = Command::new(&tool_info.path);
    cmd.arg(input);
    cmd.arg("--pages");
    cmd.arg(".");
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

    Ok(None)
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

            if let Some(cmd) = extract_pages_to_pdf_helper(input, output_path, pages, plan_only)? {
                return Ok(cmd);
            }

            Ok(format!(
                "Successfully extracted PDF pages to {}",
                output_path.display()
            ))
        }
        "png" | "jpeg" | "jpg" => {
            let output_dir_path = output_dir.ok_or_else(|| ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: format!("Output directory path required when format is '{}'", format),
            })?;

            if !plan_only {
                std::fs::create_dir_all(output_dir_path).map_err(ForgeKitError::Io)?;
            }

            // Normalization
            let ext = if format == "png" { "png" } else { "jpg" };
            let device = if format == "png" { "png16m" } else { "jpeg" };

            if plan_only {
                let temp_path = input.with_file_name("temp_extract_pages.pdf");
                let qpdf_cmd =
                    extract_pages_to_pdf_helper(input, &temp_path, pages, true)?.unwrap();

                let out_pattern = output_dir_path.join(format!("page_%d.{}", ext));
                let gs_cmd = format!(
                    "gs -sDEVICE={} -dNOPAUSE -dBATCH -dQUIET -r300 -sOutputFile={} {}",
                    device,
                    out_pattern.display(),
                    temp_path.display()
                );

                return Ok(format!(
                    "{} && {} && rm {}",
                    qpdf_cmd,
                    gs_cmd,
                    temp_path.display()
                ));
            }

            // Real execution
            // 1. Extract to temp PDF
            let temp_pdf = create_temp_file("extract", ".pdf")?;
            extract_pages_to_pdf_helper(input, &temp_pdf, pages, false)?;

            // 2. Convert temp PDF to images using Ghostscript
            let tool = GsTool;
            let config = ToolConfig::default();
            let tool_info = tool.probe(&config)?;

            let mut cmd = Command::new(&tool_info.path);
            cmd.arg(format!("-sDEVICE={}", device));
            cmd.arg("-dNOPAUSE");
            cmd.arg("-dBATCH");
            cmd.arg("-dQUIET");
            cmd.arg("-r300"); // 300 DPI for high quality extraction

            let out_pattern = output_dir_path.join(format!("page_%d.{}", ext));
            cmd.arg(format!("-sOutputFile={}", out_pattern.display()));
            cmd.arg(&temp_pdf);

            let output_result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
                tool: "gs".to_string(),
                stderr: format!("Failed to execute gs: {}", e),
            })?;

            // 3. Cleanup temp file explicitly
            let _ = std::fs::remove_file(&temp_pdf);

            if !output_result.status.success() {
                let stderr = String::from_utf8_lossy(&output_result.stderr);
                return Err(ForgeKitError::ProcessingFailed {
                    tool: "gs".to_string(),
                    stderr: stderr.to_string(),
                });
            }

            Ok(format!(
                "Successfully extracted images to directory {}",
                output_dir_path.display()
            ))
        }
        "images" => Err(ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "Format 'images' is ambiguous. Please specify 'png' or 'jpeg/jpg'.".to_string(),
        }),
        _ => Err(ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: format!(
                "Unknown format '{}'. Supported formats: pdf, png, jpeg, jpg",
                format
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::pages::PageSpec;

    // Helper to create valid page specs
    fn pages(spec: &str) -> Vec<PageSpec> {
        PageSpec::parse(spec).unwrap()
    }

    #[test]
    fn test_execute_pdf_linearize_plan() {
        let input = PathBuf::from("input.pdf");
        let output = PathBuf::from("output.pdf");

        let result = execute_pdf_linearize(&input, &output, true).unwrap();

        assert!(result.contains("qpdf"));
        assert!(result.contains("--linearize"));
        assert!(result.contains("input.pdf"));
        assert!(result.contains("output.pdf"));
    }

    #[test]
    fn test_execute_pdf_reorder_plan() {
        let input = PathBuf::from("input.pdf");
        let output = PathBuf::from("output.pdf");
        let order_spec = pages("1,3,2");
        let order: Vec<u32> = order_spec
            .iter()
            .filter_map(|s| match s {
                PageSpec::Page(n) => Some(*n as u32),
                _ => None,
            })
            .collect();

        // This invokes probe(). If qpdf missing, skip?
        // But in plan_only + input doesn't exist?
        // execute_pdf_reorder probes first.

        match execute_pdf_reorder(&input, &output, &order, true) {
            Ok(result) => {
                assert!(result.contains("qpdf"));
                assert!(result.contains("--pages"));
                assert!(result.contains("."));
                assert!(result.contains("1,3,2"));
            }
            Err(ForgeKitError::ToolNotFound { .. }) => {
                println!("Skipping test_execute_pdf_reorder_plan: qpdf not found");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_execute_pdf_extract_plan() {
        let input = PathBuf::from("input.pdf");
        let output_path = PathBuf::from("output.pdf");
        let pages_spec = pages("1-3");

        // execute_pdf_extract also probes page count if input exists.
        // If input doesn't exist, it uses default 100.
        // "input.pdf" likely doesn't exist.

        let result =
            execute_pdf_extract(&input, Some(&output_path), None, &pages_spec, "pdf", true)
                .unwrap();

        // 1-3 of 100 is 1-3.
        assert!(result.contains("qpdf"));
        assert!(result.contains("--pages"));
        assert!(result.contains("."));
        assert!(result.contains("1-3"));
    }
}
#[cfg(test)]
mod image_tests {
    use super::*;
    use crate::utils::pages::PageSpec;

    fn pages(spec: &str) -> Vec<PageSpec> {
        PageSpec::parse(spec).unwrap()
    }

    #[test]
    fn test_execute_pdf_extract_images_ambiguous() {
        let input = PathBuf::from("input.pdf");
        let output_dir = PathBuf::from("out_dir");
        let pages_spec = pages("1");

        let result = execute_pdf_extract(
            &input,
            None,
            Some(&output_dir),
            &pages_spec,
            "images",
            false,
        );

        assert!(result.is_err());
        match result {
            Err(ForgeKitError::InvalidInput { reason, .. }) => {
                assert!(reason.to_lowercase().contains("ambiguous"));
            }
            _ => panic!("Expected InvalidInput error for ambiguous format"),
        }
    }

    #[test]
    fn test_execute_pdf_extract_png_plan() {
        let input = PathBuf::from("input.pdf");
        let output_dir = PathBuf::from("out_dir");
        let pages_spec = pages("1-3");

        let result = execute_pdf_extract(
            &input,
            None,
            Some(&output_dir),
            &pages_spec,
            "png", // png format
            true,  // plan only
        )
        .unwrap();

        // Expected plan: "qpdf ... && gs ... && rm ..."
        assert!(result.contains("qpdf"));
        assert!(result.contains("gs"));
        assert!(result.contains("png16m")); // device
        assert!(result.contains("page_%d.png"));
        assert!(result.contains("rm")); // cleanup
    }

    #[test]
    fn test_execute_pdf_extract_jpeg_plan() {
        let input = PathBuf::from("input.pdf");
        let output_dir = PathBuf::from("out_dir");
        let pages_spec = pages("odd");

        let result =
            execute_pdf_extract(&input, None, Some(&output_dir), &pages_spec, "jpeg", true)
                .unwrap();

        assert!(result.contains("jpeg")); // device
        assert!(result.contains("page_%d.jpg"));
    }
}
