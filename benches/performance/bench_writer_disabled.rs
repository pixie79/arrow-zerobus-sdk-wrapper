//! Performance benchmark for writer disabled mode
//!
//! Measures operation time when writer is disabled (excluding file I/O)
//! Target: < 50ms per operation (excluding file I/O)

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use tempfile::TempDir;

fn create_test_batch(num_rows: usize) -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("value", DataType::Int64, false),
    ]);

    let id_array = Int64Array::from((0..num_rows).map(|i| i as i64).collect::<Vec<_>>());
    let name_array = StringArray::from(
        (0..num_rows)
            .map(|i| format!("name_{}", i))
            .collect::<Vec<_>>(),
    );
    let value_array = Int64Array::from((0..num_rows).map(|i| i as i64).collect::<Vec<_>>());

    RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(id_array),
            Arc::new(name_array),
            Arc::new(value_array),
        ],
    )
    .unwrap()
}

fn bench_writer_disabled_conversion(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("writer_disabled");

    // Benchmark different batch sizes
    for num_rows in [100, 1_000, 10_000] {
        let batch = create_test_batch(num_rows);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_rows", num_rows)),
            &batch,
            |b, batch| {
                b.iter(|| {
                    rt.block_on(async {
                        let temp_dir = TempDir::new().unwrap();
                        let debug_output_dir = temp_dir.path().to_path_buf();

                        let config = WrapperConfiguration::new(
                            "https://test.cloud.databricks.com".to_string(),
                            "test_table".to_string(),
                        )
                        .with_debug_output(debug_output_dir)
                        .with_zerobus_writer_disabled(true);

                        let wrapper = ZerobusWrapper::new(config).await.unwrap();
                        let _result = wrapper.send_batch(black_box(batch.clone())).await.unwrap();
                        let _ = wrapper.flush().await;
                    });
                });
            },
        );
    }

    group.finish();
}

fn bench_writer_disabled(c: &mut Criterion) {
    bench_writer_disabled_conversion(c);
}

criterion_group!(benches, bench_writer_disabled);
criterion_main!(benches);
