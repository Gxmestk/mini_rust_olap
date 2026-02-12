//! # Core Data Types Module
//!
//! This module defines the fundamental data types used throughout Mini Rust OLAP.
//!
//! ## Why Data Types Matter in Databases
//!
//! In any database system, data types are crucial for:
//! - **Memory efficiency**: Different types use different amounts of memory
//! - **Type safety**: Preventing invalid operations (e.g., adding strings)
//! - **Query optimization**: The query planner can optimize based on types
//! - **Serialization**: Storing and reading data correctly
//!
//! ## Key Types in This Module
//!
//! 1. **DataType**: Represents the schema/declared type of a column
//! 2. **Value**: Represents actual data values (can be any DataType)
//!
//! ## Design Decisions
//!
//! - **Rust Enums**: Using Rust's powerful enum system for type safety
//! - **Copy vs Clone**: Integers and floats implement `Copy` (cheap to copy)
//! - **String handling**: Strings are heap-allocated and use `String` type
//! - **NULL handling**: `Option<Value>` represents nullable values
//!
//! ## Usage Example
//!
//! ```no_run
//! use mini_rust_olap::types::{DataType, Value};
//!
//! // Define a column schema
//! let age_type = DataType::Int64;
//! let name_type = DataType::String;
//!
//! // Create actual values
//! let age = Value::Int64(25);
//! let name = Value::String("Alice".to_string());
//! ```

use std::fmt;
use std::str::FromStr;

use crate::error::{DatabaseError, Result};

/// Represents data type of a column in the database schema
///
/// This enum defines supported data types for Mini Rust OLAP columns.
/// Each variant corresponds to a specific storage layout and operations.
///
/// ## Type Characteristics
///
/// - **Int64**: 64-bit signed integer, fixed 8-byte size
/// - **Float64**: 64-bit floating point number, fixed 8-byte size (IEEE 754)
/// - **String**: Variable-length UTF-8 encoded string, heap-allocated
///
/// ## Type Safety
///
/// Rust's type system ensures that operations on values respect their declared type.
/// The compiler will catch type mismatches at compile time.
///
/// # Example
/// ```rust
/// use mini_rust_olap::types::DataType;
///
/// let column_type = DataType::Int64;
/// assert_eq!(column_type.size(), 8);
/// assert_eq!(column_type.name(), "Int64");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DataType {
    /// 64-bit signed integer
    /// Range: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
    Int64,

    /// 64-bit floating point number
    /// Follows IEEE 754 double-precision binary floating-point format
    Float64,

    /// UTF-8 encoded string
    /// Variable length, stored on the heap
    String,
}

impl DataType {
    /// Returns the name of the data type as a string
    ///
    /// # Example
    /// ```rust
    /// use mini_rust_olap::types::DataType;
    ///
    /// assert_eq!(DataType::Int64.name(), "Int64");
    /// assert_eq!(DataType::Float64.name(), "Float64");
    /// assert_eq!(DataType::String.name(), "String");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            DataType::Int64 => "Int64",
            DataType::Float64 => "Float64",
            DataType::String => "String",
        }
    }

    /// Returns the size in bytes for a single value of this type
    ///
    /// Note: For strings, this returns the size of a reference, not the actual string data.
    /// String data is stored on the heap, so the actual memory usage varies.
    ///
    /// # Example
    /// ```rust
    /// use mini_rust_olap::types::DataType;
    ///
    /// assert_eq!(DataType::Int64.size(), 8);
    /// assert_eq!(DataType::Float64.size(), 8);
    /// assert_eq!(DataType::String.size(), 24); // Pointer + length + capacity
    /// ```
    pub fn size(&self) -> usize {
        match self {
            DataType::Int64 => std::mem::size_of::<i64>(),
            DataType::Float64 => std::mem::size_of::<f64>(),
            DataType::String => std::mem::size_of::<String>(),
        }
    }

    /// Checks if this type is numeric (Int64 or Float64)
    ///
    /// This is useful for determining if arithmetic operations are valid.
    ///
    /// # Example
    /// ```rust
    /// use mini_rust_olap::types::DataType;
    ///
    /// assert!(DataType::Int64.is_numeric());
    /// assert!(DataType::Float64.is_numeric());
    /// assert!(!DataType::String.is_numeric());
    /// ```
    pub fn is_numeric(&self) -> bool {
        matches!(self, DataType::Int64 | DataType::Float64)
    }

    /// Checks if this type can be implicitly cast to another type
    ///
    /// Type coercion rules:
    /// - Int64 can be promoted to Float64
    /// - String cannot be cast to numeric types
    /// - Numeric types cannot be cast to String
    ///
    /// # Example
    /// ```rust
    /// use mini_rust_olap::types::DataType;
    ///
    /// assert!(DataType::Int64.can_cast_to(DataType::Float64));
    /// assert!(!DataType::Float64.can_cast_to(DataType::Int64));
    /// assert!(!DataType::String.can_cast_to(DataType::Int64));
    /// ```
    pub fn can_cast_to(&self, target: DataType) -> bool {
        match (self, target) {
            // Same type is always valid
            (a, b) if *a == b => true,
            // Int64 can be promoted to Float64
            (DataType::Int64, DataType::Float64) => true,
            // All other casts are invalid in our strict system
            _ => false,
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Represents a single data value in the database
///
/// This enum can hold any of the supported data types, similar to a "variant" or "any" type
/// in other languages, but with Rust's compile-time type safety.
///
/// ## Memory Layout
///
/// - Int64 and Float64 variants store values directly in the enum (Copy types)
/// - String variant owns heap-allocated data
/// - The enum itself is the size of the largest variant plus a discriminant
///
/// ## Null Values
///
/// To represent NULL/missing values, use `Option<Value>`. This is the idiomatic Rust way.
/// A value that must be present is `Value`, an optional value is `Option<Value>`.
///
/// # Example
/// ```rust
/// use mini_rust_olap::types::Value;
///
/// let age = Value::Int64(25);
/// let price = Value::Float64(19.99);
/// let name = Value::String("Alice".to_string());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 64-bit signed integer value
    Int64(i64),

    /// 64-bit floating point value
    Float64(f64),

    /// String value (UTF-8 encoded)
    String(String),
}

/// Sort direction for ORDER BY clause.
///
/// Represents the direction in which values should be sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order (ASC)
    Ascending,
    /// Descending order (DESC)
    Descending,
}

impl Value {
    /// Returns the DataType of this value
    ///
    /// This is useful for type checking and validation.
    ///
    /// # Example
    /// ```rust
    /// use mini_rust_olap::types::{Value, DataType};
    ///
    /// let v = Value::Int64(42);
    /// assert_eq!(v.data_type(), DataType::Int64);
    /// ```
    pub fn data_type(&self) -> DataType {
        match self {
            Value::Int64(_) => DataType::Int64,
            Value::Float64(_) => DataType::Float64,
            Value::String(_) => DataType::String,
        }
    }

    /// Attempts to convert this value to another DataType
    ///
    /// Returns the converted value if the conversion is valid, otherwise an error.
    ///
    /// Conversion rules:
    /// - Same type: Returns the value as-is
    /// - Int64 → Float64: Converts with possible loss of precision
    /// - Other conversions: Return an error
    ///
    /// # Example
    /// ```rust
    /// use mini_rust_olap::types::{Value, DataType};
    ///
    /// let int_val = Value::Int64(42);
    /// let float_val = int_val.cast_to(DataType::Float64).unwrap();
    /// assert!(matches!(float_val, Value::Float64(42.0)));
    /// ```
    pub fn cast_to(&self, target: DataType) -> Result<Value> {
        match (self, target) {
            // Same type - no conversion needed
            (Value::Int64(v), DataType::Int64) => Ok(Value::Int64(*v)),
            (Value::Float64(v), DataType::Float64) => Ok(Value::Float64(*v)),
            (Value::String(v), DataType::String) => Ok(Value::String(v.clone())),

            // Int64 → Float64 (promoting integer to float)
            (Value::Int64(v), DataType::Float64) => Ok(Value::Float64(*v as f64)),

            // Float64 → Int64 (truncating - potential data loss, but allowed)
            (Value::Float64(v), DataType::Int64) => Ok(Value::Int64(*v as i64)),

            // Invalid conversions
            (Value::String(_), DataType::Int64) => {
                Err(DatabaseError::type_error("Cannot cast String to Int64"))
            }
            (Value::String(_), DataType::Float64) => {
                Err(DatabaseError::type_error("Cannot cast String to Float64"))
            }
            // Invalid conversions (numeric to String)
            (Value::Int64(_), DataType::String) => {
                Err(DatabaseError::type_error("Cannot cast Int64 to String"))
            }
            (Value::Float64(_), DataType::String) => {
                Err(DatabaseError::type_error("Cannot cast Float64 to String"))
            }
        }
    }

    /// Returns true if the value is numeric (Int64 or Float64)
    ///
    /// # Example
    /// ```rust
    /// use mini_rust_olap::types::Value;
    ///
    /// assert!(Value::Int64(42).is_numeric());
    /// assert!(Value::Float64(3.5).is_numeric());
    /// assert!(!Value::String("hello".to_string()).is_numeric());
    /// ```
    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Int64(_) | Value::Float64(_))
    }

    /// Compares two values for equality
    ///
    /// This is similar to `PartialEq` but provides type checking.
    /// Returns an error if trying to compare incompatible types.
    ///
    /// # Example
    /// ```rust
    /// use mini_rust_olap::types::Value;
    ///
    /// let v1 = Value::Int64(42);
    /// let v2 = Value::Int64(42);
    /// assert_eq!(v1.equals(&v2).unwrap(), true);
    /// ```
    pub fn equals(&self, other: &Value) -> Result<bool> {
        if self.data_type() != other.data_type() {
            return Err(DatabaseError::type_error(format!(
                "Cannot compare {} with {}",
                self.data_type(),
                other.data_type()
            )));
        }
        Ok(self == other)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int64(v) => write!(f, "{}", v),
            Value::Float64(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
        }
    }
}

// ============================================================================
// FROM IMPLEMENTATIONS FOR CONVENIENCE
// ============================================================================

/// Allow creating Value from i64
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int64(value)
    }
}

/// Allow creating Value from f64
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float64(value)
    }
}

/// Allow creating Value from String
impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

/// Allow creating Value from &str
impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

// ============================================================================
// PARSING STRING REPRESENTATIONS
// ============================================================================

/// Parse a DataType from its string representation
///
/// # Example
/// ```rust
/// use mini_rust_olap::types::DataType;
/// use std::str::FromStr;
///
/// assert_eq!(DataType::from_str("Int64").unwrap(), DataType::Int64);
/// ```
impl FromStr for DataType {
    type Err = DatabaseError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "int64" | "int" => Ok(DataType::Int64),
            "float64" | "float" | "double" => Ok(DataType::Float64),
            "string" | "text" | "varchar" => Ok(DataType::String),
            _ => Err(DatabaseError::type_error(format!(
                "Unknown data type: {}",
                s
            ))),
        }
    }
}

/// Parse a Value from its string representation
///
/// This attempts to infer the type from the string content:
/// - If it looks like an integer, parse as Int64
/// - If it looks like a float, parse as Float64
/// - Otherwise, treat as String
///
/// # Example
/// ```rust
/// use mini_rust_olap::types::Value;
/// use std::str::FromStr;
///
/// assert_eq!(Value::from_str("42").unwrap(), Value::Int64(42));
/// assert_eq!(Value::from_str("1.23456").unwrap(), Value::Float64(1.23456));
/// assert_eq!(Value::from_str("hello").unwrap(), Value::String("hello".to_string()));
/// ```
impl FromStr for Value {
    type Err = DatabaseError;

    fn from_str(s: &str) -> Result<Self> {
        // Try to parse as integer first
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Value::Int64(i));
        }

        // Try to parse as float
        if let Ok(f) = s.parse::<f64>() {
            return Ok(Value::Float64(f));
        }

        // Default to string
        Ok(Value::String(s.to_string()))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // DATATYPE TESTS
    // ============================================================================

    #[test]
    fn test_datatype_names() {
        assert_eq!(DataType::Int64.name(), "Int64");
        assert_eq!(DataType::Float64.name(), "Float64");
        assert_eq!(DataType::String.name(), "String");
    }

    #[test]
    fn test_datatype_sizes() {
        // These sizes are platform-dependent but should match std::mem::size_of
        assert_eq!(DataType::Int64.size(), 8);
        assert_eq!(DataType::Float64.size(), 8);
        // String size varies but should be at least 24 bytes on 64-bit systems
        assert!(DataType::String.size() >= 16);
    }

    #[test]
    fn test_datatype_is_numeric() {
        assert!(DataType::Int64.is_numeric());
        assert!(DataType::Float64.is_numeric());
        assert!(!DataType::String.is_numeric());
    }

    #[test]
    fn test_datatype_can_cast_to() {
        // Same type is always valid
        assert!(DataType::Int64.can_cast_to(DataType::Int64));
        assert!(DataType::Float64.can_cast_to(DataType::Float64));
        assert!(DataType::String.can_cast_to(DataType::String));

        // Int64 to Float64 is valid (promotion)
        assert!(DataType::Int64.can_cast_to(DataType::Float64));

        // Float64 to Int64 is invalid in strict mode
        assert!(!DataType::Float64.can_cast_to(DataType::Int64));

        // String to numeric is invalid
        assert!(!DataType::String.can_cast_to(DataType::Int64));
        assert!(!DataType::String.can_cast_to(DataType::Float64));
    }

    #[test]
    fn test_datatype_display() {
        assert_eq!(format!("{}", DataType::Int64), "Int64");
        assert_eq!(format!("{}", DataType::Float64), "Float64");
        assert_eq!(format!("{}", DataType::String), "String");
    }

    #[test]
    fn test_datatype_equality() {
        assert_eq!(DataType::Int64, DataType::Int64);
        assert_ne!(DataType::Int64, DataType::Float64);
    }

    // ============================================================================
    // VALUE TESTS
    // ============================================================================

    #[test]
    fn test_value_creation() {
        let int_val = Value::Int64(42);
        assert!(matches!(int_val, Value::Int64(42)));

        let float_val = Value::Float64(3.5);
        assert!(matches!(float_val, Value::Float64(3.5)));

        let string_val = Value::String("hello".to_string());
        assert!(matches!(string_val, Value::String(_)));
    }

    #[test]
    fn test_value_data_type() {
        assert_eq!(Value::Int64(42).data_type(), DataType::Int64);
        assert_eq!(Value::Float64(3.5).data_type(), DataType::Float64);
        assert_eq!(
            Value::String("test".to_string()).data_type(),
            DataType::String
        );
    }

    #[test]
    fn test_value_is_numeric() {
        assert!(Value::Int64(42).is_numeric());
        assert!(Value::Float64(3.5).is_numeric());
        assert!(!Value::String("test".to_string()).is_numeric());
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Int64(42)), "42");
        assert_eq!(format!("{}", Value::Float64(3.5)), "3.5");
        assert_eq!(format!("{}", Value::String("hello".to_string())), "hello");
    }

    #[test]
    fn test_value_equality() {
        let v1 = Value::Int64(42);
        let v2 = Value::Int64(42);
        let v3 = Value::Int64(43);

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);

        let s1 = Value::String("hello".to_string());
        let s2 = Value::String("hello".to_string());
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_value_equals() {
        let v1 = Value::Int64(42);
        let v2 = Value::Int64(42);
        assert!(v1.equals(&v2).unwrap());

        let v3 = Value::Float64(3.5);
        let result = v1.equals(&v3);
        assert!(result.is_err());
        assert!(matches!(result, Err(DatabaseError::TypeError(_))));
    }

    #[test]
    fn test_value_from_traits() {
        // Test From<i64>
        let v: Value = 42i64.into();
        assert_eq!(v, Value::Int64(42));

        // Test From<f64>
        let v: Value = 3.5f64.into();
        assert_eq!(v, Value::Float64(3.5));

        // Test From<String>
        let v: Value = String::from("hello").into();
        assert_eq!(v, Value::String("hello".to_string()));

        // Test From<&str>
        let v: Value = "world".into();
        assert_eq!(v, Value::String("world".to_string()));
    }

    // ============================================================================
    // VALUE CASTING TESTS
    // ============================================================================

    #[test]
    fn test_value_cast_to_same_type() {
        let v = Value::Int64(42);
        let result = v.cast_to(DataType::Int64).unwrap();
        assert_eq!(result, Value::Int64(42));

        let v = Value::Float64(3.5);
        let result = v.cast_to(DataType::Float64).unwrap();
        assert_eq!(result, Value::Float64(3.5));

        let v = Value::String("hello".to_string());
        let result = v.cast_to(DataType::String).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_value_cast_int_to_float() {
        let v = Value::Int64(42);
        let result = v.cast_to(DataType::Float64).unwrap();
        assert!(matches!(result, Value::Float64(42.0)));

        let v = Value::Int64(-123);
        let result = v.cast_to(DataType::Float64).unwrap();
        assert!(matches!(result, Value::Float64(-123.0)));
    }

    #[test]
    fn test_value_cast_float_to_int() {
        let v = Value::Float64(42.9);
        let result = v.cast_to(DataType::Int64).unwrap();
        assert!(matches!(result, Value::Int64(42))); // Note: truncates

        let v = Value::Float64(-3.5);
        let result = v.cast_to(DataType::Int64).unwrap();
        assert!(matches!(result, Value::Int64(-3))); // Note: truncates
    }

    #[test]
    fn test_value_cast_string_to_numeric_fails() {
        let v = Value::String("42".to_string());
        let result = v.cast_to(DataType::Int64);
        assert!(result.is_err());

        let v = Value::String("hello".to_string());
        let result = v.cast_to(DataType::Float64);
        assert!(result.is_err());
    }

    // ============================================================================
    // PARSING TESTS
    // ============================================================================

    #[test]
    fn test_datatype_from_str() {
        assert_eq!(DataType::from_str("Int64").unwrap(), DataType::Int64);
        assert_eq!(DataType::from_str("int").unwrap(), DataType::Int64); // Case insensitive

        assert_eq!(DataType::from_str("Float64").unwrap(), DataType::Float64);
        assert_eq!(DataType::from_str("float").unwrap(), DataType::Float64);

        assert_eq!(DataType::from_str("String").unwrap(), DataType::String);
        assert_eq!(DataType::from_str("text").unwrap(), DataType::String);

        // Invalid type
        assert!(DataType::from_str("invalid").is_err());
    }

    #[test]
    fn test_value_from_str() {
        // Parse as integer
        let v = Value::from_str("42").unwrap();
        assert_eq!(v, Value::Int64(42));

        let v = Value::from_str("-123").unwrap();
        assert_eq!(v, Value::Int64(-123));

        // Parse as float
        let v = Value::from_str("1.23456").unwrap();
        assert_eq!(v, Value::Float64(1.23456));

        let v = Value::from_str("-0.5").unwrap();
        assert_eq!(v, Value::Float64(-0.5));

        // Parse as string (default)
        let v = Value::from_str("hello").unwrap();
        assert_eq!(v, Value::String("hello".to_string()));

        // Scientific notation parses as float
        let v = Value::from_str("1.5e3").unwrap();
        assert_eq!(v, Value::Float64(1500.0));
    }

    // ============================================================================
    // VALUE CLONE AND COPY TESTS
    // ============================================================================

    #[test]
    fn test_value_clone() {
        let v1 = Value::String("hello".to_string());
        let v2 = v1.clone();
        assert_eq!(v1, v2);
        // String variants own their data, so cloning is a deep copy
    }

    #[test]
    fn test_value_partial_eq() {
        let v1 = Value::Int64(42);
        let v2 = Value::Int64(42);
        assert_eq!(v1, v2);

        let v3 = Value::Float64(3.5);
        assert_ne!(v1, v3);
    }

    // ============================================================================
    // EDGE CASE TESTS
    // ============================================================================

    #[test]
    fn test_special_float_values() {
        let v = Value::Float64(f64::INFINITY);
        assert!(matches!(v, Value::Float64(f64::INFINITY)));

        let v = Value::Float64(f64::NEG_INFINITY);
        assert!(matches!(v, Value::Float64(f64::NEG_INFINITY)));

        let v = Value::Float64(f64::NAN);
        assert!(matches!(v, Value::Float64(f) if f.is_nan()));
    }

    #[test]
    fn test_large_integer_values() {
        let v = Value::Int64(i64::MAX);
        assert_eq!(v, Value::Int64(i64::MAX));

        let v = Value::Int64(i64::MIN);
        assert_eq!(v, Value::Int64(i64::MIN));
    }

    #[test]
    fn test_empty_string() {
        let v = Value::String("".to_string());
        assert_eq!(v.data_type(), DataType::String);
        assert_eq!(format!("{}", v), "");
    }

    #[test]
    fn test_zero_values() {
        let v1 = Value::Int64(0);
        let v2 = Value::Float64(0.0);
        assert_eq!(v1, Value::Int64(0));
        assert_eq!(v2, Value::Float64(0.0));
    }

    // ============================================================================
    // TYPE INFERENCE TESTS
    /// }

    #[test]
    fn test_type_inference_from_string() {
        // Integer with no decimal point
        assert_eq!(Value::from_str("12345").unwrap(), Value::Int64(12345));

        // Negative integer
        assert_eq!(Value::from_str("-9876").unwrap(), Value::Int64(-9876));

        // Float with decimal
        assert_eq!(Value::from_str("42.5").unwrap(), Value::Float64(42.5));

        // String that could be a number but has letters
        assert_eq!(
            Value::from_str("123abc").unwrap(),
            Value::String("123abc".to_string())
        );
    }
}
