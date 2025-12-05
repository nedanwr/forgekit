//! # Progress Reporting
//!
//! This module handles reporting progress as jobs execute. Progress events are emitted
//! as line-delimited JSON (NDJSON), making them easy to parse and stream.
//!
//! ## Why NDJSON?
//!
//! NDJSON (newline-delimited JSON) is perfect for streaming progress because:
//! - Each line is a complete, parseable JSON object
//! - You can process events as they arrive (no need to wait for completion)
//! - Tools like `jq` work great with it: `forgekit pdf merge --json | jq .progress`
//! - Future GUI can consume the same stream
//!
//! ## Event Types
//!
//! There are three types of events:
//! - **Progress**: Regular updates during execution (e.g., "Processing page 2/10")
//! - **Complete**: Job finished successfully (includes output path and stats)
//! - **Error**: Something went wrong (includes error code and helpful hints)
//!
//! All events include a `job_id` so you can track multiple jobs running concurrently.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A progress event emitted during job execution.
///
/// Events are serialized as NDJSON (one per line) when using `--json` flag.
/// The `version` field allows us to evolve the schema without breaking existing parsers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProgressEvent {
    /// Regular progress update during execution.
    ///
    /// Emitted periodically to show how much work is done. The `progress` field
    /// contains current/total counts and percentage, while `message` provides
    /// human-readable context like "Merging page 2/5".
    #[serde(rename = "progress")]
    Progress {
        /// Schema version (currently 1). Increment when breaking changes are made.
        #[serde(default = "default_version")]
        version: u32,
        /// Unique identifier for this job (UUID v4).
        job_id: String,
        /// Progress information (current, total, percent, optional stage).
        progress: ProgressInfo,
        /// Human-readable message describing what's happening.
        message: String,
    },
    /// Job completed successfully.
    ///
    /// Emitted once when the job finishes. Includes the output file path,
    /// final size, and how long it took.
    #[serde(rename = "complete")]
    Complete {
        /// Schema version (currently 1).
        #[serde(default = "default_version")]
        version: u32,
        /// Unique identifier for this job.
        job_id: String,
        /// Result information (output path, size, duration).
        result: JobResult,
    },
    /// An error occurred during execution.
    ///
    /// Emitted when something goes wrong. Includes an error code, message,
    /// and a hint about how to fix it (like install instructions).
    #[serde(rename = "error")]
    Error {
        /// Schema version (currently 1).
        #[serde(default = "default_version")]
        version: u32,
        /// Unique identifier for this job.
        job_id: String,
        /// Error information (code, message, hint).
        error: ErrorInfo,
    },
}

fn default_version() -> u32 {
    1
}

/// Progress information for a running job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    /// Current progress count (e.g., "2" when processing page 2 of 10).
    pub current: u32,
    /// Total items to process (e.g., "10" for 10 pages).
    pub total: u32,
    /// Completion percentage (0-100).
    pub percent: u32,
    /// Optional stage name (e.g., "merging", "compressing", "validating").
    /// Omitted from JSON if None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
}

/// Result information when a job completes successfully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Path to the output file (absolute or relative).
    pub output: String,
    /// Size of the output file in bytes.
    pub size_bytes: u64,
    /// How long the job took in milliseconds.
    pub duration_ms: u64,
}

/// Error information when a job fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Machine-readable error code (e.g., "TOOL_NOT_FOUND", "INVALID_INPUT").
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Actionable hint for fixing the issue (e.g., "Install with: brew install qpdf").
    pub hint: String,
}

/// Trait for reporting progress events.
///
/// Different implementations can handle events differently:
/// - `JsonProgressReporter`: Prints NDJSON to stdout (for `--json` flag)
/// - `NoOpProgressReporter`: Silently discards events (for normal CLI output)
/// - Future GUI reporter: Updates UI progress bars
///
/// This trait is `Send + Sync` so it can be used across threads if needed.
pub trait ProgressReporter: Send + Sync {
    /// Report a progress event.
    ///
    /// Called by executors whenever something interesting happens (progress update,
    /// completion, error). The reporter decides what to do with it.
    fn report(&self, event: &ProgressEvent);
}

/// Progress reporter that outputs NDJSON to stdout.
///
/// Used when the `--json` flag is set. Each event is printed as a single line
/// of JSON, making it easy to parse with tools like `jq` or stream to other processes.
pub struct JsonProgressReporter;

impl ProgressReporter for JsonProgressReporter {
    fn report(&self, event: &ProgressEvent) {
        if let Ok(json) = serde_json::to_string(event) {
            println!("{}", json);
        }
    }
}

/// Progress reporter that does nothing.
///
/// Used for normal CLI output where we don't want JSON noise. Events are silently
/// discarded, and the CLI prints friendly messages instead.
pub struct NoOpProgressReporter;

impl ProgressReporter for NoOpProgressReporter {
    fn report(&self, _event: &ProgressEvent) {
        // Do nothing - this is for non-JSON output where we print friendly messages instead
    }
}

/// Generate a new unique job ID.
///
/// Uses UUID v4 to ensure uniqueness. Each job gets its own ID so you can track
/// multiple jobs running concurrently.
pub fn new_job_id() -> String {
    Uuid::new_v4().to_string()
}
