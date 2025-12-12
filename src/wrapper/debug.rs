//! Debug file writing
//!
//! This module handles writing Arrow and Protobuf debug files for inspection.
//! Uses Arrow IPC Stream format (*.arrows) for better compatibility with DuckDB.

use crate::error::ZerobusError;
use crate::utils::file_rotation::rotate_file_if_needed;
use arrow::record_batch::RecordBatch;
use prost::Message;
use prost_types::DescriptorProto;
use regex::Regex;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Batch size for file rotation (matches BATCH_SIZE in mod.rs)
const ROTATION_BATCH_SIZE: usize = 1000;

/// Debug file writer
///
/// Handles writing Arrow RecordBatch and Protobuf files to disk for debugging.
/// Uses Arrow IPC Stream format (*.arrows) which is readable by DuckDB.
pub struct DebugWriter {
    /// Output directory for debug files
    #[allow(dead_code)]
    output_dir: PathBuf,
    /// Arrow IPC stream writer
    arrow_writer:
        Arc<tokio::sync::Mutex<Option<arrow::ipc::writer::StreamWriter<BufWriter<std::fs::File>>>>>,
    /// Protobuf file writer
    protobuf_writer: Arc<tokio::sync::Mutex<Option<BufWriter<std::fs::File>>>>,
    /// Current Arrow file path (mutable for rotation)
    arrow_file_path: Arc<tokio::sync::Mutex<PathBuf>>,
    /// Current Protobuf file path (mutable for rotation)
    protobuf_file_path: Arc<tokio::sync::Mutex<PathBuf>>,
    /// Flush interval
    flush_interval: Duration,
    /// Maximum file size before rotation (optional, secondary to record count)
    max_file_size: Option<u64>,
    /// Maximum number of rotated files to retain per type (optional, default: Some(10))
    max_files_retained: Option<usize>,
    /// Timestamp of last flush
    last_flush: Arc<Mutex<Instant>>,
    /// Number of records written to current Arrow file
    arrow_record_count: Arc<Mutex<usize>>,
    /// Number of records written to current Protobuf file
    protobuf_record_count: Arc<Mutex<usize>>,
}

impl DebugWriter {
    /// Create a new debug writer
    ///
    /// # Arguments
    ///
    /// * `output_dir` - Output directory for debug files
    /// * `table_name` - Table name (used for filename)
    /// * `flush_interval` - Interval for periodic flushing
    /// * `max_file_size` - Maximum file size before rotation (optional, secondary to record count)
    /// * `max_files_retained` - Maximum number of rotated files to retain per type (optional, default: Some(10))
    ///
    /// # Returns
    ///
    /// Returns debug writer instance, or error if initialization fails.
    pub fn new(
        output_dir: PathBuf,
        table_name: String,
        flush_interval: Duration,
        max_file_size: Option<u64>,
        max_files_retained: Option<usize>,
    ) -> Result<Self, ZerobusError> {
        // Create output directories
        let arrow_dir = output_dir.join("zerobus/arrow");
        let proto_dir = output_dir.join("zerobus/proto");

        std::fs::create_dir_all(&arrow_dir).map_err(|e| {
            ZerobusError::ConfigurationError(format!(
                "Failed to create arrow output directory: {}",
                e
            ))
        })?;

        std::fs::create_dir_all(&proto_dir).map_err(|e| {
            ZerobusError::ConfigurationError(format!(
                "Failed to create proto output directory: {}",
                e
            ))
        })?;

        // Sanitize table name for filesystem (replace dots and slashes with underscores)
        let sanitized_table_name = table_name.replace(['.', '/'], "_");
        let arrow_file_path = arrow_dir.join(format!("{}.arrows", sanitized_table_name));
        let protobuf_file_path = proto_dir.join(format!("{}.proto", sanitized_table_name));

        Ok(Self {
            output_dir,
            arrow_writer: Arc::new(tokio::sync::Mutex::new(None)),
            protobuf_writer: Arc::new(tokio::sync::Mutex::new(None)),
            arrow_file_path: Arc::new(tokio::sync::Mutex::new(arrow_file_path)),
            protobuf_file_path: Arc::new(tokio::sync::Mutex::new(protobuf_file_path)),
            flush_interval,
            max_file_size,
            max_files_retained,
            last_flush: Arc::new(Mutex::new(Instant::now())),
            arrow_record_count: Arc::new(Mutex::new(0)),
            protobuf_record_count: Arc::new(Mutex::new(0)),
        })
    }

    /// Generate rotated file path with timestamp
    ///
    /// Extracts the base filename without any existing timestamps before appending a new timestamp.
    /// This prevents recursive timestamp appending (e.g., `file_20250101_120000_20250101_120001`).
    ///
    /// # Behavior
    ///
    /// - Detects timestamp pattern `_YYYYMMDD_HHMMSS` at the end of filename (before extension)
    /// - Removes existing timestamp before appending new one
    /// - Uses sequential numbering (`_1`, `_2`, etc.) if filename would exceed filesystem limits (250 chars)
    ///
    /// # Arguments
    ///
    /// * `base_path` - Current file path to rotate
    ///
    /// # Returns
    ///
    /// New file path with timestamp or sequential number suffix
    ///
    /// # Example
    ///
    /// ```
    /// // Input: `table.arrows`
    /// // Output: `table_20251212_143022.arrows`
    ///
    /// // Input: `table_20251212_143022.arrows` (already rotated)
    /// // Output: `table_20251212_143523.arrows` (timestamp replaced, not appended)
    /// ```
    fn generate_rotated_path(base_path: &std::path::Path) -> PathBuf {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let parent = base_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let stem = base_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let extension = base_path.extension().and_then(|s| s.to_str()).unwrap_or("");

        // Pattern to match timestamp at end of filename: YYYYMMDD_HHMMSS
        // Matches exactly 8 digits, underscore, exactly 6 digits at the end
        let timestamp_pattern = Regex::new(r"_\d{8}_\d{6}$").unwrap();

        // Extract base filename without timestamp
        let base_stem = if timestamp_pattern.is_match(stem) {
            // Remove the timestamp suffix
            timestamp_pattern.replace(stem, "").to_string()
        } else {
            stem.to_string()
        };

        // Check if resulting filename would exceed filesystem limits (255 chars typical)
        let new_filename = format!("{}_{}.{}", base_stem, timestamp, extension);
        if new_filename.len() > 250 {
            // Use sequential numbering instead of timestamp if filename too long
            // Extract any existing sequential number
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
            let short_filename = format!("{}_{}.{}", clean_base, next_num, extension);
            parent.join(short_filename)
        } else {
            parent.join(new_filename)
        }
    }

    /// Ensure Arrow writer is initialized
    async fn ensure_arrow_writer(
        &self,
        schema: &arrow::datatypes::Schema,
    ) -> Result<(), ZerobusError> {
        let mut writer_guard = self.arrow_writer.lock().await;
        if writer_guard.is_none() {
            let file_path_guard = self.arrow_file_path.lock().await;
            let file_path = file_path_guard.clone();
            drop(file_path_guard);

            let file = std::fs::File::create(&file_path).map_err(|e| {
                ZerobusError::ConfigurationError(format!(
                    "Failed to create Arrow debug file: {}",
                    e
                ))
            })?;

            let buf_writer = BufWriter::new(file);
            let writer =
                arrow::ipc::writer::StreamWriter::try_new(buf_writer, schema).map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to create Arrow IPC stream writer: {}",
                        e
                    ))
                })?;

            *writer_guard = Some(writer);
            info!("✅ Created Arrow IPC stream file: {}", file_path.display());
        }
        Ok(())
    }

    /// Ensure Protobuf writer is initialized
    async fn ensure_protobuf_writer(&self) -> Result<(), ZerobusError> {
        let mut writer_guard = self.protobuf_writer.lock().await;
        if writer_guard.is_none() {
            let file_path_guard = self.protobuf_file_path.lock().await;
            let file_path = file_path_guard.clone();
            drop(file_path_guard);

            let file = std::fs::File::create(&file_path).map_err(|e| {
                ZerobusError::ConfigurationError(format!(
                    "Failed to create Protobuf debug file: {}",
                    e
                ))
            })?;
            *writer_guard = Some(BufWriter::new(file));
            info!("✅ Created Protobuf file: {}", file_path.display());
        }
        Ok(())
    }

    /// Rotate Arrow file if needed (based on record count or file size)
    async fn rotate_arrow_file_if_needed(&self, batch_rows: usize) -> Result<bool, ZerobusError> {
        let mut record_count_guard = self.arrow_record_count.lock().await;
        let current_count = *record_count_guard;
        let new_count = current_count + batch_rows;

        // Check if rotation is needed based on record count
        let needs_rotation = new_count >= ROTATION_BATCH_SIZE;

        if needs_rotation {
            // Close current writer
            let mut writer_guard = self.arrow_writer.lock().await;
            if let Some(writer) = writer_guard.take() {
                // StreamWriter doesn't need finish() - just drop it
                drop(writer);
            }
            drop(writer_guard);

            // Generate new file path
            let mut file_path_guard = self.arrow_file_path.lock().await;
            let old_path = file_path_guard.clone();
            let new_path = Self::generate_rotated_path(&old_path);
            *file_path_guard = new_path.clone();
            drop(file_path_guard);

            // Reset record count
            *record_count_guard = 0;

            info!(
                "🔄 Rotated Arrow file: {} -> {} (wrote {} records)",
                old_path.display(),
                new_path.display(),
                current_count
            );

            // Cleanup old files if retention limit is set
            if let Some(max_files) = self.max_files_retained {
                if let Err(e) = Self::cleanup_old_files(
                    old_path.parent().unwrap(),
                    "arrows",
                    max_files,
                    &new_path,
                )
                .await
                {
                    warn!("Failed to cleanup old Arrow files: {}", e);
                    // Don't fail rotation if cleanup fails
                }
            }

            Ok(true)
        } else {
            // Also check file size if configured
            if let Some(max_size) = self.max_file_size {
                let file_path_guard = self.arrow_file_path.lock().await;
                let file_path = file_path_guard.clone();
                drop(file_path_guard);

                if let Some(new_path) =
                    rotate_file_if_needed(&file_path, max_size).map_err(|e| {
                        ZerobusError::ConfigurationError(format!(
                            "Failed to check Arrow file size: {}",
                            e
                        ))
                    })?
                {
                    // Close current writer
                    let mut writer_guard = self.arrow_writer.lock().await;
                    if let Some(writer) = writer_guard.take() {
                        drop(writer);
                    }
                    drop(writer_guard);

                    // Update file path
                    let mut file_path_guard = self.arrow_file_path.lock().await;
                    *file_path_guard = new_path.clone();
                    drop(file_path_guard);

                    // Reset record count
                    *record_count_guard = 0;

                    info!(
                        "🔄 Rotated Arrow file due to size limit: {}",
                        new_path.display()
                    );

                    // Cleanup old files if retention limit is set
                    if let Some(max_files) = self.max_files_retained {
                        if let Err(e) = Self::cleanup_old_files(
                            file_path.parent().unwrap(),
                            "arrows",
                            max_files,
                            &new_path,
                        )
                        .await
                        {
                            warn!("Failed to cleanup old Arrow files: {}", e);
                            // Don't fail rotation if cleanup fails
                        }
                    }

                    return Ok(true);
                }
            }
            Ok(false)
        }
    }

    /// Rotate Protobuf file if needed (based on record count or file size)
    async fn rotate_protobuf_file_if_needed(
        &self,
        record_count: usize,
    ) -> Result<bool, ZerobusError> {
        let mut record_count_guard = self.protobuf_record_count.lock().await;
        let current_count = *record_count_guard;
        let new_count = current_count + record_count;

        // Check if rotation is needed based on record count
        let needs_rotation = new_count >= ROTATION_BATCH_SIZE;

        if needs_rotation {
            // Close current writer
            let mut writer_guard = self.protobuf_writer.lock().await;
            if let Some(mut writer) = writer_guard.take() {
                writer.flush().map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to flush Protobuf file before rotation: {}",
                        e
                    ))
                })?;
                drop(writer);
            }
            drop(writer_guard);

            // Generate new file path
            let mut file_path_guard = self.protobuf_file_path.lock().await;
            let old_path = file_path_guard.clone();
            let new_path = Self::generate_rotated_path(&old_path);
            *file_path_guard = new_path.clone();
            drop(file_path_guard);

            // Reset record count
            *record_count_guard = 0;

            info!(
                "🔄 Rotated Protobuf file: {} -> {} (wrote {} records)",
                old_path.display(),
                new_path.display(),
                current_count
            );

            // Cleanup old files if retention limit is set
            if let Some(max_files) = self.max_files_retained {
                if let Err(e) = Self::cleanup_old_files(
                    old_path.parent().unwrap(),
                    "proto",
                    max_files,
                    &new_path,
                )
                .await
                {
                    warn!("Failed to cleanup old Protobuf files: {}", e);
                    // Don't fail rotation if cleanup fails
                }
            }

            Ok(true)
        } else {
            // Also check file size if configured
            if let Some(max_size) = self.max_file_size {
                let file_path_guard = self.protobuf_file_path.lock().await;
                let file_path = file_path_guard.clone();
                drop(file_path_guard);

                if let Some(new_path) =
                    rotate_file_if_needed(&file_path, max_size).map_err(|e| {
                        ZerobusError::ConfigurationError(format!(
                            "Failed to check Protobuf file size: {}",
                            e
                        ))
                    })?
                {
                    // Close current writer
                    let mut writer_guard = self.protobuf_writer.lock().await;
                    if let Some(mut writer) = writer_guard.take() {
                        writer.flush().map_err(|e| {
                            ZerobusError::ConfigurationError(format!(
                                "Failed to flush Protobuf file before rotation: {}",
                                e
                            ))
                        })?;
                        drop(writer);
                    }
                    drop(writer_guard);

                    // Update file path
                    let mut file_path_guard = self.protobuf_file_path.lock().await;
                    *file_path_guard = new_path.clone();
                    drop(file_path_guard);

                    // Reset record count
                    *record_count_guard = 0;

                    info!(
                        "🔄 Rotated Protobuf file due to size limit: {}",
                        new_path.display()
                    );

                    // Cleanup old files if retention limit is set
                    if let Some(max_files) = self.max_files_retained {
                        if let Err(e) = Self::cleanup_old_files(
                            file_path.parent().unwrap(),
                            "proto",
                            max_files,
                            &new_path,
                        )
                        .await
                        {
                            warn!("Failed to cleanup old Protobuf files: {}", e);
                            // Don't fail rotation if cleanup fails
                        }
                    }

                    return Ok(true);
                }
            }
            Ok(false)
        }
    }

    /// Write Arrow RecordBatch to debug file
    ///
    /// # Arguments
    ///
    /// * `batch` - RecordBatch to write
    ///
    /// # Errors
    ///
    /// Returns error if file writing fails.
    pub async fn write_arrow(&self, batch: &RecordBatch) -> Result<(), ZerobusError> {
        let batch_rows = batch.num_rows();

        // Check if rotation is needed before writing
        let _rotated = self.rotate_arrow_file_if_needed(batch_rows).await?;

        // Ensure writer is initialized (with correct schema)
        self.ensure_arrow_writer(batch.schema().as_ref()).await?;

        // Write batch
        let mut writer_guard = self.arrow_writer.lock().await;
        if let Some(ref mut writer) = *writer_guard {
            writer.write(batch).map_err(|e| {
                ZerobusError::ConfigurationError(format!(
                    "Failed to write Arrow RecordBatch: {}",
                    e
                ))
            })?;
        }
        drop(writer_guard);

        // Update record count
        let mut record_count_guard = self.arrow_record_count.lock().await;
        *record_count_guard += batch_rows;
        drop(record_count_guard);

        debug!(
            "Wrote Arrow RecordBatch ({} rows) to debug file",
            batch_rows
        );
        Ok(())
    }

    /// Write Protobuf bytes to debug file
    ///
    /// # Arguments
    ///
    /// * `protobuf_bytes` - Protobuf bytes to write
    /// * `flush_immediately` - If true, flush to disk immediately after writing
    ///
    /// # Errors
    ///
    /// Returns error if file writing fails.
    pub async fn write_protobuf(
        &self,
        protobuf_bytes: &[u8],
        flush_immediately: bool,
    ) -> Result<(), ZerobusError> {
        // Check if rotation is needed (each protobuf message = 1 record)
        let _rotated = self.rotate_protobuf_file_if_needed(1).await?;

        // Ensure writer is initialized
        self.ensure_protobuf_writer().await?;

        // Write bytes
        let mut writer_guard = self.protobuf_writer.lock().await;
        if let Some(ref mut writer) = *writer_guard {
            writer.write_all(protobuf_bytes).map_err(|e| {
                ZerobusError::ConfigurationError(format!("Failed to write Protobuf bytes: {}", e))
            })?;

            // Write newline separator for readability (optional)
            writer.write_all(b"\n").map_err(|e| {
                ZerobusError::ConfigurationError(format!(
                    "Failed to write Protobuf separator: {}",
                    e
                ))
            })?;

            // Flush immediately if requested (for per-batch flushing)
            if flush_immediately {
                writer.flush().map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to flush Protobuf file: {}",
                        e
                    ))
                })?;
            }
        }
        drop(writer_guard);

        // Update record count
        let mut record_count_guard = self.protobuf_record_count.lock().await;
        *record_count_guard += 1;
        drop(record_count_guard);

        debug!(
            "Wrote {} bytes to Protobuf debug file{}",
            protobuf_bytes.len(),
            if flush_immediately { " (flushed)" } else { "" }
        );
        Ok(())
    }

    /// Write Protobuf descriptor to file (once per table)
    ///
    /// # Arguments
    ///
    /// * `table_name` - Table name (used for filename)
    /// * `descriptor` - Protobuf descriptor to write
    ///
    /// # Errors
    ///
    /// Returns error if file writing fails.
    pub async fn write_descriptor(
        &self,
        table_name: &str,
        descriptor: &DescriptorProto,
    ) -> Result<(), ZerobusError> {
        // Create descriptors directory
        let descriptors_dir = self.output_dir.join("zerobus/descriptors");
        std::fs::create_dir_all(&descriptors_dir).map_err(|e| {
            ZerobusError::ConfigurationError(format!(
                "Failed to create descriptors directory: {}",
                e
            ))
        })?;

        // Create filename from table name (sanitize for filesystem)
        let sanitized_table_name = table_name.replace(['.', '/'], "_");
        let descriptor_file_path = descriptors_dir.join(format!("{}.pb", sanitized_table_name));

        // Check if file already exists (only write once per table)
        if descriptor_file_path.exists() {
            debug!(
                "Descriptor file already exists for table {}: {}",
                table_name,
                descriptor_file_path.display()
            );
            return Ok(());
        }

        // Serialize descriptor to bytes
        let mut descriptor_bytes = Vec::new();
        descriptor.encode(&mut descriptor_bytes).map_err(|e| {
            ZerobusError::ConfigurationError(format!("Failed to encode Protobuf descriptor: {}", e))
        })?;

        // Write to file
        let mut file = std::fs::File::create(&descriptor_file_path).map_err(|e| {
            ZerobusError::ConfigurationError(format!("Failed to create descriptor file: {}", e))
        })?;

        file.write_all(&descriptor_bytes).map_err(|e| {
            ZerobusError::ConfigurationError(format!("Failed to write descriptor bytes: {}", e))
        })?;

        file.sync_all().map_err(|e| {
            ZerobusError::ConfigurationError(format!("Failed to sync descriptor file: {}", e))
        })?;

        let descriptor_name = descriptor.name.as_deref().unwrap_or("unknown");
        info!("✅ Wrote Protobuf descriptor for table '{}' to: {} (descriptor name: '{}', {} fields, {} nested types)",
              table_name, descriptor_file_path.display(), descriptor_name,
              descriptor.field.len(), descriptor.nested_type.len());

        Ok(())
    }

    /// Cleanup old rotated files, keeping only the most recent N files
    ///
    /// Scans the directory for rotated files matching the base filename pattern,
    /// sorts them by timestamp (or sequential number, or modification time),
    /// and deletes files beyond the retention limit, keeping the newest files.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory containing rotated files
    /// * `extension` - File extension (e.g., "arrows" or "proto")
    /// * `max_files` - Maximum number of files to retain (oldest are deleted first)
    /// * `active_file` - Path to active file (excluded from cleanup and count)
    ///
    /// # Behavior
    ///
    /// - Only processes files matching the base filename pattern
    /// - Excludes the active file from cleanup and retention count
    /// - Sorts files by timestamp (newest first), then by sequential number, then by modification time
    /// - Deletes files beyond the limit (oldest first)
    /// - Logs errors but doesn't fail rotation if cleanup fails
    ///
    /// # Returns
    ///
    /// Returns error if cleanup fails, but errors are logged and don't block rotation.
    ///
    /// # Example
    ///
    /// If `max_files=10` and directory contains 15 rotated files:
    /// - Keeps the 10 newest files
    /// - Deletes the 5 oldest files
    /// - Active file is excluded from count
    async fn cleanup_old_files(
        dir: &std::path::Path,
        extension: &str,
        max_files: usize,
        active_file: &std::path::Path,
    ) -> Result<(), ZerobusError> {
        // Read directory entries
        let entries = std::fs::read_dir(dir).map_err(|e| {
            ZerobusError::ConfigurationError(format!(
                "Failed to read directory {}: {}",
                dir.display(),
                e
            ))
        })?;

        // Extract base filename from active file (without extension)
        let active_stem = active_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // Extract base name without timestamp/sequence (for pattern matching)
        let timestamp_pattern = Regex::new(r"_\d{8}_\d{6}$").unwrap();
        let seq_pattern = Regex::new(r"_\d+$").unwrap();
        let base_name = timestamp_pattern.replace(active_stem, "");
        let base_name = seq_pattern.replace(&base_name, "");

        // Collect matching files with their timestamps/sequence numbers
        let mut file_entries: Vec<(
            PathBuf,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<usize>,
        )> = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                ZerobusError::ConfigurationError(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            // Skip if not a file, wrong extension, or is the active file
            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some(extension) {
                continue;
            }

            if path == active_file {
                continue;
            }

            // Check if filename matches base pattern
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if !stem.starts_with(base_name.as_ref()) {
                continue;
            }

            // Extract timestamp or sequence number
            let timestamp = if timestamp_pattern.is_match(stem) {
                // Try to parse timestamp: YYYYMMDD_HHMMSS
                let timestamp_str = timestamp_pattern.find(stem).unwrap().as_str();
                let date_part = &timestamp_str[1..9]; // Skip underscore, get YYYYMMDD
                let time_part = &timestamp_str[10..]; // Get HHMMSS

                if let (Ok(year), Ok(month), Ok(day), Ok(hour), Ok(minute), Ok(second)) = (
                    date_part[0..4].parse::<i32>(),
                    date_part[4..6].parse::<u32>(),
                    date_part[6..8].parse::<u32>(),
                    time_part[0..2].parse::<u32>(),
                    time_part[2..4].parse::<u32>(),
                    time_part[4..6].parse::<u32>(),
                ) {
                    chrono::NaiveDate::from_ymd_opt(year, month, day)
                        .and_then(|date| date.and_hms_opt(hour, minute, second))
                        .map(|dt| {
                            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                                dt,
                                chrono::Utc,
                            )
                        })
                } else {
                    None
                }
            } else {
                None
            };

            // Extract sequence number if no timestamp
            let sequence = if timestamp.is_none() {
                seq_pattern
                    .captures(stem)
                    .and_then(|c| c.get(1))
                    .and_then(|m| m.as_str().parse::<usize>().ok())
            } else {
                None
            };

            // Get file metadata for fallback sorting
            let metadata = std::fs::metadata(&path).ok();
            let modified_time = metadata
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .and_then(|d| {
                    chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                });

            file_entries.push((path, timestamp.or(modified_time), sequence));
        }

        // Sort by timestamp (newest first), then by sequence (highest first), then by modified time
        file_entries.sort_by(|a, b| {
            match (a.1, b.1) {
                (Some(ta), Some(tb)) => tb.cmp(&ta), // Newest first
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => {
                    match (a.2, b.2) {
                        (Some(sa), Some(sb)) => sb.cmp(&sa), // Highest sequence first
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
            }
        });

        // Delete files beyond the limit
        if file_entries.len() > max_files {
            let files_to_delete = &file_entries[max_files..];
            for (file_path, _, _) in files_to_delete {
                if let Err(e) = std::fs::remove_file(file_path) {
                    warn!("Failed to delete old file {}: {}", file_path.display(), e);
                    // Continue with other files even if one fails
                } else {
                    info!("🗑️  Deleted old rotated file: {}", file_path.display());
                }
            }
        }

        Ok(())
    }

    /// Flush all pending writes to disk
    ///
    /// # Errors
    ///
    /// Returns error if flush fails.
    pub async fn flush(&self) -> Result<(), ZerobusError> {
        // Flush Arrow writer (StreamWriter buffers internally)
        // StreamWriter doesn't have explicit flush, but BufWriter will flush on drop
        // For now, we just ensure the writer is still valid
        let _arrow_guard = self.arrow_writer.lock().await;

        // Flush Protobuf writer
        let mut proto_guard = self.protobuf_writer.lock().await;
        if let Some(ref mut writer) = *proto_guard {
            writer.flush().map_err(|e| {
                ZerobusError::ConfigurationError(format!("Failed to flush Protobuf file: {}", e))
            })?;
        }
        drop(proto_guard);

        // Update last flush time
        let mut last_flush = self.last_flush.lock().await;
        *last_flush = Instant::now();

        debug!("Flushed debug files to disk");
        Ok(())
    }

    /// Check if flush is needed based on interval
    ///
    /// # Returns
    ///
    /// Returns true if flush interval has elapsed.
    pub async fn should_flush(&self) -> bool {
        let last_flush = self.last_flush.lock().await;
        last_flush.elapsed() >= self.flush_interval
    }
}
