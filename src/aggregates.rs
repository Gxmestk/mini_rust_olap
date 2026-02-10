//! # Aggregate Functions Module
//!
//! This module defines aggregate functions used in OLAP queries.
//!
//! ## Why Aggregates Matter in OLAP
//!
//! In analytical queries, we often need to compute summaries of large datasets:
//! - **COUNT**: How many records match a condition
//! - **SUM**: Total of numeric values
//! - **AVG**: Average of numeric values
//! - **MIN/MAX**: Range of values
//!
//! ## Aggregate Function Design
//!
//! Each aggregate function maintains state and is updated incrementally:
//!
//! 1. Create aggregate instance
//! 2. Call update() or update_batch() for each value
//! 3. Call result() to get the final aggregate value
//!
//! # Example
//!
//! ```ignore
//! use mini_rust_olap::aggregates::{AggregateFunction, CountAggregate};
//! use mini_rust_olap::types::Value;
//!
//! let mut count = CountAggregate::new(DataType::Int64);
//! count.update(Some(Value::Int64(1))).unwrap();
//! count.update(Some(Value::Int64(2))).unwrap();
//! count.update(None).unwrap();  // NULL values are skipped
//!
//! assert_eq!(count.result(), Some(Value::Int64(2)));
//! ```

use crate::error::{DatabaseError, Result};
use crate::types::{DataType, Value};
use std::fmt;

/// A trait that defines the interface for aggregate functions.
///
/// Aggregate functions compute a single result from multiple input values.
/// They maintain state that is updated incrementally as values are processed.
///
/// # Lifecycle
///
/// 1. **Creation**: Create a new instance of the aggregate
/// 2. **Update**: Call `update()` or `update_batch()` with values
/// 3. **Result**: Call `result()` to get the final aggregate value
/// 4. **Reset**: Call `reset()` to reuse the aggregate for new data
///
/// # Example
///
/// ```ignore
/// use mini_rust_olap::aggregates::{AggregateFunction, SumAggregate};
/// use mini_rust_olap::types::{DataType, Value};
///
/// let mut sum = SumAggregate::new(DataType::Int64);
/// sum.update(Some(Value::Int64(10))).unwrap();
/// sum.update(Some(Value::Int64(20))).unwrap();
///
/// assert_eq!(sum.result(), Some(Value::Int64(30)));
/// ```
pub trait AggregateFunction: fmt::Debug {
    /// Update the aggregate state with a single value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to add to the aggregate (None for NULL)
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error if the value type is incompatible
    fn update(&mut self, value: Option<Value>) -> Result<()>;

    /// Update the aggregate state with a batch of values.
    ///
    /// This is more efficient than calling `update()` multiple times
    /// because it allows the implementation to use vectorized operations.
    ///
    /// # Arguments
    ///
    /// * `values` - Slice of `Option<Value>` to add to the aggregate
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if any value type is incompatible
    fn update_batch(&mut self, values: &[Option<Value>]) -> Result<()> {
        for value in values {
            self.update(value.clone())?;
        }
        Ok(())
    }

    /// Get the final result of the aggregate.
    ///
    /// Returns None if no non-NULL values were processed.
    ///
    /// # Returns
    ///
    /// The aggregated value, or None if no values were processed
    fn result(&self) -> Option<Value>;

    /// Reset the aggregate to its initial state.
    ///
    /// This allows the same aggregate instance to be reused
    /// for processing new data.
    fn reset(&mut self);

    /// Get the data type of the result.
    ///
    /// # Returns
    ///
    /// The DataType that this aggregate produces
    fn data_type(&self) -> DataType;
}

// ============================================================================
// COUNT AGGREGATE
// ============================================================================

/// Counts the number of non-NULL values.
///
/// COUNT is the most commonly used aggregate function and works
/// with any data type.
///
/// # Example
///
/// ```ignore
/// use mini_rust_olap::aggregates::CountAggregate;
/// use mini_rust_olap::types::Value;
///
/// let count = CountAggregate::new(DataType::Int64);
/// count.update(Some(Value::Int64(1))).unwrap();
/// count.update(None).unwrap();  // NULL is not counted
/// count.update(Some(Value::Int64(2))).unwrap();
///
/// assert_eq!(count.result(), Some(Value::Int64(2)));
/// ```
#[derive(Debug, Clone)]
pub struct CountAggregate {
    count: i64,
}

impl CountAggregate {
    /// Create a new COUNT aggregate.
    ///
    /// The data_type parameter is ignored for COUNT since it always
    /// returns Int64, but is kept for API consistency.
    pub fn new(_data_type: DataType) -> Self {
        CountAggregate { count: 0 }
    }
}

impl AggregateFunction for CountAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        if value.is_some() {
            self.count += 1;
        }
        Ok(())
    }

    fn update_batch(&mut self, values: &[Option<Value>]) -> Result<()> {
        self.count += values.iter().filter(|v| v.is_some()).count() as i64;
        Ok(())
    }

    fn result(&self) -> Option<Value> {
        Some(Value::Int64(self.count))
    }

    fn reset(&mut self) {
        self.count = 0;
    }

    fn data_type(&self) -> DataType {
        DataType::Int64
    }
}

// ============================================================================
// SUM AGGREGATE
// ============================================================================

/// Computes the sum of non-NULL numeric values.
///
/// SUM works with Int64 and Float64 data types and returns a value
/// of the same type as the input.
///
/// # Type Safety
///
/// SUM will return an error if applied to String values.
#[derive(Debug, Clone)]
pub enum SumAggregate {
    Int64(i64),
    Float64(f64),
}

impl SumAggregate {
    /// Create a new SUM aggregate for the specified data type.
    ///
    /// # Arguments
    ///
    /// * `data_type` - Must be Int64 or Float64
    ///
    /// # Returns
    ///
    /// A new SumAggregate instance, or an error if data_type is not numeric
    pub fn new(data_type: DataType) -> Result<Self> {
        match data_type {
            DataType::Int64 => Ok(SumAggregate::Int64(0)),
            DataType::Float64 => Ok(SumAggregate::Float64(0.0)),
            DataType::String => Err(DatabaseError::type_error(
                "SUM cannot be applied to String type".to_string(),
            )),
        }
    }
}

impl AggregateFunction for SumAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        if let Some(value) = value {
            match (self, value) {
                (SumAggregate::Int64(sum), Value::Int64(v)) => *sum += v,
                (SumAggregate::Float64(sum), Value::Float64(v)) => *sum += v,
                (SumAggregate::Float64(sum), Value::Int64(v)) => *sum += v as f64,
                (_, other) => {
                    return Err(DatabaseError::type_error(format!(
                        "Incompatible type for SUM: {:?}",
                        other
                    )))
                }
            }
        }
        Ok(())
    }

    fn update_batch(&mut self, values: &[Option<Value>]) -> Result<()> {
        for value in values {
            self.update(value.clone())?;
        }
        Ok(())
    }

    fn result(&self) -> Option<Value> {
        match self {
            SumAggregate::Int64(sum) => Some(Value::Int64(*sum)),
            SumAggregate::Float64(sum) => Some(Value::Float64(*sum)),
        }
    }

    fn reset(&mut self) {
        match self {
            SumAggregate::Int64(sum) => *sum = 0,
            SumAggregate::Float64(sum) => *sum = 0.0,
        }
    }

    fn data_type(&self) -> DataType {
        match self {
            SumAggregate::Int64(_) => DataType::Int64,
            SumAggregate::Float64(_) => DataType::Float64,
        }
    }
}

// ============================================================================
// MIN AGGREGATE
// ============================================================================

/// Finds the minimum non-NULL value.
///
/// MIN works with all data types (Int64, Float64, String) and
/// returns a value of the same type as the input.
#[derive(Debug, Clone)]
pub enum MinAggregate {
    Int64(Option<i64>),
    Float64(Option<f64>),
    String(Option<String>),
}

impl MinAggregate {
    /// Create a new MIN aggregate for the specified data type.
    ///
    /// # Arguments
    ///
    /// * `data_type` - The data type to find the minimum of
    pub fn new(data_type: DataType) -> Self {
        match data_type {
            DataType::Int64 => MinAggregate::Int64(None),
            DataType::Float64 => MinAggregate::Float64(None),
            DataType::String => MinAggregate::String(None),
        }
    }
}

impl AggregateFunction for MinAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        if let Some(value) = value {
            match (self, value) {
                (MinAggregate::Int64(min), Value::Int64(v)) => {
                    *min = Some(min.map_or(v, |m| m.min(v)))
                }
                (MinAggregate::Float64(min), Value::Float64(v)) => {
                    *min = Some(min.map_or(v, |m| m.min(v)))
                }
                (MinAggregate::String(min), Value::String(v)) => {
                    *min = Some(min.clone().map_or(v.clone(), |m| if m < v { m } else { v }))
                }
                (_, other) => {
                    return Err(DatabaseError::type_error(format!(
                        "Incompatible type for MIN: {:?}",
                        other
                    )))
                }
            }
        }
        Ok(())
    }

    fn update_batch(&mut self, values: &[Option<Value>]) -> Result<()> {
        for value in values {
            self.update(value.clone())?;
        }
        Ok(())
    }

    fn result(&self) -> Option<Value> {
        match self {
            MinAggregate::Int64(min) => min.map(Value::Int64),
            MinAggregate::Float64(min) => min.map(Value::Float64),
            MinAggregate::String(min) => min.clone().map(Value::String),
        }
    }

    fn reset(&mut self) {
        match self {
            MinAggregate::Int64(min) => *min = None,
            MinAggregate::Float64(min) => *min = None,
            MinAggregate::String(min) => *min = None,
        }
    }

    fn data_type(&self) -> DataType {
        match self {
            MinAggregate::Int64(_) => DataType::Int64,
            MinAggregate::Float64(_) => DataType::Float64,
            MinAggregate::String(_) => DataType::String,
        }
    }
}

// ============================================================================
// MAX AGGREGATE
// ============================================================================

/// Finds the maximum non-NULL value.
///
/// MAX works with all data types (Int64, Float64, String) and
/// returns a value of the same type as the input.
#[derive(Debug, Clone)]
pub enum MaxAggregate {
    Int64(Option<i64>),
    Float64(Option<f64>),
    String(Option<String>),
}

impl MaxAggregate {
    /// Create a new MAX aggregate for the specified data type.
    ///
    /// # Arguments
    ///
    /// * `data_type` - The data type to find the maximum of
    pub fn new(data_type: DataType) -> Self {
        match data_type {
            DataType::Int64 => MaxAggregate::Int64(None),
            DataType::Float64 => MaxAggregate::Float64(None),
            DataType::String => MaxAggregate::String(None),
        }
    }
}

impl AggregateFunction for MaxAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        if let Some(value) = value {
            match (self, value) {
                (MaxAggregate::Int64(max), Value::Int64(v)) => {
                    *max = Some(max.map_or(v, |m| m.max(v)))
                }
                (MaxAggregate::Float64(max), Value::Float64(v)) => {
                    *max = Some(max.map_or(v, |m| m.max(v)))
                }
                (MaxAggregate::String(max), Value::String(v)) => {
                    *max = Some(max.clone().map_or(v.clone(), |m| if m > v { m } else { v }))
                }
                (_, other) => {
                    return Err(DatabaseError::type_error(format!(
                        "Incompatible type for MAX: {:?}",
                        other
                    )))
                }
            }
        }
        Ok(())
    }

    fn update_batch(&mut self, values: &[Option<Value>]) -> Result<()> {
        for value in values {
            self.update(value.clone())?;
        }
        Ok(())
    }

    fn result(&self) -> Option<Value> {
        match self {
            MaxAggregate::Int64(max) => max.map(Value::Int64),
            MaxAggregate::Float64(max) => max.map(Value::Float64),
            MaxAggregate::String(max) => max.clone().map(Value::String),
        }
    }

    fn reset(&mut self) {
        match self {
            MaxAggregate::Int64(max) => *max = None,
            MaxAggregate::Float64(max) => *max = None,
            MaxAggregate::String(max) => *max = None,
        }
    }

    fn data_type(&self) -> DataType {
        match self {
            MaxAggregate::Int64(_) => DataType::Int64,
            MaxAggregate::Float64(_) => DataType::Float64,
            MaxAggregate::String(_) => DataType::String,
        }
    }
}

// ============================================================================
// AVERAGE AGGREGATE
// ============================================================================

/// Computes the average of non-NULL numeric values.
///
/// AVG works with Int64 and Float64 data types and always returns
/// Float64 to preserve fractional results.
///
/// # Type Safety
///
/// AVG will return an error if applied to String values.
#[derive(Debug, Clone)]
pub struct AvgAggregate {
    sum: f64,
    count: i64,
}

impl AvgAggregate {
    /// Create a new AVG aggregate.
    ///
    /// # Arguments
    ///
    /// * `data_type` - Must be Int64 or Float64
    ///
    /// # Returns
    ///
    /// A new AvgAggregate instance, or an error if data_type is not numeric
    pub fn new(data_type: DataType) -> Result<Self> {
        match data_type {
            DataType::Int64 | DataType::Float64 => Ok(AvgAggregate { sum: 0.0, count: 0 }),
            DataType::String => Err(DatabaseError::type_error(
                "AVG cannot be applied to String type".to_string(),
            )),
        }
    }
}

impl AggregateFunction for AvgAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        if let Some(value) = value {
            match value {
                Value::Int64(v) => {
                    self.sum += v as f64;
                    self.count += 1;
                }
                Value::Float64(v) => {
                    self.sum += v;
                    self.count += 1;
                }
                other => {
                    return Err(DatabaseError::type_error(format!(
                        "Incompatible type for AVG: {:?}",
                        other
                    )))
                }
            }
        }
        Ok(())
    }

    fn update_batch(&mut self, values: &[Option<Value>]) -> Result<()> {
        for value in values {
            self.update(value.clone())?;
        }
        Ok(())
    }

    fn result(&self) -> Option<Value> {
        if self.count == 0 {
            None
        } else {
            Some(Value::Float64(self.sum / self.count as f64))
        }
    }

    fn reset(&mut self) {
        self.sum = 0.0;
        self.count = 0;
    }

    fn data_type(&self) -> DataType {
        DataType::Float64
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // COUNT AGGREGATE TESTS
    // ============================================================================

    #[test]
    fn test_count_empty() {
        let count = CountAggregate::new(DataType::Int64);
        assert_eq!(count.result(), Some(Value::Int64(0)));
    }

    #[test]
    fn test_count_single_value() {
        let mut count = CountAggregate::new(DataType::Int64);
        count.update(Some(Value::Int64(1))).unwrap();
        assert_eq!(count.result(), Some(Value::Int64(1)));
    }

    #[test]
    fn test_count_multiple_values() {
        let mut count = CountAggregate::new(DataType::Int64);
        count.update(Some(Value::Int64(1))).unwrap();
        count.update(Some(Value::Int64(2))).unwrap();
        count.update(Some(Value::Int64(3))).unwrap();
        assert_eq!(count.result(), Some(Value::Int64(3)));
    }

    #[test]
    fn test_count_with_nulls() {
        let mut count = CountAggregate::new(DataType::Int64);
        count.update(Some(Value::Int64(1))).unwrap();
        count.update(None).unwrap();
        count.update(Some(Value::Int64(2))).unwrap();
        count.update(None).unwrap();
        assert_eq!(count.result(), Some(Value::Int64(2)));
    }

    #[test]
    fn test_count_only_nulls() {
        let mut count = CountAggregate::new(DataType::Int64);
        count.update(None).unwrap();
        count.update(None).unwrap();
        assert_eq!(count.result(), Some(Value::Int64(0)));
    }

    #[test]
    fn test_count_reset() {
        let mut count = CountAggregate::new(DataType::Int64);
        count.update(Some(Value::Int64(1))).unwrap();
        count.update(Some(Value::Int64(2))).unwrap();
        assert_eq!(count.result(), Some(Value::Int64(2)));

        count.reset();
        assert_eq!(count.result(), Some(Value::Int64(0)));

        count.update(Some(Value::Int64(3))).unwrap();
        assert_eq!(count.result(), Some(Value::Int64(1)));
    }

    #[test]
    fn test_count_batch() {
        let mut count = CountAggregate::new(DataType::Int64);
        let values = vec![
            Some(Value::Int64(1)),
            None,
            Some(Value::Int64(2)),
            None,
            Some(Value::Int64(3)),
        ];

        for value in values {
            count.update(value).unwrap();
        }

        assert_eq!(count.result(), Some(Value::Int64(3)));
    }

    #[test]
    fn test_count_data_type() {
        let count = CountAggregate::new(DataType::String);
        assert_eq!(count.data_type(), DataType::Int64);
    }

    // ============================================================================
    // SUM AGGREGATE TESTS
    // ============================================================================

    #[test]
    fn test_sum_int64_empty() {
        let sum = SumAggregate::new(DataType::Int64).unwrap();
        assert_eq!(sum.result(), Some(Value::Int64(0)));
    }

    #[test]
    fn test_sum_int64_single() {
        let mut sum = SumAggregate::new(DataType::Int64).unwrap();
        sum.update(Some(Value::Int64(10))).unwrap();
        assert_eq!(sum.result(), Some(Value::Int64(10)));
    }

    #[test]
    fn test_sum_int64_multiple() {
        let mut sum = SumAggregate::new(DataType::Int64).unwrap();
        sum.update(Some(Value::Int64(10))).unwrap();
        sum.update(Some(Value::Int64(20))).unwrap();
        sum.update(Some(Value::Int64(30))).unwrap();
        assert_eq!(sum.result(), Some(Value::Int64(60)));
    }

    #[test]
    fn test_sum_int64_with_nulls() {
        let mut sum = SumAggregate::new(DataType::Int64).unwrap();
        sum.update(Some(Value::Int64(10))).unwrap();
        sum.update(None).unwrap();
        sum.update(Some(Value::Int64(20))).unwrap();
        assert_eq!(sum.result(), Some(Value::Int64(30)));
    }

    #[test]
    fn test_sum_int64_negative() {
        let mut sum = SumAggregate::new(DataType::Int64).unwrap();
        sum.update(Some(Value::Int64(10))).unwrap();
        sum.update(Some(Value::Int64(-5))).unwrap();
        assert_eq!(sum.result(), Some(Value::Int64(5)));
    }

    #[test]
    fn test_sum_float64_empty() {
        let sum = SumAggregate::new(DataType::Float64).unwrap();
        assert_eq!(sum.result(), Some(Value::Float64(0.0)));
    }

    #[test]
    fn test_sum_float64_single() {
        let mut sum = SumAggregate::new(DataType::Float64).unwrap();
        sum.update(Some(Value::Float64(10.5))).unwrap();
        assert_eq!(sum.result(), Some(Value::Float64(10.5)));
    }

    #[test]
    fn test_sum_float64_multiple() {
        let mut sum = SumAggregate::new(DataType::Float64).unwrap();
        sum.update(Some(Value::Float64(10.5))).unwrap();
        sum.update(Some(Value::Float64(20.5))).unwrap();
        sum.update(Some(Value::Float64(30.5))).unwrap();
        assert_eq!(sum.result(), Some(Value::Float64(61.5)));
    }

    #[test]
    fn test_sum_float64_with_int64_values() {
        let mut sum = SumAggregate::new(DataType::Float64).unwrap();
        sum.update(Some(Value::Int64(10))).unwrap();
        sum.update(Some(Value::Float64(20.5))).unwrap();
        assert_eq!(sum.result(), Some(Value::Float64(30.5)));
    }

    #[test]
    fn test_sum_string_error() {
        let result = SumAggregate::new(DataType::String);
        assert!(result.is_err());
    }

    #[test]
    fn test_sum_wrong_type() {
        let mut sum = SumAggregate::new(DataType::Int64).unwrap();
        let result = sum.update(Some(Value::String("test".to_string())));
        assert!(result.is_err());
    }

    #[test]
    fn test_sum_reset() {
        let mut sum = SumAggregate::new(DataType::Int64).unwrap();
        sum.update(Some(Value::Int64(10))).unwrap();
        sum.update(Some(Value::Int64(20))).unwrap();
        assert_eq!(sum.result(), Some(Value::Int64(30)));

        sum.reset();
        assert_eq!(sum.result(), Some(Value::Int64(0)));
    }

    #[test]
    fn test_sum_data_type_int64() {
        let sum = SumAggregate::new(DataType::Int64).unwrap();
        assert_eq!(sum.data_type(), DataType::Int64);
    }

    #[test]
    fn test_sum_data_type_float64() {
        let sum = SumAggregate::new(DataType::Float64).unwrap();
        assert_eq!(sum.data_type(), DataType::Float64);
    }

    // ============================================================================
    // MIN AGGREGATE TESTS
    // ============================================================================

    #[test]
    fn test_min_int64_empty() {
        let min = MinAggregate::new(DataType::Int64);
        assert_eq!(min.result(), None);
    }

    #[test]
    fn test_min_int64_single() {
        let mut min = MinAggregate::new(DataType::Int64);
        min.update(Some(Value::Int64(10))).unwrap();
        assert_eq!(min.result(), Some(Value::Int64(10)));
    }

    #[test]
    fn test_min_int64_multiple() {
        let mut min = MinAggregate::new(DataType::Int64);
        min.update(Some(Value::Int64(10))).unwrap();
        min.update(Some(Value::Int64(5))).unwrap();
        min.update(Some(Value::Int64(15))).unwrap();
        assert_eq!(min.result(), Some(Value::Int64(5)));
    }

    #[test]
    fn test_min_int64_with_nulls() {
        let mut min = MinAggregate::new(DataType::Int64);
        min.update(Some(Value::Int64(10))).unwrap();
        min.update(None).unwrap();
        min.update(Some(Value::Int64(5))).unwrap();
        assert_eq!(min.result(), Some(Value::Int64(5)));
    }

    #[test]
    fn test_min_int64_all_nulls() {
        let mut min = MinAggregate::new(DataType::Int64);
        min.update(None).unwrap();
        min.update(None).unwrap();
        assert_eq!(min.result(), None);
    }

    #[test]
    fn test_min_int64_negative() {
        let mut min = MinAggregate::new(DataType::Int64);
        min.update(Some(Value::Int64(-10))).unwrap();
        min.update(Some(Value::Int64(-5))).unwrap();
        min.update(Some(Value::Int64(-15))).unwrap();
        assert_eq!(min.result(), Some(Value::Int64(-15)));
    }

    #[test]
    fn test_min_float64_empty() {
        let min = MinAggregate::new(DataType::Float64);
        assert_eq!(min.result(), None);
    }

    #[test]
    fn test_min_float64_multiple() {
        let mut min = MinAggregate::new(DataType::Float64);
        min.update(Some(Value::Float64(10.5))).unwrap();
        min.update(Some(Value::Float64(5.5))).unwrap();
        min.update(Some(Value::Float64(15.5))).unwrap();
        assert_eq!(min.result(), Some(Value::Float64(5.5)));
    }

    #[test]
    fn test_min_string_empty() {
        let min = MinAggregate::new(DataType::String);
        assert_eq!(min.result(), None);
    }

    #[test]
    fn test_min_string_multiple() {
        let mut min = MinAggregate::new(DataType::String);
        min.update(Some(Value::String("zebra".to_string())))
            .unwrap();
        min.update(Some(Value::String("apple".to_string())))
            .unwrap();
        min.update(Some(Value::String("banana".to_string())))
            .unwrap();
        assert_eq!(min.result(), Some(Value::String("apple".to_string())));
    }

    #[test]
    fn test_min_wrong_type() {
        let mut min = MinAggregate::new(DataType::Int64);
        let result = min.update(Some(Value::String("test".to_string())));
        assert!(result.is_err());
    }

    #[test]
    fn test_min_reset() {
        let mut min = MinAggregate::new(DataType::Int64);
        min.update(Some(Value::Int64(10))).unwrap();
        min.update(Some(Value::Int64(5))).unwrap();
        assert_eq!(min.result(), Some(Value::Int64(5)));

        min.reset();
        assert_eq!(min.result(), None);
    }

    #[test]
    fn test_min_data_type_int64() {
        let min = MinAggregate::new(DataType::Int64);
        assert_eq!(min.data_type(), DataType::Int64);
    }

    #[test]
    fn test_min_data_type_float64() {
        let min = MinAggregate::new(DataType::Float64);
        assert_eq!(min.data_type(), DataType::Float64);
    }

    #[test]
    fn test_min_data_type_string() {
        let min = MinAggregate::new(DataType::String);
        assert_eq!(min.data_type(), DataType::String);
    }

    // ============================================================================
    // MAX AGGREGATE TESTS
    // ============================================================================

    #[test]
    fn test_max_int64_empty() {
        let max = MaxAggregate::new(DataType::Int64);
        assert_eq!(max.result(), None);
    }

    #[test]
    fn test_max_int64_single() {
        let mut max = MaxAggregate::new(DataType::Int64);
        max.update(Some(Value::Int64(10))).unwrap();
        assert_eq!(max.result(), Some(Value::Int64(10)));
    }

    #[test]
    fn test_max_int64_multiple() {
        let mut max = MaxAggregate::new(DataType::Int64);
        max.update(Some(Value::Int64(10))).unwrap();
        max.update(Some(Value::Int64(5))).unwrap();
        max.update(Some(Value::Int64(15))).unwrap();
        assert_eq!(max.result(), Some(Value::Int64(15)));
    }

    #[test]
    fn test_max_int64_with_nulls() {
        let mut max = MaxAggregate::new(DataType::Int64);
        max.update(Some(Value::Int64(10))).unwrap();
        max.update(None).unwrap();
        max.update(Some(Value::Int64(5))).unwrap();
        assert_eq!(max.result(), Some(Value::Int64(10)));
    }

    #[test]
    fn test_max_int64_all_nulls() {
        let mut max = MaxAggregate::new(DataType::Int64);
        max.update(None).unwrap();
        max.update(None).unwrap();
        assert_eq!(max.result(), None);
    }

    #[test]
    fn test_max_int64_negative() {
        let mut max = MaxAggregate::new(DataType::Int64);
        max.update(Some(Value::Int64(-10))).unwrap();
        max.update(Some(Value::Int64(-5))).unwrap();
        max.update(Some(Value::Int64(-15))).unwrap();
        assert_eq!(max.result(), Some(Value::Int64(-5)));
    }

    #[test]
    fn test_max_float64_empty() {
        let max = MaxAggregate::new(DataType::Float64);
        assert_eq!(max.result(), None);
    }

    #[test]
    fn test_max_float64_multiple() {
        let mut max = MaxAggregate::new(DataType::Float64);
        max.update(Some(Value::Float64(10.5))).unwrap();
        max.update(Some(Value::Float64(5.5))).unwrap();
        max.update(Some(Value::Float64(15.5))).unwrap();
        assert_eq!(max.result(), Some(Value::Float64(15.5)));
    }

    #[test]
    fn test_max_string_empty() {
        let max = MaxAggregate::new(DataType::String);
        assert_eq!(max.result(), None);
    }

    #[test]
    fn test_max_string_multiple() {
        let mut max = MaxAggregate::new(DataType::String);
        max.update(Some(Value::String("zebra".to_string())))
            .unwrap();
        max.update(Some(Value::String("apple".to_string())))
            .unwrap();
        max.update(Some(Value::String("banana".to_string())))
            .unwrap();
        assert_eq!(max.result(), Some(Value::String("zebra".to_string())));
    }

    #[test]
    fn test_max_wrong_type() {
        let mut max = MaxAggregate::new(DataType::Int64);
        let result = max.update(Some(Value::String("test".to_string())));
        assert!(result.is_err());
    }

    #[test]
    fn test_max_reset() {
        let mut max = MaxAggregate::new(DataType::Int64);
        max.update(Some(Value::Int64(10))).unwrap();
        max.update(Some(Value::Int64(5))).unwrap();
        assert_eq!(max.result(), Some(Value::Int64(10)));

        max.reset();
        assert_eq!(max.result(), None);
    }

    #[test]
    fn test_max_data_type_int64() {
        let max = MaxAggregate::new(DataType::Int64);
        assert_eq!(max.data_type(), DataType::Int64);
    }

    #[test]
    fn test_max_data_type_float64() {
        let max = MaxAggregate::new(DataType::Float64);
        assert_eq!(max.data_type(), DataType::Float64);
    }

    #[test]
    fn test_max_data_type_string() {
        let max = MaxAggregate::new(DataType::String);
        assert_eq!(max.data_type(), DataType::String);
    }

    // ============================================================================
    // AVG AGGREGATE TESTS
    // ============================================================================

    #[test]
    fn test_avg_empty() {
        let avg = AvgAggregate::new(DataType::Int64).unwrap();
        assert_eq!(avg.result(), None);
    }

    #[test]
    fn test_avg_single_int64() {
        let mut avg = AvgAggregate::new(DataType::Int64).unwrap();
        avg.update(Some(Value::Int64(10))).unwrap();
        assert_eq!(avg.result(), Some(Value::Float64(10.0)));
    }

    #[test]
    fn test_avg_single_float64() {
        let mut avg = AvgAggregate::new(DataType::Float64).unwrap();
        avg.update(Some(Value::Float64(10.5))).unwrap();
        assert_eq!(avg.result(), Some(Value::Float64(10.5)));
    }

    #[test]
    fn test_avg_multiple_int64() {
        let mut avg = AvgAggregate::new(DataType::Int64).unwrap();
        avg.update(Some(Value::Int64(10))).unwrap();
        avg.update(Some(Value::Int64(20))).unwrap();
        avg.update(Some(Value::Int64(30))).unwrap();
        assert_eq!(avg.result(), Some(Value::Float64(20.0)));
    }

    #[test]
    fn test_avg_multiple_float64() {
        let mut avg = AvgAggregate::new(DataType::Float64).unwrap();
        avg.update(Some(Value::Float64(10.5))).unwrap();
        avg.update(Some(Value::Float64(20.5))).unwrap();
        avg.update(Some(Value::Float64(30.5))).unwrap();
        assert_eq!(avg.result(), Some(Value::Float64(20.5)));
    }

    #[test]
    fn test_avg_with_nulls() {
        let mut avg = AvgAggregate::new(DataType::Int64).unwrap();
        avg.update(Some(Value::Int64(10))).unwrap();
        avg.update(None).unwrap();
        avg.update(Some(Value::Int64(20))).unwrap();
        avg.update(None).unwrap();
        assert_eq!(avg.result(), Some(Value::Float64(15.0)));
    }

    #[test]
    fn test_avg_all_nulls() {
        let mut avg = AvgAggregate::new(DataType::Int64).unwrap();
        avg.update(None).unwrap();
        avg.update(None).unwrap();
        assert_eq!(avg.result(), None);
    }

    #[test]
    fn test_avg_negative() {
        let mut avg = AvgAggregate::new(DataType::Int64).unwrap();
        avg.update(Some(Value::Int64(-10))).unwrap();
        avg.update(Some(Value::Int64(0))).unwrap();
        avg.update(Some(Value::Int64(10))).unwrap();
        assert_eq!(avg.result(), Some(Value::Float64(0.0)));
    }

    #[test]
    fn test_avg_mixed_types() {
        let mut avg = AvgAggregate::new(DataType::Float64).unwrap();
        avg.update(Some(Value::Int64(10))).unwrap();
        avg.update(Some(Value::Float64(20.5))).unwrap();
        assert_eq!(avg.result(), Some(Value::Float64(15.25)));
    }

    #[test]
    fn test_avg_fractional() {
        let mut avg = AvgAggregate::new(DataType::Int64).unwrap();
        avg.update(Some(Value::Int64(1))).unwrap();
        avg.update(Some(Value::Int64(2))).unwrap();
        assert_eq!(avg.result(), Some(Value::Float64(1.5)));
    }

    #[test]
    fn test_avg_wrong_type() {
        let mut avg = AvgAggregate::new(DataType::Int64).unwrap();
        let result = avg.update(Some(Value::String("test".to_string())));
        assert!(result.is_err());
    }

    #[test]
    fn test_avg_reset() {
        let mut avg = AvgAggregate::new(DataType::Int64).unwrap();
        avg.update(Some(Value::Int64(10))).unwrap();
        avg.update(Some(Value::Int64(20))).unwrap();
        assert_eq!(avg.result(), Some(Value::Float64(15.0)));

        avg.reset();
        assert_eq!(avg.result(), None);
    }

    #[test]
    fn test_avg_data_type() {
        let avg = AvgAggregate::new(DataType::Int64).unwrap();
        assert_eq!(avg.data_type(), DataType::Float64);
    }
}
