//! # Query Execution Engine
//!
//! This module provides the foundation for vectorized query execution.
//! It defines the `Batch` struct for columnar data processing and the
//! `Operator` trait for implementing query operators like Scan, Filter,
//! Project, and GroupBy.

use crate::column::Column;
use crate::table::Table;
use crate::types::{DataType, SortDirection, Value};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Error type for execution operations
#[derive(Debug)]
pub enum ExecutionError {
    /// Operator is not open
    OperatorNotOpen,
    /// Operator is already open
    OperatorAlreadyOpen,
    /// Schema mismatch between operators
    SchemaMismatch(String),
    /// Schema not found
    SchemaNotFound,
    /// Invalid column index
    InvalidColumnIndex { index: usize, count: usize },
    /// Column not found
    ColumnNotFound(String),
    /// Invalid row index
    InvalidRowIndex { index: usize, count: usize },
    /// IO error during execution
    IoError(std::io::Error),
    /// Custom error message
    Custom(String),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::OperatorNotOpen => {
                write!(f, "Operator must be opened before calling this operation")
            }
            ExecutionError::OperatorAlreadyOpen => {
                write!(f, "Operator is already open")
            }
            ExecutionError::SchemaMismatch(msg) => {
                write!(f, "Schema mismatch: {}", msg)
            }
            ExecutionError::SchemaNotFound => {
                write!(f, "Schema not found")
            }
            ExecutionError::InvalidColumnIndex { index, count } => {
                write!(
                    f,
                    "Invalid column index {} (only {} columns available)",
                    index, count
                )
            }
            ExecutionError::ColumnNotFound(name) => {
                write!(f, "Column '{}' not found in batch", name)
            }
            ExecutionError::InvalidRowIndex { index, count } => {
                write!(
                    f,
                    "Invalid row index {} (only {} rows available)",
                    index, count
                )
            }
            ExecutionError::IoError(err) => {
                write!(f, "IO error during execution: {}", err)
            }
            ExecutionError::Custom(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<std::io::Error> for ExecutionError {
    fn from(err: std::io::Error) -> Self {
        ExecutionError::IoError(err)
    }
}

impl From<crate::error::DatabaseError> for ExecutionError {
    fn from(err: crate::error::DatabaseError) -> Self {
        ExecutionError::Custom(err.to_string())
    }
}

/// Result type for execution operations
pub type Result<T> = std::result::Result<T, ExecutionError>;

/// A batch of rows in columnar format for vectorized execution.
///
/// A Batch holds data in columnar format, which allows for efficient
/// vectorized operations. All columns in a batch must have the same number
/// of rows.
///
/// # Example
///
/// ```rust
/// use mini_rust_olap::execution::Batch;
/// use mini_rust_olap::column::{IntColumn, FloatColumn, Column};
/// use mini_rust_olap::types::Value;
/// use std::sync::Arc;
///
/// let mut col1 = IntColumn::new();
/// col1.push_value(Value::Int64(1)).unwrap();
/// col1.push_value(Value::Int64(2)).unwrap();
/// col1.push_value(Value::Int64(3)).unwrap();
///
/// let mut col2 = FloatColumn::new();
/// col2.push_value(Value::Float64(10.0)).unwrap();
/// col2.push_value(Value::Float64(20.0)).unwrap();
/// col2.push_value(Value::Float64(30.0)).unwrap();
///
/// let batch = Batch::new(vec![Arc::new(col1), Arc::new(col2)]);
/// assert_eq!(batch.row_count(), 3);
/// assert_eq!(batch.column_count(), 2);
/// ```
#[derive(Clone)]
pub struct Batch {
    columns: Vec<Arc<dyn Column>>,
}

impl Batch {
    /// Create a new Batch from a vector of columns.
    ///
    /// # Panics
    ///
    /// Panics if columns have different lengths or if the batch is empty.
    ///
    /// # Arguments
    ///
    /// * `columns` - Vector of columns with identical row counts
    pub fn new(columns: Vec<Arc<dyn Column>>) -> Self {
        if columns.is_empty() {
            panic!("Cannot create a batch with no columns");
        }

        let row_count = columns[0].len();

        for (i, col) in columns.iter().enumerate() {
            if col.len() != row_count {
                panic!(
                    "Column {} has {} rows, but column 0 has {} rows",
                    i,
                    col.len(),
                    row_count
                );
            }
        }

        Batch { columns }
    }

    /// Create an empty batch with the given schema.
    ///
    /// Useful for creating batches that will be populated later or for
    /// handling empty results.
    pub fn empty() -> Self {
        // Create a batch with no columns (special case for empty result)
        Batch {
            columns: Vec::new(),
        }
    }

    /// Returns the number of rows in the batch.
    pub fn row_count(&self) -> usize {
        if self.columns.is_empty() {
            0
        } else {
            self.columns[0].len()
        }
    }

    /// Returns the number of columns in the batch.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Returns true if the batch is empty (no rows).
    pub fn is_empty(&self) -> bool {
        self.row_count() == 0
    }

    /// Get a column by index.
    ///
    /// # Arguments
    ///
    /// * `index` - The column index
    pub fn column(&self, index: usize) -> Result<Arc<dyn Column>> {
        if index >= self.columns.len() {
            return Err(ExecutionError::InvalidColumnIndex {
                index,
                count: self.columns.len(),
            });
        }
        Ok(self.columns[index].clone())
    }

    /// Get all columns in the batch.
    pub fn columns(&self) -> &[Arc<dyn Column>] {
        &self.columns
    }

    /// Get the value at a specific row and column.
    ///
    /// # Arguments
    ///
    /// * `row_index` - The row index
    /// * `column_index` - The column index
    pub fn get(&self, row_index: usize, column_index: usize) -> Result<crate::types::Value> {
        if self.columns.is_empty() {
            return Err(ExecutionError::Custom("Batch is empty".to_string()));
        }
        if column_index >= self.columns.len() {
            return Err(ExecutionError::InvalidColumnIndex {
                index: column_index,
                count: self.columns.len(),
            });
        }

        let column = &self.columns[column_index];
        if row_index >= column.len() {
            return Err(ExecutionError::InvalidRowIndex {
                index: row_index,
                count: column.len(),
            });
        }

        column
            .get(row_index)
            .map_err(|e| ExecutionError::Custom(e.to_string()))
    }

    /// Get the value at a specific row and column as a string.
    ///
    /// This is a convenience method that always returns the value as a string.
    pub fn get_as_string(&self, row_index: usize, column_index: usize) -> Result<String> {
        let value = self.get(row_index, column_index)?;
        Ok(value.to_string())
    }

    /// Select specific columns to create a new batch.
    ///
    /// # Arguments
    ///
    /// * `column_indices` - Indices of columns to select
    pub fn select(&self, column_indices: &[usize]) -> Result<Batch> {
        let mut selected_columns = Vec::new();

        for &index in column_indices {
            if index >= self.columns.len() {
                return Err(ExecutionError::InvalidColumnIndex {
                    index,
                    count: self.columns.len(),
                });
            }
            selected_columns.push(self.columns[index].clone());
        }

        Ok(Batch::new(selected_columns))
    }

    /// Project columns to create a new batch with renamed columns.
    ///
    /// # Arguments
    ///
    /// * `column_indices` - Indices of columns to select
    /// * `_aliases` - New names for the selected columns (not yet implemented)
    pub fn project(&self, column_indices: &[usize], _aliases: &[String]) -> Result<Batch> {
        // For now, just select - renaming will be handled at the schema level
        self.select(column_indices)
    }

    /// Skip rows from the beginning of the batch.
    ///
    /// # Arguments
    ///
    /// * `skip_count` - Number of rows to skip
    pub fn skip_rows(&self, skip_count: usize) -> Result<Batch> {
        if skip_count >= self.row_count() {
            return Err(ExecutionError::Custom(format!(
                "Cannot skip {} rows from batch with only {} rows",
                skip_count,
                self.row_count()
            )));
        }

        let mut new_columns = Vec::new();
        for col in &self.columns {
            let data_type = col.data_type();
            let new_col: Arc<dyn Column> = match data_type {
                DataType::Int64 => {
                    let mut int_col = crate::column::IntColumn::new();
                    for row_idx in skip_count..col.len() {
                        int_col.push_value(col.get(row_idx)?)?;
                    }
                    Arc::new(int_col)
                }
                DataType::Float64 => {
                    let mut float_col = crate::column::FloatColumn::new();
                    for row_idx in skip_count..col.len() {
                        float_col.push_value(col.get(row_idx)?)?;
                    }
                    Arc::new(float_col)
                }
                DataType::String => {
                    let mut string_col = crate::column::StringColumn::new();
                    for row_idx in skip_count..col.len() {
                        string_col.push_value(col.get(row_idx)?)?;
                    }
                    Arc::new(string_col)
                }
            };
            new_columns.push(new_col);
        }

        Ok(Batch::new(new_columns))
    }

    /// Take only the first N rows from the batch.
    ///
    /// # Arguments
    ///
    /// * `take_count` - Number of rows to take
    pub fn take_rows(&self, take_count: usize) -> Result<Batch> {
        if take_count > self.row_count() {
            return Err(ExecutionError::Custom(format!(
                "Cannot take {} rows from batch with only {} rows",
                take_count,
                self.row_count()
            )));
        }

        let mut new_columns = Vec::new();
        for col in &self.columns {
            let data_type = col.data_type();
            let new_col: Arc<dyn Column> = match data_type {
                DataType::Int64 => {
                    let mut int_col = crate::column::IntColumn::new();
                    for row_idx in 0..take_count {
                        int_col.push_value(col.get(row_idx)?)?;
                    }
                    Arc::new(int_col)
                }
                DataType::Float64 => {
                    let mut float_col = crate::column::FloatColumn::new();
                    for row_idx in 0..take_count {
                        float_col.push_value(col.get(row_idx)?)?;
                    }
                    Arc::new(float_col)
                }
                DataType::String => {
                    let mut string_col = crate::column::StringColumn::new();
                    for row_idx in 0..take_count {
                        string_col.push_value(col.get(row_idx)?)?;
                    }
                    Arc::new(string_col)
                }
            };
            new_columns.push(new_col);
        }

        Ok(Batch::new(new_columns))
    }
}

impl fmt::Debug for Batch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Batch")
            .field("row_count", &self.row_count())
            .field("column_count", &self.column_count())
            .finish()
    }
}

/// Base trait for all query operators.
///
/// Every query operator (Scan, Filter, Project, GroupBy, etc.) implements
/// this trait. Operators follow a pull-based execution model:
///
/// 1. `open()` - Initialize the operator and allocate resources
/// 2. `next_batch()` - Pull the next batch of data
/// 3. `close()` - Release resources and cleanup
///
/// Operators can be chained together to form a query execution plan.
/// For example: Scan → Filter → Project
///
/// # Example
///
/// ```rust
/// use mini_rust_olap::execution::Operator;
/// use mini_rust_olap::execution::ExecutionError;
/// use mini_rust_olap::execution::Result;
/// use mini_rust_olap::types::DataType;
/// use std::collections::HashMap;
/// use std::sync::Arc;
///
/// struct MyOperator {
///     // operator state
/// }
///
/// impl Operator for MyOperator {
///     fn open(&mut self) -> Result<()> {
///         // Initialize operator
///         Ok(())
///     }
///
///     fn next_batch(&mut self) -> Result<Option<mini_rust_olap::execution::Batch>> {
///         // Return next batch, None if done
///         Ok(None)
///     }
///
///     fn close(&mut self) -> Result<()> {
///         // Cleanup resources
///         Ok(())
///     }
///
///     fn schema(&self) -> Result<HashMap<String, DataType>> {
///         // Return output schema
///         Err(ExecutionError::Custom("Not implemented".into()))
///     }
///
///     fn column_names(&self) -> Result<Vec<String>> {
///         // Return column names in order
///         Err(ExecutionError::Custom("Not implemented".into()))
///     }
/// }
/// ```
pub trait Operator {
    /// Initialize the operator and allocate any necessary resources.
    ///
    /// Must be called before `next_batch()`. This is where operators
    /// typically open file handles, allocate buffers, or initialize
    /// child operators.
    fn open(&mut self) -> Result<()>;

    /// Get the next batch of data from the operator.
    ///
    /// Returns `Ok(None)` when there are no more batches.
    /// Batches can be of varying sizes, but should be reasonably large
    /// for efficient vectorized processing (typically 1024 rows or more).
    ///
    /// # Arguments
    ///
    /// * `self` - Mutable reference to the operator
    ///
    /// # Returns
    ///
    /// * `Ok(Some(batch))` - The next batch of data
    /// * `Ok(None)` - No more data available
    /// * `Err(ExecutionError)` - An error occurred during execution
    fn next_batch(&mut self) -> Result<Option<Batch>>;

    /// Release resources and cleanup.
    ///
    /// Must be called after processing is complete. Operators should
    /// close file handles, free memory, and close child operators.
    fn close(&mut self) -> Result<()>;

    /// Get the schema of the output data.
    ///
    /// Returns the schema that will be produced by this operator.
    /// The schema should be valid after `open()` is called.
    fn schema(&self) -> Result<HashMap<String, DataType>>;

    /// Get column names in order.
    ///
    /// Returns a vector of column names in the order they appear in the output.
    /// This is important for operators like Project that need to preserve column order.
    fn column_names(&self) -> Result<Vec<String>>;

    /// Check if the operator is currently open.
    fn is_open(&self) -> bool {
        false
    }
}

/// State tracking for operator lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorState {
    NotOpen,
    Open,
    Closed,
}

/// TableScan operator for reading data from a table in batches.
///
/// TableScan reads data from a Table (typically from the catalog) and returns
/// it in columnar batches. It supports column pruning, which means it only
/// reads the columns that are needed for the query.
///
/// # Example
///
/// ```ignore
/// use mini_rust_olap::execution::TableScan;
/// use mini_rust_olap::table::Table;
///
/// let table = Table::new("users".to_string());
/// // Add columns to table...
///
/// // Create a TableScan that reads all columns
/// let scan = TableScan::new(table.clone());
///
/// // Or create a TableScan with column pruning (only read specific columns)
/// let scan_pruned = TableScan::with_columns(
///     table.clone(),
///     vec![0, 2], // Only read columns 0 and 2
/// );
/// ```
pub struct TableScan {
    /// The table to scan data from
    table: Table,

    /// Indices of columns to read (column pruning)
    /// If empty, read all columns
    column_indices: Vec<usize>,

    /// Current row position in the table
    current_row: usize,

    /// Total number of rows in the table
    total_rows: usize,

    /// Number of rows to return per batch
    batch_size: usize,

    /// Operator state
    state: OperatorState,

    /// Cached output schema
    output_schema: Option<HashMap<String, DataType>>,
}

impl TableScan {
    /// Create a new TableScan that reads all columns from the table.
    ///
    /// # Arguments
    ///
    /// * `table` - The table to scan
    ///
    /// # Example
    ///
    /// ```rust
    /// # use mini_rust_olap::execution::TableScan;
    /// # use mini_rust_olap::table::Table;
    /// let table = Table::new("users".to_string());
    /// let scan = TableScan::new(table);
    /// ```
    pub fn new(table: Table) -> Self {
        let column_count = table.column_count();
        let total_rows = table.row_count();

        TableScan {
            table,
            column_indices: (0..column_count).collect(),
            current_row: 0,
            total_rows,
            batch_size: 1024, // Default batch size
            state: OperatorState::NotOpen,
            output_schema: None,
        }
    }

    /// Create a new TableScan that reads only specific columns (column pruning).
    ///
    /// This is useful when queries only need a subset of columns, as it
    /// avoids reading unnecessary data.
    ///
    /// # Arguments
    ///
    /// * `table` - The table to scan
    /// * `column_indices` - Indices of columns to read
    ///
    /// # Example
    ///
    /// ```rust
    /// # use mini_rust_olap::execution::TableScan;
    /// # use mini_rust_olap::table::Table;
    /// let table = Table::new("users".to_string());
    /// // Only read columns 0 and 2 (e.g., id and age)
    /// let scan = TableScan::with_columns(table, vec![0, 2]);
    /// ```
    pub fn with_columns(table: Table, column_indices: Vec<usize>) -> Self {
        let total_rows = table.row_count();

        TableScan {
            table,
            column_indices,
            current_row: 0,
            total_rows,
            batch_size: 1024,
            state: OperatorState::NotOpen,
            output_schema: None,
        }
    }

    /// Set the batch size for this scan.
    ///
    /// Larger batch sizes can improve performance by reducing the number
    /// of calls to `next_batch()`, but use more memory.
    ///
    /// # Arguments
    ///
    /// * `batch_size` - Number of rows per batch (must be > 0)
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        if batch_size == 0 {
            panic!("Batch size must be greater than 0");
        }
        self.batch_size = batch_size;
        self
    }
}

impl Operator for TableScan {
    fn open(&mut self) -> Result<()> {
        if self.state == OperatorState::Open {
            return Err(ExecutionError::OperatorAlreadyOpen);
        }

        // Build output schema
        let mut schema = HashMap::new();
        let column_names = self.table.column_names();

        for &col_idx in &self.column_indices {
            if col_idx >= column_names.len() {
                return Err(ExecutionError::InvalidColumnIndex {
                    index: col_idx,
                    count: column_names.len(),
                });
            }
            let col_name = &column_names[col_idx];
            let data_type = self
                .table
                .get_column_type(col_name)
                .map_err(|e| ExecutionError::Custom(e.to_string()))?;
            schema.insert(col_name.clone(), data_type);
        }

        self.output_schema = Some(schema);
        self.state = OperatorState::Open;

        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<Batch>> {
        if self.state != OperatorState::Open {
            return Err(ExecutionError::OperatorNotOpen);
        }

        // Check if we've read all rows
        if self.current_row >= self.total_rows {
            return Ok(None);
        }

        // Calculate the number of rows in this batch
        let remaining_rows = self.total_rows - self.current_row;
        let batch_rows = self.batch_size.min(remaining_rows);

        // Build the batch columns
        let mut batch_columns = Vec::new();

        for &col_idx in &self.column_indices {
            let column_names = self.table.column_names();
            let col_name = &column_names[col_idx];

            // Get the column and slice it for this batch
            let column = self
                .table
                .get_column(col_name)
                .map_err(|e| ExecutionError::Custom(e.to_string()))?;

            let start_row = self.current_row;
            let end_row = start_row + batch_rows;

            // Get the sliced values and convert them back into a Column
            let values = column.slice(Some(start_row..end_row));

            // Create a new column of the appropriate type
            let data_type = self
                .table
                .get_column_type(col_name)
                .map_err(|e| ExecutionError::Custom(e.to_string()))?;

            let mut batch_column = crate::column::create_column(data_type);
            for value in values {
                batch_column
                    .push_value(value)
                    .map_err(|e| ExecutionError::Custom(e.to_string()))?;
            }

            batch_columns.push(batch_column.into());
        }

        // Create the batch
        let batch = Batch::new(batch_columns);

        // Advance the row position
        self.current_row += batch_rows;

        Ok(Some(batch))
    }

    fn close(&mut self) -> Result<()> {
        self.state = OperatorState::Closed;
        Ok(())
    }

    fn schema(&self) -> Result<HashMap<String, DataType>> {
        self.output_schema
            .clone()
            .ok_or(ExecutionError::SchemaNotFound)
    }

    fn column_names(&self) -> Result<Vec<String>> {
        if self.column_indices.is_empty() {
            // No column pruning, return all column names in order
            Ok(self.table.column_names())
        } else {
            // Return selected column names in the specified order
            let all_names = self.table.column_names();
            let mut selected_names = Vec::new();
            for &index in &self.column_indices {
                if index >= all_names.len() {
                    return Err(ExecutionError::InvalidColumnIndex {
                        index,
                        count: all_names.len(),
                    });
                }
                selected_names.push(all_names[index].clone());
            }
            Ok(selected_names)
        }
    }

    fn is_open(&self) -> bool {
        self.state == OperatorState::Open
    }
}

// ============================================================================
// PREDICATES AND FILTER OPERATOR
// ============================================================================

/// Trait for filter predicates that can be evaluated on batches.
///
/// Predicates are used by the Filter operator to determine which rows
/// should be included in the output. Each predicate can be evaluated
/// on a per-row basis.
///
/// # Example
///
/// ```rust
/// use mini_rust_olap::execution::Predicate;
/// use mini_rust_olap::execution::BinaryComparison;
/// use mini_rust_olap::execution::ComparisonOp;
/// use mini_rust_olap::types::Value;
///
/// // Create a predicate: column 0 equals 42
/// let predicate = BinaryComparison::new(0, ComparisonOp::Equal, Value::Int64(42));
/// ```
pub trait Predicate: Send + Sync + std::fmt::Debug {
    /// Evaluate the predicate on a specific row.
    ///
    /// # Arguments
    ///
    /// * `batch` - The batch containing the row
    /// * `row_index` - The index of the row to evaluate
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the row matches the predicate, `Ok(false)` otherwise
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<bool>;
}

/// Comparison operators for predicates
#[derive(Debug, Clone)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl std::fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOp::Equal => write!(f, "="),
            ComparisonOp::NotEqual => write!(f, "!="),
            ComparisonOp::LessThan => write!(f, "<"),
            ComparisonOp::LessThanOrEqual => write!(f, "<="),
            ComparisonOp::GreaterThan => write!(f, ">"),
            ComparisonOp::GreaterThanOrEqual => write!(f, ">="),
        }
    }
}

/// Binary comparison predicate: compare a column value to a constant.
///
/// This predicate compares the value in a specific column to a constant value
/// using a comparison operator.
///
/// # Example
///
/// ```rust
/// # use mini_rust_olap::execution::BinaryComparison;
/// # use mini_rust_olap::execution::ComparisonOp;
/// # use mini_rust_olap::types::Value;
/// // Create: age > 30
/// let predicate = BinaryComparison::new(2, ComparisonOp::GreaterThan, Value::Float64(30.0));
/// ```
#[derive(Debug, Clone)]
pub struct BinaryComparison {
    /// The column index to compare
    column_index: usize,
    /// The comparison operator
    op: ComparisonOp,
    /// The constant value to compare against
    value: Value,
}

impl BinaryComparison {
    /// Create a new binary comparison predicate.
    ///
    /// # Arguments
    ///
    /// * `column_index` - Index of the column to compare
    /// * `op` - The comparison operator to use
    /// * `value` - The constant value to compare against
    pub fn new(column_index: usize, op: ComparisonOp, value: Value) -> Self {
        Self {
            column_index,
            op,
            value,
        }
    }
}

impl Predicate for BinaryComparison {
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<bool> {
        let actual = batch.get(row_index, self.column_index)?;

        match (&self.op, &actual, &self.value) {
            // Equal
            (ComparisonOp::Equal, Value::Int64(a), Value::Int64(b)) => Ok(a == b),
            (ComparisonOp::Equal, Value::Float64(a), Value::Float64(b)) => Ok(a == b),
            (ComparisonOp::Equal, Value::String(a), Value::String(b)) => Ok(a == b),

            // NotEqual
            (ComparisonOp::NotEqual, Value::Int64(a), Value::Int64(b)) => Ok(a != b),
            (ComparisonOp::NotEqual, Value::Float64(a), Value::Float64(b)) => Ok(a != b),
            (ComparisonOp::NotEqual, Value::String(a), Value::String(b)) => Ok(a != b),

            // LessThan
            (ComparisonOp::LessThan, Value::Int64(a), Value::Int64(b)) => Ok(a < b),
            (ComparisonOp::LessThan, Value::Float64(a), Value::Float64(b)) => Ok(a < b),
            // LessThan with type conversion (Int64 <-> Float64)
            (ComparisonOp::LessThan, Value::Float64(a), Value::Int64(b)) => Ok(*a < (*b as f64)),
            (ComparisonOp::LessThan, Value::Int64(a), Value::Float64(b)) => Ok((*a as f64) < *b),

            // LessThanOrEqual
            (ComparisonOp::LessThanOrEqual, Value::Int64(a), Value::Int64(b)) => Ok(a <= b),
            (ComparisonOp::LessThanOrEqual, Value::Float64(a), Value::Float64(b)) => Ok(a <= b),
            // LessThanOrEqual with type conversion (Int64 <-> Float64)
            (ComparisonOp::LessThanOrEqual, Value::Float64(a), Value::Int64(b)) => {
                Ok(*a <= (*b as f64))
            }
            (ComparisonOp::LessThanOrEqual, Value::Int64(a), Value::Float64(b)) => {
                Ok((*a as f64) <= *b)
            }

            // GreaterThan
            (ComparisonOp::GreaterThan, Value::Int64(a), Value::Int64(b)) => Ok(a > b),
            (ComparisonOp::GreaterThan, Value::Float64(a), Value::Float64(b)) => Ok(a > b),
            // GreaterThan with type conversion (Int64 <-> Float64)
            (ComparisonOp::GreaterThan, Value::Float64(a), Value::Int64(b)) => Ok(*a > (*b as f64)),
            (ComparisonOp::GreaterThan, Value::Int64(a), Value::Float64(b)) => Ok((*a as f64) > *b),

            // GreaterThanOrEqual
            (ComparisonOp::GreaterThanOrEqual, Value::Int64(a), Value::Int64(b)) => Ok(a >= b),
            (ComparisonOp::GreaterThanOrEqual, Value::Float64(a), Value::Float64(b)) => Ok(a >= b),
            // GreaterThanOrEqual with type conversion (Int64 <-> Float64)
            (ComparisonOp::GreaterThanOrEqual, Value::Float64(a), Value::Int64(b)) => {
                Ok(*a >= (*b as f64))
            }
            (ComparisonOp::GreaterThanOrEqual, Value::Int64(a), Value::Float64(b)) => {
                Ok((*a as f64) >= *b)
            }

            // Type mismatch - for now, return false
            _ => Ok(false),
        }
    }
}

/// Logical AND predicate: both sub-predicates must be true.
///
/// # Example
///
/// ```rust
/// # use mini_rust_olap::execution::And;
/// # use mini_rust_olap::execution::BinaryComparison;
/// # use mini_rust_olap::execution::ComparisonOp;
/// # use mini_rust_olap::types::Value;
/// use std::sync::Arc;
///
/// // Create: age > 25 AND age < 50
/// let pred1 = BinaryComparison::new(2, ComparisonOp::GreaterThan, Value::Float64(25.0));
/// let pred2 = BinaryComparison::new(2, ComparisonOp::LessThan, Value::Float64(50.0));
/// let predicate = And::new(Arc::new(pred1), Arc::new(pred2));
/// ```
#[derive(Debug)]
pub struct And {
    /// Left sub-predicate
    left: Arc<dyn Predicate>,
    /// Right sub-predicate
    right: Arc<dyn Predicate>,
}

impl And {
    /// Create a new AND predicate.
    ///
    /// # Arguments
    ///
    /// * `left` - The left sub-predicate
    /// * `right` - The right sub-predicate
    pub fn new(left: Arc<dyn Predicate>, right: Arc<dyn Predicate>) -> Self {
        Self { left, right }
    }
}

impl Predicate for And {
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<bool> {
        let left_result = self.left.eval(batch, row_index)?;
        if !left_result {
            return Ok(false); // Short-circuit
        }
        self.right.eval(batch, row_index)
    }
}

/// Logical OR predicate: at least one sub-predicate must be true.
///
/// # Example
///
/// ```rust
/// # use mini_rust_olap::execution::Or;
/// # use mini_rust_olap::execution::BinaryComparison;
/// # use mini_rust_olap::execution::ComparisonOp;
/// # use mini_rust_olap::types::Value;
/// use std::sync::Arc;
///
/// // Create: age < 25 OR age > 65
/// let pred1 = BinaryComparison::new(2, ComparisonOp::LessThan, Value::Float64(25.0));
/// let pred2 = BinaryComparison::new(2, ComparisonOp::GreaterThan, Value::Float64(65.0));
/// let predicate = Or::new(Arc::new(pred1), Arc::new(pred2));
/// ```
#[derive(Debug)]
pub struct Or {
    /// Left sub-predicate
    left: Arc<dyn Predicate>,
    /// Right sub-predicate
    right: Arc<dyn Predicate>,
}

impl Or {
    /// Create a new OR predicate.
    ///
    /// # Arguments
    ///
    /// * `left` - The left sub-predicate
    /// * `right` - The right sub-predicate
    pub fn new(left: Arc<dyn Predicate>, right: Arc<dyn Predicate>) -> Self {
        Self { left, right }
    }
}

impl Predicate for Or {
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<bool> {
        let left_result = self.left.eval(batch, row_index)?;
        if left_result {
            return Ok(true); // Short-circuit
        }
        self.right.eval(batch, row_index)
    }
}

/// Filter operator that filters rows based on a predicate.
///
/// Filter reads batches from its child operator and returns only the rows
/// that match the predicate. It evaluates the predicate on each row and
/// includes the row in the output if the predicate evaluates to true.
///
/// # Example
///
/// ```rust
/// # use mini_rust_olap::execution::Filter;
/// # use mini_rust_olap::execution::TableScan;
/// # use mini_rust_olap::execution::BinaryComparison;
/// # use mini_rust_olap::execution::ComparisonOp;
/// # use mini_rust_olap::execution::Operator;
/// # use mini_rust_olap::table::Table;
/// # use mini_rust_olap::types::Value;
/// use std::sync::Arc;
///
/// let table = Table::new("users".to_string());
/// // ... add columns to table ...
///
/// let scan = TableScan::new(table);
/// let predicate = BinaryComparison::new(
///     2, // age column
///     ComparisonOp::GreaterThan,
///     Value::Float64(30.0)
/// );
/// let mut filter = Filter::new(Box::new(scan), Arc::new(predicate));
///
/// filter.open().unwrap();
/// while let Some(batch) = filter.next_batch().unwrap() {
///     // Process filtered batches
/// }
/// filter.close().unwrap();
/// ```
pub struct Filter {
    /// The child operator to read data from
    child: Box<dyn Operator>,

    /// The predicate to evaluate on each row
    predicate: Arc<dyn Predicate>,

    /// Operator state
    state: OperatorState,

    /// Cached output schema
    output_schema: Option<HashMap<String, DataType>>,
}

impl Filter {
    /// Create a new Filter operator.
    ///
    /// # Arguments
    ///
    /// * `child` - The child operator to read data from
    /// * `predicate` - The predicate to evaluate on each row
    pub fn new(child: Box<dyn Operator>, predicate: Arc<dyn Predicate>) -> Self {
        Filter {
            child,
            predicate,
            state: OperatorState::NotOpen,
            output_schema: None,
        }
    }
}

impl Operator for Filter {
    fn open(&mut self) -> Result<()> {
        if self.state == OperatorState::Open {
            return Err(ExecutionError::OperatorAlreadyOpen);
        }

        // Open the child operator
        self.child.open()?;

        // Cache the child's schema as our output schema
        self.output_schema = Some(self.child.schema()?);

        self.state = OperatorState::Open;
        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<Batch>> {
        if self.state != OperatorState::Open {
            return Err(ExecutionError::OperatorNotOpen);
        }

        // Get next batch from child
        let batch = match self.child.next_batch()? {
            Some(b) => b,
            None => return Ok(None), // No more data
        };

        // If batch is empty, return it as-is
        if batch.row_count() == 0 {
            return Ok(Some(batch));
        }

        // Evaluate predicate on each row and collect matching rows
        let mut matching_row_indices = Vec::new();
        for row_idx in 0..batch.row_count() {
            if self.predicate.eval(&batch, row_idx)? {
                matching_row_indices.push(row_idx);
            }
        }

        // If no rows match, continue to next batch
        if matching_row_indices.is_empty() {
            return self.next_batch(); // Recursively get next batch
        }

        // Create a new batch with only the matching rows
        let mut filtered_columns = Vec::new();
        let column_count = batch.column_count();

        for col_idx in 0..column_count {
            let original_column = batch.column(col_idx)?;
            let original_values = original_column.slice(None);

            // Filter the values
            let filtered_values: Vec<Value> = matching_row_indices
                .iter()
                .map(|&row_idx| original_values[row_idx].clone())
                .collect();

            // Create a new column with filtered values
            let data_type = original_column.data_type();
            let mut filtered_column = crate::column::create_column(data_type);
            for value in filtered_values {
                filtered_column
                    .push_value(value)
                    .map_err(|e| ExecutionError::Custom(e.to_string()))?;
            }

            filtered_columns.push(filtered_column.into());
        }

        Ok(Some(Batch::new(filtered_columns)))
    }

    fn close(&mut self) -> Result<()> {
        self.state = OperatorState::Closed;
        self.child.close()?;
        Ok(())
    }

    fn schema(&self) -> Result<HashMap<String, DataType>> {
        self.output_schema
            .clone()
            .ok_or(ExecutionError::SchemaNotFound)
    }

    fn column_names(&self) -> Result<Vec<String>> {
        // Filter preserves column names and order from child
        self.child.column_names()
    }

    fn is_open(&self) -> bool {
        self.state == OperatorState::Open
    }
}

/// The Project operator selects a subset of columns from its input.
///
/// This operator is used to implement the SQL SELECT clause, allowing queries to:
/// - Select specific columns from a table
/// - Reorder columns in the result set
/// - Rename columns using aliases
///
/// # Example
///
/// ```ignore
/// use mini_rust_olap::execution::{Operator, TableScan, Project};
/// use mini_rust_olap::catalog::Catalog;
///
/// let catalog = Catalog::new();
/// // Assume table with columns: id, name, age, salary
///
/// let scan = Box::new(TableScan::new(catalog.get_table("employees")?).unwrap());
///
/// // Select only name and salary, reorder them
/// let project = Box::new(Project::new(scan, vec![1, 3]));
/// ```
pub struct Project {
    /// The child operator to read data from
    child: Box<dyn Operator>,

    /// Indices of columns to project (from the child's schema)
    column_indices: Vec<usize>,

    /// Optional aliases for the projected columns
    aliases: Option<Vec<String>>,

    /// Operator state
    state: OperatorState,

    /// Cached output schema
    output_schema: Option<HashMap<String, DataType>>,
}

impl Project {
    /// Create a new Project operator.
    ///
    /// # Arguments
    ///
    /// * `child` - The child operator to read data from
    /// * `column_indices` - Indices of columns to project
    ///
    /// # Example
    ///
    /// ```ignore
    /// use mini_rust_olap::execution::{Operator, TableScan, Project};
    ///
    /// let scan = Box::new(TableScan::new(table).unwrap());
    /// let project = Box::new(Project::new(scan, vec![0, 2]));
    /// ```
    pub fn new(child: Box<dyn Operator>, column_indices: Vec<usize>) -> Self {
        Project {
            child,
            column_indices,
            aliases: None,
            state: OperatorState::NotOpen,
            output_schema: None,
        }
    }

    /// Set aliases for the projected columns.
    ///
    /// # Arguments
    ///
    /// * `aliases` - Names for the projected columns (must match length of column_indices)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use mini_rust_olap::execution::{Operator, TableScan, Project};
    ///
    /// let scan = Box::new(TableScan::new(table).unwrap());
    /// let project = Box::new(
    ///     Project::new(scan, vec![0, 2])
    ///         .with_aliases(vec!["user_id".to_string(), "user_name".to_string()])
    /// );
    /// ```
    pub fn with_aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = Some(aliases);
        self
    }
}

impl Operator for Project {
    fn open(&mut self) -> Result<()> {
        if self.state == OperatorState::Open {
            return Err(ExecutionError::OperatorAlreadyOpen);
        }

        // Open the child operator
        self.child.open()?;

        // Build the output schema
        let child_schema = self.child.schema()?;
        let child_column_names = self.child.column_names()?;
        let mut output_schema = HashMap::new();

        for (i, &index) in self.column_indices.iter().enumerate() {
            // Validate column index
            if index >= child_column_names.len() {
                return Err(ExecutionError::InvalidColumnIndex {
                    index,
                    count: child_column_names.len(),
                });
            }

            let original_name = &child_column_names[index];
            let data_type = child_schema[original_name];

            // Determine output column name (use alias if provided)
            let output_name = match &self.aliases {
                Some(aliases) => {
                    if i >= aliases.len() {
                        return Err(ExecutionError::Custom(format!(
                            "Not enough aliases provided: expected {}, got {}",
                            self.column_indices.len(),
                            aliases.len()
                        )));
                    }
                    aliases[i].clone()
                }
                None => original_name.clone(),
            };

            // Check for duplicate column names
            if output_schema.contains_key(&output_name) {
                return Err(ExecutionError::Custom(format!(
                    "Duplicate column name: {}",
                    output_name
                )));
            }

            output_schema.insert(output_name, data_type);
        }

        self.output_schema = Some(output_schema);
        self.state = OperatorState::Open;
        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<Batch>> {
        if self.state != OperatorState::Open {
            return Err(ExecutionError::OperatorNotOpen);
        }

        // Get next batch from child
        let batch = match self.child.next_batch()? {
            Some(b) => b,
            None => return Ok(None), // No more data
        };

        // Project the selected columns
        let mut projected_columns = Vec::new();

        for &index in &self.column_indices {
            let column = batch.column(index)?;
            projected_columns.push(column);
        }

        Ok(Some(Batch::new(projected_columns)))
    }

    fn close(&mut self) -> Result<()> {
        self.state = OperatorState::Closed;
        self.child.close()?;
        Ok(())
    }

    fn schema(&self) -> Result<HashMap<String, DataType>> {
        self.output_schema
            .clone()
            .ok_or(ExecutionError::SchemaNotFound)
    }

    fn column_names(&self) -> Result<Vec<String>> {
        // Rebuild column names from output schema to preserve order
        let child_column_names = self.child.column_names()?;
        let mut output_names = Vec::new();

        for (i, &index) in self.column_indices.iter().enumerate() {
            let original_name = &child_column_names[index];

            // Use alias if provided, otherwise use original name
            let output_name = match &self.aliases {
                Some(aliases) => {
                    if i < aliases.len() {
                        aliases[i].clone()
                    } else {
                        original_name.clone()
                    }
                }
                None => original_name.clone(),
            };

            output_names.push(output_name);
        }

        Ok(output_names)
    }

    fn is_open(&self) -> bool {
        self.state == OperatorState::Open
    }
}

// ============================================================================
// GROUP BY OPERATOR
// ============================================================================

/// A key for grouping rows in a GroupBy operation.
///
/// The key is a vector of values representing the group by columns.
/// It implements Hash and Eq for use as a HashMap key.
#[derive(Debug, Clone)]
struct GroupKey(Vec<Option<Value>>);

impl PartialEq for GroupKey {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            match (a, b) {
                (None, None) => continue,
                (None, Some(_)) | (Some(_), None) => return false,
                (Some(va), Some(vb)) => {
                    // Compare using string representation
                    if va.to_string() != vb.to_string() {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl Eq for GroupKey {}

impl Hash for GroupKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for value in &self.0 {
            match value {
                None => 0.hash(state),
                Some(v) => {
                    // Hash based on value type and value
                    match v {
                        Value::Int64(i) => (1, i).hash(state),
                        Value::Float64(f) => (2, f.to_bits()).hash(state),
                        Value::String(s) => (3, s).hash(state),
                    }
                }
            }
        }
    }
}

/// GroupBy operator for grouping rows and computing aggregates.
///
/// GroupBy reads all rows from a child operator, groups them by specified
/// columns, and computes aggregates for each group. The output contains
/// one row per group with the group by keys followed by the aggregate results.
///
/// # Example
///
/// ```ignore
/// use mini_rust_olap::execution::{Operator, TableScan, GroupBy};
/// use mini_rust_olap::aggregates::CountAggregate;
/// use mini_rust_olap::types::DataType;
///
/// let scan = Box::new(TableScan::new(table).unwrap());
/// let group_by = Box::new(GroupBy::new(
///     scan,
///     vec![0],  // Group by first column
///     vec![1],  // Aggregate second column
///     vec![Box::new(CountAggregate::new(DataType::Int64))]
/// ));
/// ```
pub struct GroupBy {
    /// The child operator to read data from
    child: Box<dyn Operator>,

    /// Indices of columns to group by
    group_by_columns: Vec<usize>,

    /// Indices of columns to aggregate
    aggregate_columns: Vec<usize>,

    /// Aggregates to compute for each group
    aggregates: Vec<Box<dyn crate::aggregates::AggregateFunction>>,

    /// Operator state
    state: OperatorState,

    /// Output schema will include group by columns followed by aggregates
    output_schema: Option<HashMap<String, DataType>>,

    /// Column names in output order
    output_column_names: Option<Vec<String>>,

    /// Cache of grouped data (computed during open())
    grouped_data: Option<HashMap<GroupKey, Vec<Vec<Option<Value>>>>>,

    /// Whether results have been returned
    results_returned: bool,
}

impl GroupBy {
    /// Create a new GroupBy operator.
    ///
    /// # Arguments
    ///
    /// * `child` - The child operator to read data from
    /// * `group_by_columns` - Indices of columns to group by
    /// * `aggregates` - Vector of aggregate functions to compute for each group
    ///
    /// # Example
    ///
    /// ```ignore
    /// use mini_rust_olap::execution::{Operator, TableScan, GroupBy};
    /// use mini_rust_olap::aggregates::CountAggregate;
    /// use mini_rust_olap::types::DataType;
    ///
    /// let scan = Box::new(TableScan::new(table).unwrap());
    /// let group_by = Box::new(GroupBy::new(
    ///     scan,
    ///     vec![0, 1],  // Group by first two columns
    ///     vec![2],     // Aggregate third column
    ///     vec![Box::new(CountAggregate::new(DataType::Int64))]
    /// ));
    /// ```
    pub fn new(
        child: Box<dyn Operator>,
        group_by_columns: Vec<usize>,
        aggregate_columns: Vec<usize>,
        aggregates: Vec<Box<dyn crate::aggregates::AggregateFunction>>,
    ) -> Self {
        GroupBy {
            child,
            group_by_columns,
            aggregate_columns,
            aggregates,
            state: OperatorState::NotOpen,
            output_schema: None,
            output_column_names: None,
            grouped_data: None,
            results_returned: false,
        }
    }
}

impl Operator for GroupBy {
    fn open(&mut self) -> Result<()> {
        if self.state == OperatorState::Open {
            return Err(ExecutionError::OperatorAlreadyOpen);
        }

        // Open the child operator
        self.child.open()?;

        // Get child schema and column names
        let child_schema = self.child.schema()?;
        let child_column_names = self.child.column_names()?;
        let child_column_count = child_column_names.len();

        // Validate group by column indices
        for &index in &self.group_by_columns {
            if index >= child_column_count {
                return Err(ExecutionError::InvalidColumnIndex {
                    index,
                    count: child_column_count,
                });
            }
        }

        // Validate aggregate column indices
        for &index in &self.aggregate_columns {
            if index >= child_column_count {
                return Err(ExecutionError::InvalidColumnIndex {
                    index,
                    count: child_column_count,
                });
            }
        }

        // Validate that aggregate_columns length matches aggregates length
        if self.aggregate_columns.len() != self.aggregates.len() {
            return Err(ExecutionError::Custom(format!(
                "aggregate_columns length ({}) must match aggregates length ({})",
                self.aggregate_columns.len(),
                self.aggregates.len()
            )));
        }

        // Build output schema and column names
        let mut output_schema = HashMap::new();
        let mut output_column_names = Vec::new();

        // Add group by columns to output
        for &index in &self.group_by_columns {
            let name = child_column_names[index].clone();
            let data_type = child_schema[&name];
            output_schema.insert(name.clone(), data_type);
            output_column_names.push(name);
        }

        // Add aggregates to output
        for (i, agg) in self.aggregates.iter().enumerate() {
            let name = format!("agg_{}", i);
            let data_type = agg.data_type();
            output_schema.insert(name.clone(), data_type);
            output_column_names.push(name);
        }

        self.output_schema = Some(output_schema);
        self.output_column_names = Some(output_column_names);

        // Read all data and group it
        let mut grouped_data: HashMap<GroupKey, Vec<Vec<Option<Value>>>> = HashMap::new();

        while let Some(batch) = self.child.next_batch()? {
            let row_count = batch.row_count();
            let col_count = batch.column_count();

            for row_index in 0..row_count {
                // Build group key
                let mut key_values = Vec::new();
                for &col_index in &self.group_by_columns {
                    let value = batch.get(row_index, col_index)?;
                    key_values.push(Some(value));
                }
                let key = GroupKey(key_values);

                // Get all values for this row
                let mut row_values = Vec::new();
                for col_index in 0..col_count {
                    let value = batch.get(row_index, col_index)?;
                    row_values.push(Some(value));
                }

                // Add row to its group
                grouped_data.entry(key).or_default().push(row_values);
            }
        }

        self.grouped_data = Some(grouped_data);
        self.state = OperatorState::Open;
        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<Batch>> {
        if self.state != OperatorState::Open {
            return Err(ExecutionError::OperatorNotOpen);
        }

        if self.results_returned {
            return Ok(None);
        }

        self.results_returned = true;

        let grouped_data = self.grouped_data.as_ref().unwrap();

        // If no data, return empty batch
        if grouped_data.is_empty() {
            return Ok(None);
        }

        // Prepare output columns
        let group_by_col_count = self.group_by_columns.len();
        let agg_col_count = self.aggregates.len();
        let mut output_columns: Vec<Vec<Option<Value>>> =
            vec![Vec::new(); group_by_col_count + agg_col_count];

        // Process each group
        for (key, rows) in grouped_data {
            // Add group by values
            for (col_index, value) in key.0.iter().enumerate() {
                output_columns[col_index].push(value.clone());
            }

            // Compute aggregates
            for (agg_index, agg) in self.aggregates.iter_mut().enumerate() {
                agg.reset();
                let agg_col_index = self.aggregate_columns[agg_index];

                // Add all rows in this group to the aggregate
                for row in rows {
                    if let Some(value) = &row[agg_col_index] {
                        agg.update(Some(value.clone()))?;
                    }
                }

                let result = agg.result();
                let output_index = group_by_col_count + agg_index;
                output_columns[output_index].push(result);
            }
        }

        // Convert output columns to actual column types
        let mut final_columns = Vec::new();
        let child_schema = self.child.schema()?;
        let child_column_names = self.child.column_names()?;

        // Group by columns
        for (i, &col_index) in self.group_by_columns.iter().enumerate() {
            let col_name = &child_column_names[col_index];
            let data_type = &child_schema[col_name];
            let values = &output_columns[i];

            let column: Arc<dyn Column> = match data_type {
                DataType::Int64 => {
                    let mut int_col = crate::column::IntColumn::new();
                    for value in values {
                        if let Some(Value::Int64(v)) = value {
                            int_col.push_value(Value::Int64(*v))?;
                        } else {
                            int_col.push_value(Value::Int64(0))?;
                        }
                    }
                    Arc::new(int_col)
                }
                DataType::Float64 => {
                    let mut float_col = crate::column::FloatColumn::new();
                    for value in values {
                        if let Some(Value::Float64(v)) = value {
                            float_col.push_value(Value::Float64(*v))?;
                        } else {
                            float_col.push_value(Value::Float64(0.0))?;
                        }
                    }
                    Arc::new(float_col)
                }
                DataType::String => {
                    let mut string_col = crate::column::StringColumn::new();
                    for value in values {
                        if let Some(Value::String(v)) = value {
                            string_col.push_value(Value::String(v.clone()))?;
                        } else {
                            string_col.push_value(Value::String(String::new()))?;
                        }
                    }
                    Arc::new(string_col)
                }
            };
            final_columns.push(column);
        }

        // Aggregate columns
        for (i, agg) in self.aggregates.iter().enumerate() {
            let data_type = agg.data_type();
            let values = &output_columns[group_by_col_count + i];

            let column: Arc<dyn Column> = match data_type {
                DataType::Int64 => {
                    let mut int_col = crate::column::IntColumn::new();
                    for value in values {
                        if let Some(Value::Int64(v)) = value {
                            int_col.push_value(Value::Int64(*v))?;
                        } else {
                            int_col.push_value(Value::Int64(0))?;
                        }
                    }
                    Arc::new(int_col)
                }
                DataType::Float64 => {
                    let mut float_col = crate::column::FloatColumn::new();
                    for value in values {
                        if let Some(Value::Float64(v)) = value {
                            float_col.push_value(Value::Float64(*v))?;
                        } else {
                            float_col.push_value(Value::Float64(0.0))?;
                        }
                    }
                    Arc::new(float_col)
                }
                DataType::String => {
                    let mut string_col = crate::column::StringColumn::new();
                    for value in values {
                        if let Some(Value::String(v)) = value {
                            string_col.push_value(Value::String(v.clone()))?;
                        } else {
                            string_col.push_value(Value::String(String::new()))?;
                        }
                    }
                    Arc::new(string_col)
                }
            };
            final_columns.push(column);
        }

        Ok(Some(Batch::new(final_columns)))
    }

    fn close(&mut self) -> Result<()> {
        self.state = OperatorState::Closed;
        self.child.close()?;
        self.grouped_data = None;
        self.results_returned = false;
        Ok(())
    }

    fn schema(&self) -> Result<HashMap<String, DataType>> {
        self.output_schema
            .clone()
            .ok_or(ExecutionError::SchemaNotFound)
    }

    fn column_names(&self) -> Result<Vec<String>> {
        self.output_column_names
            .clone()
            .ok_or(ExecutionError::Custom(
                "Column names not initialized".to_string(),
            ))
    }

    fn is_open(&self) -> bool {
        self.state == OperatorState::Open
    }
}

// ============================================================================
// SORT OPERATOR (ORDER BY)
// ============================================================================

/// Sort operator for ORDER BY clause.
///
/// Sort operator reads all data from the child operator and sorts it
/// according to the specified columns and directions. Since sorting
/// requires all data, this operator reads all rows in open().
pub struct Sort {
    /// The child operator to read data from
    child: Box<dyn Operator>,

    /// Indices of columns to sort by
    sort_columns: Vec<usize>,

    /// Sort direction for each column
    sort_directions: Vec<SortDirection>,

    /// Operator state
    state: OperatorState,

    /// Sorted data after sorting
    sorted_data: Option<Batch>,

    /// Current position in sorted data
    current_row: usize,

    /// Batch size for output
    batch_size: usize,
}

impl Sort {
    /// Create a new Sort operator.
    ///
    /// # Arguments
    ///
    /// * `child` - The child operator to read data from
    /// * `sort_columns` - Indices of columns to sort by
    /// * `sort_directions` - Sort direction for each column (true = ascending, false = descending)
    ///
    /// # Example
    ///
    /// ```rust
    /// # use mini_rust_olap::execution::Sort;
    /// # use mini_rust_olap::execution::TableScan;
    /// # use mini_rust_olap::execution::Operator;
    /// # use mini_rust_olap::types::SortDirection;
    /// # use mini_rust_olap::table::Table;
    /// use std::sync::Arc;
    ///
    /// let table = Table::new("users".to_string());
    /// // ... add columns to table ...
    ///
    /// let scan = TableScan::new(table);
    /// let mut sort = Sort::new(
    ///     Box::new(scan),
    ///     vec![0, 1],
    ///     vec![SortDirection::Ascending, SortDirection::Descending],
    /// );
    ///
    /// sort.open().unwrap();
    /// while let Some(batch) = sort.next_batch().unwrap() {
    ///     // Process sorted batches
    /// }
    /// sort.close().unwrap();
    /// ```
    pub fn new(
        child: Box<dyn Operator>,
        sort_columns: Vec<usize>,
        sort_directions: Vec<SortDirection>,
    ) -> Self {
        Sort {
            child,
            sort_columns,
            sort_directions,
            state: OperatorState::NotOpen,
            sorted_data: None,
            current_row: 0,
            batch_size: 1024,
        }
    }

    /// Set the batch size for output.
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl Operator for Sort {
    fn open(&mut self) -> Result<()> {
        if self.state == OperatorState::Open {
            return Err(ExecutionError::OperatorAlreadyOpen);
        }

        // Open child operator
        self.child.open()?;

        // Read all data from child
        let mut all_rows: Vec<Vec<Value>> = Vec::new();
        let mut all_batches: Vec<Batch> = Vec::new();

        while let Some(batch) = self.child.next_batch()? {
            all_batches.push(batch);
        }

        // Flatten all batches into rows
        for batch in &all_batches {
            for row_idx in 0..batch.row_count() {
                let mut row = Vec::new();
                for col_idx in 0..batch.column_count() {
                    row.push(batch.get(row_idx, col_idx)?);
                }
                all_rows.push(row);
            }
        }

        // Sort the rows
        all_rows.sort_by(|row_a, row_b| {
            for (col_idx, direction) in self.sort_columns.iter().zip(self.sort_directions.iter()) {
                let val_a = &row_a[*col_idx];
                let val_b = &row_b[*col_idx];

                let cmp = match (val_a, val_b) {
                    (Value::Int64(a), Value::Int64(b)) => a.cmp(b),
                    (Value::Float64(a), Value::Float64(b)) => {
                        // Use total_cmp for float comparison to handle NaN properly
                        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::String(a), Value::String(b)) => a.cmp(b),
                    (a, b) => {
                        // Different types - compare data types as fallback
                        a.data_type().cmp(&b.data_type())
                    }
                };

                if cmp != std::cmp::Ordering::Equal {
                    // Reverse if descending
                    return if *direction == SortDirection::Descending {
                        cmp.reverse()
                    } else {
                        cmp
                    };
                }
            }

            std::cmp::Ordering::Equal
        });

        // Convert sorted rows back to columns
        if all_rows.is_empty() {
            self.sorted_data = Some(Batch::empty());
        } else {
            let col_count = all_rows[0].len();
            let mut columns: Vec<Vec<Value>> = vec![Vec::new(); col_count];

            for row in &all_rows {
                for (col_idx, value) in row.iter().enumerate() {
                    columns[col_idx].push(value.clone());
                }
            }

            // Convert to actual column types
            let schema = self.child.schema()?;
            let column_names = self.child.column_names()?;
            let mut final_columns: Vec<Arc<dyn Column>> = Vec::new();

            for (i, col_name) in column_names.iter().enumerate() {
                let data_type = &schema[col_name];
                let values = &columns[i];

                let column: Arc<dyn Column> = match data_type {
                    DataType::Int64 => {
                        let mut int_col = crate::column::IntColumn::new();
                        for value in values {
                            int_col.push_value(value.clone())?;
                        }
                        Arc::new(int_col)
                    }
                    DataType::Float64 => {
                        let mut float_col = crate::column::FloatColumn::new();
                        for value in values {
                            float_col.push_value(value.clone())?;
                        }
                        Arc::new(float_col)
                    }
                    DataType::String => {
                        let mut string_col = crate::column::StringColumn::new();
                        for value in values {
                            string_col.push_value(value.clone())?;
                        }
                        Arc::new(string_col)
                    }
                };

                final_columns.push(column);
            }

            self.sorted_data = Some(Batch::new(final_columns));
        }

        self.current_row = 0;
        self.state = OperatorState::Open;

        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<Batch>> {
        if self.state != OperatorState::Open {
            return Err(ExecutionError::OperatorNotOpen);
        }

        let sorted_data = self.sorted_data.as_ref().unwrap();
        let total_rows = sorted_data.row_count();

        if self.current_row >= total_rows {
            return Ok(None);
        }

        let end_row = std::cmp::min(self.current_row + self.batch_size, total_rows);
        let _batch_size = end_row - self.current_row;

        // Create a batch with the rows for this batch
        let mut batch_columns: Vec<Arc<dyn Column>> = Vec::new();
        let column_count = sorted_data.column_count();

        for col_idx in 0..column_count {
            let original_column = sorted_data.column(col_idx)?;

            // Extract rows for this batch
            let batch_data: Vec<Value> = (self.current_row..end_row)
                .map(|row_idx| {
                    original_column
                        .get(row_idx)
                        .map_err(|e| ExecutionError::Custom(e.to_string()))
                })
                .collect::<Result<Vec<_>>>()?;

            let data_type = original_column.data_type();
            let batch_column: Arc<dyn Column> = match data_type {
                DataType::Int64 => {
                    let mut int_col = crate::column::IntColumn::new();
                    for value in batch_data {
                        int_col.push_value(value)?;
                    }
                    Arc::new(int_col)
                }
                DataType::Float64 => {
                    let mut float_col = crate::column::FloatColumn::new();
                    for value in batch_data {
                        float_col.push_value(value)?;
                    }
                    Arc::new(float_col)
                }
                DataType::String => {
                    let mut string_col = crate::column::StringColumn::new();
                    for value in batch_data {
                        string_col.push_value(value)?;
                    }
                    Arc::new(string_col)
                }
            };

            batch_columns.push(batch_column);
        }

        self.current_row = end_row;

        Ok(Some(Batch::new(batch_columns)))
    }

    fn close(&mut self) -> Result<()> {
        self.state = OperatorState::Closed;
        self.child.close()
    }

    fn schema(&self) -> Result<std::collections::HashMap<String, DataType>> {
        self.child.schema()
    }

    fn column_names(&self) -> Result<Vec<String>> {
        self.child.column_names()
    }

    fn is_open(&self) -> bool {
        self.state == OperatorState::Open
    }
}

// ============================================================================
// LIMIT OPERATOR (LIMIT/OFFSET)
// ============================================================================

/// Limit operator for LIMIT/OFFSET clause.
///
/// Limit operator skips a specified number of rows (OFFSET) and then
/// returns up to a specified number of rows (LIMIT). This operator can
/// stop reading from the child operator once it has read enough rows.
pub struct Limit {
    /// The child operator to read data from
    child: Box<dyn Operator>,

    /// Maximum number of rows to return
    limit: usize,

    /// Number of rows to skip
    offset: usize,

    /// Number of rows already returned
    rows_returned: usize,

    /// Number of rows already skipped
    rows_skipped: usize,

    /// Operator state
    state: OperatorState,
}

impl Limit {
    /// Create a new Limit operator.
    ///
    /// # Arguments
    ///
    /// * `child` - The child operator to read data from
    /// * `limit` - Maximum number of rows to return (if None, return all rows after offset)
    /// * `offset` - Number of rows to skip
    ///
    /// # Example
    ///
    /// ```rust
    /// # use mini_rust_olap::execution::Limit;
    /// # use mini_rust_olap::execution::TableScan;
    /// # use mini_rust_olap::execution::Operator;
    /// # use mini_rust_olap::table::Table;
    /// use std::sync::Arc;
    ///
    /// let table = Table::new("users".to_string());
    /// // ... add columns to table ...
    ///
    /// let scan = TableScan::new(table);
    /// let mut limit = Limit::new(Box::new(scan), Some(5), 10);
    ///
    /// limit.open().unwrap();
    /// while let Some(batch) = limit.next_batch().unwrap() {
    ///     // Process limited batches
    /// }
    /// limit.close().unwrap();
    /// ```
    pub fn new(child: Box<dyn Operator>, limit: Option<usize>, offset: usize) -> Self {
        Limit {
            child,
            limit: limit.unwrap_or(usize::MAX),
            offset,
            rows_returned: 0,
            rows_skipped: 0,
            state: OperatorState::NotOpen,
        }
    }
}

impl Operator for Limit {
    fn open(&mut self) -> Result<()> {
        if self.state == OperatorState::Open {
            return Err(ExecutionError::OperatorAlreadyOpen);
        }

        self.child.open()?;
        self.rows_returned = 0;
        self.rows_skipped = 0;
        self.state = OperatorState::Open;

        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<Batch>> {
        if self.state != OperatorState::Open {
            return Err(ExecutionError::OperatorNotOpen);
        }

        // If we've already returned enough rows, stop
        if self.rows_returned >= self.limit {
            return Ok(None);
        }

        // Get next batch from child
        let mut batch = match self.child.next_batch()? {
            Some(b) => b,
            None => return Ok(None),
        };

        // Apply offset if we haven't skipped enough rows yet
        if self.rows_skipped < self.offset {
            let batch_row_count = batch.row_count();

            if self.rows_skipped + batch_row_count <= self.offset {
                // Entire batch should be skipped
                self.rows_skipped += batch_row_count;
                return self.next_batch();
            } else {
                // Need to skip part of this batch
                let skip_count = self.offset - self.rows_skipped;
                batch = batch.skip_rows(skip_count)?;
                self.rows_skipped += skip_count;
            }
        }

        // Apply limit if this batch would exceed our limit
        let remaining_limit = self.limit - self.rows_returned;
        if batch.row_count() > remaining_limit {
            batch = batch.take_rows(remaining_limit)?;
        }

        self.rows_returned += batch.row_count();

        if batch.is_empty() {
            Ok(None)
        } else {
            Ok(Some(batch))
        }
    }

    fn close(&mut self) -> Result<()> {
        self.state = OperatorState::Closed;
        self.child.close()
    }

    fn schema(&self) -> Result<std::collections::HashMap<String, DataType>> {
        self.child.schema()
    }

    fn column_names(&self) -> Result<Vec<String>> {
        self.child.column_names()
    }

    fn is_open(&self) -> bool {
        self.state == OperatorState::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregates::*;
    use crate::column::{FloatColumn, IntColumn, StringColumn};
    use crate::types::{DataType, Value};

    #[test]
    fn test_batch_creation() {
        let mut col1 = IntColumn::new();
        col1.push_value(Value::Int64(1)).unwrap();
        col1.push_value(Value::Int64(2)).unwrap();
        col1.push_value(Value::Int64(3)).unwrap();

        let mut col2 = FloatColumn::new();
        col2.push_value(Value::Float64(1.0)).unwrap();
        col2.push_value(Value::Float64(2.0)).unwrap();
        col2.push_value(Value::Float64(3.0)).unwrap();

        let batch = Batch::new(vec![Arc::new(col1), Arc::new(col2)]);

        assert_eq!(batch.row_count(), 3);
        assert_eq!(batch.column_count(), 2);
    }

    #[test]
    #[should_panic(expected = "Cannot create a batch with no columns")]
    fn test_batch_empty_columns() {
        let _batch = Batch::new(vec![]);
    }

    #[test]
    #[should_panic(expected = "Column 1 has 5 rows, but column 0 has 3 rows")]
    fn test_batch_mismatched_lengths() {
        let mut col1 = IntColumn::new();
        col1.push_value(Value::Int64(1)).unwrap();
        col1.push_value(Value::Int64(2)).unwrap();
        col1.push_value(Value::Int64(3)).unwrap();

        let mut col2 = IntColumn::new();
        col2.push_value(Value::Int64(1)).unwrap();
        col2.push_value(Value::Int64(2)).unwrap();
        col2.push_value(Value::Int64(3)).unwrap();
        col2.push_value(Value::Int64(4)).unwrap();
        col2.push_value(Value::Int64(5)).unwrap();

        let _batch = Batch::new(vec![Arc::new(col1), Arc::new(col2)]);
    }

    #[test]
    fn test_batch_column_access() {
        let mut col1 = IntColumn::new();
        col1.push_value(Value::Int64(1)).unwrap();
        col1.push_value(Value::Int64(2)).unwrap();
        col1.push_value(Value::Int64(3)).unwrap();

        let mut col2 = StringColumn::new();
        col2.push_value(Value::String("a".to_string())).unwrap();
        col2.push_value(Value::String("b".to_string())).unwrap();
        col2.push_value(Value::String("c".to_string())).unwrap();

        let batch = Batch::new(vec![Arc::new(col1), Arc::new(col2)]);

        let result = batch.column(0);
        assert!(result.is_ok());

        let result = batch.column(2);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ExecutionError::InvalidColumnIndex { index: 2, count: 2 })
        ));
    }

    #[test]
    fn test_batch_get_value() {
        let mut col1 = IntColumn::new();
        col1.push_value(Value::Int64(10)).unwrap();
        col1.push_value(Value::Int64(20)).unwrap();
        col1.push_value(Value::Int64(30)).unwrap();

        let mut col2 = StringColumn::new();
        col2.push_value(Value::String("x".to_string())).unwrap();
        col2.push_value(Value::String("y".to_string())).unwrap();
        col2.push_value(Value::String("z".to_string())).unwrap();

        let batch = Batch::new(vec![Arc::new(col1), Arc::new(col2)]);

        // Valid access - returns Value enum
        let result = batch.get(1, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int64(20));

        let result = batch.get(2, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("z".to_string()));

        // Test get_as_string
        let result = batch.get_as_string(1, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "20");

        // Invalid row index
        let result = batch.get(5, 0);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ExecutionError::InvalidRowIndex { index: 5, count: 3 })
        ));

        // Invalid column index
        let result = batch.get(0, 5);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ExecutionError::InvalidColumnIndex { index: 5, count: 2 })
        ));
    }

    #[test]
    fn test_batch_get_as_string() {
        let mut col1 = StringColumn::new();
        col1.push_value(Value::String("hello".to_string())).unwrap();
        col1.push_value(Value::String("world".to_string())).unwrap();
        col1.push_value(Value::String("test".to_string())).unwrap();

        let batch = Batch::new(vec![Arc::new(col1)]);

        let result = batch.get_as_string(1, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "world");
    }

    #[test]
    fn test_batch_select() {
        let mut col1 = IntColumn::new();
        col1.push_value(Value::Int64(1)).unwrap();
        col1.push_value(Value::Int64(2)).unwrap();
        col1.push_value(Value::Int64(3)).unwrap();

        let mut col2 = StringColumn::new();
        col2.push_value(Value::String("a".to_string())).unwrap();
        col2.push_value(Value::String("b".to_string())).unwrap();
        col2.push_value(Value::String("c".to_string())).unwrap();

        let mut col3 = FloatColumn::new();
        col3.push_value(Value::Float64(1.0)).unwrap();
        col3.push_value(Value::Float64(2.0)).unwrap();
        col3.push_value(Value::Float64(3.0)).unwrap();

        let batch = Batch::new(vec![Arc::new(col1), Arc::new(col2), Arc::new(col3)]);

        let selected = batch.select(&[0, 2]).unwrap();

        assert_eq!(selected.row_count(), 3);
        assert_eq!(selected.column_count(), 2);

        let val = selected.get(0, 0).unwrap();
        assert_eq!(val, Value::Int64(1));

        let val = selected.get(1, 1).unwrap();
        assert_eq!(val, Value::Float64(2.0));
    }

    #[test]
    fn test_batch_select_invalid_index() {
        let mut col1 = IntColumn::new();
        col1.push_value(Value::Int64(1)).unwrap();
        col1.push_value(Value::Int64(2)).unwrap();
        col1.push_value(Value::Int64(3)).unwrap();

        let batch = Batch::new(vec![Arc::new(col1)]);

        let result = batch.select(&[0, 5]);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ExecutionError::InvalidColumnIndex { index: 5, count: 1 })
        ));
    }

    #[test]
    fn test_batch_empty() {
        let batch = Batch::empty();

        assert!(batch.is_empty());
        assert_eq!(batch.row_count(), 0);
    }

    #[test]
    fn test_execution_error_display() {
        let err = ExecutionError::OperatorNotOpen;
        assert_eq!(
            err.to_string(),
            "Operator must be opened before calling this operation"
        );

        let err = ExecutionError::InvalidColumnIndex { index: 5, count: 3 };
        assert_eq!(
            err.to_string(),
            "Invalid column index 5 (only 3 columns available)"
        );

        let err = ExecutionError::ColumnNotFound("age".to_string());
        assert_eq!(err.to_string(), "Column 'age' not found in batch");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let exec_err: ExecutionError = io_err.into();

        assert!(matches!(exec_err, ExecutionError::IoError(_)));
        assert_eq!(
            exec_err.to_string(),
            "IO error during execution: file not found"
        );
    }

    #[test]
    fn test_batch_debug() {
        let mut col1 = IntColumn::new();
        col1.push_value(Value::Int64(1)).unwrap();
        col1.push_value(Value::Int64(2)).unwrap();
        col1.push_value(Value::Int64(3)).unwrap();

        let mut col2 = StringColumn::new();
        col2.push_value(Value::String("a".to_string())).unwrap();
        col2.push_value(Value::String("b".to_string())).unwrap();
        col2.push_value(Value::String("c".to_string())).unwrap();

        let batch = Batch::new(vec![Arc::new(col1), Arc::new(col2)]);

        let debug_str = format!("{:?}", batch);
        assert!(debug_str.contains("Batch"));
        assert!(debug_str.contains("row_count: 3"));
        assert!(debug_str.contains("column_count: 2"));
    }

    // Simple mock operator for testing
    struct MockOperator {
        state: OperatorState,
    }

    impl Operator for MockOperator {
        fn open(&mut self) -> Result<()> {
            self.state = OperatorState::Open;
            Ok(())
        }

        fn next_batch(&mut self) -> Result<Option<Batch>> {
            if self.state != OperatorState::Open {
                return Err(ExecutionError::OperatorNotOpen);
            }
            Ok(None)
        }

        fn close(&mut self) -> Result<()> {
            self.state = OperatorState::Closed;
            Ok(())
        }

        fn schema(&self) -> Result<HashMap<String, DataType>> {
            Err(ExecutionError::Custom(
                "Mock operator has no schema".to_string(),
            ))
        }

        fn column_names(&self) -> Result<Vec<String>> {
            Err(ExecutionError::Custom(
                "Mock operator has no column names".to_string(),
            ))
        }

        fn is_open(&self) -> bool {
            self.state == OperatorState::Open
        }
    }

    #[test]
    fn test_operator_lifecycle() {
        let mut op = MockOperator {
            state: OperatorState::NotOpen,
        };

        assert!(!op.is_open());

        // Open
        assert!(op.open().is_ok());
        assert!(op.is_open());

        // Get batch
        assert!(op.next_batch().is_ok());

        // Close
        assert!(op.close().is_ok());
        assert!(!op.is_open());
    }

    #[test]
    fn test_operator_not_open_error() {
        let mut op = MockOperator {
            state: OperatorState::NotOpen,
        };

        let result = op.next_batch();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::OperatorNotOpen)));
    }

    // TableScan Operator Tests
    #[test]
    fn test_table_scan_create() {
        let table = create_test_table();
        let scan = TableScan::new(table.clone());

        assert_eq!(scan.total_rows, 5);
        assert_eq!(scan.column_indices.len(), 3); // All columns
        assert_eq!(scan.batch_size, 1024);
    }

    #[test]
    fn test_table_scan_with_columns() {
        let table = create_test_table();
        let scan = TableScan::with_columns(table.clone(), vec![0, 2]);

        assert_eq!(scan.column_indices, vec![0, 2]);
        assert_eq!(scan.total_rows, 5);
    }

    #[test]
    #[should_panic(expected = "Batch size must be greater than 0")]
    fn test_table_scan_invalid_batch_size() {
        let table = create_test_table();
        let _scan = TableScan::new(table).with_batch_size(0);
    }

    #[test]
    fn test_table_scan_lifecycle() {
        let table = create_test_table();
        let mut scan = TableScan::new(table);

        assert!(!scan.is_open());

        // Open
        assert!(scan.open().is_ok());
        assert!(scan.is_open());

        // Get batches
        let batch1 = scan.next_batch();
        assert!(batch1.is_ok());
        assert!(batch1.unwrap().is_some());

        // Should return None after exhausting data
        while scan.next_batch().unwrap().is_some() {
            // Keep consuming batches
        }

        // Close
        assert!(scan.close().is_ok());
        assert!(!scan.is_open());
    }

    #[test]
    fn test_table_scan_single_batch() {
        let table = create_test_table();
        let mut scan = TableScan::new(table).with_batch_size(10);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 5);
        assert_eq!(batch.column_count(), 3);

        // Verify data
        let val = batch.get(0, 0).unwrap();
        assert_eq!(val, Value::Int64(1));

        let val = batch.get(1, 1).unwrap();
        assert_eq!(val, Value::String("Bob".to_string()));

        let val = batch.get(2, 2).unwrap();
        assert_eq!(val, Value::Float64(35.0));

        // Next batch should be None
        let batch = scan.next_batch().unwrap();
        assert!(batch.is_none());

        scan.close().unwrap();
    }

    #[test]
    fn test_table_scan_multiple_batches() {
        let table = create_test_large_table(2500); // 2500 rows
        let mut scan = TableScan::new(table).with_batch_size(1000);

        scan.open().unwrap();

        let mut total_rows = 0;
        let mut batch_count = 0;

        loop {
            let batch = scan.next_batch().unwrap();
            if let Some(batch) = batch {
                batch_count += 1;
                total_rows += batch.row_count();
            } else {
                break;
            }
        }

        assert_eq!(batch_count, 3); // 1000 + 1000 + 500 = 2500
        assert_eq!(total_rows, 2500);

        scan.close().unwrap();
    }

    #[test]
    fn test_table_scan_column_pruning() {
        let table = create_test_table();
        let mut scan = TableScan::with_columns(table, vec![0, 2]); // Only id and age

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.column_count(), 2);
        assert_eq!(batch.row_count(), 5);

        // Verify we have the right columns
        let val = batch.get(0, 0).unwrap();
        assert_eq!(val, Value::Int64(1)); // id

        let val = batch.get(0, 1).unwrap();
        assert_eq!(val, Value::Float64(25.0)); // age

        scan.close().unwrap();
    }

    #[test]
    fn test_table_scan_schema() {
        let table = create_test_table();
        let mut scan = TableScan::new(table);

        // Schema not available before open
        assert!(scan.schema().is_err());

        scan.open().unwrap();

        let schema = scan.schema().unwrap();
        assert_eq!(schema.len(), 3);
        assert_eq!(schema.get("id"), Some(&DataType::Int64));
        assert_eq!(schema.get("name"), Some(&DataType::String));
        assert_eq!(schema.get("age"), Some(&DataType::Float64));

        scan.close().unwrap();
    }

    #[test]
    fn test_table_scan_pruned_schema() {
        let table = create_test_table();
        let mut scan = TableScan::with_columns(table, vec![0, 1]);

        scan.open().unwrap();

        let schema = scan.schema().unwrap();
        assert_eq!(schema.len(), 2);
        assert_eq!(schema.get("id"), Some(&DataType::Int64));
        assert_eq!(schema.get("name"), Some(&DataType::String));
        assert_eq!(schema.get("age"), None);

        scan.close().unwrap();
    }

    #[test]
    fn test_table_scan_invalid_column_index() {
        let table = create_test_table();
        let mut scan = TableScan::with_columns(table, vec![0, 5]); // Column 5 doesn't exist

        let result = scan.open();
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ExecutionError::InvalidColumnIndex { index: 5, count: 3 })
        ));
    }

    #[test]
    fn test_table_scan_next_batch_not_open() {
        let table = create_test_table();
        let mut scan = TableScan::new(table);

        let result = scan.next_batch();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::OperatorNotOpen)));
    }

    #[test]
    fn test_table_scan_double_open() {
        let table = create_test_table();
        let mut scan = TableScan::new(table);

        scan.open().unwrap();
        let result = scan.open();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::OperatorAlreadyOpen)));
    }

    #[test]
    fn test_table_scan_empty_table() {
        let table = Table::new("empty".to_string());
        let mut scan = TableScan::new(table);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap();
        assert!(batch.is_none());

        scan.close().unwrap();
    }

    #[test]
    fn test_table_scan_single_row_table() {
        let mut table = Table::new("single_row".to_string());

        let mut id_col = IntColumn::new();
        id_col.push_value(Value::Int64(42)).unwrap();
        table
            .add_column("id".to_string(), Box::new(id_col))
            .unwrap();

        let mut scan = TableScan::new(table);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 1);
        assert_eq!(batch.column_count(), 1);

        let val = batch.get(0, 0).unwrap();
        assert_eq!(val, Value::Int64(42));

        let batch = scan.next_batch().unwrap();
        assert!(batch.is_none());

        scan.close().unwrap();
    }

    // Predicate and Filter Operator Tests
    #[test]
    fn test_binary_comparison_equal() {
        let table = create_test_table();
        let mut scan = TableScan::new(table).with_batch_size(10);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap().unwrap();

        // Test equals with integer
        let pred = BinaryComparison::new(0, ComparisonOp::Equal, Value::Int64(1));
        assert!(pred.eval(&batch, 0).unwrap()); // row 0, id=1
        assert!(!pred.eval(&batch, 1).unwrap()); // row 1, id=2

        // Test equals with string
        let pred =
            BinaryComparison::new(1, ComparisonOp::Equal, Value::String("Alice".to_string()));
        assert!(pred.eval(&batch, 0).unwrap()); // row 0, name=Alice
        assert!(!pred.eval(&batch, 1).unwrap()); // row 1, name=Bob

        scan.close().unwrap();
    }

    #[test]
    fn test_binary_comparison_not_equal() {
        let table = create_test_table();
        let mut scan = TableScan::new(table).with_batch_size(10);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap().unwrap();

        // Test not equals with float
        let pred = BinaryComparison::new(2, ComparisonOp::NotEqual, Value::Float64(25.0));
        assert!(!pred.eval(&batch, 0).unwrap()); // row 0, age=25.0
        assert!(pred.eval(&batch, 1).unwrap()); // row 1, age=30.0

        scan.close().unwrap();
    }

    #[test]
    fn test_binary_comparison_less_than() {
        let table = create_test_table();
        let mut scan = TableScan::new(table).with_batch_size(10);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap().unwrap();

        // Test less than with integer
        let pred = BinaryComparison::new(0, ComparisonOp::LessThan, Value::Int64(3));
        assert!(pred.eval(&batch, 0).unwrap()); // row 0, id=1
        assert!(pred.eval(&batch, 1).unwrap()); // row 1, id=2
        assert!(!pred.eval(&batch, 2).unwrap()); // row 2, id=3

        scan.close().unwrap();
    }

    #[test]
    fn test_binary_comparison_greater_than() {
        let table = create_test_table();
        let mut scan = TableScan::new(table).with_batch_size(10);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap().unwrap();

        // Test greater than with float
        let pred = BinaryComparison::new(2, ComparisonOp::GreaterThan, Value::Float64(30.0));
        assert!(!pred.eval(&batch, 0).unwrap()); // row 0, age=25.0
        assert!(!pred.eval(&batch, 1).unwrap()); // row 1, age=30.0
        assert!(pred.eval(&batch, 2).unwrap()); // row 2, age=35.0

        scan.close().unwrap();
    }

    #[test]
    fn test_binary_comparison_less_than_or_equal() {
        let table = create_test_table();
        let mut scan = TableScan::new(table).with_batch_size(10);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap().unwrap();

        // Test less than or equal with integer
        let pred = BinaryComparison::new(0, ComparisonOp::LessThanOrEqual, Value::Int64(2));
        assert!(pred.eval(&batch, 0).unwrap()); // row 0, id=1
        assert!(pred.eval(&batch, 1).unwrap()); // row 1, id=2
        assert!(!pred.eval(&batch, 2).unwrap()); // row 2, id=3

        scan.close().unwrap();
    }

    #[test]
    fn test_binary_comparison_greater_than_or_equal() {
        let table = create_test_table();
        let mut scan = TableScan::new(table).with_batch_size(10);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap().unwrap();

        // Test greater than or equal with float
        let pred = BinaryComparison::new(2, ComparisonOp::GreaterThanOrEqual, Value::Float64(35.0));
        assert!(!pred.eval(&batch, 0).unwrap()); // row 0, age=25.0
        assert!(!pred.eval(&batch, 1).unwrap()); // row 1, age=30.0
        assert!(pred.eval(&batch, 2).unwrap()); // row 2, age=35.0
        assert!(pred.eval(&batch, 3).unwrap()); // row 3, age=40.0

        scan.close().unwrap();
    }

    #[test]
    fn test_comparison_op_display() {
        assert_eq!(format!("{}", ComparisonOp::Equal), "=");
        assert_eq!(format!("{}", ComparisonOp::NotEqual), "!=");
        assert_eq!(format!("{}", ComparisonOp::LessThan), "<");
        assert_eq!(format!("{}", ComparisonOp::LessThanOrEqual), "<=");
        assert_eq!(format!("{}", ComparisonOp::GreaterThan), ">");
        assert_eq!(format!("{}", ComparisonOp::GreaterThanOrEqual), ">=");
    }

    #[test]
    fn test_and_predicate() {
        let table = create_test_table();
        let mut scan = TableScan::new(table).with_batch_size(10);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap().unwrap();

        // Test: id > 1 AND id < 4
        let pred1 = Arc::new(BinaryComparison::new(
            0,
            ComparisonOp::GreaterThan,
            Value::Int64(1),
        ));
        let pred2 = Arc::new(BinaryComparison::new(
            0,
            ComparisonOp::LessThan,
            Value::Int64(4),
        ));
        let and_pred = And::new(pred1, pred2);

        assert!(!and_pred.eval(&batch, 0).unwrap()); // id=1 (not > 1)
        assert!(and_pred.eval(&batch, 1).unwrap()); // id=2 (1 < 2 < 4)
        assert!(and_pred.eval(&batch, 2).unwrap()); // id=3 (1 < 3 < 4)
        assert!(!and_pred.eval(&batch, 3).unwrap()); // id=4 (not < 4)

        scan.close().unwrap();
    }

    #[test]
    fn test_or_predicate() {
        let table = create_test_table();
        let mut scan = TableScan::new(table).with_batch_size(10);

        scan.open().unwrap();

        let batch = scan.next_batch().unwrap().unwrap();

        // Test: age < 30 OR age > 40
        let pred1 = Arc::new(BinaryComparison::new(
            2,
            ComparisonOp::LessThan,
            Value::Float64(30.0),
        ));
        let pred2 = Arc::new(BinaryComparison::new(
            2,
            ComparisonOp::GreaterThan,
            Value::Float64(40.0),
        ));
        let or_pred = Or::new(pred1, pred2);

        assert!(or_pred.eval(&batch, 0).unwrap()); // age=25.0 (< 30)
        assert!(!or_pred.eval(&batch, 1).unwrap()); // age=30.0 (not < 30 and not > 40)
        assert!(!or_pred.eval(&batch, 2).unwrap()); // age=35.0 (not < 30 and not > 40)
        assert!(!or_pred.eval(&batch, 3).unwrap()); // age=40.0 (not < 30 and not > 40)
        assert!(or_pred.eval(&batch, 4).unwrap()); // age=45.0 (> 40)

        scan.close().unwrap();
    }

    #[test]
    fn test_filter_simple() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table).with_batch_size(10));

        // Filter: age > 30
        let predicate = Arc::new(BinaryComparison::new(
            2,
            ComparisonOp::GreaterThan,
            Value::Float64(30.0),
        ));

        let mut filter = Filter::new(scan, predicate);

        filter.open().unwrap();

        let batch = filter.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        // Should only have rows with age > 30: rows 2, 3, 4 (ages 35, 40, 45)
        assert_eq!(batch.row_count(), 3);
        assert_eq!(batch.column_count(), 3);

        // Verify first row (id=3, name=Charlie, age=35.0)
        let val = batch.get(0, 0).unwrap();
        assert_eq!(val, Value::Int64(3));

        // Verify last row (id=5, name=Eve, age=45.0)
        let val = batch.get(2, 0).unwrap();
        assert_eq!(val, Value::Int64(5));

        filter.close().unwrap();
    }

    #[test]
    fn test_filter_with_and() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table).with_batch_size(10));

        // Filter: id >= 2 AND id <= 4
        let pred1 = Arc::new(BinaryComparison::new(
            0,
            ComparisonOp::GreaterThanOrEqual,
            Value::Int64(2),
        ));
        let pred2 = Arc::new(BinaryComparison::new(
            0,
            ComparisonOp::LessThanOrEqual,
            Value::Int64(4),
        ));
        let predicate = Arc::new(And::new(pred1, pred2));

        let mut filter = Filter::new(scan, predicate);

        filter.open().unwrap();

        let batch = filter.next_batch().unwrap().unwrap();
        // Should have rows 1, 2, 3 (ids 2, 3, 4)
        assert_eq!(batch.row_count(), 3);

        filter.close().unwrap();
    }

    #[test]
    fn test_filter_with_or() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table).with_batch_size(10));

        // Filter: name = "Alice" OR name = "Eve"
        let pred1 = Arc::new(BinaryComparison::new(
            1,
            ComparisonOp::Equal,
            Value::String("Alice".to_string()),
        ));
        let pred2 = Arc::new(BinaryComparison::new(
            1,
            ComparisonOp::Equal,
            Value::String("Eve".to_string()),
        ));
        let predicate = Arc::new(Or::new(pred1, pred2));

        let mut filter = Filter::new(scan, predicate);

        filter.open().unwrap();

        let batch = filter.next_batch().unwrap().unwrap();
        // Should have rows 0 and 4 (Alice and Eve)
        assert_eq!(batch.row_count(), 2);

        let val = batch.get(0, 1).unwrap();
        assert_eq!(val, Value::String("Alice".to_string()));

        let val = batch.get(1, 1).unwrap();
        assert_eq!(val, Value::String("Eve".to_string()));

        filter.close().unwrap();
    }

    #[test]
    fn test_filter_empty_result() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table).with_batch_size(10));

        // Filter: id > 100 (no rows match)
        let predicate = Arc::new(BinaryComparison::new(
            0,
            ComparisonOp::GreaterThan,
            Value::Int64(100),
        ));

        let mut filter = Filter::new(scan, predicate);

        filter.open().unwrap();

        let batch = filter.next_batch().unwrap();
        // Should return None immediately since no rows match
        assert!(batch.is_none());

        filter.close().unwrap();
    }

    #[test]
    fn test_filter_all_rows_pass() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table).with_batch_size(10));

        // Filter: id >= 1 (all rows pass)
        let predicate = Arc::new(BinaryComparison::new(
            0,
            ComparisonOp::GreaterThanOrEqual,
            Value::Int64(1),
        ));

        let mut filter = Filter::new(scan, predicate);

        filter.open().unwrap();

        let batch = filter.next_batch().unwrap().unwrap();
        // Should have all 5 rows
        assert_eq!(batch.row_count(), 5);

        filter.close().unwrap();
    }

    #[test]
    fn test_filter_multiple_batches() {
        let table = create_test_large_table(2500);
        let scan = Box::new(TableScan::new(table).with_batch_size(1000));

        // Filter: id >= 1000 (only second and third batches)
        let predicate = Arc::new(BinaryComparison::new(
            0,
            ComparisonOp::GreaterThanOrEqual,
            Value::Int64(1000),
        ));

        let mut filter = Filter::new(scan, predicate);

        filter.open().unwrap();

        let mut total_rows = 0;
        while let Some(batch) = filter.next_batch().unwrap() {
            total_rows += batch.row_count();
        }

        // Should have 1500 rows (ids 1000-2499)
        assert_eq!(total_rows, 1500);

        filter.close().unwrap();
    }

    #[test]
    fn test_filter_schema() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let predicate = Arc::new(BinaryComparison::new(
            0,
            ComparisonOp::GreaterThan,
            Value::Int64(0),
        ));

        let mut filter = Filter::new(scan, predicate);

        // Schema not available before open
        assert!(filter.schema().is_err());

        filter.open().unwrap();

        // Schema should match child's schema
        let schema = filter.schema().unwrap();
        assert_eq!(schema.len(), 3);
        assert_eq!(schema.get("id"), Some(&DataType::Int64));
        assert_eq!(schema.get("name"), Some(&DataType::String));
        assert_eq!(schema.get("age"), Some(&DataType::Float64));

        filter.close().unwrap();
    }

    #[test]
    fn test_filter_lifecycle() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let predicate = Arc::new(BinaryComparison::new(
            0,
            ComparisonOp::GreaterThan,
            Value::Int64(2),
        ));

        let mut filter = Filter::new(scan, predicate);

        assert!(!filter.is_open());

        filter.open().unwrap();
        assert!(filter.is_open());

        // Get batches
        while filter.next_batch().unwrap().is_some() {
            // Keep consuming
        }

        filter.close().unwrap();
        assert!(!filter.is_open());
    }

    #[test]
    fn test_filter_next_batch_not_open() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let predicate = Arc::new(BinaryComparison::new(
            0,
            ComparisonOp::GreaterThan,
            Value::Int64(0),
        ));

        let mut filter = Filter::new(scan, predicate);

        let result = filter.next_batch();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::OperatorNotOpen)));
    }
    // ============================================================================
    // PROJECT TESTS
    // ============================================================================

    #[test]
    fn test_project_basic() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table).with_batch_size(10));

        // Project only id and name columns (indices 0 and 1)
        let mut project = Box::new(Project::new(scan, vec![0, 1]));

        project.open().unwrap();

        let batch = project.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 5);
        assert_eq!(batch.column_count(), 2);

        // Verify column names
        let schema = project.schema().unwrap();
        assert_eq!(schema.len(), 2);
        assert!(schema.contains_key("id"));
        assert!(schema.contains_key("name"));

        // Verify data
        let val = batch.get(0, 0).unwrap();
        assert_eq!(val, Value::Int64(1));
        let val = batch.get(0, 1).unwrap();
        assert_eq!(val, Value::String("Alice".to_string()));

        // No more batches
        assert!(project.next_batch().unwrap().is_none());

        project.close().unwrap();
    }

    #[test]
    fn test_project_with_aliases() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table).with_batch_size(10));

        // Project id and name with aliases
        let mut project = Box::new(
            Project::new(scan, vec![0, 1])
                .with_aliases(vec!["user_id".to_string(), "user_name".to_string()]),
        );

        project.open().unwrap();

        // Verify schema has aliased names
        let schema = project.schema().unwrap();
        assert_eq!(schema.len(), 2);
        assert!(schema.contains_key("user_id"));
        assert!(schema.contains_key("user_name"));
        assert!(!schema.contains_key("id"));
        assert!(!schema.contains_key("name"));

        let batch = project.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 5);
        assert_eq!(batch.column_count(), 2);

        // Verify data is still accessible
        let val = batch.get(0, 0).unwrap();
        assert_eq!(val, Value::Int64(1));
        let val = batch.get(0, 1).unwrap();
        assert_eq!(val, Value::String("Alice".to_string()));

        project.close().unwrap();
    }

    #[test]
    fn test_project_column_reordering() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table).with_batch_size(10));

        // Project in reverse order: name, age, id
        let mut project = Box::new(Project::new(scan, vec![1, 2, 0]));

        project.open().unwrap();

        let batch = project.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 5);
        assert_eq!(batch.column_count(), 3);

        // Verify column order
        let columns = project.column_names().unwrap();
        assert_eq!(columns, vec!["name", "age", "id"]);

        // Verify data order
        let val = batch.get(0, 0).unwrap();
        assert_eq!(val, Value::String("Alice".to_string()));
        let val = batch.get(0, 1).unwrap();
        assert_eq!(val, Value::Float64(25.0));
        let val = batch.get(0, 2).unwrap();
        assert_eq!(val, Value::Int64(1));

        project.close().unwrap();
    }

    #[test]
    fn test_project_single_column() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table).with_batch_size(10));

        // Project only name column
        let mut project = Box::new(Project::new(scan, vec![1]));

        project.open().unwrap();

        let batch = project.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 5);
        assert_eq!(batch.column_count(), 1);

        let schema = project.schema().unwrap();
        assert_eq!(schema.len(), 1);
        assert!(schema.contains_key("name"));

        // Verify data
        for i in 0..5 {
            let val = batch.get(i, 0).unwrap();
            assert!(matches!(val, Value::String(_)));
        }

        project.close().unwrap();
    }

    #[test]
    fn test_project_invalid_column_index() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        // Try to project column index 10 (out of range)
        let mut project = Box::new(Project::new(scan, vec![10]));

        let result = project.open();
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ExecutionError::InvalidColumnIndex { .. })
        ));
    }

    #[test]
    fn test_project_insufficient_aliases() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        // Provide only 1 alias for 2 columns
        let mut project =
            Box::new(Project::new(scan, vec![0, 1]).with_aliases(vec!["user_id".to_string()]));

        let result = project.open();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::Custom(_))));
    }

    #[test]
    fn test_project_duplicate_column_names() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        // Create duplicate names via aliases
        let mut project = Box::new(
            Project::new(scan, vec![0, 1])
                .with_aliases(vec!["col1".to_string(), "col1".to_string()]),
        );

        let result = project.open();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::Custom(_))));
    }

    #[test]
    fn test_project_duplicate_original_names() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        // Project id twice (without aliases)
        let mut project = Box::new(Project::new(scan, vec![0, 0]));

        let result = project.open();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::Custom(_))));
    }

    #[test]
    fn test_project_schema() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let mut project = Box::new(Project::new(scan, vec![0, 2]));

        // Schema not available before open
        assert!(project.schema().is_err());

        project.open().unwrap();

        let schema = project.schema().unwrap();
        assert_eq!(schema.len(), 2);
        assert_eq!(schema.get("id"), Some(&DataType::Int64));
        assert_eq!(schema.get("age"), Some(&DataType::Float64));
        assert_eq!(schema.get("name"), None);

        project.close().unwrap();
    }

    #[test]
    fn test_project_lifecycle() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let mut project = Box::new(Project::new(scan, vec![0]));

        assert!(!project.is_open());

        project.open().unwrap();
        assert!(project.is_open());

        // Get batches
        while project.next_batch().unwrap().is_some() {
            // Keep consuming
        }

        project.close().unwrap();
        assert!(!project.is_open());
    }

    #[test]
    fn test_project_next_batch_not_open() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let mut project = Box::new(Project::new(scan, vec![0]));

        let result = project.next_batch();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::OperatorNotOpen)));
    }

    #[test]
    fn test_project_multiple_batches() {
        let table = create_test_large_table(2500);
        let scan = Box::new(TableScan::new(table).with_batch_size(1000));

        let mut project = Box::new(Project::new(scan, vec![0, 1]));

        project.open().unwrap();

        let mut total_rows = 0;
        let mut batch_count = 0;

        while let Some(batch) = project.next_batch().unwrap() {
            batch_count += 1;
            total_rows += batch.row_count();
            assert_eq!(batch.column_count(), 2);
        }

        assert_eq!(total_rows, 2500);
        assert_eq!(batch_count, 3); // 1000 + 1000 + 500

        project.close().unwrap();
    }

    #[test]
    fn test_project_empty_table() {
        let table = Table::new("empty".to_string());
        let scan = Box::new(TableScan::new(table));

        let mut project = Box::new(Project::new(scan, vec![]));

        project.open().unwrap();

        // Should get no batches
        let batch = project.next_batch().unwrap();
        assert!(batch.is_none());

        project.close().unwrap();
    }

    #[test]
    fn test_project_single_row_table() {
        let mut table = Table::new("single".to_string());

        let mut id_col = IntColumn::new();
        id_col.push_value(Value::Int64(1)).unwrap();
        table
            .add_column("id".to_string(), Box::new(id_col))
            .unwrap();

        let mut name_col = StringColumn::new();
        name_col
            .push_value(Value::String("Single".to_string()))
            .unwrap();
        table
            .add_column("name".to_string(), Box::new(name_col))
            .unwrap();

        let scan = Box::new(TableScan::new(table));
        let mut project = Box::new(Project::new(scan, vec![1, 0]));

        project.open().unwrap();

        let batch = project.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 1);
        assert_eq!(batch.column_count(), 2);

        let val = batch.get(0, 0).unwrap();
        assert_eq!(val, Value::String("Single".to_string()));
        let val = batch.get(0, 1).unwrap();
        assert_eq!(val, Value::Int64(1));

        assert!(project.next_batch().unwrap().is_none());

        project.close().unwrap();
    }

    #[test]
    fn test_project_double_open() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let mut project = Box::new(Project::new(scan, vec![0]));

        project.open().unwrap();
        let result = project.open();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::OperatorAlreadyOpen)));

        project.close().unwrap();
    }

    // ============================================================================
    // GROUP BY TESTS
    // ============================================================================

    #[test]
    fn test_group_by_basic() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        // Group by name (column 1), count occurrences (column 0)
        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![1], // Group by name
            vec![0], // Aggregate id column
            vec![Box::new(CountAggregate::new(DataType::Int64))],
        ));

        group_by.open().unwrap();

        let batch = group_by.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 5); // 5 unique names
        assert_eq!(batch.column_count(), 2); // name + count

        // Verify schema
        let schema = group_by.schema().unwrap();
        assert_eq!(schema.len(), 2);
        assert_eq!(schema.get("name"), Some(&DataType::String));
        assert_eq!(schema.get("agg_0"), Some(&DataType::Int64));

        // Verify column names
        let column_names = group_by.column_names().unwrap();
        assert_eq!(column_names, vec!["name".to_string(), "agg_0".to_string()]);

        // Verify data - each name should have count 1
        for i in 0..batch.row_count() {
            let _name = batch.get(i, 0).unwrap();
            let count = batch.get(i, 1).unwrap();
            assert_eq!(count, Value::Int64(1));
        }

        // No more batches
        assert!(group_by.next_batch().unwrap().is_none());

        group_by.close().unwrap();
    }

    #[test]
    fn test_group_by_multiple_groups() {
        let mut table = Table::new("test".to_string());

        // Add category column
        let mut cat_col = StringColumn::new();
        for _ in 0..9 {
            cat_col.push_value(Value::String("A".to_string())).unwrap();
        }
        table
            .add_column("category".to_string(), Box::new(cat_col))
            .unwrap();

        // Add value column
        let mut val_col = IntColumn::new();
        for i in 0..9 {
            val_col.push_value(Value::Int64(i)).unwrap();
        }
        table
            .add_column("value".to_string(), Box::new(val_col))
            .unwrap();

        let scan = Box::new(TableScan::new(table));

        // Group by category, sum value
        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0], // Group by category
            vec![1], // Aggregate value
            vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
        ));

        group_by.open().unwrap();

        let batch = group_by.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 1); // Only 1 group (all "A")
        assert_eq!(batch.get(0, 0).unwrap(), Value::String("A".to_string()));
        assert_eq!(batch.get(0, 1).unwrap(), Value::Int64(36)); // Sum of 0-8

        group_by.close().unwrap();
    }

    #[test]
    fn test_group_by_count_aggregate() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![2], // Group by age
            vec![0], // Count id
            vec![Box::new(CountAggregate::new(DataType::Int64))],
        ));

        group_by.open().unwrap();

        let batch = group_by.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 5); // 5 unique ages
        assert_eq!(batch.column_count(), 2);

        // Each age appears once
        for i in 0..batch.row_count() {
            let count = batch.get(i, 1).unwrap();
            assert_eq!(count, Value::Int64(1));
        }

        group_by.close().unwrap();
    }

    #[test]
    fn test_group_by_sum_aggregate() {
        let mut table = Table::new("test".to_string());

        // Add group column
        let mut group_col = IntColumn::new();
        for _ in 0..6 {
            group_col.push_value(Value::Int64(1)).unwrap();
        }
        for _ in 0..4 {
            group_col.push_value(Value::Int64(2)).unwrap();
        }
        table
            .add_column("group".to_string(), Box::new(group_col))
            .unwrap();

        // Add value column
        let mut val_col = IntColumn::new();
        for i in 1..=6 {
            val_col.push_value(Value::Int64(i)).unwrap();
        }
        for i in 1..=4 {
            val_col.push_value(Value::Int64(i)).unwrap();
        }
        table
            .add_column("value".to_string(), Box::new(val_col))
            .unwrap();

        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0], // Group by group
            vec![1], // Sum value
            vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
        ));

        group_by.open().unwrap();

        let batch = group_by.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 2); // 2 groups

        // Find each group and verify sum
        let mut group1_found = false;
        let mut group2_found = false;
        for i in 0..batch.row_count() {
            let group = batch.get(i, 0).unwrap();
            let sum = batch.get(i, 1).unwrap();
            if group == Value::Int64(1) {
                assert_eq!(sum, Value::Int64(21)); // Sum of 1-6
                group1_found = true;
            } else if group == Value::Int64(2) {
                assert_eq!(sum, Value::Int64(10)); // Sum of 1-4
                group2_found = true;
            }
        }
        assert!(group1_found && group2_found);

        group_by.close().unwrap();
    }

    #[test]
    fn test_group_by_min_max_aggregates() {
        let mut table = Table::new("test".to_string());

        // Add group column
        let mut group_col = IntColumn::new();
        for _ in 0..5 {
            group_col.push_value(Value::Int64(1)).unwrap();
        }
        for _ in 0..3 {
            group_col.push_value(Value::Int64(2)).unwrap();
        }
        table
            .add_column("group".to_string(), Box::new(group_col))
            .unwrap();

        // Add value column
        let mut val_col = IntColumn::new();
        for i in 5..10 {
            val_col.push_value(Value::Int64(i)).unwrap();
        }
        for i in 1..4 {
            val_col.push_value(Value::Int64(i)).unwrap();
        }
        table
            .add_column("value".to_string(), Box::new(val_col))
            .unwrap();

        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0],    // Group by group
            vec![1, 1], // Min and max value
            vec![
                Box::new(MinAggregate::new(DataType::Int64)),
                Box::new(MaxAggregate::new(DataType::Int64)),
            ],
        ));

        group_by.open().unwrap();

        let batch = group_by.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 2);
        assert_eq!(batch.column_count(), 3); // group + min + max

        // Check group 1
        for i in 0..batch.row_count() {
            let group = batch.get(i, 0).unwrap();
            let min = batch.get(i, 1).unwrap();
            let max = batch.get(i, 2).unwrap();
            if group == Value::Int64(1) {
                assert_eq!(min, Value::Int64(5));
                assert_eq!(max, Value::Int64(9));
            } else if group == Value::Int64(2) {
                assert_eq!(min, Value::Int64(1));
                assert_eq!(max, Value::Int64(3));
            }
        }

        group_by.close().unwrap();
    }

    #[test]
    fn test_group_by_avg_aggregate() {
        let mut table = Table::new("test".to_string());

        // Add group column
        let mut group_col = IntColumn::new();
        for _ in 0..3 {
            group_col.push_value(Value::Int64(1)).unwrap();
        }
        for _ in 0..3 {
            group_col.push_value(Value::Int64(2)).unwrap();
        }
        table
            .add_column("group".to_string(), Box::new(group_col))
            .unwrap();

        // Add value column
        let mut val_col = FloatColumn::new();
        val_col.push_value(Value::Float64(10.0)).unwrap();
        val_col.push_value(Value::Float64(20.0)).unwrap();
        val_col.push_value(Value::Float64(30.0)).unwrap();
        val_col.push_value(Value::Float64(40.0)).unwrap();
        val_col.push_value(Value::Float64(50.0)).unwrap();
        val_col.push_value(Value::Float64(60.0)).unwrap();
        table
            .add_column("value".to_string(), Box::new(val_col))
            .unwrap();

        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0], // Group by group
            vec![1], // Average value
            vec![Box::new(AvgAggregate::new(DataType::Float64).unwrap())],
        ));

        group_by.open().unwrap();

        let batch = group_by.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 2);

        // Check averages: (10+20+30)/3=20, (40+50+60)/3=50
        for i in 0..batch.row_count() {
            let group = batch.get(i, 0).unwrap();
            let avg = batch.get(i, 1).unwrap();
            if group == Value::Int64(1) {
                assert_eq!(avg, Value::Float64(20.0));
            } else if group == Value::Int64(2) {
                assert_eq!(avg, Value::Float64(50.0));
            }
        }

        group_by.close().unwrap();
    }

    #[test]
    fn test_group_by_multiple_group_by_columns() {
        let mut table = Table::new("test".to_string());

        // Add column1
        let mut col1 = IntColumn::new();
        for _ in 0..4 {
            col1.push_value(Value::Int64(1)).unwrap();
        }
        for _ in 0..4 {
            col1.push_value(Value::Int64(2)).unwrap();
        }
        table
            .add_column("col1".to_string(), Box::new(col1))
            .unwrap();

        // Add column2
        let mut col2 = IntColumn::new();
        for i in 0..8 {
            col2.push_value(Value::Int64(i % 2)).unwrap();
        }
        table
            .add_column("col2".to_string(), Box::new(col2))
            .unwrap();

        // Add value column
        let mut val_col = IntColumn::new();
        for i in 1..=8 {
            val_col.push_value(Value::Int64(i)).unwrap();
        }
        table
            .add_column("value".to_string(), Box::new(val_col))
            .unwrap();

        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0, 1], // Group by col1 and col2
            vec![2],    // Sum value
            vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
        ));

        group_by.open().unwrap();

        let batch = group_by.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 4); // (1,0), (1,1), (2,0), (2,1)
        assert_eq!(batch.column_count(), 3);

        // Verify column names
        let column_names = group_by.column_names().unwrap();
        assert_eq!(column_names, vec!["col1", "col2", "agg_0"]);

        group_by.close().unwrap();
    }

    #[test]
    fn test_group_by_empty_input() {
        let mut table = Table::new("empty".to_string());

        // Add column but no rows
        let col = IntColumn::new();
        table
            .add_column("value".to_string(), Box::new(col))
            .unwrap();

        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0],
            vec![0],
            vec![Box::new(CountAggregate::new(DataType::Int64))],
        ));

        group_by.open().unwrap();

        // Should return None for empty input
        assert!(group_by.next_batch().unwrap().is_none());

        group_by.close().unwrap();
    }

    #[test]
    fn test_group_by_single_group() {
        let mut table = Table::new("test".to_string());

        // Add group column (all same value)
        let mut group_col = IntColumn::new();
        for _ in 0..5 {
            group_col.push_value(Value::Int64(1)).unwrap();
        }
        table
            .add_column("group".to_string(), Box::new(group_col))
            .unwrap();

        // Add value column
        let mut val_col = IntColumn::new();
        for i in 1..=5 {
            val_col.push_value(Value::Int64(i)).unwrap();
        }
        table
            .add_column("value".to_string(), Box::new(val_col))
            .unwrap();

        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0], // Group by group
            vec![1], // Sum value
            vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
        ));

        group_by.open().unwrap();

        let batch = group_by.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 1); // Single group
        assert_eq!(batch.get(0, 1).unwrap(), Value::Int64(15)); // Sum of 1-5

        group_by.close().unwrap();
    }

    #[test]
    fn test_group_by_invalid_group_by_column_index() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![10], // Invalid column index
            vec![0],
            vec![Box::new(CountAggregate::new(DataType::Int64))],
        ));

        let result = group_by.open();
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ExecutionError::InvalidColumnIndex { .. })
        ));
    }

    #[test]
    fn test_group_by_invalid_aggregate_column_index() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0],
            vec![10], // Invalid column index
            vec![Box::new(CountAggregate::new(DataType::Int64))],
        ));

        let result = group_by.open();
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ExecutionError::InvalidColumnIndex { .. })
        ));
    }

    #[test]
    fn test_group_by_mismatched_lengths() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        // aggregate_columns has 2 elements, but aggregates has 1
        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0],
            vec![0, 1],                                           // 2 column indices
            vec![Box::new(CountAggregate::new(DataType::Int64))], // 1 aggregate
        ));

        let result = group_by.open();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::Custom(_))));
    }

    #[test]
    fn test_group_by_lifecycle() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0],
            vec![1],
            vec![Box::new(CountAggregate::new(DataType::Int64))],
        ));

        assert!(!group_by.is_open());

        group_by.open().unwrap();
        assert!(group_by.is_open());

        // Can't open twice
        let result = group_by.open();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::OperatorAlreadyOpen)));

        group_by.close().unwrap();
        assert!(!group_by.is_open());
    }

    #[test]
    fn test_group_by_next_batch_not_open() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0],
            vec![1],
            vec![Box::new(CountAggregate::new(DataType::Int64))],
        ));

        let result = group_by.next_batch();
        assert!(result.is_err());
        assert!(matches!(result, Err(ExecutionError::OperatorNotOpen)));
    }

    #[test]
    fn test_group_by_schema() {
        let table = create_test_table();
        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![1, 2], // Group by name and age
            vec![0],    // Count id
            vec![Box::new(CountAggregate::new(DataType::Int64))],
        ));

        group_by.open().unwrap();

        let schema = group_by.schema().unwrap();
        assert_eq!(schema.len(), 3);
        assert_eq!(schema.get("name"), Some(&DataType::String));
        assert_eq!(schema.get("age"), Some(&DataType::Float64));
        assert_eq!(schema.get("agg_0"), Some(&DataType::Int64));

        group_by.close().unwrap();
    }

    #[test]
    fn test_group_by_multiple_aggregates_same_column() {
        let mut table = Table::new("test".to_string());

        // Add group column
        let mut group_col = IntColumn::new();
        for _ in 0..5 {
            group_col.push_value(Value::Int64(1)).unwrap();
        }
        table
            .add_column("group".to_string(), Box::new(group_col))
            .unwrap();

        // Add value column
        let mut val_col = IntColumn::new();
        for i in 1..=5 {
            val_col.push_value(Value::Int64(i)).unwrap();
        }
        table
            .add_column("value".to_string(), Box::new(val_col))
            .unwrap();

        let scan = Box::new(TableScan::new(table));

        let mut group_by = Box::new(GroupBy::new(
            scan,
            vec![0],       // Group by group
            vec![1, 1, 1], // Sum, min, max same column
            vec![
                Box::new(SumAggregate::new(DataType::Int64).unwrap()),
                Box::new(MinAggregate::new(DataType::Int64)),
                Box::new(MaxAggregate::new(DataType::Int64)),
            ],
        ));

        group_by.open().unwrap();

        let batch = group_by.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 1);
        assert_eq!(batch.column_count(), 4); // group + sum + min + max

        // Sum=15, Min=1, Max=5
        assert_eq!(batch.get(0, 1).unwrap(), Value::Int64(15));
        assert_eq!(batch.get(0, 2).unwrap(), Value::Int64(1));
        assert_eq!(batch.get(0, 3).unwrap(), Value::Int64(5));

        group_by.close().unwrap();
    }

    // ============================================================================
    // HELPER FUNCTIONS
    // ============================================================================

    // Helper function to create a test table with sample data
    fn create_test_table() -> Table {
        let mut table = Table::new("test".to_string());

        // Add id column
        let mut id_col = IntColumn::new();
        for i in 1..=5 {
            id_col.push_value(Value::Int64(i)).unwrap();
        }
        table
            .add_column("id".to_string(), Box::new(id_col))
            .unwrap();

        // Add name column
        let mut name_col = StringColumn::new();
        name_col
            .push_value(Value::String("Alice".to_string()))
            .unwrap();
        name_col
            .push_value(Value::String("Bob".to_string()))
            .unwrap();
        name_col
            .push_value(Value::String("Charlie".to_string()))
            .unwrap();
        name_col
            .push_value(Value::String("David".to_string()))
            .unwrap();
        name_col
            .push_value(Value::String("Eve".to_string()))
            .unwrap();
        table
            .add_column("name".to_string(), Box::new(name_col))
            .unwrap();

        // Add age column
        let mut age_col = FloatColumn::new();
        age_col.push_value(Value::Float64(25.0)).unwrap();
        age_col.push_value(Value::Float64(30.0)).unwrap();
        age_col.push_value(Value::Float64(35.0)).unwrap();
        age_col.push_value(Value::Float64(40.0)).unwrap();
        age_col.push_value(Value::Float64(45.0)).unwrap();
        table
            .add_column("age".to_string(), Box::new(age_col))
            .unwrap();

        table
    }

    // Helper function to create a large test table
    fn create_test_large_table(row_count: usize) -> Table {
        let mut table = Table::new("large_test".to_string());

        // Add id column
        let mut id_col = IntColumn::new();
        for i in 0..row_count {
            id_col.push_value(Value::Int64(i as i64)).unwrap();
        }
        table
            .add_column("id".to_string(), Box::new(id_col))
            .unwrap();

        // Add value column
        let mut value_col = FloatColumn::new();
        for i in 0..row_count {
            value_col
                .push_value(Value::Float64(i as f64 * 1.5))
                .unwrap();
        }
        table
            .add_column("value".to_string(), Box::new(value_col))
            .unwrap();

        table
    }
}
