"""Python example for using Arrow Zerobus SDK Wrapper

This example demonstrates how to use the wrapper from Python to send
Arrow RecordBatch data to Zerobus.
"""

import asyncio
import os
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper, ZerobusError


async def main():
    """Main example function."""
    # Get configuration from environment variables
    endpoint = os.getenv("ZEROBUS_ENDPOINT", "https://your-workspace.cloud.databricks.com")
    table_name = os.getenv("ZEROBUS_TABLE_NAME", "my_table")
    client_id = os.getenv("ZEROBUS_CLIENT_ID", "your_client_id")
    client_secret = os.getenv("ZEROBUS_CLIENT_SECRET", "your_client_secret")
    unity_catalog_url = os.getenv("UNITY_CATALOG_URL", "https://unity-catalog-url")

    # Initialize wrapper
    print("Initializing ZerobusWrapper...")
    try:
        wrapper = ZerobusWrapper(
            endpoint=endpoint,
            table_name=table_name,
            client_id=client_id,
            client_secret=client_secret,
            unity_catalog_url=unity_catalog_url,
            debug_enabled=False,  # Set to True to enable debug file output
        )
        print("✅ Wrapper initialized successfully")
    except ZerobusError as e:
        print(f"❌ Failed to initialize wrapper: {e}")
        return

    # Create Arrow RecordBatch
    print("\nCreating Arrow RecordBatch...")
    schema = pa.schema([
        pa.field("id", pa.int64()),
        pa.field("name", pa.string()),
        pa.field("score", pa.float64()),
    ])

    arrays = [
        pa.array([1, 2, 3, 4, 5], type=pa.int64()),
        pa.array(["Alice", "Bob", "Charlie", "David", "Eve"], type=pa.string()),
        pa.array([95.5, 87.0, 92.5, 88.0, 91.0], type=pa.float64()),
    ]

    batch = pa.RecordBatch.from_arrays(arrays, schema=schema)
    print(f"✅ Created RecordBatch with {batch.num_rows} rows and {batch.num_columns} columns")

    # Send batch to Zerobus
    print("\nSending batch to Zerobus...")
    try:
        result = wrapper.send_batch(batch)
        
        if result.success:
            print(f"✅ Batch sent successfully!")
            print(f"   Latency: {result.latency_ms}ms")
            print(f"   Size: {result.batch_size_bytes} bytes")
            print(f"   Attempts: {result.attempts}")
        else:
            print(f"❌ Transmission failed")
            print(f"   Error: {result.error}")
            print(f"   Attempts: {result.attempts}")
    except ZerobusError as e:
        print(f"❌ Transmission error: {e}")

    # Shutdown wrapper
    print("\nShutting down wrapper...")
    try:
        wrapper.shutdown()
        print("✅ Wrapper shut down successfully")
    except ZerobusError as e:
        print(f"❌ Shutdown error: {e}")


async def main_with_context_manager():
    """Example using async context manager."""
    endpoint = os.getenv("ZEROBUS_ENDPOINT", "https://your-workspace.cloud.databricks.com")
    table_name = os.getenv("ZEROBUS_TABLE_NAME", "my_table")
    client_id = os.getenv("ZEROBUS_CLIENT_ID", "your_client_id")
    client_secret = os.getenv("ZEROBUS_CLIENT_SECRET", "your_client_secret")
    unity_catalog_url = os.getenv("UNITY_CATALOG_URL", "https://unity-catalog-url")

    # Use async context manager for automatic cleanup
    async with ZerobusWrapper(
        endpoint=endpoint,
        table_name=table_name,
        client_id=client_id,
        client_secret=client_secret,
        unity_catalog_url=unity_catalog_url,
    ) as wrapper:
        # Create and send batch
        schema = pa.schema([
            pa.field("id", pa.int64()),
            pa.field("name", pa.string()),
        ])
        arrays = [
            pa.array([1, 2, 3], type=pa.int64()),
            pa.array(["Alice", "Bob", "Charlie"], type=pa.string()),
        ]
        batch = pa.RecordBatch.from_arrays(arrays, schema=schema)
        
        result = wrapper.send_batch(batch)
        print(f"Result: success={result.success}, latency={result.latency_ms}ms")


if __name__ == "__main__":
    # Run the main example
    asyncio.run(main())
    
    # Or use context manager
    # asyncio.run(main_with_context_manager())

