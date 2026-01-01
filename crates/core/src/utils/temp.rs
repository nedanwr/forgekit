//! # Temporary File Utilities
//!
//! Helpers for creating temporary files safely. All temp files use UUIDs in
//! their names to avoid collisions, even when multiple ForgeKit processes
//! run simultaneously.
//!
//! ## Atomic Writes
//!
//! The `atomic_write()` function ensures that output files are either completely
//! written or not written at all. It writes to a temp file first, then atomically
//! renames it to the final location. This prevents partial/corrupted files if
//! the process is interrupted.

use crate::utils::error::{ForgeKitError, Result};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Create a temporary file with a unique name in the system temp directory.
///
/// The filename format is `{prefix}-{uuid}{suffix}`. The UUID ensures uniqueness
/// even if multiple processes create temp files simultaneously.
///
/// # Example
///
/// ```
/// use forgekit_core::utils::temp::create_temp_file;
/// use forgekit_core::utils::error::Result;
///
/// # fn main() -> Result<()> {
/// let temp = create_temp_file("forgekit", ".pdf")?;
/// // temp might be: /tmp/forgekit-550e8400-e29b-41d4-a716-446655440000.pdf
/// # Ok(())
/// # }
/// ```
pub fn create_temp_file(prefix: &str, suffix: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let unique_id = Uuid::new_v4();
    let filename = format!("{}-{}{}", prefix, unique_id, suffix);
    let temp_path = temp_dir.join(filename);

    Ok(temp_path)
}

/// Create a temporary file in the same directory as the given path.
///
/// Useful when you want the temp file on the same filesystem as the target
/// (required for atomic renames on some systems). The filename format is
/// `{prefix}-{uuid}{suffix}`.
///
/// # Example
///
/// ```
/// use forgekit_core::utils::temp::create_temp_file_near;
/// use forgekit_core::utils::error::Result;
/// use std::path::PathBuf;
///
/// # fn main() -> Result<()> {
/// let target = PathBuf::from("/home/user/output.pdf");
/// let temp = create_temp_file_near(&target, "forgekit", ".tmp")?;
/// // temp might be: /home/user/forgekit-550e8400-e29b-41d4-a716-446655440000.tmp
/// # Ok(())
/// # }
/// ```
pub fn create_temp_file_near(path: &Path, prefix: &str, suffix: &str) -> Result<PathBuf> {
    let parent = path.parent().ok_or_else(|| ForgeKitError::InvalidInput {
        path: path.to_path_buf(),
        reason: "Path has no parent directory".to_string(),
    })?;

    let unique_id = Uuid::new_v4();
    let filename = format!("{}-{}{}", prefix, unique_id, suffix);
    let temp_path = parent.join(filename);

    Ok(temp_path)
}

/// Atomically write to a file by writing to a temp file first, then renaming.
///
/// This ensures the target file is either completely written or doesn't exist
/// at all. If the process is interrupted (Ctrl+C, crash, etc.), you won't end
/// up with a partial/corrupted file.
///
/// The temp file is created in the same directory as the target (required for
/// atomic renames on some filesystems), then atomically renamed to the final
/// location.
///
/// # Example
///
/// ```
/// use forgekit_core::utils::temp::atomic_write;
/// use forgekit_core::utils::error::Result;
///
/// # fn main() -> Result<()> {
/// let target_path = std::env::temp_dir().join("forgekit_test_output.txt");
/// atomic_write(&target_path, |temp_path| {
///     // Write your data to temp_path
///     std::fs::write(temp_path, b"hello world")?;
///     Ok(())
/// })?;
/// // target_path now exists with the data, or an error was returned
/// # std::fs::remove_file(&target_path).ok(); // cleanup
/// # Ok(())
/// # }
/// ```
pub fn atomic_write<F>(target: &Path, write_fn: F) -> Result<()>
where
    F: FnOnce(&Path) -> Result<()>,
{
    let temp_path = create_temp_file_near(target, "forgekit", ".tmp")?;

    // Write to temp file
    write_fn(&temp_path)?;

    // Atomic rename
    std::fs::rename(&temp_path, target).map_err(ForgeKitError::Io)?;

    Ok(())
}
