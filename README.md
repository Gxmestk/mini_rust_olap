# Mini Rust OLAP - Mini OLAP Database Engine

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-127%20passing-green.svg)]()
[![Phase](https://img.shields.io/badge/phase-1%20complete-success.svg)]()

**A lightweight, in-memory OLAP database engine built with Rust for educational purposes**

[Features](#-features) • [Architecture](#-architecture) • [Quick Start](#-quick-start) • [Learning](#-learning)

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

### Current Implementation (Phase 1 - Complete ✅)

#### 🏗️ Core Foundation
- **Error Handling**: Comprehensive error types using `thiserror`
- **Data Types**: Support for `Int64`, `Float64`, and `String` with type safety
- **Columnar Storage**: Efficient column-oriented data layout

#### 📊 Column Types
- **IntColumn**: 64-bit integer storage in `Vec<i64>`
- **FloatColumn**: 64-bit floating point storage in `Vec<f64>`
- **StringColumn**: UTF-8 string storage in `Vec<String>`

#### 🔍 Manual Query Operations
- **Aggregations**: SUM, AVG, COUNT, MIN, MAX
- **Filtering**: WHERE clause with AND/OR logic
- **Projection**: SELECT specific columns
- **Grouping**: GROUP BY with aggregation

### Planned Features (Roadmap)

- [ ] Phase 2: Table & Catalog management
- [ ] Phase 3: CSV data ingestion with type inference
- [ ] Phase 4: Physical query operators (Scan, Filter, Project, Aggregate)
- [ ] Phase 5: SQL parser for SELECT statements
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

- **Total Tests**: 127 (all passing ✅)
- **Unit Tests**: 72
- **Integration Tests**: 15
- **Manual Query Tests**: 40
- **Code Coverage**: ~20% (growing with each phase)

### Phase 1 Deliverables

✅ **Comprehensive Documentation**
- **Phase 1 Learning Guide** (`docs/phase1-learning-guide.md`) - 2,668 lines covering 10 chapters
  - Rust programming fundamentals and best practices
  - Database internals and column-oriented storage
  - Code examples with detailed explanations
  - Self-assessment questions and exercises
  
✅ **Phase 1 Assessment** (`docs/code-review-assessment.md`) - 432 lines
  - 35 multiple-choice questions covering Phase 1 concepts
  - Comprehensive answer key with detailed explanations
  - Scoring guide and study recommendations
  - Tests understanding of Rust patterns and database concepts

✅ **CI/CD Pipeline** (`.githooks/`)
- Pre-commit hook with comprehensive checks (formatting, linting, tests, documentation)
- Pre-push hook with extended validation
- Automated quality assurance pipeline
- 998 lines of automation code


### 📝 Phase 1 Code Review Assessment

Comprehensive evaluation materials are available to assess your understanding of Phase 1 concepts:

**Code Review Assessment** (`docs/code-review-assessment.md`) - 432 lines
- 35 multiple-choice questions covering Rust patterns and database concepts
- Tests understanding of traits, error handling, generics, and columnar storage
- Detailed answer key with explanations for each question
- Scoring guide to track your progress
- Study recommendations based on your score

> 💡 **Assessment Tip**: After completing Phase 1, take the assessment to identify areas for improvement before moving to Phase 2.

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
5. *(Future)* `src/table.rs` - Table structure
6. *(Future)* `src/execution.rs` - Query execution engine

---

## 📊 Development Status

### Phase Progress

| Phase | Description | Status | Tests |
|-------|-------------|--------|-------|
| 1 | Foundation (Types, Columns) | ✅ Complete | 127 |
| 2 | Storage Layer (Table, Catalog) | 🚀 Starting | - |
| 3 | CSV Ingestion | ❌ Not Started | - |
| 4 | Query Operators | ❌ Not Started | - |
| 5 | SQL Parser | ❌ Not Started | - |
| 6 | Query Planning | ❌ Not Started | - |
| 7 | REPL Interface | ❌ Not Started | - |

### Module Status

- ✅ `error` - Error handling complete (11 tests)
- ✅ `types` - Core types complete (26 tests)
- ✅ `column` - Column implementations complete (33 tests)
- ❌ `table` - Table structure (next phase)
- ❌ `catalog` - Metadata management (next phase)
- ❌ `ingest` - CSV ingestion
- ❌ `parser` - SQL parsing
- ❌ `execution` - Query execution
- ❌ `operators` - Physical operators
- ❌ `aggregates` - Aggregate functions

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
│   ├── table.rs             # Table structure (next)
│   ├── catalog.rs           # Metadata management (next)
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

- **Lines of Code**: ~2000
- **Test Count**: 127 (and growing!)
- **Number of Modules**: 3 implemented, 6 planned
- **Dependencies**: 8 (minimal for learning purposes)
- **Build Time**: ~2 seconds (optimized for fast iteration)

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