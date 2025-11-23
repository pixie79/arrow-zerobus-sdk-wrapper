//! File rotation utility
//!
//! This module handles file rotation based on size limits.

use std::path::PathBuf;
use tracing::debug;

/// Rotate file if it exceeds maximum size
///
/// Creates a new file path with timestamp suffix when the current file
/// exceeds the maximum size. The caller is responsible for actually
/// creating the new file and closing the old one.
///
/// # Arguments
///
/// * `file_path` - Current file path
/// * `max_size` - Maximum file size in bytes
///
/// # Returns
///
/// Returns the new file path if rotation is needed, or None if not.
pub fn rotate_file_if_needed(
    file_path: &PathBuf,
    max_size: u64,
) -> Result<Option<PathBuf>, std::io::Error> {
    if !file_path.exists() {
        return Ok(None);
    }

    let metadata = std::fs::metadata(file_path)?;
    // Only rotate if file size exceeds max_size (not equal)
    if metadata.len() <= max_size {
        return Ok(None);
    }

    // Generate new file path with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let parent = file_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let extension = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let new_path = parent.join(format!("{}_{}.{}", stem, timestamp, extension));

    debug!(
        "Rotating file {} ({} bytes) to {}",
        file_path.display(),
        metadata.len(),
        new_path.display()
    );

    Ok(Some(new_path))
}

