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
    ///   # Merge two PDFs
    ///   forgekit pdf merge doc1.pdf doc2.pdf --output merged.pdf
    ///
    ///   # Merge with linearization and JSON progress
    ///   forgekit pdf merge *.pdf --output all.pdf --linearize --json
    ///
    ///   # See what commands would run
    ///   forgekit pdf merge a.pdf b.pdf --output c.pdf --plan
    ///
    /// Exit codes:
    ///   0  Success
    ///   1  Processing failed (check logs)
    ///   2  Tool not found (install qpdf)
    ///   3  Invalid input (file not found or invalid format)
    ///   4  Permission denied
    ///   5  Disk full
    Merge(MergeArgs),
    /// Split PDF into separate files by page ranges
    ///
    /// Examples:
    ///   # Extract pages 1-3
    ///   forgekit pdf split book.pdf --output-dir pages/ --pages 1-3
    ///
    ///   # Extract odd pages
    ///   forgekit pdf split book.pdf --output-dir pages/ --pages odd
    ///
    ///   # Extract complex range
    ///   forgekit pdf split book.pdf --output-dir pages/ --pages "1-5,10-20,odd"
    ///
    /// Page specification:
    ///   - Numbers: 1, 42
    ///   - Ranges: 1-5, 10-20, 7- (7 to end), -10 (1 to 10)
    ///   - Keywords: odd, even, first, last
    ///   - Exclusions: !2, !5-10
    ///   - Combined: 1-3,5,7-,odd,!2
    Split(SplitArgs),
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
    #[arg(short = 'o', long, required = true, help = "Output directory for split files")]
    pub output_dir: PathBuf,

    /// Page specification (e.g., "1-3,5,7-", "odd", "even", "!2")
    #[arg(short, long, required = true, help = "Page specification: numbers (1, 42), ranges (1-5, 7-), keywords (odd, even, first, last), or exclusions (!2)")]
    pub pages: String,
}

pub fn handle_pdf_command(cmd: PdfCommand, plan_only: bool, json_output: bool) -> Result<()> {
    match cmd {
        PdfCommand::Merge(args) => handle_merge(args, plan_only, json_output),
        PdfCommand::Split(args) => handle_split(args, plan_only, json_output),
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

