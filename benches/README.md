# Benchmarks

This directory contains performance benchmarks for the mini_rust_olap engine using the `criterion` crate. These benchmarks help identify performance bottlenecks and track improvements over time.

## Running Benchmarks

Run all benchmarks:

```bash
cargo bench
```

Run a specific benchmark:

```bash
cargo bench -- <benchmark_name>
```

For detailed performance profiling with flamegraphs:

```bash
cargo bench -- --profile-time 10
```

## Available Benchmarks

### `query_benchmark.rs`

Comprehensive benchmark suite measuring various aspects of query execution:

#### 1. Full Scan (`full_scan`)
Measures the performance of scanning entire tables of different sizes.
- **Sizes tested**: 100, 1,000, 10,000 rows
- **Purpose**: Baseline performance for table scanning operations

#### 2. Filter Operations (`filter`)
Benchmarks WHERE clause filtering performance.
- **Operation**: Filter rows where `value > 100`
- **Purpose**: Measure predicate evaluation efficiency

#### 3. Project Operations (`project`)
Benchmarks column selection (SELECT specific columns).
- **Operation**: Select id, value, category columns
- **Purpose**: Measure projection efficiency

#### 4. SQL Parsing (`sql_parsing`)
Benchmarks the SQL parser performance.
- **Tests**: Simple vs. complex query parsing
- **Purpose**: Identify parser bottlenecks

#### 5. Full Query Execution (`full_query`)
End-to-end benchmark of the complete query pipeline.
- **Operations**: Parse → Plan → Execute
- **Query**: `SELECT category, COUNT(*) FROM benchmark WHERE value > 100 GROUP BY category`
- **Purpose**: Measure overall query performance

#### 6. Aggregation Operations (`aggregation`)
Benchmarks GROUP BY and aggregate function performance.
- **Operation**: Count rows per category
- **Purpose**: Measure hash aggregation efficiency

## Understanding Benchmark Results

Benchmark results are saved in `target/criterion/` after each run. You can view detailed reports by opening:

```
target/criterion/report/index.html
```

### Key Metrics

- **Mean**: Average execution time
- **Std Dev**: Standard deviation (lower is better)
- **Median**: Middle value of execution times
- **Throughput**: Operations per second

## Writing New Benchmarks

When adding new benchmarks:

1. Use the `criterion` crate
2. Follow the pattern in `query_benchmark.rs`
3. Test multiple data sizes when appropriate
4. Use `black_box()` to prevent compiler optimizations
5. Group related benchmarks with `c.benchmark_group()`

Example:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Your benchmark code here
            let result = my_function(black_box(input));
        });
    });
}

criterion_group!(benches, benchmark_my_feature);
criterion_main!(benches);
```

## Benchmarking Best Practices

1. **Run multiple times**: Performance can vary between runs due to system factors
2. **Use consistent hardware**: Benchmark on the same machine for comparable results
3. **Minimize background processes**: Close unnecessary applications while benchmarking
4. **Warm up the system**: First run may be slower due to cold caches
5. **Profile bottlenecks**: Use `cargo bench -- --profile-time` to identify slow code paths

## Performance Targets

Target performance for key operations (single thread):

| Operation | 100 rows | 1,000 rows | 10,000 rows |
|-----------|----------|------------|-------------|
| Full Scan | < 1ms | < 5ms | < 50ms |
| Filter | < 2ms | < 10ms | < 100ms |
| Group By | < 5ms | < 20ms | < 200ms |
| SQL Parse | < 1ms | < 1ms | < 1ms |

*These targets are guidelines for expected performance on modern hardware.*

## Contributing Improvements

If you optimize the codebase:

1. Run benchmarks before and after your changes
2. Document the performance improvement
3. Update the performance targets if the improvements are significant
4. Add new benchmarks for any new features you add

## Related Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Project README](../README.md) - Main project documentation