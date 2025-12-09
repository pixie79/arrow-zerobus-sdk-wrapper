//! Zerobus SDK integration
//!
//! This module handles integration with the Databricks Zerobus SDK,
//! including stream creation and management.

use crate::error::ZerobusError;
use databricks_zerobus_ingest_sdk::{
    StreamConfigurationOptions, TableProperties, ZerobusSdk, ZerobusStream,
};
use prost_types::DescriptorProto;
use tracing::{debug, error, info, warn};
use std::time::{Duration, Instant};
use rand::Rng;

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

/// Tracks error 6006 state for backoff logic (per-table)
use std::sync::OnceLock;
static ERROR_6006_STATE: OnceLock<std::sync::Mutex<std::collections::HashMap<String, (Instant, Instant)>>> = OnceLock::new();

fn get_error_6006_state() -> &'static std::sync::Mutex<std::collections::HashMap<String, (Instant, Instant)>> {
    ERROR_6006_STATE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// Check if we're currently in backoff period for error 6006 (per-table)
/// This can be called before attempting writes to prevent writes during backoff
pub async fn check_error_6006_backoff(table_name: &str) -> Result<(), ZerobusError> {
    let state = get_error_6006_state();
    let state_guard = state.lock().unwrap();
    if let Some((_, backoff_until)) = state_guard.get(table_name) {
        if *backoff_until > Instant::now() {
            let remaining = backoff_until.duration_since(Instant::now());
            warn!("⏸️  Error 6006 backoff active for table {} - pipeline writes disabled. Remaining backoff: {:.1}s. Will retry after backoff period.", 
                  table_name, remaining.as_secs_f64());
            return Err(ZerobusError::ConnectionError(format!(
                "Pipeline temporarily blocked due to error 6006. Backoff period active for {:.1} more seconds. Writes are disabled during backoff.",
                remaining.as_secs_f64()
            )));
        }
    }
    Ok(())
}

/// Create or get Zerobus stream
///
/// Creates a new stream if one doesn't exist, or returns the existing stream.
///
/// # Arguments
///
/// * `sdk` - Zerobus SDK instance
/// * `table_name` - Target table name
/// * `descriptor_proto` - Protobuf descriptor for schema
/// * `client_id` - OAuth2 client ID
/// * `client_secret` - OAuth2 client secret
///
/// # Returns
///
/// Returns stream instance, or error if stream creation fails.
pub async fn ensure_stream(
    sdk: &ZerobusSdk,
    table_name: String,
    descriptor_proto: DescriptorProto,
    client_id: String,
    client_secret: String,
) -> Result<ZerobusStream, ZerobusError> {
    // Check if we're in backoff period for error 6006 (per-table)
    check_error_6006_backoff(&table_name).await?;
    
    // Log descriptor info in debug mode
    let descriptor_name = descriptor_proto.name.as_deref().unwrap_or("unknown");
    let field_count = descriptor_proto.field.len();
    let nested_count = descriptor_proto.nested_type.len();
    info!("🔍 [DEBUG] Creating Zerobus stream for table: {} with Protobuf descriptor: name='{}', fields={}, nested_types={}", 
          table_name, descriptor_name, field_count, nested_count);
    if field_count <= 20 {
        let field_names: Vec<&str> = descriptor_proto.field.iter()
            .map(|f| f.name.as_deref().unwrap_or("?"))
            .collect();
        debug!("🔍 [DEBUG] Descriptor fields: {:?}", field_names);
    }

    let table_properties = TableProperties {
        table_name: table_name.clone(),
        descriptor_proto,
    };

    #[allow(clippy::default_constructed_unit_structs)]
    let options = StreamConfigurationOptions::default();

    let stream_result = sdk
        .create_stream(table_properties, client_id, client_secret, Some(options))
        .await;
    
    match stream_result {
        Ok(stream) => {
            info!(
                "✅ Zerobus stream created successfully for table: {}",
                table_name
            );
            Ok(stream)
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            
            // Check for error 6006 - pipeline blocked, need backoff
            if error_msg.contains("6006") || error_msg.contains("Error Code: 6006") 
                || error_msg.contains("Pipeline creation is temporarily blocked") {
                // Calculate backoff with jitter (min 60 seconds)
                let base_delay_secs = 60;
                let jitter_range_secs = 30;
                let mut rng = rand::thread_rng();
                let jitter = rng.gen_range(0..=jitter_range_secs);
                let backoff_duration = Duration::from_secs(base_delay_secs + jitter);
                let backoff_until = Instant::now() + backoff_duration;
                
                // Store backoff state per table
                {
                    let state = get_error_6006_state();
                    let mut state_guard = state.lock().unwrap();
                    state_guard.insert(table_name.clone(), (Instant::now(), backoff_until));
                }
                
                error!("🚫 Error 6006 detected: Data ingestion pipeline for table \"{}\" has failed multiple times recently. Pipeline creation is temporarily blocked.", table_name);
                warn!("⏸️  Disabling writes to pipeline for {} seconds (jitter-based backoff, min 60s). Will retry after backoff period.", backoff_duration.as_secs());
                warn!("⏸️  This is a temporary block by Databricks. The system will automatically retry after the backoff period.");
                
                return Err(ZerobusError::ConnectionError(format!(
                    "Error 6006: Pipeline temporarily blocked for table {}. Writes disabled for {} seconds (backoff period). Will automatically retry after backoff.",
                    table_name, backoff_duration.as_secs()
                )));
            }
            
            // Check if this is a schema validation error
            if error_msg.contains("schema") || error_msg.contains("Schema") || 
               error_msg.contains("validation") || error_msg.contains("Validation") ||
               error_msg.contains("mismatch") || error_msg.contains("Mismatch") {
                error!("❌ Schema validation error when creating stream for table {}: {}", table_name, error_msg);
            }
            
            Err(ZerobusError::ConnectionError(format!(
                "Failed to create Zerobus stream for table {}: {}",
                table_name, e
            )))
        }
    }
}
