//! # ForgeKit Core Library
//!
//! This is the heart of ForgeKit - a pure Rust library that handles all the actual work
//! of processing PDFs, images, and media files. The CLI is just a thin wrapper around this.
//!
//! ## Architecture Overview
//!
//! ForgeKit follows a simple, layered architecture:
//!
//! 1. **Job Specs** (`job::spec`) - Define what you want to do (merge PDFs, resize images, etc.)
//! 2. **Executors** (`job::executor`) - Actually run the jobs by calling external tools
//! 3. **Tools** (`tools`) - Adapters for external command-line tools (qpdf, ffmpeg, etc.)
//! 4. **Progress** (`job::progress`) - Report what's happening as jobs run
//! 5. **Utils** (`utils`) - Helpers for parsing, errors, temp files, etc.
//!
//! ## Key Concepts
//!
//! ### Jobs
//! A `JobSpec` describes what you want to do. For example, "merge these 3 PDFs into one file"
//! or "extract pages 1-5 from this PDF". Jobs are pure data - they don't do anything until
//! you execute them.
//!
//! ### Tools
//! ForgeKit doesn't reinvent the wheel. Instead, it wraps existing tools like `qpdf` for PDF
//! operations and `ffmpeg` for media. Each tool implements the `Tool` trait, which handles
//! finding the tool on your system, checking its version, and running it.
//!
//! ### Progress Reporting
//! When a job runs, it emits progress events. These can be consumed by the CLI (for JSON output)
//! or by a future GUI. The `ProgressReporter` trait lets you plug in different reporting strategies.
//!
//! ## Example: Adding a New Tool
//!
//! Want to add support for a new external tool? Here's the pattern:
//!
//! ```rust,no_run
//! use forgekit_core::tools::{Tool, ToolConfig, ToolInfo};
//! use forgekit_core::utils::error::Result;
//! use std::path::PathBuf;
//!
//! pub struct MyTool;
//!
//! impl Tool for MyTool {
//!     fn name(&self) -> &'static str { "mytool" }
//!
//!     fn probe(&self, config: &ToolConfig) -> Result<ToolInfo> {
//!         // Find the tool in PATH or use config.override_path
//!         // Return ToolInfo with path, version, and availability
//!         # todo!()
//!     }
//!
//!     fn version(&self, path: &PathBuf) -> Result<String> {
//!         // Run `mytool --version` and parse the output
//!         # todo!()
//!     }
//! }
//! ```
//!
//! Then add a `JobSpec` variant and wire it up in the executor. That's it!

pub mod job;
pub mod tools;
pub mod utils;

pub use utils::error::{ExitCode, ForgeKitError, Result};
pub use utils::pages::PageSpec;
