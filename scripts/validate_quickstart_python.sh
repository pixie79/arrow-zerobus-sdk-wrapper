#!/bin/bash
# Quickstart validation script for Python examples
# Validates that Python examples from quickstart.md work correctly

set -e

echo "🔍 Validating Python Quickstart Examples..."
echo ""

# Create a temporary test file based on quickstart.md examples
TEMP_TEST_FILE=$(mktemp /tmp/quickstart_test_XXXXXX.py)
trap "rm -f $TEMP_TEST_FILE" EXIT

cat > "$TEMP_TEST_FILE" << 'EOF'
"""Quickstart validation test for Python examples."""
import asyncio
import os
import tempfile
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper, WrapperConfiguration, ZerobusError

async def test_basic_usage():
    """Test basic usage with writer disabled."""
    print("Test 1: Basic usage with writer disabled...")
    temp_dir = tempfile.mkdtemp()
    debug_output_dir = os.path.join(temp_dir, "debug_output")
    
    try:
        wrapper = ZerobusWrapper(
            endpoint="https://workspace.cloud.databricks.com",
            table_name="my_table",
            debug_enabled=True,
            debug_output_dir=debug_output_dir,
            zerobus_writer_disabled=True,  # Enable disabled mode
            # Note: credentials not required when writer is disabled
        )
        print("✅ Wrapper initialization succeeded")
        
        # Create test batch
        schema = pa.schema([
            pa.field("id", pa.int64()),
            pa.field("name", pa.string()),
        ])
        arrays = [
            pa.array([1, 2, 3], type=pa.int64()),
            pa.array(["Alice", "Bob", "Charlie"], type=pa.string()),
        ]
        batch = pa.RecordBatch.from_arrays(arrays, schema=schema)
        
        # Send batch - writes debug files but skips network calls
        result = await wrapper.send_batch(batch)
        assert result.success, "send_batch should succeed when writer disabled"
        print("✅ Batch send succeeded")
        
        await wrapper.shutdown()
        print("✅ Shutdown succeeded")
    finally:
        # Cleanup
        import shutil
        shutil.rmtree(temp_dir, ignore_errors=True)

async def test_configuration_validation():
    """Test configuration validation error case."""
    print("\nTest 2: Configuration validation error case...")
    try:
        config = WrapperConfiguration(
            endpoint="https://workspace.cloud.databricks.com",
            table_name="my_table",
            zerobus_writer_disabled=True,  # But debug_enabled is False (default)
        )
        config.validate()
        print("❌ Validation should have failed")
        assert False, "Validation should fail when writer disabled but debug not enabled"
    except ZerobusError as e:
        if "debug_enabled must be true" in str(e):
            print("✅ Validation correctly rejected invalid config")
        else:
            raise

async def test_cicd_example():
    """Test CI/CD testing example."""
    print("\nTest 3: CI/CD testing example...")
    temp_dir = tempfile.mkdtemp()
    debug_output_dir = os.path.join(temp_dir, "test_debug")
    
    try:
        wrapper = ZerobusWrapper(
            endpoint=os.getenv("ZEROBUS_ENDPOINT", "https://test.cloud.databricks.com"),
            table_name="test_table",
            debug_enabled=True,
            debug_output_dir=debug_output_dir,
            zerobus_writer_disabled=True,  # No credentials needed
        )
        
        # Create test batch
        schema = pa.schema([
            pa.field("id", pa.int64()),
            pa.field("name", pa.string()),
        ])
        arrays = [
            pa.array([1, 2, 3], type=pa.int64()),
            pa.array(["Alice", "Bob", "Charlie"], type=pa.string()),
        ]
        batch = pa.RecordBatch.from_arrays(arrays, schema=schema)
        
        result = await wrapper.send_batch(batch)
        assert result.success, "Conversion should succeed"
        print("✅ CI/CD example test succeeded")
        
        await wrapper.shutdown()
    finally:
        import shutil
        shutil.rmtree(temp_dir, ignore_errors=True)

async def main():
    """Run all quickstart validation tests."""
    try:
        await test_basic_usage()
        await test_configuration_validation()
        await test_cicd_example()
        print("\n✅ All Python quickstart examples validated successfully!")
    except Exception as e:
        print(f"\n❌ Quickstart validation failed: {e}")
        raise

if __name__ == "__main__":
    asyncio.run(main())
EOF

echo "📝 Created test file: $TEMP_TEST_FILE"
echo "🐍 Running Python quickstart validation..."

python3 "$TEMP_TEST_FILE" 2>&1 || {
    echo "⚠️  Python quickstart validation requires Python bindings to be built"
    echo "   Run: maturin develop (or maturin build) first"
    exit 1
}

echo ""
echo "✅ Python quickstart validation complete!"

