//! File rotation utility
//!
//! This module handles file rotation based on size limits.

use regex::Regex;
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
    // Extract base filename without existing timestamps to prevent recursive appending
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let parent = file_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");

    // Pattern to match timestamp at end of filename: YYYYMMDD_HHMMSS
    let timestamp_pattern = Regex::new(r"_\d{8}_\d{6}$").unwrap();

    // Extract base filename without timestamp
    let base_stem = if timestamp_pattern.is_match(stem) {
        timestamp_pattern.replace(stem, "").to_string()
    } else {
        stem.to_string()
    };

    // Check if resulting filename would exceed filesystem limits (255 chars typical)
    let new_filename = format!("{}_{}.{}", base_stem, timestamp, extension);
    let new_path = if new_filename.len() > 250 {
        // Use sequential numbering instead of timestamp if filename too long
        let seq_pattern = Regex::new(r"_(\d+)$").unwrap();
        let next_num = if let Some(captures) = seq_pattern.captures(&base_stem) {
            captures
                .get(1)
                .and_then(|m| m.as_str().parse::<usize>().ok())
                .map(|n| n + 1)
                .unwrap_or(1)
        } else {
            1
        };

        // Remove any existing sequential number
        let clean_base = seq_pattern.replace(&base_stem, "").to_string();
        parent.join(format!("{}_{}.{}", clean_base, next_num, extension))
    } else {
        parent.join(new_filename)
    };

    debug!(
        "Rotating file {} ({} bytes) to {}",
        file_path.display(),
        metadata.len(),
        new_path.display()
    );

    Ok(Some(new_path))
}
