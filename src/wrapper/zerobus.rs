//! Zerobus SDK integration
//!
//! This module handles integration with the Databricks Zerobus SDK,
//! including stream creation and management.

use crate::error::ZerobusError;
use databricks_zerobus_ingest_sdk::{
    StreamConfigurationOptions, TableProperties, ZerobusSdk, ZerobusStream,
};
use prost_types::FileDescriptorProto;
use tracing::{debug, info};

/// Create or get Zerobus SDK instance
///
/// # Arguments
///
/// * `endpoint` - Zerobus endpoint URL
/// * `unity_catalog_url` - Unity Catalog URL for OAuth
///
/// # Returns
///
/// Returns initialized SDK instance, or error if initialization fails.
pub async fn create_sdk(
    endpoint: String,
    unity_catalog_url: String,
) -> Result<ZerobusSdk, ZerobusError> {
    info!("Creating Zerobus SDK with endpoint: {}", endpoint);

    let sdk = ZerobusSdk::new(endpoint, unity_catalog_url).map_err(|e| {
        ZerobusError::ConfigurationError(format!("Failed to initialize Zerobus SDK: {}", e))
    })?;

    debug!("Zerobus SDK created successfully");
    Ok(sdk)
}

/// Create or get Zerobus stream
///
/// Creates a new stream if one doesn't exist, or returns the existing stream.
///
/// # Arguments
///
/// * `sdk` - Zerobus SDK instance
/// * `table_name` - Target table name
/// * `file_descriptor_proto` - Protobuf file descriptor for schema
/// * `client_id` - OAuth2 client ID
/// * `client_secret` - OAuth2 client secret
///
/// # Returns
///
/// Returns stream instance, or error if stream creation fails.
pub async fn ensure_stream(
    sdk: &ZerobusSdk,
    table_name: String,
    file_descriptor_proto: FileDescriptorProto,
    client_id: String,
    client_secret: String,
) -> Result<ZerobusStream, ZerobusError> {
    info!("Creating Zerobus stream for table: {}", table_name);

    // Create FileDescriptorSet from FileDescriptorProto
    use prost_types::FileDescriptorSet;
    let file_descriptor_set = FileDescriptorSet {
        file: vec![file_descriptor_proto],
    };

    let table_properties = TableProperties {
        table_name: table_name.clone(),
        descriptor_proto: None,
        file_descriptor_set: Some(file_descriptor_set),
    };

    let options = StreamConfigurationOptions;

    let stream = sdk
        .create_stream(table_properties, client_id, client_secret, Some(options))
        .await
        .map_err(|e| {
            ZerobusError::ConnectionError(format!(
                "Failed to create Zerobus stream for table {}: {}",
                table_name, e
            ))
        })?;

    debug!(
        "Zerobus stream created successfully for table: {}",
        table_name
    );
    Ok(stream)
}
