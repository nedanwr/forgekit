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

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::job::progress::{
    new_job_id, ErrorInfo, JobResult, ProgressEvent, ProgressInfo, ProgressReporter,
};
use crate::job::spec::MetadataAction;
use crate::job::JobSpec;
use crate::presets::get_compression_strategy;
use crate::tools::exiftool::ExiftoolTool;
use crate::tools::ffmpeg::FfmpegTool;
use crate::tools::gs::GsTool;
use crate::tools::libvips::LibvipsTool;
use crate::tools::ocrmypdf::OcrmypdfTool;
use crate::tools::qpdf::QpdfTool;
use crate::tools::{Tool, ToolConfig, ToolInfo};
use crate::utils::audio::{AudioFormat, LoudnessTarget};
use crate::utils::error::{ForgeKitError, Result};
use crate::utils::image::ImageFormat;
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
        JobSpec::PdfOcr {
            input,
            output,
            language,
            skip_text,
            deskew,
            force_ocr,
        } => execute_pdf_ocr_with_progress(
            input, output, language, *skip_text, *deskew, *force_ocr, plan_only, reporter,
        ),
        JobSpec::PdfMetadata {
            input,
            output,
            action,
        } => execute_pdf_metadata(input, output.as_deref(), action, plan_only),

        // Image operations
        JobSpec::ImageConvert {
            input,
            output,
            format,
            quality,
            compression,
            strip_metadata,
        } => execute_image_convert(
            input,
            output,
            format,
            *quality,
            *compression,
            *strip_metadata,
            plan_only,
        ),
        JobSpec::ImageResize {
            input,
            output,
            width,
            height,
        } => execute_image_resize(input, output, *width, *height, plan_only),
        JobSpec::ImageStrip { input, output } => execute_image_strip(input, output, plan_only),

        // Audio operations
        JobSpec::AudioConvert {
            input,
            output,
            format,
            bitrate,
        } => execute_audio_convert(input, output, format, *bitrate, plan_only),
        JobSpec::AudioNormalize {
            input,
            output,
            target,
        } => execute_audio_normalize(input, output, target, plan_only),
        JobSpec::AudioExtract {
            input,
            output,
            format,
            bitrate,
        } => execute_audio_extract(input, output, format, *bitrate, plan_only),
        JobSpec::AudioTrim {
            input,
            output,
            start,
            end,
        } => execute_audio_trim(input, output, *start, *end, plan_only),
        JobSpec::AudioJoin { inputs, output } => execute_audio_join(inputs, output, plan_only),
        JobSpec::AudioVolume {
            input,
            output,
            gain_db,
        } => execute_audio_volume(input, output, *gain_db, plan_only),
        JobSpec::AudioMono { input, output } => execute_audio_mono(input, output, plan_only),
        JobSpec::VideoTranscode {
            input,
            output,
            crf,
            preset,
            scale,
            copy_audio,
        } => execute_video_transcode(input, output, *crf, preset, *scale, *copy_audio, plan_only),
        JobSpec::VideoTrim {
            input,
            output,
            start,
            end,
        } => execute_video_trim(input, output, *start, *end, plan_only),
        JobSpec::VideoJoin { inputs, output } => execute_video_join(inputs, output, plan_only),
        JobSpec::VideoThumbnail {
            input,
            output,
            timestamp,
        } => execute_video_thumbnail(input, output, *timestamp, plan_only),
        JobSpec::VideoConvert {
            input,
            output,
            format,
            start,
            duration,
            width,
            fps,
        } => execute_video_convert(input, output, format, *start, *duration, *width, *fps, plan_only),
        JobSpec::VideoSpeed {
            input,
            output,
            speed,
        } => execute_video_speed(input, output, *speed, plan_only),
        JobSpec::VideoRotate {
            input,
            output,
            degrees,
        } => execute_video_rotate(input, output, *degrees, plan_only),
        JobSpec::VideoMute { input, output } => execute_video_mute(input, output, plan_only),
        JobSpec::VideoStitch {
            inputs,
            output,
            format,
            fps,
            width,
        } => execute_video_stitch(inputs, output, format, *fps, *width, plan_only),
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

#[allow(clippy::too_many_arguments)]
fn execute_pdf_ocr_with_progress(
    input: &PathBuf,
    output: &PathBuf,
    language: &str,
    skip_text: bool,
    deskew: bool,
    force_ocr: bool,
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
            total: 100,
            percent: 0,
            stage: Some("starting".to_string()),
        },
        message: format!("Starting OCR with language '{}'", language),
    });

    // Build command parts for plan output
    let mut cmd_parts = vec!["ocrmypdf".to_string()];

    cmd_parts.push("-l".to_string());
    cmd_parts.push(language.to_string());

    if skip_text {
        cmd_parts.push("--skip-text".to_string());
    }

    if deskew {
        cmd_parts.push("--deskew".to_string());
    }

    if force_ocr {
        cmd_parts.push("--force-ocr".to_string());
    }

    // Add progress flag for real execution
    if !plan_only {
        cmd_parts.push("--progress".to_string());
    }

    cmd_parts.push(input.display().to_string());
    cmd_parts.push(output.display().to_string());

    if plan_only {
        return Ok(cmd_parts.join(" "));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.clone(),
            reason: "Input file does not exist".to_string(),
        });
    }

    // Probe for ocrmypdf
    let tool = OcrmypdfTool;
    let config = ToolConfig::default();
    let tool_info = tool.probe(&config)?;

    // Build actual command
    let mut cmd = Command::new(&tool_info.path);
    cmd.arg("-l").arg(language);

    if skip_text {
        cmd.arg("--skip-text");
    }

    if deskew {
        cmd.arg("--deskew");
    }

    if force_ocr {
        cmd.arg("--force-ocr");
    }

    cmd.arg(input);
    cmd.arg(output);

    // Report progress as OCR starts
    reporter.report(&ProgressEvent::Progress {
        version: 1,
        job_id: job_id.clone(),
        progress: ProgressInfo {
            current: 10,
            total: 100,
            percent: 10,
            stage: Some("ocr".to_string()),
        },
        message: "Running OCR (this may take a while for large documents)...".to_string(),
    });

    // Execute
    let output_result = cmd.output().map_err(|e| ForgeKitError::ProcessingFailed {
        tool: "ocrmypdf".to_string(),
        stderr: format!("Failed to execute: {}", e),
    })?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        let error = ForgeKitError::ProcessingFailed {
            tool: "ocrmypdf".to_string(),
            stderr: stderr.to_string(),
        };

        // Emit error event
        reporter.report(&ProgressEvent::Error {
            version: 1,
            job_id: job_id.clone(),
            error: ErrorInfo {
                code: "OCR_FAILED".to_string(),
                message: error.to_string(),
                hint:
                    "Check that the input PDF is valid and tesseract language packs are installed"
                        .to_string(),
            },
        });

        return Err(error);
    }

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
        "Successfully added OCR text layer to PDF: {}",
        output.display()
    ))
}

fn execute_pdf_metadata(
    input: &Path,
    output: Option<&Path>,
    action: &MetadataAction,
    plan_only: bool,
) -> Result<String> {
    match action {
        MetadataAction::GetAll => execute_pdf_metadata_get_all(input, plan_only),
        MetadataAction::Get(field) => execute_pdf_metadata_get(input, field, plan_only),
        MetadataAction::Set(fields) => {
            let output_path = output.ok_or_else(|| ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: "Output file path required for set operations".to_string(),
            })?;
            execute_pdf_metadata_set(input, output_path, fields, plan_only)
        }
    }
}

fn execute_pdf_metadata_get_all(input: &Path, plan_only: bool) -> Result<String> {
    if plan_only {
        return Ok(format!(
            "exiftool -json -PDF:all -XMP:all {}",
            input.display()
        ));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    // Probe for exiftool
    let tool = ExiftoolTool;
    let config = ToolConfig::default();
    let tool_info = tool.probe(&config)?;

    // Read metadata as JSON
    let json = tool.read_metadata_json(&tool_info.path, input)?;

    Ok(json)
}

fn execute_pdf_metadata_get(input: &Path, field: &str, plan_only: bool) -> Result<String> {
    if plan_only {
        return Ok(format!("exiftool -s -s -s -{} {}", field, input.display()));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    // Probe for exiftool
    let tool = ExiftoolTool;
    let config = ToolConfig::default();
    let tool_info = tool.probe(&config)?;

    // Read specific field
    let value = tool.read_field(&tool_info.path, input, field)?;

    if value.is_empty() {
        Ok(format!("Field '{}' is not set", field))
    } else {
        Ok(value)
    }
}

fn execute_pdf_metadata_set(
    input: &Path,
    output: &Path,
    fields: &[(String, String)],
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        let mut cmd_parts = vec!["exiftool".to_string()];
        for (field, value) in fields {
            cmd_parts.push(format!("-{}={}", field, value));
        }
        // If input != output, we need to copy first
        if input != output {
            cmd_parts.push("-o".to_string());
            cmd_parts.push(output.display().to_string());
        } else {
            cmd_parts.push("-overwrite_original".to_string());
        }
        cmd_parts.push(input.display().to_string());
        return Ok(cmd_parts.join(" "));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    // Probe for exiftool
    let tool = ExiftoolTool;
    let config = ToolConfig::default();
    let tool_info = tool.probe(&config)?;

    // If output is different from input, copy first
    if input != output {
        std::fs::copy(input, output).map_err(ForgeKitError::Io)?;
    }

    // Write metadata (to output file, overwrite original to avoid backup)
    tool.write_metadata(&tool_info.path, output, fields, true)?;

    Ok(format!(
        "Successfully set {} metadata field(s) in {}",
        fields.len(),
        output.display()
    ))
}

// ========== Image Operations ==========

/// Probe for libvips tool.
fn probe_libvips() -> Result<ToolInfo> {
    let tool = LibvipsTool;
    let config = ToolConfig::default();
    tool.probe(&config)
}

fn execute_image_convert(
    input: &Path,
    output: &Path,
    format: &ImageFormat,
    quality: Option<u8>,
    compression: Option<u8>,
    strip: bool,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(LibvipsTool::plan_convert(
            input,
            output,
            quality,
            compression,
            strip,
        ));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_libvips()?;
    let tool = LibvipsTool;
    tool.convert(
        &tool_info.path,
        input,
        output,
        format,
        quality,
        compression,
        strip,
    )?;

    Ok(format!(
        "Successfully converted image to {} ({})",
        output.display(),
        format.extension()
    ))
}

fn execute_image_resize(
    input: &Path,
    output: &Path,
    width: Option<u32>,
    height: Option<u32>,
    plan_only: bool,
) -> Result<String> {
    if width.is_none() && height.is_none() {
        return Err(ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "Width or height required for resize".to_string(),
        });
    }

    if plan_only {
        return Ok(LibvipsTool::plan_resize(input, output, width, height));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_libvips()?;
    let tool = LibvipsTool;
    tool.resize(&tool_info.path, input, output, width, height)?;

    let size_str = match (width, height) {
        (Some(w), Some(h)) => format!("{}x{}", w, h),
        (Some(w), None) => format!("width {}", w),
        (None, Some(h)) => format!("height {}", h),
        (None, None) => "?".to_string(),
    };

    Ok(format!(
        "Successfully resized image to {} ({})",
        output.display(),
        size_str
    ))
}

fn execute_image_strip(input: &Path, output: &Path, plan_only: bool) -> Result<String> {
    if plan_only {
        return Ok(LibvipsTool::plan_strip(input, output));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_libvips()?;
    let tool = LibvipsTool;
    tool.strip_metadata(&tool_info.path, input, output)?;

    Ok(format!(
        "Successfully stripped metadata from image: {}",
        output.display()
    ))
}

// ========== Audio Operations ==========

fn probe_ffmpeg() -> Result<ToolInfo> {
    let tool = FfmpegTool;
    let config = ToolConfig::default();
    tool.probe(&config)
}

fn execute_audio_convert(
    input: &Path,
    output: &Path,
    format: &AudioFormat,
    bitrate: Option<u32>,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_convert(input, output, format, bitrate));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.convert(&tool_info.path, input, output, format, bitrate)?;

    let bitrate_str = bitrate
        .map(|b| format!(" at {}kbps", b))
        .unwrap_or_default();

    Ok(format!(
        "Successfully converted audio to {} ({}){}",
        output.display(),
        format.extension(),
        bitrate_str
    ))
}

fn execute_audio_normalize(
    input: &Path,
    output: &Path,
    target: &LoudnessTarget,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_normalize(input, output, target));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.normalize(&tool_info.path, input, output, target)?;

    Ok(format!(
        "Successfully normalized audio to {} ({})",
        output.display(),
        target
    ))
}

fn execute_audio_extract(
    input: &Path,
    output: &Path,
    format: &AudioFormat,
    bitrate: Option<u32>,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_extract(input, output, format, bitrate));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.extract(&tool_info.path, input, output, format, bitrate)?;

    let bitrate_str = bitrate
        .map(|b| format!(" at {}kbps", b))
        .unwrap_or_default();

    Ok(format!(
        "Successfully extracted audio to {} ({}){}",
        output.display(),
        format.extension(),
        bitrate_str
    ))
}

fn execute_audio_trim(
    input: &Path,
    output: &Path,
    start: Option<f64>,
    end: Option<f64>,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_trim(input, output, start, end));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.trim(&tool_info.path, input, output, start, end)?;

    let time_str = match (start, end) {
        (Some(s), Some(e)) => format!(" from {:.1}s to {:.1}s", s, e),
        (Some(s), None) => format!(" from {:.1}s", s),
        (None, Some(e)) => format!(" to {:.1}s", e),
        (None, None) => String::new(),
    };

    Ok(format!(
        "Successfully trimmed audio to {}{}",
        output.display(),
        time_str
    ))
}

fn execute_audio_join(inputs: &[PathBuf], output: &Path, plan_only: bool) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_join(inputs, output));
    }

    // Validate inputs
    for input in inputs {
        if !input.exists() {
            return Err(ForgeKitError::InvalidInput {
                path: input.clone(),
                reason: "Input file does not exist".to_string(),
            });
        }
    }

    if inputs.len() < 2 {
        return Err(ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "At least 2 input files are required for join".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.join(&tool_info.path, inputs, output)?;

    Ok(format!(
        "Successfully joined {} audio files to {}",
        inputs.len(),
        output.display()
    ))
}

fn execute_audio_volume(
    input: &Path,
    output: &Path,
    gain_db: f64,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_volume(input, output, gain_db));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.volume(&tool_info.path, input, output, gain_db)?;

    let gain_str = if gain_db >= 0.0 {
        format!("+{:.1}dB", gain_db)
    } else {
        format!("{:.1}dB", gain_db)
    };

    Ok(format!(
        "Successfully adjusted volume by {} to {}",
        gain_str,
        output.display()
    ))
}

fn execute_audio_mono(input: &Path, output: &Path, plan_only: bool) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_mono(input, output));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.mono(&tool_info.path, input, output)?;

    Ok(format!(
        "Successfully converted to mono: {}",
        output.display()
    ))
}

// ========== Video Operations ==========

fn execute_video_transcode(
    input: &Path,
    output: &Path,
    crf: u8,
    preset: &str,
    scale: Option<(i32, i32)>,
    copy_audio: bool,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_transcode(
            input, output, crf, preset, scale, copy_audio,
        ));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.transcode(&tool_info.path, input, output, crf, preset, scale, copy_audio)?;

    let scale_str = scale
        .map(|(w, h)| {
            if h == -1 {
                format!(" scaled to {}p", w)
            } else {
                format!(" scaled to {}x{}", w, h)
            }
        })
        .unwrap_or_default();

    Ok(format!(
        "Successfully transcoded video to {} (H.264, CRF {}){}",
        output.display(),
        crf,
        scale_str
    ))
}

fn execute_video_trim(
    input: &Path,
    output: &Path,
    start: Option<f64>,
    end: Option<f64>,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_video_trim(input, output, start, end));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.video_trim(&tool_info.path, input, output, start, end)?;

    let time_str = match (start, end) {
        (Some(s), Some(e)) => format!(" from {:.1}s to {:.1}s", s, e),
        (Some(s), None) => format!(" from {:.1}s to end", s),
        (None, Some(e)) => format!(" from start to {:.1}s", e),
        (None, None) => String::new(),
    };

    Ok(format!(
        "Successfully trimmed video{}",
        time_str
    ))
}

fn execute_video_join(
    inputs: &[PathBuf],
    output: &Path,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_video_join(inputs, output));
    }

    if inputs.len() < 2 {
        return Err(ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: format!("At least 2 files required for join, got {}", inputs.len()),
        });
    }

    for input in inputs {
        if !input.exists() {
            return Err(ForgeKitError::InvalidInput {
                path: input.clone(),
                reason: "Input file does not exist".to_string(),
            });
        }
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.video_join(&tool_info.path, inputs, output)?;

    Ok(format!(
        "Successfully joined {} videos into {}",
        inputs.len(),
        output.display()
    ))
}

fn execute_video_thumbnail(
    input: &Path,
    output: &Path,
    timestamp: f64,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_video_thumbnail(input, output, timestamp));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.video_thumbnail(&tool_info.path, input, output, timestamp)?;

    Ok(format!(
        "Successfully extracted thumbnail at {:.1}s to {}",
        timestamp,
        output.display()
    ))
}

#[allow(clippy::too_many_arguments)]
fn execute_video_convert(
    input: &Path,
    output: &Path,
    format: &str,
    start: Option<f64>,
    duration: Option<f64>,
    width: Option<u32>,
    fps: Option<u32>,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_video_convert(input, output, format, start, duration, width, fps));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.video_convert(&tool_info.path, input, output, format, start, duration, width, fps)?;

    Ok(format!(
        "Successfully converted video to {}: {}",
        format.to_uppercase(),
        output.display()
    ))
}

fn execute_video_speed(
    input: &Path,
    output: &Path,
    speed: f64,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_video_speed(input, output, speed));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    if speed <= 0.0 {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Speed must be greater than 0".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.video_speed(&tool_info.path, input, output, speed)?;

    Ok(format!(
        "Successfully changed video speed to {:.1}x: {}",
        speed,
        output.display()
    ))
}

fn execute_video_rotate(
    input: &Path,
    output: &Path,
    degrees: u32,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_video_rotate(input, output, degrees));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    if degrees != 90 && degrees != 180 && degrees != 270 {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: format!("Invalid rotation angle {}. Use 90, 180, or 270.", degrees),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.video_rotate(&tool_info.path, input, output, degrees)?;

    Ok(format!(
        "Successfully rotated video {}°: {}",
        degrees,
        output.display()
    ))
}

fn execute_video_mute(
    input: &Path,
    output: &Path,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_video_mute(input, output));
    }

    if !input.exists() {
        return Err(ForgeKitError::InvalidInput {
            path: input.to_path_buf(),
            reason: "Input file does not exist".to_string(),
        });
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.video_mute(&tool_info.path, input, output)?;

    Ok(format!(
        "Successfully removed audio from video: {}",
        output.display()
    ))
}

fn execute_video_stitch(
    inputs: &[PathBuf],
    output: &Path,
    format: &str,
    fps: u32,
    width: Option<u32>,
    plan_only: bool,
) -> Result<String> {
    if plan_only {
        return Ok(FfmpegTool::plan_video_stitch(inputs, output, format, fps, width));
    }

    if inputs.is_empty() {
        return Err(ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "No input images provided".to_string(),
        });
    }

    for input in inputs {
        if !input.exists() {
            return Err(ForgeKitError::InvalidInput {
                path: input.clone(),
                reason: "Input file does not exist".to_string(),
            });
        }
    }

    let tool_info = probe_ffmpeg()?;
    let tool = FfmpegTool;
    tool.video_stitch(&tool_info.path, inputs, output, format, fps, width)?;

    Ok(format!(
        "Successfully stitched {} images into {}: {}",
        inputs.len(),
        format.to_uppercase(),
        output.display()
    ))
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

#[cfg(test)]
mod ocr_tests {
    use super::*;
    use crate::job::progress::NoOpProgressReporter;

    #[test]
    fn test_execute_pdf_ocr_plan() {
        let input = PathBuf::from("scan.pdf");
        let output = PathBuf::from("searchable.pdf");
        let reporter = NoOpProgressReporter;

        let result = execute_pdf_ocr_with_progress(
            &input, &output, "eng", true, false, false, true, &reporter,
        )
        .unwrap();

        assert!(result.contains("ocrmypdf"));
        assert!(result.contains("-l eng"));
        assert!(result.contains("--skip-text"));
        assert!(result.contains("scan.pdf"));
        assert!(result.contains("searchable.pdf"));
    }

    #[test]
    fn test_execute_pdf_ocr_plan_with_deskew() {
        let input = PathBuf::from("tilted.pdf");
        let output = PathBuf::from("fixed.pdf");
        let reporter = NoOpProgressReporter;

        let result = execute_pdf_ocr_with_progress(
            &input, &output, "deu", false, true, false, true, &reporter,
        )
        .unwrap();

        assert!(result.contains("ocrmypdf"));
        assert!(result.contains("-l deu"));
        assert!(result.contains("--deskew"));
        assert!(!result.contains("--skip-text"));
    }

    #[test]
    fn test_execute_pdf_ocr_plan_force() {
        let input = PathBuf::from("existing_text.pdf");
        let output = PathBuf::from("reocr.pdf");
        let reporter = NoOpProgressReporter;

        let result = execute_pdf_ocr_with_progress(
            &input, &output, "eng", false, false, true, true, &reporter,
        )
        .unwrap();

        assert!(result.contains("ocrmypdf"));
        assert!(result.contains("--force-ocr"));
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::*;
    use crate::job::spec::MetadataAction;

    #[test]
    fn test_execute_pdf_metadata_get_all_plan() {
        let input = PathBuf::from("doc.pdf");

        let result = execute_pdf_metadata(&input, None, &MetadataAction::GetAll, true).unwrap();

        assert!(result.contains("exiftool"));
        assert!(result.contains("-json"));
        assert!(result.contains("-PDF:all"));
        assert!(result.contains("-XMP:all"));
        assert!(result.contains("doc.pdf"));
    }

    #[test]
    fn test_execute_pdf_metadata_get_field_plan() {
        let input = PathBuf::from("doc.pdf");
        let action = MetadataAction::Get("title".to_string());

        let result = execute_pdf_metadata(&input, None, &action, true).unwrap();

        assert!(result.contains("exiftool"));
        assert!(result.contains("-s -s -s"));
        assert!(result.contains("-title"));
        assert!(result.contains("doc.pdf"));
    }

    #[test]
    fn test_execute_pdf_metadata_set_plan() {
        let input = PathBuf::from("doc.pdf");
        let output = PathBuf::from("updated.pdf");
        let fields = vec![
            ("title".to_string(), "My Document".to_string()),
            ("author".to_string(), "John Doe".to_string()),
        ];
        let action = MetadataAction::Set(fields);

        let result = execute_pdf_metadata(&input, Some(&output), &action, true).unwrap();

        assert!(result.contains("exiftool"));
        assert!(result.contains("-title=My Document"));
        assert!(result.contains("-author=John Doe"));
        assert!(result.contains("-o"));
        assert!(result.contains("updated.pdf"));
    }

    #[test]
    fn test_execute_pdf_metadata_set_in_place_plan() {
        let path = PathBuf::from("doc.pdf");
        let fields = vec![("title".to_string(), "Updated Title".to_string())];
        let action = MetadataAction::Set(fields);

        let result = execute_pdf_metadata(&path, Some(&path), &action, true).unwrap();

        assert!(result.contains("exiftool"));
        assert!(result.contains("-overwrite_original"));
        // Check that "-o " (with space) is not present - don't confuse with "-overwrite_original"
        assert!(!result.contains(" -o "));
    }

    #[test]
    fn test_execute_pdf_metadata_set_requires_output() {
        let input = PathBuf::from("doc.pdf");
        let fields = vec![("title".to_string(), "My Document".to_string())];
        let action = MetadataAction::Set(fields);

        let result = execute_pdf_metadata(&input, None, &action, false);

        assert!(result.is_err());
        match result {
            Err(ForgeKitError::InvalidInput { reason, .. }) => {
                assert!(reason.contains("Output file path required"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }
}

#[cfg(test)]
mod image_operation_tests {
    use super::*;
    use crate::utils::image::ImageFormat;

    #[test]
    fn test_execute_image_convert_plan() {
        let input = PathBuf::from("photo.jpg");
        let output = PathBuf::from("photo.webp");

        let result = execute_image_convert(
            &input,
            &output,
            &ImageFormat::WebP,
            Some(80),
            None,
            true,
            true,
        )
        .unwrap();

        // Plan uses libvips format
        assert!(result.contains("vips copy"));
        assert!(result.contains("photo.jpg"));
        assert!(result.contains("Q=80"));
        assert!(result.contains("strip"));
    }

    #[test]
    fn test_execute_image_convert_plan_no_strip() {
        let input = PathBuf::from("photo.jpg");
        let output = PathBuf::from("photo.png");

        let result = execute_image_convert(
            &input,
            &output,
            &ImageFormat::Png,
            None,
            Some(0),
            false,
            true,
        )
        .unwrap();

        assert!(result.contains("vips copy"));
        assert!(result.contains("photo.png"));
        assert!(!result.contains("Q="));
        assert!(result.contains("compression=0"));
        assert!(!result.contains("strip"));
    }

    #[test]
    fn test_execute_image_resize_plan() {
        let input = PathBuf::from("photo.jpg");
        let output = PathBuf::from("thumb.jpg");

        let result = execute_image_resize(&input, &output, Some(800), Some(600), true).unwrap();

        assert!(result.contains("vipsthumbnail"));
        assert!(result.contains("-s 800x600"));
    }

    #[test]
    fn test_execute_image_resize_plan_width_only() {
        let input = PathBuf::from("photo.jpg");
        let output = PathBuf::from("thumb.jpg");

        let result = execute_image_resize(&input, &output, Some(800), None, true).unwrap();

        assert!(result.contains("vipsthumbnail"));
        assert!(result.contains("-s 800x"));
    }

    #[test]
    fn test_execute_image_resize_plan_height_only() {
        let input = PathBuf::from("photo.jpg");
        let output = PathBuf::from("thumb.jpg");

        let result = execute_image_resize(&input, &output, None, Some(600), true).unwrap();

        assert!(result.contains("vipsthumbnail"));
        assert!(result.contains("-s x600"));
    }

    #[test]
    fn test_execute_image_resize_requires_dimensions() {
        let input = PathBuf::from("photo.jpg");
        let output = PathBuf::from("thumb.jpg");

        let result = execute_image_resize(&input, &output, None, None, true);

        assert!(result.is_err());
        match result {
            Err(ForgeKitError::InvalidInput { reason, .. }) => {
                assert!(reason.contains("Width or height required"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_execute_image_strip_plan() {
        let input = PathBuf::from("photo.jpg");
        let output = PathBuf::from("clean.jpg");

        let result = execute_image_strip(&input, &output, true).unwrap();

        assert!(result.contains("vips copy"));
        assert!(result.contains("photo.jpg"));
        assert!(result.contains("[strip]"));
    }

    // Compress tests (compress uses ImageConvert with specific settings)

    #[test]
    fn test_execute_image_compress_jpeg_plan() {
        // JPEG compress: quality reduction + strip metadata
        let input = PathBuf::from("photo.jpg");
        let output = PathBuf::from("photo_compressed.jpg");

        let result = execute_image_convert(
            &input,
            &output,
            &ImageFormat::Jpeg,
            Some(80), // quality 80
            None,     // no PNG compression
            true,     // strip metadata
            true,     // plan only
        )
        .unwrap();

        assert!(result.contains("vips copy"));
        assert!(result.contains("photo.jpg"));
        assert!(result.contains("Q=80"));
        assert!(result.contains("strip"));
    }

    #[test]
    fn test_execute_image_compress_png_plan() {
        // PNG compress: max compression level + strip metadata
        let input = PathBuf::from("image.png");
        let output = PathBuf::from("image_compressed.png");

        let result = execute_image_convert(
            &input,
            &output,
            &ImageFormat::Png,
            None,    // no quality for PNG
            Some(9), // max compression
            true,    // strip metadata
            true,    // plan only
        )
        .unwrap();

        assert!(result.contains("vips copy"));
        assert!(result.contains("image.png"));
        assert!(result.contains("compression=9"));
        assert!(result.contains("strip"));
    }

    #[test]
    fn test_execute_image_compress_webp_plan() {
        // WebP compress: quality reduction + strip metadata
        let input = PathBuf::from("photo.webp");
        let output = PathBuf::from("photo_compressed.webp");

        let result = execute_image_convert(
            &input,
            &output,
            &ImageFormat::WebP,
            Some(60), // lower quality for more compression
            None,
            true, // strip metadata
            true, // plan only
        )
        .unwrap();

        assert!(result.contains("vips copy"));
        assert!(result.contains("photo.webp"));
        assert!(result.contains("Q=60"));
        assert!(result.contains("strip"));
    }
}

#[cfg(test)]
mod audio_operation_tests {
    use super::*;
    use crate::utils::audio::{AudioFormat, LoudnessTarget};

    #[test]
    fn test_execute_audio_convert_mp3_plan() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio.mp3");

        let result =
            execute_audio_convert(&input, &output, &AudioFormat::Mp3, Some(192), true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-i audio.wav"));
        assert!(result.contains("-c:a libmp3lame"));
        assert!(result.contains("-b:a 192k"));
        assert!(result.contains("audio.mp3"));
    }

    #[test]
    fn test_execute_audio_convert_opus_plan() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio.opus");

        let result =
            execute_audio_convert(&input, &output, &AudioFormat::Opus, Some(128), true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-c:a libopus"));
        assert!(result.contains("-b:a 128k"));
        assert!(result.contains("-f opus"));
    }

    #[test]
    fn test_execute_audio_convert_flac_no_bitrate() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio.flac");

        // FLAC doesn't support bitrate, so it shouldn't be in the plan
        let result =
            execute_audio_convert(&input, &output, &AudioFormat::Flac, Some(320), true).unwrap();

        assert!(result.contains("-c:a flac"));
        assert!(!result.contains("-b:a"));
    }

    #[test]
    fn test_execute_audio_convert_aac_plan() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio.m4a");

        let result =
            execute_audio_convert(&input, &output, &AudioFormat::M4a, Some(256), true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-c:a aac"));
        assert!(result.contains("-b:a 256k"));
    }

    #[test]
    fn test_execute_audio_normalize_ebu_r128_plan() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio_normalized.wav");

        let result =
            execute_audio_normalize(&input, &output, &LoudnessTarget::EbuR128, true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-af"));
        assert!(result.contains("loudnorm"));
        assert!(result.contains("I=-23"));
        assert!(result.contains("TP=-1"));
    }

    #[test]
    fn test_execute_audio_normalize_streaming_plan() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio_normalized.wav");

        let result =
            execute_audio_normalize(&input, &output, &LoudnessTarget::Streaming, true).unwrap();

        assert!(result.contains("I=-14"));
    }

    #[test]
    fn test_execute_audio_normalize_custom_plan() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio_normalized.wav");

        let result =
            execute_audio_normalize(&input, &output, &LoudnessTarget::Custom(-16.0), true).unwrap();

        assert!(result.contains("I=-16"));
    }

    #[test]
    fn test_execute_audio_extract_mp3_plan() {
        let input = PathBuf::from("video.mp4");
        let output = PathBuf::from("audio.mp3");

        let result =
            execute_audio_extract(&input, &output, &AudioFormat::Mp3, Some(192), true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-i video.mp4"));
        assert!(result.contains("-vn")); // Strip video
        assert!(result.contains("-c:a libmp3lame"));
        assert!(result.contains("-b:a 192k"));
        assert!(result.contains("audio.mp3"));
    }

    #[test]
    fn test_execute_audio_extract_opus_plan() {
        let input = PathBuf::from("video.mkv");
        let output = PathBuf::from("audio.opus");

        let result =
            execute_audio_extract(&input, &output, &AudioFormat::Opus, Some(128), true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-vn"));
        assert!(result.contains("-c:a libopus"));
        assert!(result.contains("-b:a 128k"));
        assert!(result.contains("-f opus"));
    }

    #[test]
    fn test_execute_audio_extract_flac_no_bitrate() {
        let input = PathBuf::from("video.mov");
        let output = PathBuf::from("audio.flac");

        // FLAC doesn't support bitrate
        let result =
            execute_audio_extract(&input, &output, &AudioFormat::Flac, Some(320), true).unwrap();

        assert!(result.contains("-vn"));
        assert!(result.contains("-c:a flac"));
        assert!(!result.contains("-b:a"));
    }

    #[test]
    fn test_execute_audio_trim_plan() {
        let input = PathBuf::from("song.mp3");
        let output = PathBuf::from("clip.mp3");

        let result = execute_audio_trim(&input, &output, Some(30.0), Some(120.0), true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-ss 30"));
        assert!(result.contains("-t 90")); // duration = 120 - 30
        assert!(result.contains("-c copy"));
    }

    #[test]
    fn test_execute_audio_trim_start_only_plan() {
        let input = PathBuf::from("song.mp3");
        let output = PathBuf::from("clip.mp3");

        let result = execute_audio_trim(&input, &output, Some(60.0), None, true).unwrap();

        assert!(result.contains("-ss 60"));
        assert!(!result.contains("-t "));
        assert!(!result.contains("-to "));
    }

    #[test]
    fn test_execute_audio_trim_end_only_plan() {
        let input = PathBuf::from("song.mp3");
        let output = PathBuf::from("clip.mp3");

        let result = execute_audio_trim(&input, &output, None, Some(90.0), true).unwrap();

        assert!(!result.contains("-ss"));
        assert!(result.contains("-to 90"));
    }

    #[test]
    fn test_execute_audio_join_plan() {
        let inputs = vec![
            PathBuf::from("part1.mp3"),
            PathBuf::from("part2.mp3"),
            PathBuf::from("part3.mp3"),
        ];
        let output = PathBuf::from("joined.mp3");

        let result = execute_audio_join(&inputs, &output, true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-f concat"));
        assert!(result.contains("-c copy"));
        assert!(result.contains("part1.mp3"));
        assert!(result.contains("part2.mp3"));
        assert!(result.contains("part3.mp3"));
    }

    #[test]
    fn test_execute_audio_volume_positive_plan() {
        let input = PathBuf::from("quiet.wav");
        let output = PathBuf::from("louder.wav");

        let result = execute_audio_volume(&input, &output, 6.0, true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-af"));
        assert!(result.contains("volume=6dB"));
    }

    #[test]
    fn test_execute_audio_volume_negative_plan() {
        let input = PathBuf::from("loud.wav");
        let output = PathBuf::from("quieter.wav");

        let result = execute_audio_volume(&input, &output, -3.0, true).unwrap();

        assert!(result.contains("volume=-3dB"));
    }

    #[test]
    fn test_execute_audio_mono_plan() {
        let input = PathBuf::from("stereo.wav");
        let output = PathBuf::from("mono.wav");

        let result = execute_audio_mono(&input, &output, true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-ac 1"));
    }
}

#[cfg(test)]
mod video_operation_tests {
    use super::*;

    #[test]
    fn test_execute_video_transcode_plan() {
        let input = PathBuf::from("video.mp4");
        let output = PathBuf::from("output.mp4");

        let result =
            execute_video_transcode(&input, &output, 23, "medium", None, true, true).unwrap();

        assert!(result.contains("ffmpeg"));
        assert!(result.contains("-i video.mp4"));
        assert!(result.contains("-c:v libx264"));
        assert!(result.contains("-crf 23"));
        assert!(result.contains("-preset medium"));
        assert!(result.contains("-c:a copy"));
    }

    #[test]
    fn test_execute_video_transcode_with_scale_plan() {
        let input = PathBuf::from("video.mp4");
        let output = PathBuf::from("output.mp4");

        let result = execute_video_transcode(
            &input,
            &output,
            23,
            "fast",
            Some((1920, 1080)),
            true,
            true,
        )
        .unwrap();

        assert!(result.contains("-vf scale=1920:1080"));
    }

    #[test]
    fn test_execute_video_transcode_width_only_plan() {
        let input = PathBuf::from("video.mp4");
        let output = PathBuf::from("output.mp4");

        // -1 height = preserve aspect ratio
        let result = execute_video_transcode(
            &input,
            &output,
            20,
            "slow",
            Some((1280, -1)),
            false,
            true,
        )
        .unwrap();

        assert!(result.contains("-vf scale=1280:-2")); // -2 ensures divisible by 2
        assert!(result.contains("-c:a aac"));
        assert!(result.contains("-b:a 128k"));
    }

    #[test]
    fn test_execute_video_transcode_reencode_audio_plan() {
        let input = PathBuf::from("video.mkv");
        let output = PathBuf::from("output.mp4");

        let result =
            execute_video_transcode(&input, &output, 18, "fast", None, false, true).unwrap();

        assert!(result.contains("-c:v libx264"));
        assert!(result.contains("-crf 18"));
        assert!(result.contains("-c:a aac"));
        assert!(result.contains("-b:a 128k"));
    }
}
