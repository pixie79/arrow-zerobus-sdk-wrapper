//! Debug file writing
//!
//! This module handles writing Arrow and Protobuf debug files for inspection.

use crate::error::ZerobusError;
use crate::utils::file_rotation::rotate_file_if_needed;
use arrow::record_batch::RecordBatch;
use prost_types::DescriptorProto;
use prost::Message;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Debug file writer
///
/// Handles writing Arrow RecordBatch and Protobuf files to disk for debugging.
pub struct DebugWriter {
    /// Output directory for debug files
    #[allow(dead_code)]
    output_dir: PathBuf,
    /// Arrow IPC file writer
    arrow_writer: Arc<tokio::sync::Mutex<Option<arrow::ipc::writer::FileWriter<std::fs::File>>>>,
    /// Protobuf file writer
    protobuf_writer: Arc<tokio::sync::Mutex<Option<std::fs::File>>>,
    /// Arrow file path
    arrow_file_path: PathBuf,
    /// Protobuf file path
    protobuf_file_path: PathBuf,
    /// Flush interval
    flush_interval: Duration,
    /// Maximum file size before rotation
    max_file_size: Option<u64>,
    /// Timestamp of last flush
    last_flush: Arc<Mutex<Instant>>,
}

impl DebugWriter {
    /// Create a new debug writer
    ///
    /// # Arguments
    ///
    /// * `output_dir` - Output directory for debug files
    /// * `flush_interval` - Interval for periodic flushing
    /// * `max_file_size` - Maximum file size before rotation (optional)
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
        let sanitized_table_name = table_name.replace('.', "_").replace('/', "_");
        let arrow_file_path = arrow_dir.join(format!("{}.arrow", sanitized_table_name));
        let protobuf_file_path = proto_dir.join(format!("{}.proto", sanitized_table_name));

        Ok(Self {
            output_dir,
            arrow_writer: Arc::new(tokio::sync::Mutex::new(None)),
            protobuf_writer: Arc::new(tokio::sync::Mutex::new(None)),
            arrow_file_path,
            protobuf_file_path,
            flush_interval,
            max_file_size,
            last_flush: Arc::new(Mutex::new(Instant::now())),
        })
    }

    /// Ensure Arrow writer is initialized
    async fn ensure_arrow_writer(&self) -> Result<(), ZerobusError> {
        let mut writer_guard = self.arrow_writer.lock().await;
        if writer_guard.is_none() {
            let file = std::fs::File::create(&self.arrow_file_path).map_err(|e| {
                ZerobusError::ConfigurationError(format!(
                    "Failed to create Arrow debug file: {}",
                    e
                ))
            })?;

            let schema = arrow::datatypes::Schema::empty();
            let writer = arrow::ipc::writer::FileWriter::try_new(file, &schema).map_err(|e| {
                ZerobusError::ConfigurationError(format!(
                    "Failed to create Arrow IPC writer: {}",
                    e
                ))
            })?;

            *writer_guard = Some(writer);
        }
        Ok(())
    }

    /// Ensure Protobuf writer is initialized
    async fn ensure_protobuf_writer(&self) -> Result<(), ZerobusError> {
        let mut writer_guard = self.protobuf_writer.lock().await;
        if writer_guard.is_none() {
            let file = std::fs::File::create(&self.protobuf_file_path).map_err(|e| {
                ZerobusError::ConfigurationError(format!(
                    "Failed to create Protobuf debug file: {}",
                    e
                ))
            })?;
            *writer_guard = Some(file);
        }
        Ok(())
    }

    /// Rotate Arrow file if needed
    async fn rotate_arrow_file_if_needed(&self) -> Result<(), ZerobusError> {
        if let Some(max_size) = self.max_file_size {
            if let Some(new_path) =
                rotate_file_if_needed(&self.arrow_file_path, max_size).map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to check Arrow file size: {}",
                        e
                    ))
                })?
            {
                // Close current writer
                let mut writer_guard = self.arrow_writer.lock().await;
                if let Some(mut writer) = writer_guard.take() {
                    writer.finish().map_err(|e| {
                        ZerobusError::ConfigurationError(format!(
                            "Failed to finish Arrow writer: {}",
                            e
                        ))
                    })?;
                }

                // Update file path and create new writer
                // Note: We'd need to update arrow_file_path, but it's immutable
                // For now, we'll create a new file with the rotated name
                let file = std::fs::File::create(&new_path).map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to create rotated Arrow file: {}",
                        e
                    ))
                })?;

                let schema = arrow::datatypes::Schema::empty();
                let writer =
                    arrow::ipc::writer::FileWriter::try_new(file, &schema).map_err(|e| {
                        ZerobusError::ConfigurationError(format!(
                            "Failed to create rotated Arrow IPC writer: {}",
                            e
                        ))
                    })?;

                *writer_guard = Some(writer);
            }
        }
        Ok(())
    }

    /// Rotate Protobuf file if needed
    async fn rotate_protobuf_file_if_needed(&self) -> Result<(), ZerobusError> {
        if let Some(max_size) = self.max_file_size {
            if let Some(new_path) = rotate_file_if_needed(&self.protobuf_file_path, max_size)
                .map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to check Protobuf file size: {}",
                        e
                    ))
                })?
            {
                // Close current writer
                let mut writer_guard = self.protobuf_writer.lock().await;
                if let Some(file) = writer_guard.take() {
                    file.sync_all().map_err(|e| {
                        ZerobusError::ConfigurationError(format!(
                            "Failed to sync Protobuf file: {}",
                            e
                        ))
                    })?;
                }

                // Create new file
                let file = std::fs::File::create(&new_path).map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to create rotated Protobuf file: {}",
                        e
                    ))
                })?;

                *writer_guard = Some(file);
            }
        }
        Ok(())
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
        // Check if rotation is needed
        self.rotate_arrow_file_if_needed().await?;

        // Ensure writer is initialized
        self.ensure_arrow_writer().await?;

        // Write batch
        let mut writer_guard = self.arrow_writer.lock().await;
        if let Some(ref mut writer) = *writer_guard {
            // Update schema if needed (first write)
            if writer.schema().fields().is_empty() {
                // Recreate writer with actual schema
                drop(writer_guard);
                let file = std::fs::File::create(&self.arrow_file_path).map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to recreate Arrow file: {}",
                        e
                    ))
                })?;

                let writer = arrow::ipc::writer::FileWriter::try_new(file, batch.schema().as_ref())
                    .map_err(|e| {
                        ZerobusError::ConfigurationError(format!(
                            "Failed to create Arrow IPC writer with schema: {}",
                            e
                        ))
                    })?;

                let mut new_guard = self.arrow_writer.lock().await;
                *new_guard = Some(writer);
                writer_guard = new_guard;
            }

            if let Some(ref mut writer) = *writer_guard {
                writer.write(batch).map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to write Arrow RecordBatch: {}",
                        e
                    ))
                })?;
            }
        }

        debug!("Wrote Arrow RecordBatch to debug file");
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
    pub async fn write_protobuf(&self, protobuf_bytes: &[u8], flush_immediately: bool) -> Result<(), ZerobusError> {
        // Check if rotation is needed
        self.rotate_protobuf_file_if_needed().await?;

        // Ensure writer is initialized
        self.ensure_protobuf_writer().await?;

        // Write bytes
        let mut writer_guard = self.protobuf_writer.lock().await;
        if let Some(ref mut file) = *writer_guard {
            file.write_all(protobuf_bytes).map_err(|e| {
                ZerobusError::ConfigurationError(format!("Failed to write Protobuf bytes: {}", e))
            })?;

            // Write newline separator for readability (optional)
            file.write_all(b"\n").map_err(|e| {
                ZerobusError::ConfigurationError(format!(
                    "Failed to write Protobuf separator: {}",
                    e
                ))
            })?;

            // Flush immediately if requested (for per-batch flushing)
            if flush_immediately {
                file.sync_all().map_err(|e| {
                    ZerobusError::ConfigurationError(format!("Failed to flush Protobuf file: {}", e))
                })?;
            }
        }

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
    pub async fn write_descriptor(&self, table_name: &str, descriptor: &DescriptorProto) -> Result<(), ZerobusError> {
        // Create descriptors directory
        let descriptors_dir = self.output_dir.join("zerobus/descriptors");
        std::fs::create_dir_all(&descriptors_dir).map_err(|e| {
            ZerobusError::ConfigurationError(format!(
                "Failed to create descriptors directory: {}",
                e
            ))
        })?;

        // Create filename from table name (sanitize for filesystem)
        let sanitized_table_name = table_name.replace('.', "_").replace('/', "_");
        let descriptor_file_path = descriptors_dir.join(format!("{}.pb", sanitized_table_name));

        // Check if file already exists (only write once per table)
        if descriptor_file_path.exists() {
            debug!("Descriptor file already exists for table {}: {}", table_name, descriptor_file_path.display());
            return Ok(());
        }

        // Serialize descriptor to bytes
        let mut descriptor_bytes = Vec::new();
        descriptor.encode(&mut descriptor_bytes).map_err(|e| {
            ZerobusError::ConfigurationError(format!(
                "Failed to encode Protobuf descriptor: {}",
                e
            ))
        })?;

        // Write to file
        let mut file = std::fs::File::create(&descriptor_file_path).map_err(|e| {
            ZerobusError::ConfigurationError(format!(
                "Failed to create descriptor file: {}",
                e
            ))
        })?;

        file.write_all(&descriptor_bytes).map_err(|e| {
            ZerobusError::ConfigurationError(format!(
                "Failed to write descriptor bytes: {}",
                e
            ))
        })?;

        file.sync_all().map_err(|e| {
            ZerobusError::ConfigurationError(format!(
                "Failed to sync descriptor file: {}",
                e
            ))
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
        // Flush Arrow writer
        let arrow_guard = self.arrow_writer.lock().await;
        if let Some(ref _writer) = *arrow_guard {
            // Arrow FileWriter doesn't have explicit flush, but we can ensure it's written
            // The writer buffers internally and writes on finish
        }
        drop(arrow_guard);

        // Flush Protobuf writer
        let mut proto_guard = self.protobuf_writer.lock().await;
        if let Some(ref mut file) = *proto_guard {
            file.sync_all().map_err(|e| {
                ZerobusError::ConfigurationError(format!("Failed to sync Protobuf file: {}", e))
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
