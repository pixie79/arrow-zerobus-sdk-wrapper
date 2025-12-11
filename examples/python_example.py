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
    endpoint = os.getenv(
        "ZEROBUS_ENDPOINT", "https://your-workspace.cloud.databricks.com"
    )
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
    schema = pa.schema(
        [
            pa.field("id", pa.int64()),
            pa.field("name", pa.string()),
            pa.field("score", pa.float64()),
        ]
    )

    arrays = [
        pa.array([1, 2, 3, 4, 5], type=pa.int64()),
        pa.array(["Alice", "Bob", "Charlie", "David", "Eve"], type=pa.string()),
        pa.array([95.5, 87.0, 92.5, 88.0, 91.0], type=pa.float64()),
    ]

    batch = pa.RecordBatch.from_arrays(arrays, schema=schema)
    print(
        f"✅ Created RecordBatch with {batch.num_rows} rows and {batch.num_columns} columns"
    )

    # Send batch to Zerobus
    print("\nSending batch to Zerobus...")
    original_batch = batch  # Keep reference for quarantine workflow
    try:
        result = wrapper.send_batch(batch)

        if result.success:
            print("✅ Batch sent successfully!")
            print(f"   Latency: {result.latency_ms}ms")
            print(f"   Size: {result.batch_size_bytes} bytes")
            print(f"   Attempts: {result.attempts}")
            
            # Handle per-row errors with quarantine workflow
            if result.is_partial_success():
                print("\n⚠️  Partial success detected:")
                print(f"   Total rows: {result.total_rows}")
                print(f"   Successful: {result.successful_count}")
                print(f"   Failed: {result.failed_count}")
                
                # Extract and write successful rows to main table
                successful_batch = result.extract_successful_batch(original_batch)
                if successful_batch is not None:
                    print(f"\n✅ Writing {successful_batch.num_rows} successful rows to main table...")
                    # In a real application, you would write successful_batch to your main table here
                    # await write_to_main_table(successful_batch)
                
                # Extract and quarantine failed rows
                failed_batch = result.extract_failed_batch(original_batch)
                if failed_batch is not None:
                    print(f"\n❌ Quarantining {failed_batch.num_rows} failed rows...")
                    failed_indices = result.get_failed_row_indices()
                    if result.failed_rows:
                        for row_idx, error_msg in result.failed_rows:
                            print(f"   Row {row_idx}: {error_msg}")
                    # In a real application, you would quarantine failed_batch here
                    # await quarantine_batch(failed_batch)
            elif result.has_failed_rows():
                print("\n❌ All rows failed")
                failed_batch = result.extract_failed_batch(original_batch)
                if failed_batch is not None:
                    print(f"   Quarantining {failed_batch.num_rows} failed rows...")
                    # In a real application, you would quarantine failed_batch here
                    # await quarantine_batch(failed_batch)
            else:
                print(f"\n✅ All {result.successful_count} rows succeeded!")
            
            # Error analysis and pattern detection
            if result.has_failed_rows():
                print("\n📊 Error Analysis:")
                stats = result.get_error_statistics()
                print(f"   Success rate: {stats['success_rate'] * 100:.1}%")
                print(f"   Failure rate: {stats['failure_rate'] * 100:.1}%")
                
                grouped = result.group_errors_by_type()
                if grouped:
                    print("   Error breakdown by type:")
                    for error_type, indices in grouped.items():
                        print(f"     {error_type}: {len(indices)} rows (indices: {indices})")
                
                # Get all error messages for debugging
                error_messages = result.get_error_messages()
                if error_messages:
                    print("   Sample error messages:")
                    for i, msg in enumerate(error_messages[:3]):
                        print(f"     {i + 1}. {msg}")
                    if len(error_messages) > 3:
                        print(f"     ... and {len(error_messages) - 3} more")
        else:
            print("❌ Transmission failed")
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
    endpoint = os.getenv(
        "ZEROBUS_ENDPOINT", "https://your-workspace.cloud.databricks.com"
    )
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
        schema = pa.schema(
            [
                pa.field("id", pa.int64()),
                pa.field("name", pa.string()),
            ]
        )
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
