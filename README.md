# Mini Rust OLAP - Mini OLAP Database Engine

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-446%20passing-green.svg)]()
[![Phase](https://img.shields.io/badge/phase-8%20complete-success.svg)]()

**A lightweight, in-memory OLAP database engine built with Rust for educational purposes**

[Features](#-features) • [Architecture](#%EF%B8%8F-architecture) • [Quick Start](#-quick-start) • [Learning Path](#-learning-path)

</div>

---

## 📖 About

**Mini Rust OLAP** is a miniature Online Analytical Processing (OLAP) database engine designed specifically for educational purposes. It demonstrates core database internals concepts through clean, well-documented Rust code with a **fully interactive command-line interface** for real-time data exploration.

### Why Mini Rust OLAP?

This project was created to help developers learn:

- **Database Internals**: How column-oriented storage differs from row-oriented systems
- **Rust Programming**: Advanced Rust patterns including traits, generics, and error handling
- **Systems Programming**: Memory layout, CPU cache awareness, and performance optimization
- **Query Execution**: From SQL parsing to physical operator execution

Unlike production databases that are complex and hard to understand, Mini Rust OLAP is intentionally simple while still demonstrating fundamental OLAP concepts.

### Educational Goals

- ✅ **Learnability**: Clean, well-commented code for intermediate Rust developers
- ✅ **Correctness**: Tested implementations that match mathematical expectations
- ✅ **Performance Awareness**: Understanding why column-stores excel at analytical queries
- ✅ **Zero Dependencies**: Core logic implemented from scratch (no heavy external crates)

---

## ✨ Features

### Current Implementation (Phase 8 - Complete ✅)

#### 🏗️ Core Foundation
- **Error Handling**: Comprehensive error types using `thiserror` (11 tests)
- **Data Types**: Support for `Int64`, `Float64`, and `String` with type safety (26 tests)
- **Columnar Storage**: Efficient column-oriented data layout (33 tests)

#### 📊 Column Types
- **IntColumn**: 64-bit integer storage in `Vec<i64>`
- **FloatColumn**: 64-bit floating point storage in `Vec<f64>`
- **StringColumn**: UTF-8 string storage in `Vec<String>`

#### 🔍 Manual Query Operations
- **Aggregations**: SUM, AVG, COUNT, MIN, MAX
- **Filtering**: WHERE clause with AND/OR logic
- **Projection**: SELECT specific columns
- **Grouping**: GROUP BY with aggregation

#### 🗃️ Table Management (Phase 2)
- **Schema Definition**: Define table structure with column names and types
- **Data Insertion**: Add rows with automatic type conversion
- **Column Operations**: Add, drop, select, and query columns (33 tests)
- **Schema Validation**: Ensure data integrity with type checks

#### 📚 Catalog System (Phase 2)
- **Table Registry**: Central metadata repository for all tables
- **Table Operations**: Register, retrieve, drop, and rename tables (25 tests)
- **Query Support**: Check existence, list tables, and access metadata
- **Integration**: Seamless table-catalog coordination

#### 📄 CSV Ingestion (Phase 3)
- **CSV Parsing**: Robust CSV file reading using csv crate
- **Type Inference**: Automatic detection of Int64, Float64, and String types
- **Data Loading**: Row-to-column transposition for efficient storage
- **Error Handling**: Comprehensive error handling for malformed CSVs
- **Integration**: Direct catalog registration with load_csv_into_catalog (38 tests)

#### 🚀 Query Execution Engine (Phase 4)
- **Vectorized Processing**: Batch-based columnar execution for performance
- **TableScan Operator**: Read data from tables with column pruning and batch sizing (33 tests)
- **Filter Operator**: Predicate evaluation with BinaryComparison, AND, and OR logic (19 tests)
- **Project Operator**: Column selection, reordering, and aliasing (22 tests)
- **Aggregate Functions**: Count, Sum, Min, Max, Avg with stateful design (65 tests)
- **GroupBy Operator**: Hash-based grouping with multiple aggregates per group (16 tests)
- **Operator Chaining**: Seamless integration of operators in query pipelines
- **Integration Testing**: 16 comprehensive tests for operator chains

#### 🔧 SQL Parser (Phase 5)
- **Tokenizer**: Lexical analysis with 20+ token types including keywords, operators, and literals (10 tests)
- **AST Design**: Abstract Syntax Tree with Query, SelectStatement, and Expression structures
- **Recursive Descent Parser**: Full SQL SELECT statement parsing (9 tests)
- **Expression Parsing**: Support for arithmetic, logical, and comparison operators with proper precedence
- **Aggregate Functions**: Parse COUNT, SUM, AVG, MIN, MAX with column and wildcard arguments
- **WHERE Clauses**: Complex boolean expressions with AND, OR, NOT, and nested predicates
- **GROUP BY Parsing**: Multi-column grouping with aggregate function support
- **Error Handling**: Descriptive error messages with line/column tracking using thiserror
- **Case Sensitivity**: Keywords are case-insensitive, identifiers converted to lowercase
- **Comprehensive Testing**: 19 tests covering queries, expressions, edge cases, and tokenizer

#### 🧠 Query Planning (Phase 6)
- **Query Planner**: Converts parsed SQL to optimized execution plans (8 tests)
- **Column Pruning**: Removes unused columns from queries for efficiency (2 tests)
- **Operator Ordering**: Correct placement of operators in execution tree
- **Schema Integration**: Maintains schema throughout query pipeline
- **Expression Analysis**: Validates and optimizes expressions

#### 📊 Advanced Query Features (Phase 6.2)
- **ORDER BY Clause**: Sort results by one or more columns (4 tests)
  - Single and multi-column sorting with ASC/DESC directions
  - Support for Int64, Float64, and String data types
  - Proper column index mapping for simple and GROUP BY queries
- **LIMIT Clause**: Restrict number of rows returned (2 tests)
  - Efficient row counting with early termination
  - Works independently and combined with ORDER BY
- **OFFSET Clause**: Skip specified number of rows (1 test)
  - Pagination support with proper row skipping
  - Works with and without LIMIT
- **Combined Features**: Full pagination support (1 test)
  - ORDER BY + LIMIT: Top N sorted results
  - ORDER BY + OFFSET: Skip and sort
  - LIMIT + OFFSET: Pagination functionality
  - **Multi-column ORDER BY with LIMIT/OFFSET**

  #### 💻 Interactive REPL (Phase 7) - NEW!
  - **Command History**: Full readline support with `rustyline` for persistent command history to `.olap_history`
  - **CSV Loading**: LOAD command to import CSV files into catalog with automatic type inference (Int64, Float64, String)
  - **SQL Query Execution**: Full support for SELECT queries including WITH clause for CTEs
  - **Complete Clauses**: WHERE, GROUP BY, ORDER BY (ASC/DESC), LIMIT all supported
  - **Catalog Management**: SHOW TABLES (also `.TABLES`) and DESCRIBE (also `.SCHEMA`) commands
  - **Professional Output**: Clean ASCII table formatting with box-drawing characters (┌─┐│├─┤└─┘)
  - **Error Handling**: Visual error messages in formatted boxes with helpful context
  - **Performance Metrics**: Execution timing for all operations (ms or s based on duration)
  - **Signal Handling**: Graceful Ctrl+C (continue) and Ctrl+D (exit) behavior
  - **Command Aliases**: Multiple formats supported (HELP/.HELP/?, EXIT/QUIT/.EXIT, CLEAR/.CLEAR)
  - **Utility Commands**: HELP, CLEAR, and EXIT for enhanced user interaction
  - **Aggregate Functions**: COUNT(*), SUM, AVG, MIN, MAX working seamlessly
  - **Pagination**: Result sets limited to 50 rows by default with pagination messages
  - **Empty Result Handling**: Clear messages for empty result sets
  - **Welcome Screen**: Professional startup banner with version information

  #### 📋 Quality Improvements & Documentation (Phase 8) - NEW!
  - **API Documentation**: Comprehensive API reference generated with `cargo doc --no-deps`
  - **Test Strategy Documentation**: Complete testing guide at `docs/testing/test_strategy.md` (561 lines)
  - **Performance Analysis**: Detailed memory and optimization guide at `docs/performance/memory_and_optimization.md` (1,581 lines)
    - Memory architecture and usage patterns
    - Performance bottleneck identification
    - Optimization strategies (zero-copy, SIMD, compression)
    - Hot path analysis for all operators
    - Profiling tools guide (flamegraph, criterion, valgrind, perf)
    - Optimization roadmap (4 phases over 3+ months)
  - **Property-Based Tests**: 20 property-based tests for SQL parser (tests/parser_properties.rs)
    - Robustness tests (6 tests)
    - Round-trip properties (1 test)
    - Semantic properties (3 tests)
    - Algebraic properties (2 tests)
    - Edge cases (4 tests)
    - Regression tests (4 tests)
  - **Benchmark Suite**: Comprehensive performance benchmarks at `benches/query_benchmark.rs`
    - SQL parsing benchmarks
    - Table scan benchmarks
    - Filter and project benchmarks
    - Aggregation benchmarks
    - ORDER BY benchmarks
    - Full query execution benchmarks
  - **Proptest Integration**: Added `proptest` dependency for property-based testing
  - **Learning Guide**: Comprehensive Phase 8 learning guide at `docs/phase8-learning-guide.md` (2,371 lines)
    - API Documentation (cargo doc, rustdoc, doc tests)
    - Test Strategy (unit, integration, property-based tests)
    - Performance & Memory Optimization (profiling, SIMD, compression)
    - Property-Based Testing (proptest framework, strategies, shrinking)
    - Performance Benchmarks (Criterion framework, best practices)
    - Code Coverage (measurement tools and goals)
    - Best Practices for all topics
    - 4 detailed practical examples with code
  - **Assessment**: Phase 8 assessment at `docs/phase8-assessment.md` (447 lines)
    - 45 questions across 5 sections
    - Complete answer key with explanations
    - Scoring guide (80% passing threshold)
    - Time estimate: 60-90 minutes

  ### Planned Features (Roadmap)

  - [x] Phase 4: Physical query operators (Scan, Filter, Project, Aggregate) ✅
  - [x] Phase 5: SQL parser for SELECT statements ✅
  - [x] Phase 6.1: Query planning and optimization ✅
  - [x] Phase 6.2: ORDER BY, LIMIT, OFFSET clauses ✅
  - [x] Phase 7: Interactive REPL (Read-Eval-Print Loop) ✅
  - [x] Phase 8: Additional Tasks & Quality Improvements ✅

---

## 🏛️ Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                  Mini Rust OLAP Architecture                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐   │
│  │    REPL     │───▶│   Parser    │───▶│   Planner   │   │
│  │  (Phase 7)  │    │  (Phase 5)  │    │  (Phase 6)  │   │
│  └─────────────┘    └─────────────┘    └─────────────┘   │
│                                                │           │
│                                                ▼           │
│  ┌───────────────────────────────────────────────────────┐   │
│  │              Physical Operators (Phase 4)            │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────┐ │   │
│  │  │  Scan   │─▶│ Filter  │─▶│ Project │─▶│ Aggr  │ │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └──────┘ │   │
│  └───────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          ▼                                  │
│  ┌───────────────────────────────────────────────────────┐   │
│  │              Storage Layer (Phase 2)                  │   │
│  │  ┌─────────────────────────────────────────────────┐  │   │
│  │  │  Catalog ──────▶ Table 1                     │  │   │
│  │  │   (metadata)       ├─ Column 1 (Int64)         │  │   │
│  │  │                  ├─ Column 2 (Float64)        │  │   │
│  │  │                  └─ Column 3 (String)         │  │   │
│  │  └─────────────────────────────────────────────────┘  │   │
│  └───────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          ▼                                  │
│  ┌───────────────────────────────────────────────────────┐   │
│  │              Core Modules (Phase 1 - Complete)      │   │
│  │  • Error Handling (DatabaseError)                   │   │
│  │  • Data Types (DataType, Value)                     │   │
│  │  • Column Trait & Implementations                   │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Columnar Storage Explained

Traditional row-oriented databases (OLTP) store data like this:

```
Row 1: [id: 1, name: "Alice", age: 25]
Row 2: [id: 2, name: "Bob",   age: 30]
Row 3: [id: 3, name: "Charlie", age: 35]
```

Mini Rust OLAP (column-oriented OLAP) stores data like this:

```
id column:   [1, 2, 3, ...]
name column: ["Alice", "Bob", "Charlie", ...]
age column:  [25, 30, 35, ...]
```

**Why Columnar?**

1. **Compression**: Similar values in columns compress better
2. **Cache Efficiency**: Read only needed columns into CPU cache
3. **Vectorized Execution**: Process entire vectors with SIMD instructions
4. **I/O Reduction**: Skip reading irrelevant columns from disk

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.70 or later
- Basic understanding of Rust concepts (ownership, traits, enums)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/mini_rust_olap.git
cd mini_rust_olap

# Build the project
cargo build

# Run tests
cargo test

# Run examples (after building)
cargo run --example simple_table
cargo run --example csv_loading

# Run benchmarks (after building)
cargo bench

# Start the REPL
cargo run --release
```

### Basic Usage Example

```rust
use mini_rust_olap::{
    Column, IntColumn, FloatColumn, StringColumn, Value
};

fn main() -> mini_rust_olap::Result<()> {
    // Create columns
    let mut ids = IntColumn::new();
    let mut names = StringColumn::new();
    let mut ages = FloatColumn::new();

    // Insert data
    ids.push_value(Value::Int64(1))?;
    names.push_value(Value::String("Alice".to_string()))?;
    ages.push_value(Value::Float64(25.0))?;

    ids.push_value(Value::Int64(2))?;
    names.push_value(Value::String("Bob".to_string()))?;
    ages.push_value(Value::Float64(30.0))?;

    // Manual aggregation: Calculate average age
    let mut sum = 0.0;
    for i in 0..ages.len() {
        if let Value::Float64(age) = ages.get(i)? {
            sum += age;
        }
    }
    let avg_age = sum / ages.len() as f64;

    println!("Average age: {:.1}", avg_age);

    Ok(())
}
```

### Library API Example

```rust
use mini_rust_olap::{Column, IntColumn, FloatColumn, Value};

fn main() -> mini_rust_olap::Result<()> {
    let mut scores = IntColumn::new();
    
    // Add some test scores
    scores.push_value(Value::Int64(85))?;
    scores.push_value(Value::Int64(92))?;
    scores.push_value(Value::Int64(78))?;
    scores.push_value(Value::Int64(95))?;

    // Find high scores (> 90)
    let mut high_scorers = Vec::new();
    for i in 0..scores.len() {
        if let Value::Int64(score) = scores.get(i)? {
            if score > 90 {
                high_scorers.push(score);
            }
        }
    }

    println!("High scores: {:?}", high_scorers);

    Ok(())
}
```

### REPL Usage Example

The interactive REPL provides an easy way to explore data without writing code:

```bash
# Build and start the REPL
cargo build --release
./target/release/mini_rust_olap

# You'll see the welcome screen
╔═════════════════════════════════════════════════════════╗
║     Mini Rust OLAP - Interactive REPL v0.1.0            ║
╚═════════════════════════════════════════════════════════╝

Welcome to Mini Rust OLAP! Type HELP for available commands.

olap> # Load some data
olap> LOAD examples/sales.csv AS sales
Loading CSV from 'examples/sales.csv' as 'sales'...
✓ Loaded table 'sales' successfully.
⏱ Executed in 7.62ms

olap> # Explore the catalog
olap> SHOW TABLES
Tables in catalog:
  - sales
⏱ Executed in 0.02ms

olap> # Inspect the table schema
olap> DESCRIBE sales

Table: sales
┌────────────────────────┬──────────┬────────────────┐
│ Column Name            │ Type     │ Description    │
├────────────────────────┼──────────┼────────────────┤
│ id                     │ Int64    │        1000 rows│
│ product                │ String   │        1000 rows│
│ region                 │ String   │        1000 rows│
│ amount                 │ Float64  │        1000 rows│
│ date                   │ String   │        1000 rows│
└────────────────────────┴──────────┴────────────────┘
Total rows: 1000
⏱ Executed in 0.47ms

olap> # Run a simple query
olap> SELECT * FROM sales LIMIT 5
┌────┬────────────────┬────────────┬────────────┬────────────┐
│col_0│ col_1          │ col_2      │ col_3      │ col_4      │
├────┼────────────────┼────────────┼────────────┼────────────┤
│  1 │ Widget A       │ North      │    1000.0  │2024-01-01  │
│  2 │ Widget B       │ South      │    1500.0  │2024-01-02  │
│  3 │ Widget A       │ East       │    1200.0  │2024-01-03  │
│  4 │ Widget C       │ West       │     800.0  │2024-01-04  │
│  5 │ Widget B       │ North      │    1100.0  │2024-01-05  │
└────┴────────────────┴────────────┴────────────┴────────────┘
+(5 rows)
⏱ Executed in 0.62ms

olap> # Filter with WHERE clause
olap> SELECT product, amount FROM sales WHERE amount > 1400
┌────────────────┬──────────────────┐
│ col_0          │ col_1             │
├────────────────┼──────────────────┤
│ Widget B       │      1500.0       │
│ Widget A       │      1450.0       │
│ Widget C       │      1600.0       │
└────────────────┴──────────────────┘
+(3 rows)
⏱ Executed in 0.41ms

olap> # Aggregate by region
olap> SELECT region, COUNT(*), SUM(amount) FROM sales GROUP BY region
┌────────────────┬──────────────┬──────────────────┐
│ col_0          │ col_1        │ col_2             │
├────────────────┼──────────────┼──────────────────┤
│ East           │          250  │        125000.0   │
│ North          │          250  │        118000.0   │
│ South          │          250  │        132000.0   │
│ West           │          250  │         95000.0   │
└────────────────┴──────────────┴──────────────────┘
+(4 rows)
⏱ Executed in 0.35ms

olap> # Complex query with sorting
olap> SELECT product, COUNT(*) AS count 
    > FROM sales 
    > WHERE amount > 1000 
    > GROUP BY product 
    > ORDER BY count DESC
┌────────────────┬──────────────┐
│ col_0          │ col_1        │
├────────────────┼──────────────┤
│ Widget A       │          180  │
│ Widget B       │          165  │
│ Widget C       │          155  │
└────────────────┴──────────────┘
+(3 rows)
⏱ Executed in 0.41ms

olap> # Get help
olap> HELP

Mini Rust OLAP - Available Commands:
═══════════════════════════════════════════

Data Loading:
  LOAD <path> AS <table_name>      Load a CSV file into the catalog

Querying:
  SELECT <columns> FROM <table>    Execute a SQL SELECT query
  WHERE <condition>                Add filtering conditions
  GROUP BY <columns>               Group results
  ORDER BY <columns> [ASC|DESC]    Sort results
  LIMIT <n>                        Limit number of rows

Catalog Management:
  SHOW TABLES                       List all tables
  DESCRIBE <table_name>             Show table schema

Utility:
  HELP or ?                         Show this help message
  CLEAR                             Clear screen
  EXIT or QUIT                      Exit the REPL

Features:
  • Columnar storage for fast analytics
  • SQL-like query language
  • Automatic type inference from CSV
  • Aggregations: COUNT, SUM, AVG, MIN, MAX
⏱ Executed in 0.01ms

olap> # Use command history (up arrow)
olap> SELECT product, COUNT(*) AS count FROM sales GROUP BY product ORDER BY count DESC LIMIT 2
┌────────────────┬──────────────┐
│ col_0          │ col_1        │
├────────────────┼──────────────┤
│ Widget A       │          180  │
│ Widget B       │          165  │
└────────────────┴──────────────┘
+(2 rows)
⏱ Executed in 0.38ms

olap> # Exit cleanly
olap> EXIT
Goodbye!
⏱ Executed in 0.01ms
```

**Key REPL Features:**
- **Command History**: Use up/down arrows to navigate previous commands
- **Case Insensitive**: Commands work in any case (HELP, help, Help)
- **Command Aliases**: Multiple formats supported (HELP/.HELP/?, SHOW TABLES/.TABLES)
- **Error Recovery**: Errors don't crash the REPL, just show a message
- **Performance**: All operations show timing in milliseconds
- **Formatting**: Clean ASCII tables with proper alignment

---

## 🎯 Examples

The project includes comprehensive examples demonstrating library usage:

### Running Examples

```bash
# Run all examples
cargo run --example simple_table
cargo run --example csv_loading

# List all available examples
cargo run --example --help
```

### Available Examples

#### 1. **`examples/simple_table.rs`**
Demonstrates programmatic table creation and SQL queries.

**Sample Output:**
```bash
$ cargo run --example simple_table
🚀 Mini Rust OLAP - Simple Table Example

📋 Creating 'employees' table...
✓ Table created and registered

🔍 Example 1: SELECT * FROM employees
SQL: SELECT * FROM employees
  +----+----------+-------------+--------+
  | id | name     | department  | salary |
  +----+----------+-------------+--------+
  | 1  | Alice    | Engineering | 90000  |
  | 2  | Bob      | Marketing   | 75000  |
  | 3  | Charlie  | Engineering | 95000  |
  | 4  | Diana    | Sales       | 80000  |
  | 5  | Eve      | Marketing   | 82000  |

🔍 Example 2: SELECT department, AVG(salary) FROM employees GROUP BY department
SQL: SELECT department, AVG(salary) FROM employees GROUP BY department
  +-------------+-------------+
  | col_0       | col_1       |
  +-------------+-------------+
  | Engineering | 92500.0     |
  | Marketing   | 78500.0     |
  | Sales       | 80000.0     |

All examples completed successfully! ✓
```

#### 2. **`examples/csv_loading.rs`**
Demonstrates CSV loading and various SQL operations.

**Sample Output:**
```bash
$ cargo run --example csv_loading
🚀 Mini Rust OLAP - CSV Loading Example

📂 Loading data from: tests/data/test_data.csv
✓ Data loaded successfully (10 rows, 6 columns)

🔍 Example 1: SELECT * FROM employees LIMIT 5
SQL: SELECT * FROM employees LIMIT 5
  +----+----------+-------------+--------+
  | id | name     | department  | salary |
  +----+----------+-------------+--------+
  | 1  | Alice    | Engineering | 90000  |
  | 2  | Bob      | Marketing   | 75000  |
  | 3  | Charlie  | Engineering | 95000  |
  | 4  | Diana    | Sales       | 80000  |
  | 5  | Eve      | Marketing   | 82000  |

🔍 Example 2: SELECT department, COUNT(*) FROM employees WHERE salary > 80000 GROUP BY department
SQL: SELECT department, COUNT(*) FROM employees WHERE salary > 80000 GROUP BY department
  +-------------+-------+
  | col_0       | col_1 |
  +-------------+-------+
  | Engineering | 2     |
  | Marketing   | 1     |

🔍 Example 3: SELECT name, salary FROM employees ORDER BY salary DESC LIMIT 3
SQL: SELECT name, salary FROM employees ORDER BY salary DESC LIMIT 3
  +----------+--------+
  | col_0    | col_1  |
  +----------+--------+
  | Charlie  | 95000  |
  | Alice    | 90000  |
  | Eve      | 82000  |

All examples completed successfully! ✓
```

For detailed explanations of each example, see `examples/README.md`.

## 🔬 Benchmarks

Performance benchmarks are provided using the criterion crate to measure query performance:

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench full_scan
cargo bench aggregation

# Run with custom settings
cargo bench -- --profile-time 10  # For flamegraphs

# View detailed report (HTML)
open target/criterion/report/index.html
```

### What the Benchmarks Measure

The comprehensive benchmark suite measures performance across all query execution stages:

- **SQL Parsing** - How fast queries are parsed and converted to AST
- **Full Scan** - Performance of reading entire tables into memory
- **Filter Operations** - WHERE clause evaluation and predicate filtering
- **Project Operations** - Column selection and reordering performance
- **Aggregation** - GROUP BY and aggregate function (COUNT, SUM, AVG, MIN, MAX) performance
- **Order By** - Sorting performance for single and multi-column sorting
- **Full Query Execution** - End-to-end query performance (parse → plan → execute)

Each benchmark tests multiple data sizes (100, 1,000, 10,000 rows) to provide comprehensive performance insights.

### Sample Benchmark Output

```bash
$ cargo bench

Running target/release/deps/query_benchmark-xxx

running 7 tests
test full_scan::scan_100_rows      ... bench:       1,234 ns/iter (+/- 123)
test full_scan::scan_1000_rows     ... bench:      12,345 ns/iter (+/- 1,234)
test filter::filter_1000_rows      ... bench:      8,765 ns/iter (+/- 876)
test projection::project_1000_rows ... bench:      3,456 ns/iter (+/- 345)
test aggregation::group_by_1000     ... bench:     15,678 ns/iter (+/- 1,567)
test order_by::sort_1000_rows      ... bench:      9,876 ns/iter (+/- 987)
test complex_query::full_1000      ... bench:     23,456 ns/iter (+/- 2,345)

test result: ok. 0.00s; 0.00s; 0.00s for 7 tests

Gnuplot not found, using plotters backend
```

Detailed HTML reports are generated in `target/criterion/report/index.html` with:
- Performance comparisons between runs
- Graphs showing performance over time
- Statistical analysis (mean, median, standard deviation)
- Regression/improvement detection

For detailed interpretation of benchmark results, see `benches/README.md`.

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run integration tests
cargo test --test manual_query

# Run tests with output
cargo test -- --nocapture

# Run tests with filtering
cargo test test_manual_sum

# Run test scripts
./scripts/test_repl.sh          # Comprehensive REPL testing
./scripts/test_repl_simple.sh   # Basic REPL testing
./scripts/final_test.sh         # Final integration testing
```

### Test Coverage

```bash
# Install tarpaulin for coverage reports
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

### Current Test Status

- **Total Tests**: 446 passing ✅ (361 library tests + 15 integration tests + 16 manual tests + 34 active doc tests [17 ignored] + 20 property-based tests)
- **Library Tests**: 361 (error: 10, types: 26, column: 33, table: 33, catalog: 25, ingest: 38, execution: 77, aggregates: 65, parser: 19, planner: 10, lib: 3)
- **Integration Tests**: 15 (operator chaining: 16)
- **Manual Query Tests**: 16
- **Documentation Tests**: 51 total (34 active, 17 ignored)
- **Code Coverage**: High test coverage across all implemented phases (Foundation, Storage Layer, CSV Ingestion, Query Operators, SQL Parser, Query Planning, Advanced Query Features)

### Documentation & Assessments

#### Phase 1: Foundation

✅ **Phase 1 Learning Guide** (`docs/phase1-learning-guide.md`) - 2,668 lines covering 10 chapters
  - Rust programming fundamentals and best practices
  - Database internals and column-oriented storage
  - Code examples with detailed explanations
  - Self-assessment questions and exercises
  
✅ **Phase 1 Assessment** (`docs/phase1-assessment.md`) - 431 lines
  - 35 multiple-choice questions covering Phase 1 concepts
  - Comprehensive answer key with detailed explanations
  - Scoring guide and study recommendations

#### Phase 2: Storage Layer

✅ **Phase 2 Learning Guide** (`docs/phase2-learning-guide.md`) - Comprehensive guide covering:
  - Table design fundamentals and data organization
  - HashMap and collections in Rust for efficient lookups
  - Advanced trait implementations and type safety
  - Schema validation and enforcement mechanisms
  - Catalog design patterns for metadata management
  - Error handling for complex data structures
  - Testing strategies for storage layer components
  - Integration and modularity best practices

✅ **Phase 2 Assessment** (`docs/phase2-assessment.md`)
  - 35 multiple-choice questions across 4 parts
  - Topics: Rust fundamentals, Table operations, Catalog management, Advanced topics
  - Comprehensive answer key with detailed explanations
  - Scoring guide with readiness indicators for Phase 3

#### Phase 3: CSV Ingestion

✅ **Phase 3 Learning Guide** (`docs/phase3-learning-guide.md`) - 2,009 lines covering 11 chapters
  - CSV parsing fundamentals and type inference algorithms
  - Row-to-column transposition and data transformation
  - Error handling strategies and testing approaches
  - Advanced topics like streaming and parallel processing
  - Self-assessment questions and practical exercises
  
✅ **Phase 3 Assessment** (`docs/phase3-assessment.md`) - 1,919 lines
  - 45 multiple-choice questions covering CSV ingestion concepts
  - Comprehensive answer key with detailed explanations
  - Scoring guide and preparation checklist for Phase 4
  - Tests understanding of Rust patterns and database concepts

#### Phase 4: Query Execution Engine

✅ **Phase 4 Learning Guide** (`docs/phase4-learning-guide.md`) - 2,895 lines covering:
  - Query execution foundation and vectorized processing
  - TableScan operator with column pruning and batch sizing
  - Filter operator with predicate system (BinaryComparison, AND, OR)
  - Project operator with column selection and aliasing
  - Aggregate functions (Count, Sum, Min, Max, Avg) with stateful design
  - GroupBy operator with hash-based grouping and GroupKey
  - Operator integration testing patterns for chaining
  - Advanced topics (predicate/projection pushdown, vectorization, streaming)
  - Best practices and design patterns
  - Learning outcomes and self-assessment questions
  - Practical exercises (Limit, Distinct, Variance, Streaming GroupBy)
  - Appendices (code summary, benchmarks, common errors, glossary)

✅ **Phase 4 Assessment** (`docs/phase4-assessment.md`) - 1,220 lines
  - 75 multiple-choice questions across 8 parts covering all Phase 4 topics
  - Complete answer key with explanations for each question
  - Scoring guide (70% passing threshold)
  - Performance breakdown by topic with mastery levels
  - Self-reflection questions for understanding evaluation
  - Preparation checklist for Phase 5 (SQL Parser)
  - Study tips for before/during/after assessment

#### Phase 5: SQL Parser

✅ **Phase 5 Learning Guide** (`docs/phase5-learning-guide.md`) - 2,170 lines covering:
  - Introduction to SQL parsing and its role in database systems
  - Tokenizer/Lexer design with 20+ token types
  - Abstract Syntax Tree (AST) design principles
  - Recursive descent parsing methodology
  - Expression parsing with operator precedence handling
  - SQL clause parsing (SELECT, FROM, WHERE, GROUP BY)
  - Aggregate function parsing (COUNT, SUM, AVG, MIN, MAX)
  - Error handling strategies with descriptive messages
  - Testing strategies for parsers
  - Best practices and design patterns
  - Learning outcomes and self-assessment questions
  - 15 practical exercises (beginner, intermediate, advanced)

✅ **Phase 5 Assessment** (`docs/phase5-assessment.md`) - 785 lines
  - 67 multiple-choice questions across 9 parts
  - Parts: Tokenizer/Lexer Fundamentals, AST Design, Recursive Descent Parsing, Expression Parsing, SQL Clauses, Aggregate Functions, Error Handling, Testing Strategies, Advanced Topics
  - Complete answer key with detailed explanations
  - Scoring guide with readiness indicators for Phase 6
  - Self-reflection questions and preparation checklist

#### Phase 6: Query Planning & Advanced Features

✅ **Phase 6.2 Learning Guide** (`phase6_2-learning-guide.md`) - 1,108 lines covering:
  - ORDER BY clause: Single and multi-column sorting with ASC/DESC directions
  - LIMIT clause: Row restriction for performance and pagination
  - OFFSET clause: Row skipping for pagination
  - Combined usage: ORDER BY + LIMIT + OFFSET patterns
  - Parser implementation: New token types, AST changes, parsing logic
  - Execution engine: Sort and Limit operators with implementation details
  - Query planning: Operator ordering, column index mapping
  - Code examples: 7 detailed examples with execution plans
  - Best practices: Performance considerations and common patterns
  - Known limitations: Current constraints and future enhancements
  - 10 main sections covering all aspects of Phase 6.2

✅ **Phase 6.2 Assessment** (`phase6_2-assessment.md`) - 533 lines
  - 9-part comprehensive assessment covering all Phase 6.2 concepts
  - Multiple choice and true/false questions (30 points)
  - Short answer questions (30 points)
  - Code analysis exercises (20 points)
  - Implementation challenge for NULLS FIRST/LAST (15 points)
  - Debugging challenge (15 points)
  - Critical thinking scenarios (20 points)
  - Code writing challenge (20 points)
  - Advanced topics (20 points)
  - Bonus questions (20 points)
  - Total: 170 points (passing: 70%)

#### Phase 7: Interactive REPL

✅ **Phase 7 Learning Guide** (`docs/phase7-learning-guide.md`) - 462 lines covering:
  - REPL Overview and Learning Objectives
  - Rust Concepts: rustyline crate, error handling, command pattern
  - Database Concepts: end-to-end query processing, catalog management
  - Implementation Walkthrough: REPL structure, command processing, query execution
  - Testing Strategies: manual testing with shell scripts, integration testing
  - Common Challenges & Solutions: column names, error conversion, borrowing issues
  - Code Organization: file structure, adding new commands
  - Key Takeaways: UX importance, incremental development
  - Further Improvements: short-term, medium-term, long-term features
  - Completion Checklist

✅ **Phase 7 Assessment** (`docs/phase7-assessment.md`) - 620 lines
  - Comprehensive assessment across 5 parts
  - Part 1: Knowledge Questions (25 points, 15 questions)
  - Part 2: Practical Tasks (35 points, 5 tasks)
  - Part 3: Code Review (20 points, 3 reviews)
  - Part 4: Challenge Exercises (20 points each)
  - Part 5: Integration Verification (optional extra credit)
  - Complete answer keys and suggested improvements
  - Self-check checklist with tips for success
  - Time estimates for each section

#### Phase 8: Additional Tasks & Quality Improvements

✅ **Phase 8 Learning Guide** (`docs/phase8-learning-guide.md`) - 2,371 lines covering 12 sections:
  - Overview and Learning Objectives for Phase 8
  - API Documentation (cargo doc, rustdoc, documentation comments, doc tests)
  - Test Strategy (unit tests, integration tests, test pyramid, AAA pattern)
  - Performance & Memory Optimization (profiling tools, columnar storage, string handling, SIMD, compression)
  - Property-Based Testing (proptest framework, strategies, properties, shrinking)
  - Performance Benchmarks (Criterion framework, benchmarking best practices)
  - Code Coverage (measurement tools, goals, improvement strategies)
  - Best Practices (documentation, testing, performance, benchmarking)
  - Key Concepts (comprehensive terminology and definitions)
  - Practical Examples (detailed code examples for all topics)
  - Resources & Further Reading (links to external documentation)
  - Summary and key takeaways

✅ **Phase 8 Assessment** (`docs/phase8-assessment.md`) - 447 lines
  - 45 questions across 5 sections
  - Section 1: API Documentation (10 questions)
  - Section 2: Test Strategy (10 questions)
  - Section 3: Performance & Memory Optimization (10 questions)
  - Section 4: Property-Based Testing (8 questions)
  - Section 5: Performance Benchmarks (7 questions)
  - Complete answer key with explanations
  - Scoring guide with performance levels (80% passing)
  - Time estimates for completion (60-90 minutes)
  - Difficulty assessment (Intermediate)

#### General Documentation

✅ **CI/CD Pipeline** (`.githooks/`)
- Pre-commit hook with comprehensive checks (formatting, linting, tests, documentation)
- Pre-push hook with extended validation
- Automated quality assurance pipeline
- 998 lines of automation code

✅ **Code Review Workflow** (`docs/code-review-workflow.md`) - 2,590 lines
  - Complete guide to Pull Requests and code reviews
  - Git workflow best practices
  - Communication guidelines for code reviews

✅ **Code Review Assessment** (`docs/code-review-assessment.md`) - 234 lines


### 📝 Code Review Assessment

**Code Review Assessment** (`docs/code-review-assessment.md`) - 234 lines
- 15 multiple-choice questions covering Git workflow, Pull Requests, and code review best practices
- Tests understanding of version control, collaborative development, and review processes
- Detailed answer key with explanations for each question
- Scoring guide to evaluate your code review knowledge
- Study recommendations based on your score

> 💡 **Assessment Tip**: Practice with Git and GitHub to reinforce your understanding of the code review workflow before reviewing Phase 2 pull requests.

---

## 📚 Learning Path

### For Rust Beginners

If you're new to Rust, this project teaches:

1. **Ownership & Borrowing**: Understanding memory management
2. **Traits**: Defining shared behavior across types
3. **Enums & Pattern Matching**: Handling different value types
4. **Error Handling**: Using `Result` and `thiserror`
5. **Generics**: Writing reusable, type-safe code

### For Database Learners

This project demonstrates:

1. **Columnar Storage**: How analytical databases organize data
2. **Vectorized Execution**: Processing data in batches
3. **Query Operators**: Physical execution of queries
4. **Type Systems**: Ensuring data integrity in databases
5. **Aggregation**: How GROUP BY and aggregations work

> 💡 **Pro Tip**: Read the [Phase 1 Learning Guide](docs/phase1-learning-guide.md) for a comprehensive walkthrough of database concepts as implemented in this project.

### Suggested Reading Order

1. `src/error.rs` - Error handling patterns
2. `src/types.rs` - Core data type design
3. `src/column.rs` - Columnar storage implementation
4. `tests/manual_query.rs` - Manual query operations
5. `src/table.rs` - Table structure and schema management
6. `src/catalog.rs` - Metadata repository for table management
7. *(Future)* `src/execution.rs` - Query execution engine

---

## 📊 Development Status

### Phase Progress

| Phase | Description | Status | Tests |
|-------|-------------|--------|-------|
| 1 | Foundation (Types, Columns) | ✅ Complete | 69 |
| 2 | Storage Layer (Table, Catalog) | ✅ Complete | 58 |
| 3 | CSV Ingestion | ✅ Complete | 38 |
| 4 | Query Operators | ✅ Complete | 145 |
| 5 | SQL Parser | ✅ Complete | 19 |
| 6.1 | Query Planning | ✅ Complete | 10 |
| 6.2 | ORDER BY, LIMIT, OFFSET | ✅ Complete | 8 |
| 7 | REPL Interface | ✅ Complete | 165 |
| 8 | Additional Tasks & Quality Improvements | ✅ Complete | 20 |
| - | Project Reorganization | ✅ Complete | - |

**Total Tests**: 446 (361 library tests + 15 integration tests + 16 manual tests + 34 active doc tests [17 ignored] + 20 property-based tests)
**Examples**: 2 working examples with comprehensive documentation
**Benchmarks**: 7 benchmark categories using criterion


### Module Status

- ✅ `error` - Error handling complete (10 tests)
- ✅ `types` - Core types complete with SortDirection (26 tests)
- ✅ `column` - Column implementations complete (33 tests)
- ✅ `table` - Table structure complete (33 tests)
- ✅ `catalog` - Metadata management complete (25 tests)
- ✅ `ingest` - CSV ingestion complete (38 tests)
- ✅ `parser` - SQL parsing complete with ORDER BY/LIMIT/OFFSET (19 tests)
- ✅ `execution` - Query execution engine with Sort/Limit operators (77 tests)
- ✅ `planner` - Query planning with column pruning (10 tests)
- ✅ `aggregates` - Aggregate functions (65 tests)
- ✅ `lib` - Library exports and integration (3 tests)

---

## 🔬 Project Structure

```
mini_rust_olap/
├── src/                     # Core library implementation
│   ├── main.rs              # Entry point (REPL)
│   ├── lib.rs               # Library exports and integration tests
│   ├── error.rs             # Error types (complete)
│   ├── types.rs             # Data types (complete)
│   ├── column.rs            # Column implementations (complete)
│   ├── table.rs             # Table structure (complete)
│   ├── catalog.rs           # Metadata management (complete)
│   ├── ingest.rs            # CSV ingestion (complete)
│   ├── parser.rs            # SQL parser (complete)
│   ├── execution.rs         # Query execution and operators (complete)
│   └── aggregates.rs        # Aggregate functions (complete)
├── examples/                # Example programs demonstrating library usage
│   ├── simple_table.rs      # Programmatic table creation and SQL queries
│   ├── csv_loading.rs       # CSV loading and various SQL operations
│   └── README.md            # Guide for running and understanding examples
├── benches/                 # Performance benchmarks using criterion
│   ├── query_benchmark.rs   # Comprehensive benchmark suite
│   └── README.md            # Guide for running benchmarks
├── tests/                   # Integration tests
│   ├── data/                # Test data files
│   │   └── test_data.csv    # Sample data for testing
│   ├── manual_query.rs      # Manual query integration tests
│   └── integration_tests.rs # Operator chain integration tests
├── scripts/                 # Shell scripts for testing and automation
│   ├── test_repl.sh         # Comprehensive REPL testing
│   ├── test_repl_simple.sh  # Basic REPL testing
│   └── final_test.sh        # Final integration testing
├── docs/                    # Learning guides and assessments
│   ├── phase{1-7}-learning-guide.md
│   ├── phase{1-7}-assessment.md
│   ├── ci-pipeline-setup.md
│   ├── ci-pipeline-assessment.md
│   ├── code-review-workflow.md
│   ├── code-review-assessment.md
│   └── references/          # Reference documentation
│       ├── prd.md           # Project Requirements Document
│       ├── progress.md      # Development tracking
│       └── REORGANIZATION_SUMMARY.md
├── .githooks/               # Git hooks for CI/CD
├── Cargo.toml               # Dependencies
├── Cargo.lock               # Dependency lock file
├── README.md                # This file ✅
└── .gitignore               # Git ignore rules
```

---

## 🎯 Design Principles

### 1. Simplicity Over Complexity
- Prioritize understandability over performance optimizations
- Avoid premature optimization
- Keep code paths straightforward

### 2. Type Safety
- Leverage Rust's type system
- No `unsafe` code unless absolutely necessary
- Compile-time guarantees where possible

### 3. Comprehensive Documentation
- Public APIs must have documentation comments
- Explain "why", not just "what"
- Include examples in doc comments
- Maintain detailed learning guides and assessments

### 4. Test-Driven Development
- Write tests before implementation
- Maintain high test coverage
- Tests serve as usage examples

### 5. Educational Value
- Code should be readable by intermediate developers
- Include comments explaining database concepts
- Compare to industry practices

---

## 🛠️ Development

### Setting Up Development Environment

```bash
# Clone and setup
git clone https://github.com/yourusername/mini_rust_olap.git
cd mini_rust_olap

# Install development tools
rustup update
cargo install cargo-tarpaulin  # For code coverage

# Setup git hooks (optional)
chmod +x scripts/setup-ci-hooks.sh
./scripts/setup-ci-hooks.sh

# Build and test
cargo build
cargo test
cargo bench
```
### Development Workflow

```bash
# Watch for changes and run tests
cargo watch -x test

# Check for linting issues
cargo clippy

# Format code
cargo fmt

# Build documentation
cargo doc --open
```

### Adding New Features

1. Update `docs/references/progress.md` to track the feature
2. Write tests first (TDD approach)
3. Implement the feature
4. Add documentation comments
5. Run `cargo test` to verify
6. Run `cargo clippy` to check for warnings
7. Update this README if applicable

---

## ⚡ Quick Reference

### Essential Commands

```bash
# Build project
cargo build

# Run tests
cargo test

# Run REPL
cargo run --release

# Run examples
cargo run --example simple_table
cargo run --example csv_loading

# Run benchmarks
cargo bench

# Run test scripts
./scripts/test_repl.sh
./scripts/test_repl_simple.sh
./scripts/final_test.sh
```

### Key File Locations

| Item | Location |
|-------|----------|
| Examples | `examples/` |
| Benchmarks | `benches/` |
| Test Data | `tests/data/` |
| Test Scripts | `scripts/` |
| Reference Docs | `docs/references/` |
| Main Docs | `docs/` |
| Source Code | `src/` |

### Loading CSV Data in REPL

```bash
# Load test data
LOAD tests/data/test_data.csv AS employees

# Query the loaded data
SELECT * FROM employees
SELECT COUNT(*) FROM employees
```

---

## 📖 Code Examples

### Example 1: Creating a Simple Table

```rust
use mini_rust_olap::{
    Column, create_column, DataType, Value
};

fn main() -> mini_rust_olap::Result<()> {
    // Create columns dynamically based on data type
    let mut ids = create_column(DataType::Int64);
    let mut names = create_column(DataType::String);
    let mut salaries = create_column(DataType::Int64);

    // Insert data
    ids.push_value(Value::Int64(1))?;
    names.push_value(Value::String("Alice".to_string()))?;
    salaries.push_value(Value::Int64(50000))?;

    ids.push_value(Value::Int64(2))?;
    names.push_value(Value::String("Bob".to_string()))?;
    salaries.push_value(Value::Int64(60000))?;

    Ok(())
}
```

### Example 2: Type Conversion

```rust
use mini_rust_olap::{Value, DataType};

fn main() -> mini_rust_olap::Result<()> {
    let int_value = Value::Int64(42);
    
    // Cast to float
    let float_value = int_value.cast_to(DataType::Float64)?;
    assert_eq!(float_value, Value::Float64(42.0));

    // Parse from string
    let parsed_value: Value = "123.45".parse()?;
    assert_eq!(parsed_value, Value::Float64(123.45));

    Ok(())
}
```

### Example 3: Manual GROUP BY

```rust
use mini_rust_olap::{Column, IntColumn, StringColumn, Value};
use std::collections::HashMap;

fn main() -> mini_rust_olap::Result<()> {
    let mut departments = StringColumn::new();
    let mut salaries = IntColumn::new();

    // Insert data
    departments.push_value(Value::String("Engineering".to_string()))?;
    salaries.push_value(Value::Int64(100000))?;

    departments.push_value(Value::String("Sales".to_string()))?;
    salaries.push_value(Value::Int64(50000))?;

    departments.push_value(Value::String("Engineering".to_string()))?;
    salaries.push_value(Value::Int64(120000))?;

    // Group by department and sum salaries
    let mut dept_totals: HashMap<String, i64> = HashMap::new();
    
    for i in 0..departments.len() {
        let dept = departments.get(i)?;
        let salary = salaries.get(i)?;
        
        if let (Value::String(d), Value::Int64(s)) = (dept, salary) {
            *dept_totals.entry(d).or_insert(0) += s;
        }
    }

    println!("Department salaries: {:?}", dept_totals);

    Ok(())
}
```

### Example 4: SQL Parsing

```rust
use mini_rust_olap::parser::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a parser instance
    let sql = "SELECT id, name, age FROM users WHERE age > 25";
    let mut parser = Parser::new(sql);
    
    // Parse the SQL statement
    let query = parser.parse()?;
    
    // Access the parsed query structure
    if let mini_rust_olap::parser::Query::Select(stmt) = query {
        println!("Selected columns: {:?}", stmt.select_items);
        println!("From table: {}", stmt.from_table);
        
        if let Some(where_expr) = stmt.where_clause {
            println!("Where clause: {:?}", where_expr);
        }
        
        if let Some(group_by) = stmt.group_by {
            println!("Group by: {:?}", group_by);
        }
    }
    
    Ok(())
}
```

---

## 🤝 Contributing

This is primarily an educational project, but contributions are welcome! Areas where help is appreciated:

1. **Documentation**: Improving explanations and examples
2. **Tests**: Adding more test cases for edge conditions
3. **Examples**: Creating usage examples in the `examples/` directory
4. **Performance**: Non-breaking optimizations with explanations
5. **Bug Reports**: Found an issue? Please open an issue with details

### Contribution Guidelines

1. Keep code readable and well-commented
2. Add tests for new functionality
3. Update documentation
4. Follow existing code style
5. Ensure all tests pass before submitting

---

## 📋 Future Enhancements

### Post-MVP Ideas

- **Predicate Pushdown**: Move filters closer to data source
- **Index Support**: B-tree or Bloom filter indexes
- **Parquet Format**: Support for reading/writing Parquet files
- **Multi-threading**: Parallel query execution
- **More SQL Features**: JOIN, ORDER BY, HAVING, LIMIT
- **Query Caching**: Cache query results
- **Web UI**: Browser-based query interface
- **Persistence**: Write-ahead log for durability

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

## 🙏 Acknowledgments

- **ClickHouse**: Inspiration for column-oriented design
- **Apache Arrow**: Influenced by Arrow memory model concepts
- **Rust Community**: Excellent documentation and community support
- **Database Internals Course**: Design patterns from academic databases

---

## 📞 Support & Questions

### Getting Help

- 📖 **Documentation**: Check the inline code documentation
- 📝 **Issues**: Open a GitHub issue for bugs or questions
- 💬 **Discussions**: Use GitHub Discussions for general questions

### Learning Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Database Internals by Alex Petrov](https://www.databass.dev/)
- [ClickHouse Documentation](https://clickhouse.com/docs/en/)
- [Apache Arrow Documentation](https://arrow.apache.org/docs/)

---

## 📊 Project Statistics

- **Lines of Code**: 11,200 (comprehensive implementation)
- **Test Count**: 446 passing tests (high test coverage)
- **Number of Modules**: 10 implemented (error, types, column, table, catalog, ingest, parser, execution, aggregates, lib)
- **Dependencies**: 8 (minimal for learning purposes)
- **Build Time**: ~3 seconds (optimized for fast iteration)

---

## 🎓 Educational Value

This project includes comprehensive educational materials:

### 📚 Documentation & Assessments

- **Phase 1 Learning Guide**: A 2,668-line comprehensive guide covering:
  - 10 chapters on Rust programming and database internals
  - Detailed code examples with explanations
  - Self-assessment questions for each chapter
  - Practical exercises to reinforce learning

- **Phase 1 Assessment**: A 431-line evaluation tool with:
  - 35 multiple-choice questions testing Phase 1 knowledge
  - Detailed answer key with explanations for each answer
  - Scoring rubric to track your progress
  - Study recommendations based on your score

- **CI/CD Pipeline Documentation**: 998 lines of automation setup including:
  - Pre-commit hooks for code quality checks (171 lines)
  - Pre-push hooks for comprehensive validation (373 lines)
  - Setup script for hook installation (454 lines)
  - Ensures code quality through automated testing and formatting

### 🔧 CI/CD for Learning

The project includes a robust CI/CD pipeline that:
- Runs automated checks before commits (formatting, linting, tests)
- Validates documentation quality
- Ensures all tests pass before pushing changes
- Serves as an example of professional development workflows

### 🎯 Core Learning Objectives

This project is designed to help you understand:

1. **How databases store data** - Columnar vs row-oriented
2. **How queries execute** - From SQL to physical operators
3. **How to write idiomatic Rust** - Best practices and patterns
4. **How to design systems** - Trade-offs and architectural decisions

### For Different Learners

- **Students**: See database theory in practice
- **Rust Developers**: Apply Rust to systems programming
- **Data Engineers**: Understand query engines better
- **Curious Minds**: Learn how modern databases work

> 🎓 **Ready to learn?** Start with the [Phase 1 Learning Guide](docs/phase1-learning-guide.md) and test your knowledge with the [Phase 1 Assessment](docs/phase1-assessment.md)!

---

<div align="center">

**Built with ❤️ for learning**

**Mini Rust OLAP - Where databases meet education**

[⬆ Back to Top](#mini-rust-olap---mini-olap-database-engine)

</div>
