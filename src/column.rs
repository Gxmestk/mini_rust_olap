//! # Column Module
//!
//! This module defines the columnar storage abstraction for RustyCube.
//!
//! ## Why Columnar Storage?
//!
//! In traditional row-oriented databases (OLTP), data is stored like this:
//! ```text
//! Row 1: [id: 1, name: "Alice", age: 25]
//! Row 2: [id: 2, name: "Bob",   age: 30]
//! ```
//!
//! In column-oriented databases (OLAP), data is stored like this:
//! ```text
//! id column:   [1, 2, 3, ...]
//! name column: ["Alice", "Bob", "Charlie", ...]
//! age column:  [25, 30, 35, ...]
//! ```
//!
//! ## Benefits of Columnar Storage
//!
//! 1. **Compression**: Similar values in a column can be compressed more efficiently
//! 2. **Cache Efficiency**: Analytical queries often read only specific columns
//! 3. **Vectorized Execution**: Process entire vectors of data at once
//! 4. **I/O Reduction**: Read only what you need from disk
//!
//! ## Architecture
//!
//! The `Column` trait defines a common interface for all column types.
//! Concrete implementations (`IntColumn`, `FloatColumn`, `StringColumn`)
//! store data in typed `Vec` structures for optimal performance.
//!
//! ## Usage Example
//!
//! ```no_run
//! use crate::column::{Column, IntColumn, FloatColumn, StringColumn};
//! use crate::types::{DataType, Value};
//!
//! // Create columns
//! let mut ids = IntColumn::new();
//! let mut ages = FloatColumn::new();
//! let mut names = StringColumn::new();
//!
//! // Insert data
//! ids.push_value(Value::Int64(1))?;
//! ages.push_value(Value::Float64(25.0))?;
//! names.push_value(Value::String("Alice".to_string()))?;
//!
//! // Retrieve data (returns owned Value)
//! let first_id = ids.get(0)?;
//! assert_eq!(first_id, Value::Int64(1));
//! ```

use crate::error::{DatabaseError, Result};
use crate::types::{DataType, Value};

// ============================================================================
// COLUMN TRAIT
// ============================================================================

/// A trait that defines the interface for columnar storage
///
/// This trait provides a common abstraction for all column types in RustyCube.
/// Each column type implements this trait to provide type-safe storage and
/// retrieval of data.
///
/// # Why Use a Trait?
///
/// Using a trait allows us to:
/// 1. **Polymorphism**: Work with any column type through the same interface
/// 2. **Type Safety**: Ensure operations respect the column's data type
/// 3. **Extensibility**: Add new column types without changing query logic
///
/// # Design Decisions
///
/// - **Owned Values**: Methods return `Value` instead of `&Value` for simplicity
/// - **Type Safety**: Push operations check type compatibility at runtime
/// - **Zero-Cost Abstraction**: Static dispatch, no runtime overhead
///
/// # Example
///
/// ```rust
/// use crate::column::{Column, IntColumn};
/// use crate::types::{DataType, Value};
///
/// let mut col = IntColumn::new();
/// col.push_value(Value::Int64(42))?;
/// assert_eq!(col.len(), 1);
/// assert_eq!(col.get(0).unwrap(), Value::Int64(42));
/// ```
pub trait Column {
    /// Returns the data type of this column
    ///
    /// This is used for schema validation and query planning.
    ///
    /// # Returns
    /// The `DataType` enum value for this column
    ///
    /// # Example
    /// ```rust
    /// use crate::column::{Column, IntColumn};
    ///
    /// let col = IntColumn::new();
    /// assert_eq!(col.data_type(), DataType::Int64);
    /// ```
    fn data_type(&self) -> DataType;

    /// Returns the number of rows in this column
    ///
    /// This is equivalent to `vec.len()` for the underlying storage.
    ///
    /// # Returns
    /// The number of values stored in the column
    ///
    /// # Example
    /// ```rust
    /// use crate::column::{Column, IntColumn};
    /// use crate::types::Value;
    ///
    /// let mut col = IntColumn::new();
    /// assert_eq!(col.len(), 0);
    /// col.push_value(Value::Int64(1))?;
    /// assert_eq!(col.len(), 1);
    /// ```
    fn len(&self) -> usize;

    /// Returns true if the column is empty
    ///
    /// This is a convenience method equivalent to `self.len() == 0`.
    ///
    /// # Returns
    /// `true` if the column has no rows, `false` otherwise
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Adds a value to the end of the column
    ///
    /// This is similar to `Vec::push()`. The value is type-checked against
    /// the column's data type. If the type doesn't match, an error is returned.
    ///
    /// # Arguments
    /// * `value` - The value to add to the column
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err` if the value type doesn't match
    ///
    /// # Errors
    /// Returns `DatabaseError::TypeError` if the value's type doesn't match
    ///
    /// # Example
    /// ```rust
    /// use crate::column::{Column, IntColumn};
    /// use crate::types::Value;
    ///
    /// let mut col = IntColumn::new();
    /// col.push_value(Value::Int64(42))?;
    /// # Ok::<(), crate::error::DatabaseError>(())
    /// ```
    fn push_value(&mut self, value: Value) -> Result<()>;

    /// Retrieves a value by index
    ///
    /// Returns an owned copy of the value at the specified index.
    /// For integer and float types, this is very cheap (simple copy).
    /// For strings, this involves cloning the string data.
    ///
    /// # Arguments
    /// * `index` - The row index to retrieve (0-based)
    ///
    /// # Returns
    /// An owned `Value` at the specified index
    ///
    /// # Errors
    /// Returns `DatabaseError::ColumnError` if the index is out of bounds
    ///
    /// # Example
    /// ```rust
    /// use crate::column::{Column, IntColumn};
    /// use crate::types::Value;
    ///
    /// let mut col = IntColumn::new();
    /// col.push_value(Value::Int64(42))?;
    /// let value = col.get(0)?;
    /// assert_eq!(value, Value::Int64(42));
    /// # Ok::<(), crate::error::DatabaseError>(())
    /// ```
    fn get(&self, index: usize) -> Result<Value>;

    /// Returns a slice of the column's values
    ///
    /// This is useful for vectorized operations and batch processing.
    /// Returns a vector of owned values in the specified range.
    ///
    /// # Arguments
    /// * `range` - Optional range to slice. If `None`, returns all values.
    ///
    /// # Returns
    /// A vector of `Value` objects in the specified range
    ///
    /// # Example
    /// ```rust
    /// use crate::column::{Column, IntColumn};
    /// use crate::types::Value;
    ///
    /// let mut col = IntColumn::new();
    /// col.push_value(Value::Int64(1))?;
    /// col.push_value(Value::Int64(2))?;
    /// col.push_value(Value::Int64(3))?;
    ///
    /// let values = col.slice(None);
    /// assert_eq!(values.len(), 3);
    /// # Ok::<(), crate::error::DatabaseError>(())
    /// ```
    fn slice(&self, range: Option<std::ops::Range<usize>>) -> Vec<Value>;

    /// Clears all values from the column
    ///
    /// This removes all data but preserves the column's schema/type.
    /// Equivalent to `Vec::clear()`.
    ///
    /// # Example
    /// ```rust
    /// use crate::column::{Column, IntColumn};
    /// use crate::types::Value;
    ///
    /// let mut col = IntColumn::new();
    /// col.push_value(Value::Int64(1))?;
    /// col.push_value(Value::Int64(2))?;
    /// assert_eq!(col.len(), 2);
    /// col.clear();
    /// assert_eq!(col.len(), 0);
    /// # Ok::<(), crate::error::DatabaseError>(())
    /// ```
    fn clear(&mut self);
}

// ============================================================================
// INT COLUMN IMPLEMENTATION
// ============================================================================

/// A column that stores 64-bit integers
///
/// This is the most efficient column type for integer data.
/// Internally uses `Vec<i64>` for storage.
///
/// # Memory Layout
///
/// ```text
/// IntColumn {
///     data: [i64, i64, i64, ...]
/// }
/// ```
/// Each value takes exactly 8 bytes of memory.
///
/// # Example
///
/// ```rust
/// use crate::column::{Column, IntColumn};
/// use crate::types::Value;
///
/// let mut col = IntColumn::new();
/// col.push_value(Value::Int64(42))?;
/// col.push_value(Value::Int64(100))?;
/// assert_eq!(col.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct IntColumn {
    /// The underlying vector storing the integer values
    data: Vec<i64>,
}

impl IntColumn {
    /// Creates a new empty `IntColumn`
    ///
    /// # Returns
    /// A new `IntColumn` with no values
    ///
    /// # Example
    /// ```rust
    /// use crate::column::IntColumn;
    ///
    /// let col = IntColumn::new();
    /// assert_eq!(col.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Creates a new `IntColumn` with pre-allocated capacity
    ///
    /// This is more efficient than `new()` when you know the approximate
    /// number of values you'll be adding, as it reduces reallocations.
    ///
    /// # Arguments
    /// * `capacity` - The number of elements to pre-allocate space for
    ///
    /// # Returns
    /// A new `IntColumn` with the specified capacity
    ///
    /// # Example
    /// ```rust
    /// use crate::column::IntColumn;
    /// use crate::types::Value;
    ///
    /// let mut col = IntColumn::with_capacity(100);
    /// // Adding up to 100 values won't cause reallocation
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Returns a reference to the underlying data vector
    ///
    /// This provides access to the raw `Vec<i64>` for advanced operations.
    /// Be careful when using this method, as it bypasses type safety.
    ///
    /// # Returns
    /// A reference to the `Vec<i64>` containing all values
    ///
    /// # Example
    /// ```rust
    /// use crate::column::IntColumn;
    /// use crate::types::Value;
    ///
    /// let mut col = IntColumn::new();
    /// col.push_value(Value::Int64(42)).unwrap();
    /// let data = col.as_vec();
    /// assert_eq!(data, &[42]);
    /// ```
    pub fn as_vec(&self) -> &[i64] {
        &self.data
    }
}

impl Default for IntColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl Column for IntColumn {
    fn data_type(&self) -> DataType {
        DataType::Int64
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn push_value(&mut self, value: Value) -> Result<()> {
        match value {
            Value::Int64(v) => {
                self.data.push(v);
                Ok(())
            }
            _ => Err(DatabaseError::type_error(format!(
                "Cannot push {:?} into IntColumn",
                value.data_type()
            ))),
        }
    }

    fn get(&self, index: usize) -> Result<Value> {
        self.data
            .get(index)
            .map(|v| Value::Int64(*v))
            .ok_or_else(|| {
                DatabaseError::column_error(format!(
                    "Index {} out of bounds (len: {})",
                    index,
                    self.len()
                ))
            })
    }

    fn slice(&self, range: Option<std::ops::Range<usize>>) -> Vec<Value> {
        let data = &self.data;
        let range = range.unwrap_or(0..data.len());
        data[range].iter().map(|v| Value::Int64(*v)).collect()
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

// ============================================================================
// FLOAT COLUMN IMPLEMENTATION
// ============================================================================

/// A column that stores 64-bit floating point numbers
///
/// Uses `Vec<f64>` for storage. Suitable for decimal numbers, percentages,
/// and any data requiring fractional precision.
///
/// # Memory Layout
///
/// ```text
/// FloatColumn {
///     data: [f64, f64, f64, ...]
/// }
/// ```
/// Each value takes exactly 8 bytes of memory.
///
/// # Example
///
/// ```rust
/// use crate::column::{Column, FloatColumn};
/// use crate::types::Value;
///
/// let mut col = FloatColumn::new();
/// col.push_value(Value::Float64(3.14))?;
/// col.push_value(Value::Float64(2.718))?;
/// assert_eq!(col.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct FloatColumn {
    /// The underlying vector storing the floating point values
    data: Vec<f64>,
}

impl FloatColumn {
    /// Creates a new empty `FloatColumn`
    ///
    /// # Returns
    /// A new `FloatColumn` with no values
    ///
    /// # Example
    /// ```rust
    /// use crate::column::FloatColumn;
    ///
    /// let col = FloatColumn::new();
    /// assert_eq!(col.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Creates a new `FloatColumn` with pre-allocated capacity
    ///
    /// # Arguments
    /// * `capacity` - The number of elements to pre-allocate space for
    ///
    /// # Returns
    /// A new `FloatColumn` with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Returns a reference to the underlying data vector
    ///
    /// # Returns
    /// A reference to the `Vec<f64>` containing all values
    pub fn as_vec(&self) -> &[f64] {
        &self.data
    }
}

impl Default for FloatColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl Column for FloatColumn {
    fn data_type(&self) -> DataType {
        DataType::Float64
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn push_value(&mut self, value: Value) -> Result<()> {
        match value {
            Value::Float64(v) => {
                self.data.push(v);
                Ok(())
            }
            _ => Err(DatabaseError::type_error(format!(
                "Cannot push {:?} into FloatColumn",
                value.data_type()
            ))),
        }
    }

    fn get(&self, index: usize) -> Result<Value> {
        self.data
            .get(index)
            .map(|v| Value::Float64(*v))
            .ok_or_else(|| {
                DatabaseError::column_error(format!(
                    "Index {} out of bounds (len: {})",
                    index,
                    self.len()
                ))
            })
    }

    fn slice(&self, range: Option<std::ops::Range<usize>>) -> Vec<Value> {
        let data = &self.data;
        let range = range.unwrap_or(0..data.len());
        data[range].iter().map(|v| Value::Float64(*v)).collect()
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

// ============================================================================
// STRING COLUMN IMPLEMENTATION
// ============================================================================

/// A column that stores UTF-8 strings
///
/// Uses `Vec<String>` for storage. Each string is heap-allocated.
/// This is the most flexible column type but has higher memory overhead.
///
/// # Memory Layout
///
/// ```text
/// StringColumn {
///     data: [String, String, String, ...]
/// }
/// ```
/// Each entry is a pointer to heap-allocated string data.
///
/// # Example
///
/// ```rust
/// use crate::column::{Column, StringColumn};
/// use crate::types::Value;
///
/// let mut col = StringColumn::new();
/// col.push_value(Value::String("Hello".to_string()))?;
/// col.push_value(Value::String("World".to_string()))?;
/// assert_eq!(col.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct StringColumn {
    /// The underlying vector storing the string values
    data: Vec<String>,
}

impl StringColumn {
    /// Creates a new empty `StringColumn`
    ///
    /// # Returns
    /// A new `StringColumn` with no values
    ///
    /// # Example
    /// ```rust
    /// use crate::column::StringColumn;
    ///
    /// let col = StringColumn::new();
    /// assert_eq!(col.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Creates a new `StringColumn` with pre-allocated capacity
    ///
    /// # Arguments
    /// * `capacity` - The number of elements to pre-allocate space for
    ///
    /// # Returns
    /// A new `StringColumn` with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Returns a reference to the underlying data vector
    ///
    /// # Returns
    /// A reference to the `Vec<String>` containing all values
    pub fn as_vec(&self) -> &[String] {
        &self.data
    }
}

impl Default for StringColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl Column for StringColumn {
    fn data_type(&self) -> DataType {
        DataType::String
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn push_value(&mut self, value: Value) -> Result<()> {
        match value {
            Value::String(v) => {
                self.data.push(v);
                Ok(())
            }
            _ => Err(DatabaseError::type_error(format!(
                "Cannot push {:?} into StringColumn",
                value.data_type()
            ))),
        }
    }

    fn get(&self, index: usize) -> Result<Value> {
        self.data
            .get(index)
            .map(|v| Value::String(v.clone()))
            .ok_or_else(|| {
                DatabaseError::column_error(format!(
                    "Index {} out of bounds (len: {})",
                    index,
                    self.len()
                ))
            })
    }

    fn slice(&self, range: Option<std::ops::Range<usize>>) -> Vec<Value> {
        let data = &self.data;
        let range = range.unwrap_or(0..data.len());
        data[range]
            .iter()
            .map(|v| Value::String(v.clone()))
            .collect()
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

/// Creates a new column of the specified type
///
/// This is a convenience function for creating columns dynamically
/// based on a `DataType`. Useful when building tables from CSV data
/// where types are inferred at runtime.
///
/// # Arguments
/// * `data_type` - The type of column to create
///
/// # Returns
/// A boxed `Column` trait object of the appropriate type
///
/// # Example
/// ```rust
/// use crate::column::{Column, create_column};
/// use crate::types::DataType;
///
/// let mut col = create_column(DataType::Int64);
/// assert_eq!(col.data_type(), DataType::Int64);
/// ```
pub fn create_column(data_type: DataType) -> Box<dyn Column> {
    match data_type {
        DataType::Int64 => Box::new(IntColumn::new()),
        DataType::Float64 => Box::new(FloatColumn::new()),
        DataType::String => Box::new(StringColumn::new()),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // INT COLUMN TESTS
    // ============================================================================

    #[test]
    fn test_int_column_new() {
        let col = IntColumn::new();
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
        assert_eq!(col.data_type(), DataType::Int64);
    }

    #[test]
    fn test_int_column_with_capacity() {
        let col = IntColumn::with_capacity(10);
        assert_eq!(col.len(), 0);
        // We can't directly test capacity without internal access,
        // but we can verify it doesn't crash
    }

    #[test]
    fn test_int_column_push_value() {
        let mut col = IntColumn::new();

        col.push_value(Value::Int64(42)).unwrap();
        assert_eq!(col.len(), 1);

        col.push_value(Value::Int64(100)).unwrap();
        assert_eq!(col.len(), 2);
    }

    #[test]
    fn test_int_column_push_wrong_type() {
        let mut col = IntColumn::new();

        let result = col.push_value(Value::String("hello".to_string()));
        assert!(result.is_err());
        assert!(matches!(result, Err(DatabaseError::TypeError(_))));
    }

    #[test]
    fn test_int_column_get() {
        let mut col = IntColumn::new();
        col.push_value(Value::Int64(42)).unwrap();
        col.push_value(Value::Int64(100)).unwrap();

        let v1 = col.get(0).unwrap();
        assert_eq!(v1, Value::Int64(42));

        let v2 = col.get(1).unwrap();
        assert_eq!(v2, Value::Int64(100));
    }

    #[test]
    fn test_int_column_get_out_of_bounds() {
        let col = IntColumn::new();

        let result = col.get(0);
        assert!(result.is_err());
        assert!(matches!(result, Err(DatabaseError::ColumnError(_))));
    }

    #[test]
    fn test_int_column_slice() {
        let mut col = IntColumn::new();
        for i in 0..5 {
            col.push_value(Value::Int64(i)).unwrap();
        }

        let all = col.slice(None);
        assert_eq!(all.len(), 5);
        assert_eq!(all[0], Value::Int64(0));
        assert_eq!(all[4], Value::Int64(4));

        let some = col.slice(Some(1..3));
        assert_eq!(some.len(), 2);
        assert_eq!(some[0], Value::Int64(1));
        assert_eq!(some[1], Value::Int64(2));
    }

    #[test]
    fn test_int_column_clear() {
        let mut col = IntColumn::new();
        col.push_value(Value::Int64(1)).unwrap();
        col.push_value(Value::Int64(2)).unwrap();
        col.push_value(Value::Int64(3)).unwrap();

        assert_eq!(col.len(), 3);

        col.clear();
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn test_int_column_as_vec() {
        let mut col = IntColumn::new();
        col.push_value(Value::Int64(1)).unwrap();
        col.push_value(Value::Int64(2)).unwrap();

        let data = col.as_vec();
        assert_eq!(data, &[1, 2]);
    }

    #[test]
    fn test_int_column_default() {
        let col: IntColumn = Default::default();
        assert_eq!(col.len(), 0);
    }

    #[test]
    fn test_int_column_clone() {
        let mut col1 = IntColumn::new();
        col1.push_value(Value::Int64(42)).unwrap();

        let col2 = col1.clone();
        assert_eq!(col2.len(), col1.len());
        assert_eq!(col2.as_vec(), col1.as_vec());
    }

    // ============================================================================
    // FLOAT COLUMN TESTS
    // ============================================================================

    #[test]
    fn test_float_column_new() {
        let col = FloatColumn::new();
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
        assert_eq!(col.data_type(), DataType::Float64);
    }

    #[test]
    fn test_float_column_push_value() {
        let mut col = FloatColumn::new();

        col.push_value(Value::Float64(3.14)).unwrap();
        assert_eq!(col.len(), 1);

        col.push_value(Value::Float64(2.718)).unwrap();
        assert_eq!(col.len(), 2);
    }

    #[test]
    fn test_float_column_push_wrong_type() {
        let mut col = FloatColumn::new();

        let result = col.push_value(Value::Int64(42));
        assert!(result.is_err());
        assert!(matches!(result, Err(DatabaseError::TypeError(_))));
    }

    #[test]
    fn test_float_column_get() {
        let mut col = FloatColumn::new();
        col.push_value(Value::Float64(3.14)).unwrap();
        col.push_value(Value::Float64(2.718)).unwrap();

        let v1 = col.get(0).unwrap();
        assert_eq!(v1, Value::Float64(3.14));

        let v2 = col.get(1).unwrap();
        assert_eq!(v2, Value::Float64(2.718));
    }

    #[test]
    fn test_float_column_slice() {
        let mut col = FloatColumn::new();
        for i in 0..3 {
            col.push_value(Value::Float64(i as f64)).unwrap();
        }

        let all = col.slice(None);
        assert_eq!(all.len(), 3);
        assert_eq!(all[0], Value::Float64(0.0));
        assert_eq!(all[2], Value::Float64(2.0));
    }

    #[test]
    fn test_float_column_clear() {
        let mut col = FloatColumn::new();
        col.push_value(Value::Float64(1.0)).unwrap();
        col.push_value(Value::Float64(2.0)).unwrap();

        assert_eq!(col.len(), 2);
        col.clear();
        assert_eq!(col.len(), 0);
    }

    // ============================================================================
    // STRING COLUMN TESTS
    // ============================================================================

    #[test]
    fn test_string_column_new() {
        let col = StringColumn::new();
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
        assert_eq!(col.data_type(), DataType::String);
    }

    #[test]
    fn test_string_column_push_value() {
        let mut col = StringColumn::new();

        col.push_value(Value::String("Hello".to_string())).unwrap();
        assert_eq!(col.len(), 1);

        col.push_value(Value::String("World".to_string())).unwrap();
        assert_eq!(col.len(), 2);
    }

    #[test]
    fn test_string_column_push_wrong_type() {
        let mut col = StringColumn::new();

        let result = col.push_value(Value::Int64(42));
        assert!(result.is_err());
        assert!(matches!(result, Err(DatabaseError::TypeError(_))));
    }

    #[test]
    fn test_string_column_get() {
        let mut col = StringColumn::new();
        col.push_value(Value::String("Hello".to_string())).unwrap();
        col.push_value(Value::String("World".to_string())).unwrap();

        let v1 = col.get(0).unwrap();
        assert_eq!(v1, Value::String("Hello".to_string()));

        let v2 = col.get(1).unwrap();
        assert_eq!(v2, Value::String("World".to_string()));
    }

    #[test]
    fn test_string_column_slice() {
        let mut col = StringColumn::new();
        col.push_value(Value::String("a".to_string())).unwrap();
        col.push_value(Value::String("b".to_string())).unwrap();
        col.push_value(Value::String("c".to_string())).unwrap();

        let all = col.slice(None);
        assert_eq!(all.len(), 3);
        assert_eq!(all[0], Value::String("a".to_string()));

        let some = col.slice(Some(1..3));
        assert_eq!(some.len(), 2);
        assert_eq!(some[0], Value::String("b".to_string()));
        assert_eq!(some[1], Value::String("c".to_string()));
    }

    #[test]
    fn test_string_column_clear() {
        let mut col = StringColumn::new();
        col.push_value(Value::String("a".to_string())).unwrap();
        col.push_value(Value::String("b".to_string())).unwrap();

        assert_eq!(col.len(), 2);
        col.clear();
        assert_eq!(col.len(), 0);
    }

    #[test]
    fn test_string_column_empty_string() {
        let mut col = StringColumn::new();
        col.push_value(Value::String("".to_string())).unwrap();

        assert_eq!(col.len(), 1);
        let v = col.get(0).unwrap();
        assert_eq!(v, Value::String("".to_string()));
    }

    // ============================================================================
    // FACTORY FUNCTION TESTS
    // ============================================================================

    #[test]
    fn test_create_column_int() {
        let col = create_column(DataType::Int64);
        assert_eq!(col.data_type(), DataType::Int64);
    }

    #[test]
    fn test_create_column_float() {
        let col = create_column(DataType::Float64);
        assert_eq!(col.data_type(), DataType::Float64);
    }

    #[test]
    fn test_create_column_string() {
        let col = create_column(DataType::String);
        assert_eq!(col.data_type(), DataType::String);
    }

    // ============================================================================
    // TRAIT OBJECT TESTS
    // ============================================================================

    #[test]
    fn test_column_trait_polymorphism() {
        // Test that we can work with any column type through the trait
        let mut columns: Vec<Box<dyn Column>> = vec![
            Box::new(IntColumn::new()),
            Box::new(FloatColumn::new()),
            Box::new(StringColumn::new()),
        ];

        columns[0].push_value(Value::Int64(42)).unwrap();
        columns[1].push_value(Value::Float64(3.14)).unwrap();
        columns[2]
            .push_value(Value::String("test".to_string()))
            .unwrap();

        assert_eq!(columns[0].len(), 1);
        assert_eq!(columns[1].len(), 1);
        assert_eq!(columns[2].len(), 1);
    }

    // ============================================================================
    // EDGE CASE TESTS
    // ============================================================================

    #[test]
    fn test_large_numbers() {
        let mut col = IntColumn::new();
        col.push_value(Value::Int64(i64::MAX)).unwrap();
        col.push_value(Value::Int64(i64::MIN)).unwrap();

        let v1 = col.get(0).unwrap();
        assert_eq!(v1, Value::Int64(i64::MAX));

        let v2 = col.get(1).unwrap();
        assert_eq!(v2, Value::Int64(i64::MIN));
    }

    #[test]
    fn test_special_float_values() {
        let mut col = FloatColumn::new();
        col.push_value(Value::Float64(f64::INFINITY)).unwrap();
        col.push_value(Value::Float64(f64::NEG_INFINITY)).unwrap();
        col.push_value(Value::Float64(f64::NAN)).unwrap();

        let v1 = col.get(0).unwrap();
        if let Value::Float64(f) = v1 {
            assert!(f.is_infinite());
        } else {
            panic!("Expected Float64");
        }
    }

    #[test]
    fn test_string_with_special_chars() {
        let mut col = StringColumn::new();
        col.push_value(Value::String("Hello\nWorld".to_string()))
            .unwrap();
        col.push_value(Value::String("🎉 Emoji".to_string()))
            .unwrap();

        let v1 = col.get(0).unwrap();
        assert_eq!(v1, Value::String("Hello\nWorld".to_string()));
    }

    // ============================================================================
    // PERFORMANCE TESTS (simple sanity checks)
    // ============================================================================

    #[test]
    fn test_bulk_insert_int() {
        let mut col = IntColumn::new();
        let start = std::time::Instant::now();

        for i in 0..1000 {
            col.push_value(Value::Int64(i)).unwrap();
        }

        assert_eq!(col.len(), 1000);
        let duration = start.elapsed();
        println!("Inserted 1000 integers in {:?}", duration);

        // Verify all values
        for i in 0..1000 {
            assert_eq!(col.get(i).unwrap(), Value::Int64(i as i64));
        }
    }

    #[test]
    fn test_bulk_slice() {
        let mut col = IntColumn::new();
        for i in 0..100 {
            col.push_value(Value::Int64(i)).unwrap();
        }

        let values = col.slice(Some(10..90));
        assert_eq!(values.len(), 80);
        assert_eq!(values[0], Value::Int64(10));
        assert_eq!(values[79], Value::Int64(89));
    }
}
