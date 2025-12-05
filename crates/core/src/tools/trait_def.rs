//! # Tool Trait System
//!
//! ForgeKit doesn't bundle external tools - instead, it finds and uses tools
//! already installed on your system. This keeps the binary small and lets you
//! use your system's package manager to keep tools updated.
//!
//! ## How It Works
//!
//! Each external tool (qpdf, ffmpeg, etc.) implements the `Tool` trait. The trait
//! handles three things:
//!
//! 1. **Finding the tool** - Checks PATH, config files, and CLI flags
//! 2. **Checking version** - Verifies it meets minimum requirements
//! 3. **Running the tool** - Executes commands and parses output
//!
//! ## Adding a New Tool
//!
//! To add support for a new tool:
//!
//! 1. Create a struct (e.g., `MyTool`) and implement `Tool`
//! 2. Implement `probe()` to find the tool (check PATH, handle overrides)
//! 3. Implement `version()` to parse version output
//! 4. Add execution logic in the appropriate executor function
//!
//! See `qpdf.rs` for a complete example.

use std::path::PathBuf;

use crate::utils::error::Result;

/// Configuration for tool detection.
///
/// Allows overriding the tool path via CLI flags (`--tools.qpdf=/path/to/qpdf`)
/// or config files. If `override_path` is set, that path is used instead of
/// searching PATH.
#[derive(Debug, Clone, Default)]
pub struct ToolConfig {
    /// Override path for this tool (from `--tools.<name>=path` or config file).
    /// If set, this path is used directly without checking PATH.
    pub override_path: Option<PathBuf>,
}

/// Information about a tool discovered on the system.
#[derive(Debug, Clone)]
pub struct ToolInfo {
    /// Full path to the tool executable.
    pub path: PathBuf,
    /// Version string (e.g., "11.0.0" or "qpdf version 11.0.0").
    pub version: String,
    /// Whether the tool is available and working.
    pub available: bool,
}

/// Trait for external command-line tools that ForgeKit uses.
///
/// Each external tool (qpdf, ffmpeg, libvips, etc.) implements this trait.
/// The implementation handles finding the tool, checking its version, and
/// (in the executor) running it with the right arguments.
///
/// ## Why Not Just Shell Out?
///
/// We could just run `qpdf` directly, but the trait system gives us:
/// - Consistent error handling (tool not found, wrong version, etc.)
/// - Easy testing (mock the trait)
/// - Clear separation of concerns (tool detection vs. execution)
/// - Better error messages with install hints
pub trait Tool: Send + Sync {
    /// Name of the tool (e.g., "qpdf", "ffmpeg").
    ///
    /// Used in error messages and install hints.
    fn name(&self) -> &'static str;

    /// Probe for the tool on the system.
    ///
    /// Checks PATH first, then falls back to `config.override_path` if set.
    /// Returns `ToolInfo` if found, or an error with install hints if not.
    ///
    /// # Errors
    ///
    /// Returns `ForgeKitError::ToolNotFound` if the tool isn't found, with
    /// a helpful hint like "Install with: brew install qpdf".
    fn probe(&self, config: &ToolConfig) -> Result<ToolInfo>;

    /// Get the version of the tool at the given path.
    ///
    /// Runs `tool --version` (or similar) and parses the output. Used to
    /// verify minimum version requirements.
    ///
    /// # Errors
    ///
    /// Returns an error if the tool can't be executed or version can't be parsed.
    fn version(&self, path: &PathBuf) -> Result<String>;
}
