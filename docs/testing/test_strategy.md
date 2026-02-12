# Test Strategy for Mini Rust OLAP

## Overview

This document outlines the comprehensive testing strategy for the Mini Rust OLAP database engine. The testing approach is designed to ensure reliability, correctness, and performance across all components of the system.

### Testing Philosophy

- **Test-Driven Development (TDD)**: Where feasible, tests are written before or alongside code
- **Comprehensive Coverage**: Aim for high test coverage across all critical paths
- **Fast Feedback**: Unit tests should be fast; integration tests can be slower
- **Real-World Scenarios**: Tests should reflect actual use cases
- **Maintainability**: Tests should be clear, concise, and easy to understand
- **Automation**: All tests should run automatically in CI/CD pipelines

## Testing Levels

### 1. Unit Tests

**Location**: `src/*.rs` files (embedded in each module)

**Purpose**: Test individual functions, methods, and components in isolation

**Coverage Areas**:
- Core data types (`types.rs`, `column.rs`)
- Error handling (`error.rs`)
- SQL parsing (`parser.rs`)
- Query planning (`planner.rs`)
- Execution operators (`execution.rs`, `aggregates.rs`)
- CSV ingestion (`ingest.rs`)
- Catalog management (`catalog.rs`)
- Table operations (`table.rs`)

**Current Status**: 361 unit tests passing

**Example Structure**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = ...;
        
        // Act
        let result = function(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### 2. Integration Tests

**Location**: `tests/` directory

**Purpose**: Test how components work together

**Current Files**:
- `integration_tests.rs`: Comprehensive integration tests (68 tests, 51 passing)
- `manual_query.rs`: Manual query execution tests (16 tests)

**Test Categories**:
1. **CSV Ingestion Integration**
   - Load CSV files into catalog
   - Type inference
   - Data validation

2. **Query Execution Integration**
   - End-to-end query processing
   - Parse → Plan → Execute pipeline
   - Multiple operations in single query

3. **REPL Integration**
   - Command processing
   - Session management
   - Error handling

### 3. Documentation Tests

**Location**: Embedded in `//` doc comments

**Purpose**: Verify that code examples in documentation compile and run correctly

**Current Status**: 51 doc tests passing

**Example**:
```rust
/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// use mini_rust_olap::types::Value;
/// let result = Value::Int(1) + Value::Int(2);
/// assert_eq!(result, Value::Int(3));
/// ```
```

### 4. Benchmark Tests

**Location**: `benches/query_benchmark.rs`

**Purpose**: Measure and track performance over time

**Benchmark Categories**:
1. **SQL Parsing Performance**
   - Simple queries
   - Complex queries with multiple clauses

2. **Full Table Scan**
   - Select all rows
   - Various dataset sizes

3. **Filter Operations**
   - Numeric filters (WHERE salary > 80000)
   - String filters (WHERE department = 'Engineering')

4. **Projection Operations**
   - Select specific columns
   - Column count variations

5. **Aggregation Operations**
   - GROUP BY with COUNT
   - GROUP BY with AVG
   - Multiple aggregations

6. **ORDER BY Operations**
   - Ascending and descending
   - Multiple sort keys

7. **Full Query Execution**
   - Combined filter and aggregation
   - Complex multi-clause queries

**Running Benchmarks**:
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench full_scan

# Generate flamegraph for profiling
cargo bench -- --profile-time 5
```

## Test Organization

### Directory Structure

```
mini_rust_olap/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── main.rs          # Binary entry point (REPL)
│   ├── types.rs         # Core data types + unit tests
│   ├── column.rs        # Column implementations + unit tests
│   ├── error.rs         # Error types + unit tests
│   ├── parser.rs        # SQL parser + unit tests
│   ├── planner.rs       # Query planner + unit tests
│   ├── execution.rs     # Execution engine + unit tests
│   ├── aggregates.rs    # Aggregate functions + unit tests
│   ├── catalog.rs       # Catalog management + unit tests
│   ├── table.rs         # Table operations + unit tests
│   └── ingest.rs        # CSV ingestion + unit tests
├── tests/
│   ├── data/            # Test data files
│   │   └── test_data.csv
│   ├── integration_tests.rs  # Integration test suite
│   └── manual_query.rs       # Manual query tests
├── benches/
│   └── query_benchmark.rs    # Performance benchmarks
└── docs/
    └── testing/
        └── test_strategy.md # This document
```

### Test Data Management

**Location**: `tests/data/`

**Files**:
- `test_data.csv`: Sample CSV file for integration testing

**Test Data Characteristics**:
- Small and manageable for fast tests
- Representative of real-world data
- Includes edge cases (null values, various types, etc.)
- Version-controlled with the project

## Code Coverage Goals

### Target Coverage

- **Overall Target**: >80% line coverage
- **Critical Path**: 100% coverage for core execution operators
- **Parser Components**: >90% coverage (error handling intensive)
- **Ingestion Layer**: >85% coverage

### Coverage Tools

**Recommended Tools**:
- `cargo-tarpaulin`: Linux-based coverage reporting
- `cargo-llvm-cov`: Alternative coverage tool
- `grcov`: Coverage report generator

**Running Coverage Reports**:
```bash
# Using cargo-tarpaulin
cargo tarpaulin --out Html --output-dir target/coverage

# Using cargo-llvm-cov
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

### Coverage Metrics to Track

- Line coverage
- Branch coverage
- Function coverage
- Region coverage

## Testing Tools and Frameworks

### Core Dependencies

From `Cargo.toml`:
```toml
[dev-dependencies]
pretty_assertions = "1.4"  # Better error messages
tempfile = "3"            # Temporary file management
criterion = "0.5"         # Benchmarking framework
```

### Tools Description

1. **cargo test**: Built-in Rust test runner
   - Runs unit tests, integration tests, and doc tests
   - Supports filtering with `--test` and `--` flags

2. **pretty_assertions**: Enhanced assertion output
   - Colorized diffs
   - Clear failure messages

3. **tempfile**: Temporary file handling
   - Create and clean up temp files automatically
   - Useful for CSV ingestion tests

4. **criterion**: Statistical benchmarking
   - Handles warmup, sampling, and statistical analysis
   - Generates HTML reports with graphs

## Best Practices

### 1. Test Naming

- Use descriptive names: `test_parse_select_with_multiple_columns`
- Follow pattern: `test_<function>_<scenario>`
- Group related tests with `mod tests` blocks

### 2. Test Structure (AAA Pattern)

```rust
#[test]
fn test_filter_with_numeric_comparison() {
    // Arrange: Set up test data and expected results
    let column = IntColumn::from_vec(vec![1, 2, 3, 4, 5]);
    let predicate = BinaryExpr {
        op: BinaryOp::Gt,
        left: Box::new(Expr::Column("value".to_string())),
        right: Box::new(Expr::Literal(Value::Int(3))),
    };
    
    // Act: Execute the function being tested
    let result = filter(&column, &predicate).unwrap();
    
    // Assert: Verify the result matches expectations
    assert_eq!(result.len(), 2);
    assert_eq!(result.as_int_slice(), &[4, 5]);
}
```

### 3. Test Data Management

- Use fixtures for common test data
- Keep test data minimal but representative
- Document assumptions about test data
- Clean up resources in test teardown

### 4. Edge Case Testing

- Test boundary conditions (empty, single element, max size)
- Test error conditions (invalid input, parse errors)
- Test type conversions and edge cases
- Test with null/None values where applicable

### 5. Isolation

- Tests should be independent of each other
- Use test setup/teardown properly
- Avoid shared mutable state between tests
- Mock external dependencies when necessary

### 6. Performance Considerations

- Keep unit tests fast (<100ms each)
- Use `#[ignore]` for slow tests
- Separate unit tests from integration tests
- Use test fixtures to avoid repeated setup

## Continuous Testing

### Pre-Commit Hooks

**File**: `.githooks/pre-commit`

**Checks Performed**:
1. Run `cargo fmt --check` - Code formatting
2. Run `cargo clippy` - Lint checks
3. Run `cargo test --lib` - Quick unit tests
4. Run `cargo test --doc` - Documentation tests

**Purpose**: Catch issues before they reach the repository

### Pre-Push Hooks

**File**: `.githooks/pre-push`

**Checks Performed**:
1. All pre-commit checks
2. Full test suite including integration tests
3. Build verification
4. Optional: Coverage checks

**Purpose**: Ensure code quality before sharing changes

### CI/CD Pipeline

**Triggers**:
- Pull requests
- Merges to main branch
- Scheduled nightly builds

**Stages**:
1. **Build**: Verify code compiles
2. **Lint**: Run clippy and rustfmt checks
3. **Test**: Run full test suite
4. **Coverage**: Generate coverage reports
5. **Benchmarks**: Run performance benchmarks (optional)

## Performance Testing

### Benchmark Baselines

**Current Benchmark Results**:
- SQL Parsing: ~microseconds per query
- Full Scan: Depends on dataset size
- Filter: Sub-millisecond for typical datasets
- Aggregation: Varies with grouping complexity

### Performance Regression Detection

**Strategy**:
- Benchmark on every merge to main
- Compare against previous run
- Alert on significant (>10%) performance degradation

### Profiling Tools

**For Hot Path Analysis**:
1. `cargo flamegraph`: Generate flamegraphs
2. `perf`: Linux profiler
3. ` criterion -- --profile-time`: Benchmark-specific profiling

**Example**:
```bash
cargo install flamegraph
cargo flamegraph --bench query_benchmark
```

## Property-Based Testing (Optional)

### Future Enhancement

**Tool**: `proptest` - Property-based testing framework

**Use Cases**:
- Parser correctness with random inputs
- Algebraic properties (e.g., filter after filter = single filter)
- Round-trip properties (parse → string → parse)
- Invariants (aggregation always produces correct type)

**Example**:
```rust
proptest! {
    #[test]
    fn test_round_trip_parse(query in "SELECT * FROM t") {
        let parsed = Parser::new(&query).parse().unwrap();
        let reconstructed = parsed.to_string();
        let reparsed = Parser::new(&reconstructed).parse().unwrap();
        assert_eq!(parsed, reparsed);
    }
}
```

## Test Execution

### Running All Tests

```bash
# Run all tests (unit, integration, doc)
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode (faster)
cargo test --release
```

### Running Specific Tests

```bash
# Run tests in a specific module
cargo test parser

# Run a specific test
cargo test test_parse_select

# Run integration tests only
cargo test --test integration_tests

# Run doc tests only
cargo test --doc
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench full_scan

# Save benchmark results
cargo bench -- --save-baseline main
```

## Current Test Status

### Test Count Summary

- **Unit Tests**: 361 passing
- **Integration Tests**: 51 passing (17 ignored)
- **Manual Query Tests**: 16 passing
- **Doc Tests**: 51 passing
- **Total Active Tests**: 479 passing

### Test Categories by Module

| Module          | Tests | Status |
|-----------------|-------|--------|
| Core Types      | ~50   | ✅ Pass |
| Columns         | ~40   | ✅ Pass |
| Error Handling  | ~20   | ✅ Pass |
| Parser          | ~60   | ✅ Pass |
| Planner         | ~40   | ✅ Pass |
| Execution       | ~50   | ✅ Pass |
| Aggregates      | ~30   | ✅ Pass |
| Catalog         | ~25   | ✅ Pass |
| Table           | ~20   | ✅ Pass |
| Ingestion       | ~26   | ✅ Pass |
| Integration     | ~67   | ✅ Pass (17 ignored) |

## Future Improvements

### Short Term

1. **Achieve >80% code coverage**
   - Add tests for error paths
   - Increase integration test coverage
   - Add property-based tests for parser

2. **Add performance regression testing**
   - Set up benchmark baselines
   - Automated comparison
   - Alert on degradation

3. **Test data expansion**
   - Add larger datasets for performance testing
   - Add edge case datasets
   - Include real-world data samples

### Medium Term

1. **Fuzz Testing**
   - Use `cargo-fuzz` for parser
   - Find edge cases in CSV parsing
   - Robustness testing

2. **Concurrency Testing**
   - Test catalog thread safety
   - Test query execution under concurrent load
   - Race condition detection

3. **Snapshot Testing**
   - Validate query output formats
   - Test backwards compatibility
   - Version-to-version comparison

### Long Term

1. **Distributed Testing**
   - Multi-node scenarios
   - Network failure simulation
   - Consistency verification

2. **Stress Testing**
   - Large dataset handling
   - Memory pressure tests
   - Long-running query tests

3. **Production-like Testing**
   - Real-world workload simulation
   - Customer query patterns
   - Production data sanitization

## Conclusion

The test strategy for Mini Rust OLAP emphasizes comprehensive coverage across unit, integration, and benchmark testing. With 479 passing tests and a robust CI/CD pipeline, the project maintains high code quality and reliability. Future work will focus on increasing code coverage, adding performance regression testing, and exploring advanced testing techniques like fuzz testing and snapshot testing.

### Quick Reference

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Generate coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html

# Run linting
cargo clippy

# Format code
cargo fmt
```

---

**Document Version**: 1.0  
**Last Updated**: Phase 7 Complete  
**Maintained By**: Development Team