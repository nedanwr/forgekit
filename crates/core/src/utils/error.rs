//! # Error Handling
//!
//! ForgeKit uses a custom error type (`ForgeKitError`) that maps to exit codes
//! suitable for shell scripting. This makes it easy to write scripts that check
//! exit codes and handle errors appropriately.
//!
//! ## Exit Codes
//!
//! - `0` - Success
//! - `1` - General error (processing failed)
//! - `2` - Missing tool (with install hint)
//! - `3` - Invalid input (file not found, bad page spec, etc.)
//! - `4` - Permission denied
//! - `5` - Disk full
//! - `130` - Cancelled (SIGINT)
//!
//! ## Error Messages
//!
//! All errors include actionable hints. For example, if qpdf isn't found, the
//! error includes: "Install with: brew install qpdf (macOS) | apt install qpdf (Linux)".

use std::path::PathBuf;
use thiserror::Error;

/// Exit codes for the CLI.
///
/// These follow common Unix conventions and are designed for shell scripting.
/// Scripts can check exit codes to handle different error conditions:
///
/// ```bash
/// if forgekit pdf merge a.pdf b.pdf --output c.pdf; then
///     echo "Success!"
/// elif [ $? -eq 2 ]; then
///     echo "Tool missing - install dependencies"
/// elif [ $? -eq 3 ]; then
///     echo "Invalid input - check file paths"
/// fi
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Operation completed successfully.
    Success = 0,
    /// General error (processing failed, tool error, etc.).
    GeneralError = 1,
    /// Required external tool not found (includes install hint).
    MissingTool = 2,
    /// Invalid input (file not found, bad page spec, etc.).
    InvalidInput = 3,
    /// Permission denied (can't read input or write output).
    PermissionDenied = 4,
    /// Disk full (can't write output file).
    DiskFull = 5,
    /// Operation cancelled by user (SIGINT).
    Cancelled = 130,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

/// Main error type for ForgeKit.
///
/// All errors in ForgeKit are represented by this enum. Each variant includes
/// enough context to provide helpful error messages and actionable hints.
///
/// Errors automatically convert to `ExitCode` via `exit_code()`, which is used
/// by the CLI to set the process exit code.
#[derive(Error, Debug)]
pub enum ForgeKitError {
    /// External tool not found in PATH or config.
    ///
    /// The `hint` field contains OS-specific install instructions like
    /// "Install with: brew install qpdf (macOS) | apt install qpdf (Linux)".
    #[error("Tool '{tool}' not found: {hint}")]
    ToolNotFound { tool: String, hint: String },

    /// External tool version doesn't meet minimum requirements.
    ///
    /// Used when a tool is found but is too old. The `required` field specifies
    /// the minimum version needed.
    #[error("Tool '{tool}' version mismatch: required {required}, found {found}")]
    ToolVersionMismatch {
        tool: String,
        required: String,
        found: String,
    },

    /// Invalid input file or parameter.
    ///
    /// Used for things like: file doesn't exist, invalid page spec, malformed
    /// arguments, etc. The `reason` field explains what's wrong.
    #[error("Invalid input '{path:?}': {reason}")]
    InvalidInput { path: PathBuf, reason: String },

    /// External tool execution failed.
    ///
    /// The tool was found and executed, but it returned an error. The `stderr`
    /// field contains the tool's error output (often helpful for debugging).
    #[error("Processing failed with {tool}: {stderr}")]
    ProcessingFailed { tool: String, stderr: String },

    /// Permission denied when accessing a file or directory.
    #[error("Permission denied: {path:?}")]
    PermissionDenied { path: PathBuf },

    /// Disk is full, can't write output file.
    #[error("Disk full: {path:?}")]
    DiskFull { path: PathBuf },

    /// Operation was cancelled by user (e.g., Ctrl+C).
    #[error("Operation cancelled")]
    Cancelled,

    /// I/O error (wrapped from `std::io::Error`).
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Other error (wrapped from `anyhow::Error`).
    ///
    /// Catch-all for errors that don't fit the other categories. Should be
    /// rare - prefer specific variants when possible.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ForgeKitError {
    /// Get the exit code that corresponds to this error.
    ///
    /// Used by the CLI to set the process exit code. Scripts can check this
    /// to handle different error conditions appropriately.
    pub fn exit_code(&self) -> ExitCode {
        match self {
            ForgeKitError::ToolNotFound { .. } => ExitCode::MissingTool,
            ForgeKitError::InvalidInput { .. } => ExitCode::InvalidInput,
            ForgeKitError::PermissionDenied { .. } => ExitCode::PermissionDenied,
            ForgeKitError::DiskFull { .. } => ExitCode::DiskFull,
            ForgeKitError::Cancelled => ExitCode::Cancelled,
            _ => ExitCode::GeneralError,
        }
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, ForgeKitError>;

