//! # Error Handling Module
//!
//! This module defines all error types for the RustyCube OLAP database.
//!
//! ## Design Philosophy
//!
//! Error handling in Rust is a first-class citizen. This module demonstrates:
//! - Using `thiserror` for creating descriptive, strongly-typed errors
//! - Proper error chaining to preserve context
//! - Making errors informative for debugging
//! - Converting between error types with `From` implementations
//!
//! ## Error Categories
//!
//! Errors are organized by subsystem:
//! 1. **Storage Errors**: Column and table operations
//! 2. **Ingestion Errors**: CSV parsing and data loading
//! 3. **Execution Errors**: Query execution problems
//! 4. **Parser Errors**: SQL syntax and parsing issues
//! 5. **Catalog Errors**: Metadata management issues
//!
//! ## Usage Example
//!
//! ```no_run
//! use crate::error::{Result, DatabaseError};
//!
//! fn load_data() -> Result<()> {
//!     // This function returns a Result<T> which is aliased to std::result::Result<T, DatabaseError>
//!     // If any operation fails, we use the `?` operator to propagate the error
//!     Err(DatabaseError::StorageError("Failed to load data".to_string()))
//! }
//! ```

use std::io;
use thiserror::Error;

/// A type alias for `Result<T, DatabaseError>`
///
/// This is a common pattern in Rust to simplify error handling.
/// Instead of writing `std::result::Result<T, DatabaseError>` everywhere,
/// we can simply write `Result<T>`.
///
/// # Example
/// ```rust
/// use crate::error::Result;
///
/// fn get_value() -> Result<i32> {
///     Ok(42)
/// }
/// ```
pub type Result<T> = std::result::Result<T, DatabaseError>;

/// The main error type for RustyCube database operations
///
/// This enum encompasses all possible errors that can occur throughout the database system.
/// Each variant represents a different category of errors, making it easy to match and handle
/// specific error cases.
///
/// # Error Propagation
///
/// When an error occurs deep in the call stack, we use the `?` operator to propagate it up.
/// The error type remains consistent throughout, eliminating the need for constant conversions.
///
/// # Example
/// ```rust
/// use crate::error::DatabaseError;
///
/// fn process() -> Result<()> {
///     // Some operation that might fail
///     Err(DatabaseError::StorageError("Out of memory".to_string()))
/// }
/// ```
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Errors related to column operations (insertion, retrieval, type mismatch)
    ///
    /// This variant captures any errors that occur when working with columns.
    /// Column operations are fundamental to the database, so errors here are critical.
    #[error("Column error: {0}")]
    ColumnError(String),

    /// Errors related to table operations (creation, schema validation, etc.)
    ///
    /// Table-level errors include schema mismatches, duplicate table names,
    /// and operations on non-existent tables.
    #[error("Table error: {0}")]
    TableError(String),

    /// Errors related to metadata and table catalog management
    ///
    /// The catalog manages table metadata. Errors here include:
    /// - Table not found in catalog
    /// - Duplicate registration attempts
    #[error("Catalog error: {0}")]
    CatalogError(String),

    /// Errors during CSV file ingestion and parsing
    ///
    /// This includes I/O errors when reading files, parsing errors for malformed CSVs,
    /// and type inference failures.
    #[error("Ingestion error: {0}")]
    IngestionError(String),

    /// Errors during query execution
    ///
    /// Query execution errors occur when the physical operators fail.
    /// This includes invalid column references, aggregation errors, and more.
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Errors during SQL parsing
    ///
    /// Parser errors occur when the input SQL doesn't match the expected syntax.
    /// This includes syntax errors, unexpected tokens, and invalid query structure.
    #[error("Parser error: {0}")]
    ParserError(String),

    /// Errors related to data types and conversions
    ///
    /// Type errors occur when attempting operations between incompatible types,
    /// or when converting values fails.
    #[error("Type error: {0}")]
    TypeError(String),

    /// Generic I/O errors (file operations, network, etc.)
    ///
    /// This wraps standard I/O errors to include them in our error chain.
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Generic error for uncategorized issues
    ///
    /// Use this sparingly. Prefer creating specific error variants for better error handling.
    #[error("Generic error: {0}")]
    GenericError(String),
}

// ============================================================================
// IMPLEMENTATIONS FOR CREATING SPECIFIC ERRORS
// ============================================================================

impl DatabaseError {
    /// Creates a column error with a descriptive message
    ///
    /// # Arguments
    /// * `msg` - The error message describing what went wrong
    ///
    /// # Returns
    /// A `DatabaseError::ColumnError` variant
    ///
    /// # Example
    /// ```rust
    /// use crate::error::DatabaseError;
    ///
    /// let err = DatabaseError::column_error("Cannot insert String into IntColumn");
    /// ```
    pub fn column_error(msg: impl Into<String>) -> Self {
        Self::ColumnError(msg.into())
    }

    /// Creates a table error with a descriptive message
    ///
    /// # Example
    /// ```rust
    /// use crate::error::DatabaseError;
    ///
    /// let err = DatabaseError::table_error("Table 'users' already exists");
    /// ```
    pub fn table_error(msg: impl Into<String>) -> Self {
        Self::TableError(msg.into())
    }

    /// Creates a catalog error with a descriptive message
    ///
    /// # Example
    /// ```rust
    /// use crate::error::DatabaseError;
    ///
    /// let err = DatabaseError::catalog_error("Table 'users' not found in catalog");
    /// ```
    pub fn catalog_error(msg: impl Into<String>) -> Self {
        Self::CatalogError(msg.into())
    }

    /// Creates an ingestion error with a descriptive message
    ///
    /// # Example
    /// ```rust
    /// use crate::error::DatabaseError;
    ///
    /// let err = DatabaseError::ingestion_error("Failed to parse CSV at line 42");
    /// ```
    pub fn ingestion_error(msg: impl Into<String>) -> Self {
        Self::IngestionError(msg.into())
    }

    /// Creates an execution error with a descriptive message
    ///
    /// # Example
    /// ```rust
    /// use crate::error::DatabaseError;
    ///
    /// let err = DatabaseError::execution_error("Column 'age' not found in table");
    /// ```
    pub fn execution_error(msg: impl Into<String>) -> Self {
        Self::ExecutionError(msg.into())
    }

    /// Creates a parser error with a descriptive message
    ///
    /// # Example
    /// ```rust
    /// use crate::error::DatabaseError;
    ///
    /// let err = DatabaseError::parser_error("Expected 'FROM' keyword at position 10");
    /// ```
    pub fn parser_error(msg: impl Into<String>) -> Self {
        Self::ParserError(msg.into())
    }

    /// Creates a type error with a descriptive message
    ///
    /// # Example
    /// ```rust
    /// use crate::error::DatabaseError;
    ///
    /// let err = DatabaseError::type_error("Cannot add String to Int64");
    /// ```
    pub fn type_error(msg: impl Into<String>) -> Self {
        Self::TypeError(msg.into())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the Result type alias works correctly
    #[test]
    fn test_result_type_alias() {
        // Test Ok variant
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Test Err variant
        let result: Result<i32> = Err(DatabaseError::column_error("test error"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Column error: test error");
    }

    /// Tests error creation using convenience methods
    #[test]
    fn test_error_creation() {
        let col_err = DatabaseError::column_error("Column full");
        assert_eq!(col_err.to_string(), "Column error: Column full");

        let tab_err = DatabaseError::table_error("Table not found");
        assert_eq!(tab_err.to_string(), "Table error: Table not found");

        let cat_err = DatabaseError::catalog_error("Duplicate table");
        assert_eq!(cat_err.to_string(), "Catalog error: Duplicate table");

        let ing_err = DatabaseError::ingestion_error("CSV malformed");
        assert_eq!(ing_err.to_string(), "Ingestion error: CSV malformed");

        let exe_err = DatabaseError::execution_error("Query failed");
        assert_eq!(exe_err.to_string(), "Execution error: Query failed");

        let par_err = DatabaseError::parser_error("Syntax error");
        assert_eq!(par_err.to_string(), "Parser error: Syntax error");

        let typ_err = DatabaseError::type_error("Type mismatch");
        assert_eq!(typ_err.to_string(), "Type error: Type mismatch");
    }

    /// Tests error display formatting
    #[test]
    fn test_error_display() {
        let err = DatabaseError::column_error("Test error message");
        assert_eq!(format!("{}", err), "Column error: Test error message");
    }

    /// Tests error debug formatting
    #[test]
    fn test_error_debug() {
        let err = DatabaseError::table_error("Debug test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("TableError"));
        assert!(debug_str.contains("Debug test"));
    }

    /// Tests that errors can be converted to the generic Error type
    #[test]
    fn test_error_as_dyn_error() {
        let err: Box<dyn std::error::Error> = Box::new(DatabaseError::column_error("test"));
        assert_eq!(err.to_string(), "Column error: test");
    }

    /// Tests I/O error conversion using From trait
    #[test]
    fn test_io_error_conversion() {
        // Create an IO error
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");

        // Convert it to DatabaseError (automatically via From trait)
        let db_err: DatabaseError = io_err.into();

        // Verify it's an IoError variant
        assert!(matches!(db_err, DatabaseError::IoError(_)));
        assert!(db_err.to_string().contains("file not found"));
    }

    /// Tests error propagation with the ? operator
    #[test]
    fn test_error_propagation() {
        fn inner_function() -> Result<()> {
            Err(DatabaseError::column_error("inner error"))
        }

        fn outer_function() -> Result<()> {
            // The ? operator propagates the error
            inner_function()?;
            Ok(())
        }

        let result = outer_function();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Column error: inner error");
    }

    /// Tests that we can match on error variants
    #[test]
    fn test_error_matching() {
        let err = DatabaseError::column_error("test");

        match err {
            DatabaseError::ColumnError(msg) => assert_eq!(msg, "test"),
            _ => panic!("Should have matched ColumnError"),
        }
    }

    /// Tests string type conversion
    #[test]
    fn test_string_conversion() {
        // Test that we can pass &str to error functions
        let err1 = DatabaseError::column_error("string literal");
        assert_eq!(err1.to_string(), "Column error: string literal");

        // Test that we can pass String to error functions
        let msg = String::from("owned string");
        let err2 = DatabaseError::column_error(msg);
        assert_eq!(err2.to_string(), "Column error: owned string");

        // Test that we can pass anything that implements Into<String>
        let owned: String = "convertible".to_string();
        let err3 = DatabaseError::column_error(owned);
        assert_eq!(err3.to_string(), "Column error: convertible");
    }

    /// Tests multiple error propagation levels
    #[test]
    fn test_deep_error_propagation() {
        fn level_3() -> Result<i32> {
            Err(DatabaseError::execution_error("level 3 failed"))
        }

        fn level_2() -> Result<i32> {
            Ok(level_3()?) // Propagate from level 3
        }

        fn level_1() -> Result<i32> {
            Ok(level_2()?) // Propagate from level 2
        }

        let result = level_1();
        assert!(result.is_err());
        // The error message should be preserved through all levels
        assert!(result.unwrap_err().to_string().contains("level 3 failed"));
    }
}
