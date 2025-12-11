//! Debug file writing
//!
//! This module handles writing Arrow and Protobuf debug files for inspection.
//! Uses Arrow IPC Stream format (*.arrows) for better compatibility with DuckDB.

use crate::error::ZerobusError;
use crate::utils::file_rotation::rotate_file_if_needed;
use arrow::record_batch::RecordBatch;
use prost::Message;
use prost_types::DescriptorProto;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info};

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
    ///
    /// # Returns
    ///
    /// Returns debug writer instance, or error if initialization fails.
    pub fn new(
        output_dir: PathBuf,
        table_name: String,
        flush_interval: Duration,
        max_file_size: Option<u64>,
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
            last_flush: Arc::new(Mutex::new(Instant::now())),
            arrow_record_count: Arc::new(Mutex::new(0)),
            protobuf_record_count: Arc::new(Mutex::new(0)),
        })
    }

    /// Generate rotated file path with timestamp
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

        parent.join(format!("{}_{}.{}", stem, timestamp, extension))
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
