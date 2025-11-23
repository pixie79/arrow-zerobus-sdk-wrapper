# Rust API Contract: Zerobus SDK Wrapper

**Feature**: 001-zerobus-wrapper  
**Date**: 2025-11-23  
**Version**: 0.1.0

## Public API

### ZerobusWrapper

Main wrapper struct for sending data to Zerobus.

```rust
pub struct ZerobusWrapper {
    // Internal fields (not public)
}

impl ZerobusWrapper {
    /// Create a new ZerobusWrapper with default configuration
    /// 
    /// # Errors
    /// Returns error if configuration is invalid or SDK initialization fails
    pub async fn new(config: WrapperConfiguration) -> Result<Self, ZerobusError>;
    
    /// Send a data batch to Zerobus
    /// 
    /// # Arguments
    /// * `batch` - Arrow RecordBatch to send
    /// 
    /// # Returns
    /// TransmissionResult indicating success or failure
    /// 
    /// # Errors
    /// Returns error if transmission fails after all retry attempts
    pub async fn send_batch(&self, batch: RecordBatch) -> Result<TransmissionResult, ZerobusError>;
    
    /// Flush any pending operations and ensure data is transmitted
    /// 
    /// # Errors
    /// Returns error if flush operation fails
    pub async fn flush(&self) -> Result<(), ZerobusError>;
    
    /// Shutdown the wrapper gracefully, closing connections and cleaning up resources
    /// 
    /// # Errors
    /// Returns error if shutdown fails
    pub async fn shutdown(&self) -> Result<(), ZerobusError>;
}
```

### WrapperConfiguration

Configuration for initializing the wrapper.

```rust
pub struct WrapperConfiguration {
    pub zerobus_endpoint: String,
    pub unity_catalog_url: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub table_name: String,
    pub observability_enabled: bool,
    pub observability_config: Option<OtlpConfig>,
    pub debug_enabled: bool,
    pub debug_output_dir: Option<PathBuf>,
    pub debug_flush_interval_secs: u64,
    pub debug_max_file_size: Option<u64>,
    pub retry_max_attempts: u32,
    pub retry_base_delay_ms: u64,
    pub retry_max_delay_ms: u64,
}

impl WrapperConfiguration {
    /// Create a new configuration with defaults
    pub fn new(endpoint: String, table_name: String) -> Self;
    
    /// Builder pattern for configuration
    pub fn with_credentials(mut self, client_id: String, client_secret: String) -> Self;
    pub fn with_unity_catalog(mut self, url: String) -> Self;
    pub fn with_observability(mut self, config: OtlpConfig) -> Self;
    pub fn with_debug_output(mut self, output_dir: PathBuf) -> Self;
    pub fn with_retry_config(mut self, max_attempts: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self;
    
    /// Validate configuration
    /// 
    /// # Errors
    /// Returns error if configuration is invalid
    pub fn validate(&self) -> Result<(), ZerobusError>;
}
```

### TransmissionResult

Result of a data transmission operation.

```rust
pub struct TransmissionResult {
    pub success: bool,
    pub error: Option<ZerobusError>,
    pub attempts: u32,
    pub latency_ms: Option<u64>,
    pub batch_size_bytes: usize,
}
```

### ZerobusError

Error type for wrapper operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum ZerobusError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Conversion error: {0}")]
    ConversionError(String),
    
    #[error("Transmission error: {0}")]
    TransmissionError(String),
    
    #[error("Retry exhausted: {0}")]
    RetryExhausted(String),
    
    #[error("Token refresh error: {0}")]
    TokenRefreshError(String),
}
```

## Usage Example

```rust
use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
use arrow::record_batch::RecordBatch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = WrapperConfiguration::new(
        "https://workspace.cloud.databricks.com".to_string(),
        "my_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string());
    
    // Initialize wrapper
    let wrapper = ZerobusWrapper::new(config).await?;
    
    // Create Arrow RecordBatch (example)
    let batch = create_record_batch()?;
    
    // Send batch
    let result = wrapper.send_batch(batch).await?;
    
    if result.success {
        println!("Batch sent successfully in {}ms", result.latency_ms.unwrap_or(0));
    } else {
        eprintln!("Transmission failed: {:?}", result.error);
    }
    
    // Shutdown
    wrapper.shutdown().await?;
    
    Ok(())
}
```

## Thread Safety

All public methods are thread-safe and can be called concurrently from multiple threads or async tasks. The wrapper uses internal synchronization (Arc<Mutex>) to ensure safe concurrent access.

## Error Handling

All errors are returned as `Result<T, ZerobusError>`. Errors are descriptive and actionable, providing sufficient information for developers to diagnose and resolve issues.

## Performance Characteristics

- **Latency**: p95 latency under 150ms for batches up to 10MB
- **Success Rate**: 99.999% under normal network conditions
- **Concurrency**: Thread-safe, supports concurrent operations
- **Memory**: Bounded memory usage per wrapper instance

## Versioning

The API follows semantic versioning. Breaking changes will be indicated by major version bumps and will include migration guides.

