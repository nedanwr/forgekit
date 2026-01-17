//! # ForgeKit CLI
//!
//! This is the command-line interface for ForgeKit. It's a thin wrapper around
//! the core library (`forgekit_core`) that handles argument parsing, progress
//! output formatting, and exit codes.
//!
//! ## Architecture
//!
//! The CLI follows a simple pattern:
//!
//! 1. **Parse arguments** - Use `clap` to parse CLI args into structured data
//! 2. **Convert to JobSpec** - Transform CLI args into `JobSpec` (core library types)
//! 3. **Execute** - Call `forgekit_core::job::executor::execute_job_with_progress()`
//! 4. **Format output** - Print results or JSON progress events
//! 5. **Exit** - Set exit code based on result
//!
//! ## Adding a New Command
//!
//! 1. Add a variant to the appropriate `*Command` enum (e.g., `PdfCommand`)
//! 2. Add an `Args` struct for the command's arguments
//! 3. Add a handler function (e.g., `handle_pdf_compress()`)
//! 4. Wire it up in the match statement in `main()`
//!
//! See `commands/pdf.rs` for examples.

mod commands;

use clap::{Parser, Subcommand};
use commands::audio::{handle_audio_command, AudioCommand};
use commands::check::handle_check_deps;
use commands::image::{handle_image_command, ImageCommand};
use commands::pdf::{handle_pdf_command, PdfCommand};
use commands::video::{handle_video_command, VideoCommand};
use forgekit_core::utils::error::{ExitCode, ForgeKitError};

/// Main CLI structure.
///
/// Global flags apply to all commands. Subcommands are defined in the `Commands` enum.
#[derive(Parser)]
#[command(
    name = "forgekit",
    about = "Local-first media and PDF toolkit",
    long_about = "ForgeKit - Fast, lightweight, and privacy-focused media toolkit.\n\nQuick Start:\n  forgekit pdf merge doc1.pdf doc2.pdf --output merged.pdf\n  forgekit pdf split book.pdf --output-dir pages/ --pages 1-5\n  forgekit pdf compress large.pdf --output small.pdf --level high"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show what commands would be executed without running them
    #[arg(long, global = true)]
    plan: bool,

    /// Output progress as JSON (NDJSON format, one event per line)
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// PDF operations (merge, split, compress, extract, etc.)
    #[command(subcommand)]
    Pdf(PdfCommand),

    /// Image operations (convert, resize, strip metadata)
    #[command(subcommand)]
    Image(ImageCommand),

    /// Audio operations (convert, normalize, trim, join, etc.)
    #[command(subcommand)]
    Audio(AudioCommand),

    /// Video operations (convert, transcode, trim, thumbnail, etc.)
    #[command(subcommand)]
    Video(VideoCommand),

    /// Check if required dependencies are installed
    CheckDeps,
}

fn main() {
    let cli = Cli::parse();
    let plan_only = cli.plan;
    let json_output = cli.json;

    let result = match &cli.command {
        Some(Commands::Pdf(ref cmd)) => handle_pdf_command(cmd.clone(), plan_only, json_output),
        Some(Commands::Image(ref cmd)) => handle_image_command(cmd.clone(), plan_only, json_output),
        Some(Commands::Audio(ref cmd)) => handle_audio_command(cmd, plan_only, json_output),
        Some(Commands::Video(ref cmd)) => handle_video_command(cmd, plan_only, json_output),
        Some(Commands::CheckDeps) => handle_check_deps(),
        None => {
            println!("ForgeKit - Local-first media and PDF toolkit");
            println!("Use --help for usage information");
            Ok(())
        }
    };

    match result {
        Ok(()) => {
            std::process::exit(ExitCode::Success as i32);
        }
        Err(e) => {
            if json_output {
                // Emit error as JSON
                let error_event = forgekit_core::job::progress::ProgressEvent::Error {
                    version: 1,
                    job_id: forgekit_core::job::progress::new_job_id(),
                    error: forgekit_core::job::progress::ErrorInfo {
                        code: format!("{:?}", e.exit_code()),
                        message: e.to_string(),
                        hint: match &e {
                            ForgeKitError::ToolNotFound { hint, .. } => hint.clone(),
                            _ => "Check logs for details".to_string(),
                        },
                    },
                };
                if let Ok(json) = serde_json::to_string(&error_event) {
                    eprintln!("{}", json);
                }
            } else {
                eprintln!("Error: {}", e);
            }
            std::process::exit(e.exit_code() as i32);
        }
    }
}
