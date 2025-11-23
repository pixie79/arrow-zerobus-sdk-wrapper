//! Rust example for using Arrow Zerobus SDK Wrapper
//!
//! This example demonstrates how to use the wrapper from Rust to send
//! Arrow RecordBatch data to Zerobus.

use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusError, ZerobusWrapper};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration from environment variables
    let endpoint = env::var("ZEROBUS_ENDPOINT")
        .unwrap_or_else(|_| "https://your-workspace.cloud.databricks.com".to_string());
    let table_name = env::var("ZEROBUS_TABLE_NAME").unwrap_or_else(|_| "my_table".to_string());
    let client_id = env::var("ZEROBUS_CLIENT_ID").unwrap_or_else(|_| "your_client_id".to_string());
    let client_secret =
        env::var("ZEROBUS_CLIENT_SECRET").unwrap_or_else(|_| "your_client_secret".to_string());
    let unity_catalog_url =
        env::var("UNITY_CATALOG_URL").unwrap_or_else(|_| "https://unity-catalog-url".to_string());

    // Create configuration
    println!("Initializing ZerobusWrapper...");
    let config = WrapperConfiguration::new(endpoint, table_name)
        .with_credentials(client_id, client_secret)
        .with_unity_catalog(unity_catalog_url)
        .with_retry_config(5, 100, 30000); // 5 attempts, 100ms base delay, 30s max delay

    // Initialize wrapper
    let wrapper = match ZerobusWrapper::new(config).await {
        Ok(w) => {
            println!("✅ Wrapper initialized successfully");
            w
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize wrapper: {:?}", e);
            return Err(e.into());
        }
    };

    // Create Arrow RecordBatch
    println!("\nCreating Arrow RecordBatch...");
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("score", DataType::Float64, false),
    ]);

    let id_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie", "David", "Eve"]);
    let score_array = Float64Array::from(vec![95.5, 87.0, 92.5, 88.0, 91.0]);

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(id_array),
            Arc::new(name_array),
            Arc::new(score_array),
        ],
    )?;

    println!(
        "✅ Created RecordBatch with {} rows and {} columns",
        batch.num_rows(),
        batch.num_columns()
    );

    // Send batch to Zerobus
    println!("\nSending batch to Zerobus...");
    match wrapper.send_batch(batch).await {
        Ok(result) => {
            if result.success {
                println!("✅ Batch sent successfully!");
                println!("   Latency: {}ms", result.latency_ms.unwrap_or(0));
                println!("   Size: {} bytes", result.batch_size_bytes);
                println!("   Attempts: {}", result.attempts);
            } else {
                println!("❌ Transmission failed");
                if let Some(error) = result.error {
                    println!("   Error: {:?}", error);
                }
                println!("   Attempts: {}", result.attempts);
            }
        }
        Err(e) => {
            eprintln!("❌ Transmission error: {:?}", e);
        }
    }

    // Shutdown wrapper
    println!("\nShutting down wrapper...");
    match wrapper.shutdown().await {
        Ok(()) => {
            println!("✅ Wrapper shut down successfully");
        }
        Err(e) => {
            eprintln!("❌ Shutdown error: {:?}", e);
        }
    }

    Ok(())
}
