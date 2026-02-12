# Mini Rust OLAP Examples

This directory contains example programs demonstrating how to use the `mini_rust_olap` library both as a standalone application and as a Rust library.

## Running Examples

All examples can be run using `cargo run`:

```bash
# Run the simple table example
cargo run --example simple_table

# Run the CSV loading example
cargo run --example csv_loading
```

## Available Examples

### 1. `simple_table.rs`

**Purpose**: Demonstrates basic table creation, data insertion, and query execution using the library's API directly.

**What you'll learn**:
- How to create a `Table` and define columns
- How to insert data rows programmatically
- How to use execution operators (`ScanOperator`, `FilterOperator`, `ProjectOperator`)
- How to execute SQL queries using the parser and planner

**Use case**: Perfect for understanding the internal architecture and API design of the library.

### 2. `csv_loading.rs`

**Purpose**: Shows how to load data from CSV files and query it using SQL.

**What you'll learn**:
- How to load CSV data into the catalog using `load_csv_into_catalog`
- How to execute various SQL queries (SELECT, WHERE, GROUP BY, ORDER BY, LIMIT)
- How to perform aggregations (COUNT, AVG, SUM, MIN, MAX)
- How to format and display query results

**Use case**: Demonstrates the typical workflow for analytical queries on CSV data.

## Tips for Learning

1. **Start with `simple_table.rs`** if you want to understand the internals
2. **Start with `csv_loading.rs`** if you want to see practical SQL queries
3. **Read the source code comments** - they explain each step in detail
4. **Modify the examples** - try adding your own queries or data to experiment

## Related Documentation

- [Project README](../README.md) - Main project documentation
- [Learning Guides](../docs/) - Detailed educational guides for each development phase
- [API Documentation](https://docs.rs/mini_rust_olap) - Official Rust API documentation