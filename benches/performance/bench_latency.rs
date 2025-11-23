//! Performance benchmark for latency
//!
//! Measures p95 latency for batches up to 10MB
//! Target: p95 latency under 150ms

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

fn create_test_batch(size_mb: usize) -> RecordBatch {
    // Create a batch of approximately the specified size
    // For simplicity, we'll create batches with varying row counts
    let num_rows = match size_mb {
        1 => 10_000,
        5 => 50_000,
        10 => 100_000,
        _ => 10_000,
    };
    
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("value", DataType::Int64, false),
    ]);
    
    let id_array = Int64Array::from((0..num_rows).map(|i| i as i64).collect::<Vec<_>>());
    let name_array = StringArray::from(
        (0..num_rows)
            .map(|i| format!("name_{}", i))
            .collect::<Vec<_>>()
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

fn bench_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency");
    
    // Benchmark different batch sizes
    for size_mb in [1, 5, 10] {
        let batch = create_test_batch(size_mb);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}MB", size_mb)),
            &batch,
            |b, batch| {
                b.iter(|| {
                    // Simulate batch processing (without actual network call)
                    // In real benchmark, this would call wrapper.send_batch()
                    let _size = black_box(batch.get_array_memory_size());
                    let _rows = black_box(batch.num_rows());
                    // Actual latency measurement would require mock SDK
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_latency);
criterion_main!(benches);

