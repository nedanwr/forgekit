use clap::{Args, Subcommand};
use forgekit_core::job::JobSpec;
use forgekit_core::utils::error::Result;
use forgekit_core::utils::pages::PageSpec;
use std::path::PathBuf;

#[derive(Subcommand, Clone)]
pub enum PdfCommand {
    /// Merge multiple PDFs into a single file
    ///
    /// Examples:
    ///   forgekit pdf merge doc1.pdf doc2.pdf --output merged.pdf
    ///   forgekit pdf merge *.pdf --output all.pdf --linearize
    Merge(MergeArgs),
    /// Split PDF into separate files by page ranges
    ///
    /// Examples:
    ///   forgekit pdf split book.pdf --output-dir pages/ --pages 1-5
    ///   forgekit pdf split book.pdf --output-dir pages/ --pages odd
    ///
    /// Page spec: numbers (1), ranges (1-5, 7-), keywords (odd, even), exclusions (!2)
    Split(SplitArgs),
    /// Compress a PDF to reduce file size
    ///
    /// Examples:
    ///   forgekit pdf compress input.pdf --output compressed.pdf
    ///   forgekit pdf compress input.pdf --output compressed.pdf --level light
    ///   forgekit pdf compress input.pdf --output compressed.pdf --level high
    ///
    /// Compression levels (using Ghostscript):
    ///   - light: High quality (~4.2MB)
    ///   - standard: Medium quality (~3.4MB, default)
    ///   - high: Low quality (~2.7MB)
    Compress(CompressArgs),
    /// Linearize a PDF for fast web viewing
    ///
    /// Example:
    ///   forgekit pdf linearize input.pdf --output linearized.pdf
    Linearize(LinearizeArgs),
    /// Reorder pages in a PDF
    ///
    /// Examples:
    ///   forgekit pdf reorder input.pdf --output reordered.pdf --pages 3,1,2
    ///   forgekit pdf reorder input.pdf --output reversed.pdf --pages 5,4,3,2,1
    Reorder(ReorderArgs),
    /// Extract specific pages from a PDF
    ///
    /// Examples:
    ///   forgekit pdf extract book.pdf --output pages.pdf --pages 1-5
    ///   forgekit pdf extract book.pdf --output odd.pdf --pages odd
    ///   forgekit pdf extract book.pdf --output-dir images/ --pages 1-5 --format png
    ///   forgekit pdf extract book.pdf --output-dir images/ --pages 1-5 --format jpeg
    ///
    /// Page spec: numbers (1), ranges (1-5, 7-), keywords (odd, even), exclusions (!2)
    /// Formats: pdf (default), png, jpeg/jpg (image files per page)
    Extract(ExtractArgs),
}

#[derive(Args, Clone)]
pub struct MergeArgs {
    /// Input PDF files (at least 2 required)
    #[arg(required = true, num_args = 1.., help = "Input PDF files (at least 2 required)")]
    pub inputs: Vec<PathBuf>,

    /// Output PDF file path
    #[arg(short, long, required = true, help = "Output PDF file path")]
    pub output: PathBuf,

    /// Optimize for fast web view
    #[arg(long, help = "Optimize for fast web view (linearize)")]
    pub linearize: bool,
}

#[derive(Args, Clone)]
pub struct SplitArgs {
    /// Input PDF file
    #[arg(required = true, help = "Input PDF file")]
    pub input: PathBuf,

    /// Output directory for split files
    #[arg(
        short = 'o',
        long,
        required = true,
        help = "Output directory for split files"
    )]
    pub output_dir: PathBuf,

    /// Page specification (e.g., "1-5", "odd", "1-10,!2")
    #[arg(short, long, required = true)]
    pub pages: String,
}

#[derive(Args, Clone)]
pub struct CompressArgs {
    /// Input PDF file
    #[arg(required = true, help = "Input PDF file")]
    pub input: PathBuf,

    /// Output PDF file path
    #[arg(short, long, required = true, help = "Output PDF file path")]
    pub output: PathBuf,

    /// Compression level: light (preserves quality), standard (default), or high (smallest size)
    #[arg(short, long, default_value = "standard")]
    pub level: String,
}

#[derive(Args, Clone)]
pub struct LinearizeArgs {
    /// Input PDF file
    #[arg(required = true, help = "Input PDF file")]
    pub input: PathBuf,

    /// Output PDF file path
    #[arg(short, long, required = true, help = "Output PDF file path")]
    pub output: PathBuf,
}

#[derive(Args, Clone)]
pub struct ReorderArgs {
    /// Input PDF file
    #[arg(required = true, help = "Input PDF file")]
    pub input: PathBuf,

    /// Output PDF file path
    #[arg(short, long, required = true, help = "Output PDF file path")]
    pub output: PathBuf,

    /// Page order (comma-separated, 1-indexed). Example: "3,1,2" means page 3, then 1, then 2
    #[arg(
        short,
        long,
        required = true,
        help = "Page order (comma-separated, 1-indexed). Example: 3,1,2 means page 3, then 1, then 2"
    )]
    pub pages: String,
}

#[derive(Args, Clone)]
pub struct ExtractArgs {
    /// Input PDF file
    #[arg(required = true, help = "Input PDF file")]
    pub input: PathBuf,

    /// Output PDF file path (required when format is 'pdf')
    #[arg(
        short,
        long,
        help = "Output PDF file path (required when format is 'pdf')"
    )]
    pub output: Option<PathBuf>,

    /// Output directory path (required when format is 'png' or 'jpeg')
    #[arg(
        short = 'd',
        long,
        help = "Output directory path (required when format is 'png' or 'jpeg')"
    )]
    pub output_dir: Option<PathBuf>,

    /// Page specification (e.g., "1-5", "odd", "1-10,!2")
    #[arg(short, long, required = true)]
    pub pages: String,

    /// Output format: pdf (default), png, or jpeg/jpg
    #[arg(long, default_value = "pdf")]
    pub format: String,
}

pub fn handle_pdf_command(cmd: PdfCommand, plan_only: bool, json_output: bool) -> Result<()> {
    match cmd {
        PdfCommand::Merge(args) => handle_merge(args, plan_only, json_output),
        PdfCommand::Split(args) => handle_split(args, plan_only, json_output),
        PdfCommand::Compress(args) => handle_compress(args, plan_only, json_output),
        PdfCommand::Linearize(args) => handle_linearize(args, plan_only, json_output),
        PdfCommand::Reorder(args) => handle_reorder(args, plan_only, json_output),
        PdfCommand::Extract(args) => handle_extract(args, plan_only, json_output),
    }
}

fn handle_merge(args: MergeArgs, plan_only: bool, json_output: bool) -> Result<()> {
    let spec = JobSpec::PdfMerge {
        inputs: args.inputs,
        output: args.output,
        linearize: args.linearize,
    };

    if plan_only {
        let plan = forgekit_core::job::executor::execute_job(&spec, true)?;
        if json_output {
            // For plan mode, just output the plan as a message
            let event = forgekit_core::job::progress::ProgressEvent::Progress {
                version: 1,
                job_id: forgekit_core::job::progress::new_job_id(),
                progress: forgekit_core::job::progress::ProgressInfo {
                    current: 0,
                    total: 1,
                    percent: 0,
                    stage: Some("plan".to_string()),
                },
                message: plan.clone(),
            };
            println!("{}", serde_json::to_string(&event).unwrap());
        } else {
            println!("{}", plan);
        }
        Ok(())
    } else {
        if json_output {
            let reporter = forgekit_core::job::progress::JsonProgressReporter;
            forgekit_core::job::executor::execute_job_with_progress(&spec, false, &reporter)?;
        } else {
            let result = forgekit_core::job::executor::execute_job(&spec, false)?;
            println!("{}", result);
        }
        Ok(())
    }
}

fn handle_split(args: SplitArgs, plan_only: bool, json_output: bool) -> Result<()> {
    let pages = PageSpec::parse(&args.pages)?;

    let spec = JobSpec::PdfSplit {
        input: args.input,
        output_dir: args.output_dir,
        pages,
    };

    if plan_only {
        let plan = forgekit_core::job::executor::execute_job(&spec, true)?;
        if json_output {
            let event = forgekit_core::job::progress::ProgressEvent::Progress {
                version: 1,
                job_id: forgekit_core::job::progress::new_job_id(),
                progress: forgekit_core::job::progress::ProgressInfo {
                    current: 0,
                    total: 1,
                    percent: 0,
                    stage: Some("plan".to_string()),
                },
                message: plan.clone(),
            };
            println!("{}", serde_json::to_string(&event).unwrap());
        } else {
            println!("{}", plan);
        }
        Ok(())
    } else {
        let result = forgekit_core::job::executor::execute_job(&spec, false)?;
        if json_output {
            // For split, we don't have progress reporting yet, so just output result
            let event = forgekit_core::job::progress::ProgressEvent::Complete {
                version: 1,
                job_id: forgekit_core::job::progress::new_job_id(),
                result: forgekit_core::job::progress::JobResult {
                    output: result.clone(),
                    size_bytes: 0,
                    duration_ms: 0,
                },
            };
            println!("{}", serde_json::to_string(&event).unwrap());
        } else {
            println!("{}", result);
        }
        Ok(())
    }
}

fn handle_compress(args: CompressArgs, plan_only: bool, json_output: bool) -> Result<()> {
    let spec = JobSpec::PdfCompress {
        input: args.input,
        output: args.output,
        level: args.level,
    };

    if plan_only {
        let plan = forgekit_core::job::executor::execute_job(&spec, true)?;
        if json_output {
            let event = forgekit_core::job::progress::ProgressEvent::Progress {
                version: 1,
                job_id: forgekit_core::job::progress::new_job_id(),
                progress: forgekit_core::job::progress::ProgressInfo {
                    current: 0,
                    total: 1,
                    percent: 0,
                    stage: Some("plan".to_string()),
                },
                message: plan.clone(),
            };
            println!("{}", serde_json::to_string(&event).unwrap());
        } else {
            println!("{}", plan);
        }
        Ok(())
    } else {
        if json_output {
            let reporter = forgekit_core::job::progress::JsonProgressReporter;
            forgekit_core::job::executor::execute_job_with_progress(&spec, false, &reporter)?;
        } else {
            let result = forgekit_core::job::executor::execute_job(&spec, false)?;
            println!("{}", result);
        }
        Ok(())
    }
}

fn handle_linearize(args: LinearizeArgs, plan_only: bool, json_output: bool) -> Result<()> {
    let spec = JobSpec::PdfLinearize {
        input: args.input,
        output: args.output,
    };

    if plan_only {
        let plan = forgekit_core::job::executor::execute_job(&spec, true)?;
        if json_output {
            let event = forgekit_core::job::progress::ProgressEvent::Progress {
                version: 1,
                job_id: forgekit_core::job::progress::new_job_id(),
                progress: forgekit_core::job::progress::ProgressInfo {
                    current: 0,
                    total: 1,
                    percent: 0,
                    stage: Some("plan".to_string()),
                },
                message: plan.clone(),
            };
            println!("{}", serde_json::to_string(&event).unwrap());
        } else {
            println!("{}", plan);
        }
        Ok(())
    } else {
        if json_output {
            let reporter = forgekit_core::job::progress::JsonProgressReporter;
            forgekit_core::job::executor::execute_job_with_progress(&spec, false, &reporter)?;
        } else {
            let result = forgekit_core::job::executor::execute_job(&spec, false)?;
            println!("{}", result);
        }
        Ok(())
    }
}

fn handle_reorder(args: ReorderArgs, plan_only: bool, json_output: bool) -> Result<()> {
    // Parse page order from comma-separated string
    let page_order: Vec<u32> = args
        .pages
        .split(',')
        .map(|s| s.trim().parse::<u32>())
        .collect::<std::result::Result<Vec<u32>, _>>()
        .map_err(
            |_| forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason:
                    "Invalid page order format. Expected comma-separated numbers (e.g., '3,1,2')"
                        .to_string(),
            },
        )?;

    if page_order.is_empty() {
        return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "Page order cannot be empty".to_string(),
        });
    }

    let spec = JobSpec::PdfReorder {
        input: args.input,
        output: args.output,
        page_order,
    };

    if plan_only {
        let plan = forgekit_core::job::executor::execute_job(&spec, true)?;
        if json_output {
            let event = forgekit_core::job::progress::ProgressEvent::Progress {
                version: 1,
                job_id: forgekit_core::job::progress::new_job_id(),
                progress: forgekit_core::job::progress::ProgressInfo {
                    current: 0,
                    total: 1,
                    percent: 0,
                    stage: Some("plan".to_string()),
                },
                message: plan.clone(),
            };
            println!("{}", serde_json::to_string(&event).unwrap());
        } else {
            println!("{}", plan);
        }
        Ok(())
    } else {
        if json_output {
            let reporter = forgekit_core::job::progress::JsonProgressReporter;
            forgekit_core::job::executor::execute_job_with_progress(&spec, false, &reporter)?;
        } else {
            let result = forgekit_core::job::executor::execute_job(&spec, false)?;
            println!("{}", result);
        }
        Ok(())
    }
}

fn handle_extract(args: ExtractArgs, plan_only: bool, json_output: bool) -> Result<()> {
    // Validate format and required paths
    match args.format.as_str() {
        "pdf" => {
            if args.output.is_none() {
                return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: "Output file path required when format is 'pdf'. Use --output <file>"
                        .to_string(),
                });
            }
        }
        "png" | "jpeg" | "jpg" => {
            if args.output_dir.is_none() {
                return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: format!("Output directory path required when format is '{}'. Use --output-dir <dir>", args.format),
                });
            }
        }
        _ => {
            return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: format!(
                    "Unknown format '{}'. Supported formats: pdf, png, jpeg, jpg",
                    args.format
                ),
            });
        }
    }

    let pages = PageSpec::parse(&args.pages)?;

    let spec = JobSpec::PdfExtract {
        input: args.input,
        output: args.output,
        output_dir: args.output_dir,
        pages,
        format: args.format,
    };

    if plan_only {
        let plan = forgekit_core::job::executor::execute_job(&spec, true)?;
        if json_output {
            let event = forgekit_core::job::progress::ProgressEvent::Progress {
                version: 1,
                job_id: forgekit_core::job::progress::new_job_id(),
                progress: forgekit_core::job::progress::ProgressInfo {
                    current: 0,
                    total: 1,
                    percent: 0,
                    stage: Some("plan".to_string()),
                },
                message: plan.clone(),
            };
            println!("{}", serde_json::to_string(&event).unwrap());
        } else {
            println!("{}", plan);
        }
        Ok(())
    } else {
        if json_output {
            let reporter = forgekit_core::job::progress::JsonProgressReporter;
            forgekit_core::job::executor::execute_job_with_progress(&spec, false, &reporter)?;
        } else {
            let result = forgekit_core::job::executor::execute_job(&spec, false)?;
            println!("{}", result);
        }
        Ok(())
    }
}
