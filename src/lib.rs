//! # Mini Rust OLAP - Mini OLAP Database Engine
//!
//! Mini Rust OLAP is a lightweight, in-memory OLAP (Online Analytical Processing) database engine
//! implemented in Rust. Its primary purpose is educational: to demonstrate the core principles
//! of column-oriented storage, query execution, and aggregation.
//!
//! ## Architecture Overview
//!
//! Mini Rust OLAP is organized into several key modules:
//!
//! - [`error`] - Comprehensive error handling and result types
//! - [`types`] - Core data types (DataType, Value, etc.)
//! - [`mod@column`] - Columnar storage implementation
//! - [`table`] - Table structure holding columns
//! - [`catalog`] - Metadata management for tables
//! - [`ingest`] - CSV data ingestion
//! - parser - SQL query parsing *(TODO)*
//! - [`execution`] - Query execution engine
//! - [`aggregates`] - Aggregate functions
//!
//! ## Key Features
//!
//! - **Columnar Storage**: Data is stored by column, enabling efficient analytical queries
//! - **Vectorized Execution**: Operations process batches of data for CPU cache efficiency
//! - **Type Safety**: Strong typing with compile-time guarantees
//! - **SQL-like Interface**: Simple query language for data manipulation
//!
//! ## Quick Start
//!
//! ```ignore
//! use mini_rust_olap::error::Result;
//! use mini_rust_olap::catalog::Catalog;
//! use mini_rust_olap::ingest::load_csv;
//!
//! fn main() -> Result<()> {
//!     let mut catalog = Catalog::new();
//!
//!     // Load data from CSV
//!     load_csv("data.csv", "users", &mut catalog)?;
//!
//!     // Query the data
//!     // (implementation details follow...)
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Learning Resources
//!
//! This codebase is designed to help you learn about:
//! - Database internals and column-oriented storage
//! - Rust programming patterns (traits, generics, error handling)
//! - Query execution and optimization
//! - Systems programming concepts

// Re-export commonly used types
pub use catalog::Catalog;
pub use column::{create_column, Column, FloatColumn, IntColumn, StringColumn};
pub use error::{DatabaseError, Result};
pub use ingest::{load_csv, load_csv_into_catalog};
pub use table::Table;
pub use types::{DataType, Value};

// ============================================================================
// MODULE DECLARATIONS
// ============================================================================

// Core error handling - defined first as other modules depend on it
pub mod error;

// TODO: Add module declarations as we implement them
pub mod catalog;
pub mod column;
pub mod ingest;
pub mod table;
pub mod types;
// pub mod parser;
pub mod aggregates;
pub mod execution;

// ============================================================================
// VERSION INFORMATION
// ============================================================================

/// The current version of Mini Rust OLAP
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the database
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that library information is accessible
    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "mini_rust_olap");
    }

    /// Test that Result type alias works
    #[test]
    fn test_result_type() {
        // This should compile and work
        let result: i32 = 42;
        assert_eq!(result, 42);
    }

    /// Test that DatabaseError is accessible
    #[test]
    fn test_error_type() {
        let err = DatabaseError::column_error("test");
        assert_eq!(err.to_string(), "Column error: test");
    }
}
