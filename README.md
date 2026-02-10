# Mini Rust OLAP - Mini OLAP Database Engine

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-341%20passing-green.svg)]()
[![Phase](https://img.shields.io/badge/phase-4%20complete-success.svg)]()

**A lightweight, in-memory OLAP database engine built with Rust for educational purposes**

[Features](#-features) • [Architecture](#%EF%B8%8F-architecture) • [Quick Start](#-quick-start) • [Learning Path](#-learning-path)

</div>

---

## 📖 About

**Mini Rust OLAP** is a miniature Online Analytical Processing (OLAP) database engine designed specifically for educational purposes. It demonstrates core database internals concepts through clean, well-documented Rust code.

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

### Current Implementation (Phase 5 - Complete ✅)

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

### Planned Features (Roadmap)

- [x] Phase 4: Physical query operators (Scan, Filter, Project, Aggregate) ✅
- [x] Phase 5: SQL parser for SELECT statements ✅
- [ ] Phase 6: Query planning and optimization
- [ ] Phase 7: Interactive REPL (Read-Eval-Print Loop)

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

# (Optional) Run with debug logging
RUST_LOG=debug cargo run
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

### Manual Filtering Example

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

---

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
```

### Test Coverage

```bash
# Install tarpaulin for coverage reports
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

### Current Test Status

- **Total Tests**: 329 passing ✅ (298 library tests + 31 integration tests)
- **Library Tests**: 298 (error: 10, types: 26, column: 33, table: 33, catalog: 25, ingest: 38, execution: 77, aggregates: 65, parser: 19, lib: 3)
- **Integration Tests**: 31 (operator chaining: 16, manual query: 15)
- **Code Coverage**: High test coverage across all implemented phases (Foundation, Storage Layer, CSV Ingestion, Query Operators, SQL Parser)

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
| 6 | Query Planning | ❌ Not Started | - |
| 7 | REPL Interface | ❌ Not Started | - |

**Total Tests**: 329 (329 passing ✅, including 31 integration tests)


### Module Status

- ✅ `error` - Error handling complete (10 tests)
- ✅ `types` - Core types complete (26 tests)
- ✅ `column` - Column implementations complete (33 tests)
- ✅ `table` - Table structure complete (33 tests)
- ✅ `catalog` - Metadata management complete (25 tests)
- ✅ `ingest` - CSV ingestion complete (38 tests)
- ✅ `parser` - SQL parsing complete (19 tests)
- ✅ `execution` - Query execution engine (77 tests)
- ✅ `aggregates` - Aggregate functions (65 tests)
- ✅ `lib` - Library exports and integration (3 tests)

---

## 🔬 Project Structure

```
mini_rust_olap/
├── src/
│   ├── main.rs              # Entry point (REPL - future)
│   ├── lib.rs               # Library exports
│   ├── error.rs             # Error types (complete)
│   ├── types.rs             # Data types (complete)
│   ├── column.rs            # Column implementations (complete)
│   ├── table.rs             # Table structure (complete)
│   ├── catalog.rs           # Metadata management (complete)
│   ├── ingest.rs            # CSV ingestion
│   ├── parser.rs            # SQL parser
│   ├── execution.rs         # Query execution
│   ├── operators.rs         # Physical operators
│   └── aggregates.rs        # Aggregate functions
├── tests/
│   └── manual_query.rs      # Integration tests
├── Cargo.toml               # Dependencies
├── README.md                # This file ✅
└── progress.md              # Development tracking
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
# Install Rust toolchain
rustup install stable
rustup default stable

# Install development tools
cargo install cargo-watch    # Auto-reload on file changes
cargo install cargo-edit     # Easy dependency management
cargo install cargo-tarpaulin # Coverage reports

# Enable pre-commit hooks (optional)
cargo install cargo-husky
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

1. Update `progress.md` to track the feature
2. Write tests first (TDD approach)
3. Implement the feature
4. Add documentation comments
5. Run `cargo test` to verify
6. Run `cargo clippy` to check for warnings
7. Update this README if applicable

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

- **Lines of Code**: 9,723 (comprehensive implementation)
- **Test Count**: 341 passing tests (high test coverage)
- **Number of Modules**: 8 implemented (error, catalog, column, ingest, table, types, aggregates, execution), with additional modules planned for future phases
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

- **Phase 1 Assessment**: A 432-line evaluation tool with:
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

> 🎓 **Ready to learn?** Start with the [Phase 1 Learning Guide](docs/phase1-learning-guide.md) and test your knowledge with the [Phase 1 Assessment](docs/code-review-assessment.md)!

---

<div align="center">

**Built with ❤️ for learning**

**Mini Rust OLAP - Where databases meet education**

[⬆ Back to Top](#mini-rust-olap---mini-olap-database-engine)

</div>
