# Phase 4 Learning Guide: Query Operators Implementation

## Table of Contents

1. [Introduction & Learning Objectives](#1-introduction--learning-objectives)
2. [Query Execution Foundation](#2-query-execution-foundation)
3. [TableScan Operator](#3-tablescan-operator)
4. [Filter Operator & Predicates](#4-filter-operator--predicates)
5. [Project Operator](#5-project-operator)
6. [Aggregate Functions](#6-aggregate-functions)
7. [GroupBy Operator](#7-groupby-operator)
8. [Operator Integration Testing](#8-operator-integration-testing)
9. [Advanced Topics](#9-advanced-topics)
10. [Best Practices & Design Patterns](#10-best-practices--design-patterns)
11. [Learning Outcomes & Self-Assessment](#11-learning-outcomes--self-assessment)
12. [Appendices](#12-appendices)

---

## 1. Introduction & Learning Objectives

### Overview

Phase 4 focuses on **Query Execution Engine** - the heart of the Mini Rust OLAP database that processes queries and returns results. This phase implements vectorized query operators that work on batches of data efficiently, forming the foundation for SQL query execution.

### Why Query Execution Matters

Query execution is what makes a database useful:

- **Performance**: Determines how fast queries run
- **Flexibility**: Enables complex data transformations
- **Scalability**: Handles large datasets efficiently
- **Correctness**: Ensures accurate results

Implementing query execution teaches:
- Operator-based query execution model
- Vectorized processing for performance
- Predicate evaluation and filtering
- Aggregation algorithms
- Schema management and transformations
- Error handling in query pipelines
- Testing strategies for complex systems

### Learning Objectives

By the end of Phase 4, you will understand:

1. **Vectorized Query Execution**
   - Columnar data processing in batches
   - Operator lifecycle management (open, next_batch, close)
   - Schema propagation through operator chains
   - Memory-efficient batch processing

2. **Table Scanning**
   - Reading data from tables in batches
   - Column pruning for performance
   - Schema management and validation
   - Batch size optimization

3. **Filtering with Predicates**
   - Binary comparison predicates
   - Boolean logic (AND, OR)
   - Predicate evaluation strategies
   - Short-circuit optimization

4. **Projection**
   - Column selection and reordering
   - Column aliasing
   - Schema reduction
   - Expression evaluation

5. **Aggregation**
   - Aggregate function design pattern
   - State management for incremental aggregation
   - Type-specific implementations
   - Handling NULL values

6. **Grouping**
   - Hash-based grouping algorithms
   - Group key design
   - Multiple grouping columns
   - Group-aware aggregation

7. **Operator Integration**
   - Chaining operators into pipelines
   - Schema transformation validation
   - End-to-end query execution
   - Performance testing

### Prerequisites

Before starting Phase 4, you should be comfortable with:

- **Phase 1**: Core data types (Int64, Float64, String), Value enum, and error handling
- **Phase 2**: Table structure, Column traits, schema management
- **Phase 3**: CSV ingestion and data loading
- **Rust**: Traits, Generics, Box<dyn T>, Arc<T>, Error handling
- **Data Structures**: HashMap, Vec, Option, Result
- **Testing**: Unit tests, integration tests, test organization

---

## 2. Query Execution Foundation

### 2.1 Vectorized Execution Model

Traditional row-based databases process one row at a time:
```
for each row in table:
    if row satisfies predicate:
        select columns from row
        add to results
```

**Vectorized execution** processes batches of rows in columnar format:
```
for each batch of rows:
    filter_column = evaluate_predicate(batch[predicate_column])
    filtered_batch = apply_filter(batch, filter_column)
    projected_batch = select_columns(filtered_batch, column_indices)
    add projected_batch to results
```

**Benefits of Vectorized Execution:**

1. **CPU Cache Efficiency**: Processing columns sequentially uses cache better
2. **SIMD Optimization**: Modern CPUs can process multiple values simultaneously
3. **Memory Efficiency**: Columnar layout avoids loading unused columns
4. **Parallelization**: Easier to parallelize column operations

### 2.2 The Batch Structure

A `Batch` represents a subset of data in columnar format:

```rust
pub struct Batch {
    columns: Vec<Arc<dyn Column>>,
    row_count: usize,
}
```

**Key Design Decisions:**

1. **Arc<Column>**: Shared ownership allows efficient cloning
2. **Dynamic Dispatch**: Supports different column types (Int, Float, String)
3. **Row Count Tracking**: Optimizes row iteration
4. **Columnar Layout**: Columns stored separately for vectorization

**Example Creating a Batch:**

```rust
use mini_rust_olap::{IntColumn, FloatColumn, StringColumn, Value, Column};

fn create_sample_batch() -> Batch {
    // Create columns
    let mut id_col = IntColumn::new();
    let mut name_col = StringColumn::new();
    let mut age_col = FloatColumn::new();
    
    // Add values
    id_col.push_value(Value::Int64(1)).unwrap();
    id_col.push_value(Value::Int64(2)).unwrap();
    
    name_col.push_value(Value::String("Alice".to_string())).unwrap();
    name_col.push_value(Value::String("Bob".to_string())).unwrap();
    
    age_col.push_value(Value::Float64(25.0)).unwrap();
    age_col.push_value(Value::Float64(30.0)).unwrap();
    
    // Wrap in Arc and create batch
    let columns: Vec<Arc<dyn Column>> = vec![
        Arc::new(id_col),
        Arc::new(name_col),
        Arc::new(age_col),
    ];
    
    Batch::new(columns)
}
```

### 2.3 The Operator Trait

All query operators implement the `Operator` trait:

```rust
pub trait Operator: Send + Sync {
    /// Open the operator and prepare for execution
    fn open(&mut self) -> Result<()>;
    
    /// Get the next batch of results
    fn next_batch(&mut self) -> Result<Option<Batch>>;
    
    /// Close the operator and release resources
    fn close(&mut self) -> Result<()>;
    
    /// Get the output schema
    fn schema(&self) -> Result<HashMap<String, DataType>>;
    
    /// Get the output column names
    fn column_names(&self) -> Result<Vec<String>>;
    
    /// Check if operator is open
    fn is_open(&self) -> bool;
}
```

**Lifecycle Management:**

Operators follow a strict lifecycle:
1. **Closed**: Initial state, no resources allocated
2. **Open**: After `open()` is called, resources allocated, can process data
3. **Closed Again**: After `close()` is called, resources released

**State Enum:**

```rust
pub enum OperatorState {
    NotOpen,
    Open,
    Closed,
}
```

**Common Lifecycle Errors:**

```rust
// Error: Can't call next_batch before opening
let mut op = TableScan::new(table);
op.next_batch()?; // Returns Err(OperatorNotOpen)

// Error: Can't open twice
let mut op = TableScan::new(table);
op.open()?;
op.open()?; // Returns Err(OperatorAlreadyOpen)

// Error: Can't get schema before opening
let op = TableScan::new(table);
op.schema()?; // Returns Err(SchemaNotFound)
```

### 2.4 Error Handling

Query execution has its own error type:

```rust
pub enum ExecutionError {
    OperatorNotOpen,
    OperatorAlreadyOpen,
    SchemaMismatch(String),
    SchemaNotFound,
    InvalidColumnIndex { index: usize, count: usize },
    ColumnNotFound(String),
    InvalidRowIndex { index: usize, count: usize },
    IoError(std::io::Error),
    Custom(String),
}
```

**Error Propagation Pattern:**

```rust
impl Operator for MyOperator {
    fn open(&mut self) -> Result<()> {
        if self.state == OperatorState::Open {
            return Err(ExecutionError::OperatorAlreadyOpen);
        }
        
        // Open child operator
        self.child.open()?;  // Propagates errors using ?
        
        // Validate something
        if self.column_index >= child_schema.len() {
            return Err(ExecutionError::InvalidColumnIndex {
                index: self.column_index,
                count: child_schema.len(),
            });
        }
        
        self.state = OperatorState::Open;
        Ok(())
    }
}
```

### 2.5 Schema Management

Each operator must manage its output schema:

```rust
// Input schema from child
let child_schema = self.child.schema()?;
let child_columns = self.child.column_names()?;

// Build output schema
let mut output_schema = HashMap::new();
let mut output_columns = Vec::new();

for &index in &self.selected_columns {
    let name = child_columns[index].clone();
    let data_type = child_schema[&name];
    output_schema.insert(name.clone(), data_type);
    output_columns.push(name);
}

self.output_schema = Some(output_schema);
self.output_columns = Some(output_columns);
```

**Schema Validation Rules:**

1. **Filter**: Output schema must match input schema (no transformation)
2. **Project**: Output schema is subset of input schema
3. **GroupBy**: Output schema = group columns + aggregate columns
4. **Aggregates**: Must handle type conversions (e.g., Int64 → Float64 for Avg)

### 2.6 Common Pitfalls

**Pitfall 1: Forgetting to Open Before Using Schema**

```rust
// WRONG
let scan = TableScan::new(table);
let schema = scan.schema()?; // Error: SchemaNotFound

// RIGHT
let mut scan = TableScan::new(table);
scan.open()?;
let schema = scan.schema()?;
```

**Pitfall 2: Not Closing Operators**

```rust
// WRONG - Resource leak
let mut scan = TableScan::new(table);
scan.open()?;
// ... process data ...
// Forgot to close!

// RIGHT
let mut scan = TableScan::new(table);
scan.open()?;
// ... process data ...
scan.close()?;
```

**Pitfall 3: Ignoring Batch Results**

```rust
// WRONG - Infinite loop
while let Some(_batch) = operator.next_batch()? {
    // Forgot to break or advance
}

// RIGHT
while let Some(batch) = operator.next_batch()? {
    // Process batch
    if batch.is_empty() {
        break;  // Handle empty batches
    }
}
```

---

## 3. TableScan Operator

### 3.1 Purpose and Function

`TableScan` is the source operator that reads data from a table and produces batches. It's typically the first operator in a query execution pipeline.

**Responsibilities:**
1. Read rows from table
2. Organize data into batches
3. Apply column pruning (select only needed columns)
4. Manage batch size for optimal performance
5. Provide schema information

### 3.2 Basic Implementation

```rust
pub struct TableScan {
    table: Table,
    batch_size: usize,
    column_indices: Vec<usize>,  // For column pruning
    state: OperatorState,
    current_row: usize,
}
```

**Open Implementation:**

```rust
fn open(&mut self) -> Result<()> {
    if self.state == OperatorState::Open {
        return Err(ExecutionError::OperatorAlreadyOpen);
    }
    
    // Validate column indices
    let table_schema = self.table.schema()?;
    for &index in &self.column_indices {
        if index >= table_schema.len() {
            return Err(ExecutionError::InvalidColumnIndex {
                index,
                count: table_schema.len(),
            });
        }
    }
    
    self.state = OperatorState::Open;
    Ok(())
}
```

**Next Batch Implementation:**

```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    if self.state != OperatorState::Open {
        return Err(ExecutionError::OperatorNotOpen);
    }
    
    let total_rows = self.table.row_count();
    
    // Check if we've read all rows
    if self.current_row >= total_rows {
        return Ok(None);
    }
    
    // Calculate batch size
    let end_row = std::cmp::min(self.current_row + self.batch_size, total_rows);
    let batch_row_count = end_row - self.current_row;
    
    // Create batch columns
    let mut batch_columns = Vec::new();
    
    for &col_index in &self.column_indices {
        let column = self.table.get_column(col_index)?;
        batch_columns.push(column);
    }
    
    // Create batch from selected rows
    let batch = create_batch_from_rows(batch_columns, self.current_row, end_row)?;
    
    self.current_row = end_row;
    Ok(Some(batch))
}
```

### 3.3 Column Pruning

Column pruning reads only the columns needed by the query:

```rust
// Query: SELECT name, age FROM users
let scan = TableScan::with_columns(table, vec![1, 2]);  // name, age

// Benefits:
// - Less memory used
// - Better cache locality
// - Faster I/O
```

**Without Pruning:**
```rust
// Reads all columns (id, name, age, email, address, ...)
let scan = TableScan::new(table);  // Reads 100 columns
```

**With Pruning:**
```rust
// Reads only 2 columns
let scan = TableScan::with_columns(table, vec![1, 2]);  // Reads 2 columns
// 50x less data read!
```

### 3.4 Batch Size Selection

Choosing the right batch size is crucial for performance:

```rust
let scan = TableScan::new(table).with_batch_size(1000);
```

**Factors to Consider:**

1. **Cache Size**: Batches should fit in CPU cache (typically 64-256 KB)
2. **Memory Usage**: Larger batches use more memory
3. **Parallelism**: Smaller batches enable better parallelization
4. **Overhead**: Too small batches incur overhead

**Typical Batch Sizes:**

| Scenario | Recommended Batch Size | Reason |
|----------|----------------------|---------|
| Small tables (< 10K rows) | 1000-5000 | Minimize overhead |
| Medium tables (10K-1M rows) | 5000-10000 | Balance memory and cache |
| Large tables (> 1M rows) | 10000-50000 | Maximize throughput |

### 3.5 Schema Handling

TableScan must provide both schema and column names:

```rust
fn schema(&self) -> Result<HashMap<String, DataType>> {
    if self.output_schema.is_none() {
        return Err(ExecutionError::SchemaNotFound);
    }
    
    Ok(self.output_schema.clone().unwrap())
}

fn column_names(&self) -> Result<Vec<String>> {
    if self.output_columns.is_none() {
        return Err(ExecutionError::Custom("Columns not initialized".to_string()));
    }
    
    Ok(self.output_columns.clone().unwrap())
}
```

**Schema Building in Open:**

```rust
let table_schema = self.table.schema()?;
let table_columns = self.table.column_names()?;

let mut output_schema = HashMap::new();
let mut output_columns = Vec::new();

for &index in &self.column_indices {
    let name = table_columns[index].clone();
    let data_type = table_schema[&name].clone();
    output_schema.insert(name.clone(), data_type);
    output_columns.push(name);
}

self.output_schema = Some(output_schema);
self.output_columns = Some(output_columns);
```

### 3.6 Testing TableScan

**Basic Test:**

```rust
#[test]
fn test_table_scan_lifecycle() {
    let table = create_test_table();
    let mut scan = TableScan::new(table);
    
    // Schema not available before open
    assert!(scan.schema().is_err());
    
    scan.open().unwrap();
    
    // Schema available after open
    let schema = scan.schema().unwrap();
    assert_eq!(schema.len(), 3);
    
    // Get batches
    let mut total_rows = 0;
    while let Some(batch) = scan.next_batch().unwrap() {
        total_rows += batch.row_count();
    }
    
    assert_eq!(total_rows, 100);
    
    scan.close().unwrap();
}
```

**Column Pruning Test:**

```rust
#[test]
fn test_table_scan_column_pruning() {
    let table = create_test_table();
    let scan = TableScan::with_columns(table, vec![0, 2]);  // id, age
    
    scan.open().unwrap();
    
    let schema = scan.schema().unwrap();
    assert_eq!(schema.len(), 2);
    assert!(schema.contains_key("id"));
    assert!(schema.contains_key("age"));
    assert!(!schema.contains_key("name"));
    
    let batch = scan.next_batch().unwrap().unwrap();
    assert_eq!(batch.column_count(), 2);
    
    scan.close().unwrap();
}
```

---

## 4. Filter Operator & Predicates

### 4.1 Predicate System Design

Predicates evaluate conditions on row data. Mini Rust OLAP uses a trait-based design:

```rust
pub trait Predicate: Send + Sync + std::fmt::Debug {
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<bool>;
}
```

**Benefits of Trait-Based Design:**

1. **Extensibility**: Easy to add new predicate types
2. **Composability**: Predicates can be combined (AND, OR)
3. **Type Safety**: Compile-time checking
4. **Dynamic Dispatch**: Runtime flexibility

### 4.2 Binary Comparison Predicate

The most common predicate compares a column to a value:

```rust
pub struct BinaryComparison {
    column_index: usize,
    op: ComparisonOp,
    value: Value,
}

pub enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}
```

**Implementation:**

```rust
impl Predicate for BinaryComparison {
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<bool> {
        let actual = batch.get(row_index, self.column_index)?;
        
        match (&self.op, &actual, &self.value) {
            // Equal
            (ComparisonOp::Equal, Value::Int64(a), Value::Int64(b)) => Ok(a == b),
            (ComparisonOp::Equal, Value::Float64(a), Value::Float64(b)) => Ok(a == b),
            (ComparisonOp::Equal, Value::String(a), Value::String(b)) => Ok(a == b),
            
            // Greater Than
            (ComparisonOp::GreaterThan, Value::Int64(a), Value::Int64(b)) => Ok(a > b),
            (ComparisonOp::GreaterThan, Value::Float64(a), Value::Float64(b)) => Ok(a > b),
            
            // ... other comparisons
        }
    }
}
```

**Usage:**

```rust
// Filter: age > 30
let predicate = Arc::new(BinaryComparison::new(
    2,  // age column index
    ComparisonOp::GreaterThan,
    Value::Float64(30.0),
));

let filter = Filter::new(child, predicate);
```

### 4.3 Composite Predicates

Predicates can be combined using AND and OR:

```rust
pub struct And {
    left: Arc<dyn Predicate>,
    right: Arc<dyn Predicate>,
}

pub struct Or {
    left: Arc<dyn Predicate>,
    right: Arc<dyn Predicate>,
}
```

**AND Implementation with Short-Circuit:**

```rust
impl Predicate for And {
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<bool> {
        let left_result = self.left.eval(batch, row_index)?;
        if !left_result {
            return Ok(false);  // Short-circuit: if left is false, result is false
        }
        self.right.eval(batch, row_index)
    }
}
```

**OR Implementation with Short-Circuit:**

```rust
impl Predicate for Or {
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<bool> {
        let left_result = self.left.eval(batch, row_index)?;
        if left_result {
            return Ok(true);  // Short-circuit: if left is true, result is true
        }
        self.right.eval(batch, row_index)
    }
}
```

**Using Composite Predicates:**

```rust
// Filter: age > 25 AND age < 50
let pred1 = Arc::new(BinaryComparison::new(
    2, ComparisonOp::GreaterThan, Value::Float64(25.0)
));
let pred2 = Arc::new(BinaryComparison::new(
    2, ComparisonOp::LessThan, Value::Float64(50.0)
));
let predicate = Arc::new(And::new(pred1, pred2));

// Filter: department = 'Engineering' OR department = 'Sales'
let pred1 = Arc::new(BinaryComparison::new(
    1, ComparisonOp::Equal, Value::String("Engineering".to_string())
));
let pred2 = Arc::new(BinaryComparison::new(
    1, ComparisonOp::Equal, Value::String("Sales".to_string())
));
let predicate = Arc::new(Or::new(pred1, pred2));
```

### 4.4 Filter Operator Implementation

```rust
pub struct Filter {
    child: Box<dyn Operator>,
    predicate: Arc<dyn Predicate>,
    state: OperatorState,
    output_schema: Option<HashMap<String, DataType>>,
    output_columns: Option<Vec<String>>,
}
```

**Next Batch Implementation:**

```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    if self.state != OperatorState::Open {
        return Err(ExecutionError::OperatorNotOpen);
    }
    
    let mut output_columns: Vec<Vec<Option<Value>>> = Vec::new();
    let mut output_rows = 0;
    
    // Process batches until we have enough rows or child is exhausted
    while output_rows < self.batch_size {
        match self.child.next_batch()? {
            Some(input_batch) => {
                // Evaluate predicate on each row
                for row_index in 0..input_batch.row_count() {
                    if self.predicate.eval(&input_batch, row_index)? {
                        // Row passes filter - add to output
                        for col_index in 0..input_batch.column_count() {
                            let value = input_batch.get(row_index, col_index)?;
                            output_columns[col_index].push(Some(value));
                        }
                        output_rows += 1;
                    }
                }
                
                if output_rows >= self.batch_size {
                    break;
                }
            }
            None => break,  // No more batches from child
        }
    }
    
    if output_rows == 0 {
        return Ok(None);
    }
    
    // Create output batch
    let batch = create_batch_from_columns(output_columns)?;
    Ok(Some(batch))
}
```

**Schema Preservation:**

```rust
// Filter preserves child's schema exactly
fn open(&mut self) -> Result<()> {
    self.child.open()?;
    
    // Copy child's schema
    self.output_schema = Some(self.child.schema()?);
    self.output_columns = Some(self.child.column_names()?);
    
    self.state = OperatorState::Open;
    Ok(())
}
```

### 4.5 Performance Optimizations

**1. Vectorized Predicate Evaluation:**

Instead of evaluating row by row, evaluate entire columns:

```rust
// Row-by-row (slow)
for row in 0..batch.row_count() {
    if predicate.eval(batch, row)? {
        add_row_to_output(row);
    }
}

// Vectorized (fast)
let column = batch.get_column(predicate.column_index)?;
let mask = column.compare(predicate.op, &predicate.value)?;
let filtered_batch = batch.apply_mask(mask);
```

**2. Early Termination:**

```rust
// If all rows in a batch fail the predicate, skip to next batch
let mut all_false = true;
for row in 0..batch.row_count() {
    if predicate.eval(batch, row)? {
        all_false = false;
        break;
    }
}
if all_false {
    continue;  // Skip this batch entirely
}
```

**3. Predicate Pushdown:**

Move filters earlier in the pipeline to reduce data processed:

```rust
// BAD: Filter after expensive operation
Scan → GroupBy → Filter → Project

// GOOD: Filter before expensive operation
Scan → Filter → GroupBy → Project  // Processes much less data
```

### 4.6 Testing Filters

**Simple Filter Test:**

```rust
#[test]
fn test_filter_simple() {
    let table = create_test_table();
    let scan = Box::new(TableScan::new(table));
    
    // Filter: age > 30
    let predicate = Arc::new(BinaryComparison::new(
        2, ComparisonOp::GreaterThan, Value::Float64(30.0)
    ));
    let mut filter = Filter::new(scan, predicate);
    
    filter.open().unwrap();
    
    let batch = filter.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 3);  // 3 rows with age > 30
    
    filter.close().unwrap();
}
```

**AND Predicate Test:**

```rust
#[test]
fn test_filter_with_and() {
    let table = create_test_table();
    let scan = Box::new(TableScan::new(table));
    
    // Filter: id >= 2 AND id <= 4
    let pred1 = Arc::new(BinaryComparison::new(
        0, ComparisonOp::GreaterThanOrEqual, Value::Int64(2)
    ));
    let pred2 = Arc::new(BinaryComparison::new(
        0, ComparisonOp::LessThanOrEqual, Value::Int64(4)
    ));
    let predicate = Arc::new(And::new(pred1, pred2));
    let mut filter = Filter::new(scan, predicate);
    
    filter.open().unwrap();
    
    let batch = filter.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 3);  // ids 2, 3, 4
    
    filter.close().unwrap();
}
```

---

## 5. Project Operator

### 5.1 Purpose and Function

Project operator selects and reorders columns from the input, optionally providing aliases. It's used to:

1. **Column Selection**: Choose only needed columns (column pruning)
2. **Column Reordering**: Arrange columns in desired order
3. **Column Aliasing**: Rename columns in output

**SQL Equivalent:**
```sql
SELECT name AS user_name, age AS user_age FROM users
```

### 5.2 Implementation

```rust
pub struct Project {
    child: Box<dyn Operator>,
    column_indices: Vec<usize>,
    aliases: Option<Vec<String>>,
    state: OperatorState,
    output_schema: Option<HashMap<String, DataType>>,
    output_columns: Option<Vec<String>>,
}
```

**Constructor Methods:**

```rust
// Without aliases
let project = Project::new(child, vec![1, 2]);

// With aliases
let project = Project::new(child, vec![1, 2])
    .with_aliases(vec!["user_name".to_string(), "user_age".to_string()]);
```

**Next Batch Implementation:**

```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    if self.state != OperatorState::Open {
        return Err(ExecutionError::OperatorNotOpen);
    }
    
    // Get next batch from child
    let input_batch = match self.child.next_batch()? {
        Some(batch) => batch,
        None => return Ok(None),
    };
    
    // Project columns
    let mut output_columns = Vec::new();
    for &col_index in &self.column_indices {
        let column = input_batch.get_column(col_index)?;
        output_columns.push(column.clone());
    }
    
    Ok(Some(Batch::new(output_columns)))
}
```

**Schema Building:**

```rust
fn open(&mut self) -> Result<()> {
    self.child.open()?;
    
    let child_schema = self.child.schema()?;
    let child_columns = self.child.column_names()?;
    
    let mut output_schema = HashMap::new();
    let mut output_columns = Vec::new();
    
    for (i, &col_index) in self.column_indices.iter().enumerate() {
        let child_name = &child_columns[col_index];
        let data_type = child_schema[child_name];
        
        // Use alias if provided, otherwise use child's name
        let output_name = match &self.aliases {
            Some(aliases) => aliases[i].clone(),
            None => child_name.clone(),
        };
        
        output_schema.insert(output_name.clone(), data_type);
        output_columns.push(output_name);
    }
    
    self.output_schema = Some(output_schema);
    self.output_columns = Some(output_columns);
    self.state = OperatorState::Open;
    Ok(())
}
```

### 5.3 Column Pruning

Project operator is the primary mechanism for column pruning:

```rust
// Query: SELECT name, email FROM users WHERE age > 25

// Bad: Select all columns, then filter, then project
Scan(all columns) → Filter(age > 25) → Project(name, email)

// Good: Project only needed columns first
Scan(all columns) → Project(name, email) → Filter(age > 25)
// But wait - Filter needs age column!

// Best: Scan only needed columns, then filter
Scan(name, age, email) → Filter(age > 25) → Project(name, email)
```

**Rule of Thumb:** Include columns needed by:
- Filter predicates
- GroupBy clauses
- Selected output columns

### 5.4 Expression Evaluation (Advanced)

Current implementation supports simple column selection. Future enhancements could support expressions:

```sql
SELECT name, salary * 1.1 AS adjusted_salary FROM employees
```

**Design Pattern for Expressions:**

```rust
pub trait Expression: Send + Sync {
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<Value>;
    fn data_type(&self) -> DataType;
}

pub struct ColumnExpression {
    column_index: usize,
}

pub struct ArithmeticExpression {
    left: Box<dyn Expression>,
    op: ArithmeticOp,
    right: Box<dyn Expression>,
}
```

### 5.5 Testing Project

**Basic Projection Test:**

```rust
#[test]
fn test_project_basic() {
    let table = create_test_table();
    let scan = Box::new(TableScan::new(table));
    let mut project = Project::new(scan, vec![0, 1]);  // id, name
    
    project.open().unwrap();
    
    let schema = project.schema().unwrap();
    assert_eq!(schema.len(), 2);
    
    let batch = project.next_batch().unwrap().unwrap();
    assert_eq!(batch.column_count(), 2);
    
    // Verify column names
    let columns = project.column_names().unwrap();
    assert_eq!(columns, vec!["id".to_string(), "name".to_string()]);
    
    project.close().unwrap();
}
```

**Column Reordering Test:**

```rust
#[test]
fn test_project_column_reordering() {
    let table = create_test_table();
    let scan = Box::new(TableScan::new(table));
    let mut project = Project::new(scan, vec![2, 0, 1]);  // age, id, name
    
    project.open().unwrap();
    
    let columns = project.column_names().unwrap();
    assert_eq!(columns, vec!["age".to_string(), "id".to_string(), "name".to_string()]);
    
    let batch = project.next_batch().unwrap().unwrap();
    
    // Verify order: first column is age, second is id
    let age = batch.get(0, 0).unwrap();
    assert!(matches!(age, Value::Float64(_)));
    
    let id = batch.get(0, 1).unwrap();
    assert!(matches!(id, Value::Int64(_)));
    
    project.close().unwrap();
}
```

**Column Aliasing Test:**

```rust
#[test]
fn test_project_with_aliases() {
    let table = create_test_table();
    let scan = Box::new(TableScan::new(table));
    let mut project = Project::new(scan, vec![1, 2])
        .with_aliases(vec!["user_name".to_string(), "user_age".to_string()]);
    
    project.open().unwrap();
    
    let schema = project.schema().unwrap();
    assert!(schema.contains_key("user_name"));
    assert!(schema.contains_key("user_age"));
    assert!(!schema.contains_key("name"));
    assert!(!schema.contains_key("age"));
    
    let columns = project.column_names().unwrap();
    assert_eq!(columns, vec!["user_name".to_string(), "user_age".to_string()]);
    
    project.close().unwrap();
}
```

---

## 6. Aggregate Functions

### 6.1 Aggregate Function Design Pattern

Aggregate functions process multiple values and produce a single result. The design pattern uses:

1. **State**: Maintain intermediate results
2. **Update**: Process new values incrementally
3. **Result**: Extract final result

```rust
pub trait AggregateFunction: Send + Sync + std::fmt::Debug {
    /// Reset the aggregate to initial state
    fn reset(&mut self);
    
    /// Add a new value to the aggregate
    fn update(&mut self, value: Option<Value>) -> Result<()>;
    
    /// Get the final result
    fn result(&self) -> Value;
    
    /// Get the data type of the result
    fn data_type(&self) -> DataType;
}
```

**Benefits of This Pattern:**

1. **Incremental**: Processes one value at a time
2. **Stateful**: Maintains partial results
3. **Reusable**: Works for streaming data
4. **Type-Safe**: Enforces result type

### 6.2 Count Aggregate

Counts the number of values:

```rust
pub struct CountAggregate {
    count: i64,
    data_type: DataType,
}

impl CountAggregate {
    pub fn new(data_type: DataType) -> Self {
        CountAggregate {
            count: 0,
            data_type,
        }
    }
}

impl AggregateFunction for CountAggregate {
    fn reset(&mut self) {
        self.count = 0;
    }
    
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        // Count increments for both Some and None
        self.count += 1;
        Ok(())
    }
    
    fn result(&self) -> Value {
        Value::Int64(self.count)
    }
    
    fn data_type(&self) -> DataType {
        DataType::Int64
    }
}
```

**Example:**
```rust
let mut count = CountAggregate::new(DataType::Int64);
count.update(Some(Value::Int64(1)))?;
count.update(Some(Value::Int64(2)))?;
count.update(None)?;  // NULL values counted
assert_eq!(count.result(), Value::Int64(3));
```

### 6.3 Sum Aggregate

Sums numeric values:

```rust
pub struct SumAggregate {
    sum: i64,
    data_type: DataType,
}

impl SumAggregate {
    pub fn new(data_type: DataType) -> Result<Self> {
        if data_type != DataType::Int64 && data_type != DataType::Float64 {
            return Err(DatabaseError::InvalidType("Sum requires Int64 or Float64".to_string()));
        }
        Ok(SumAggregate { sum: 0, data_type })
    }
}

impl AggregateFunction for SumAggregate {
    fn reset(&mut self) {
        self.sum = 0;
    }
    
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        match value {
            Some(Value::Int64(v)) => self.sum += v,
            Some(Value::Float64(v)) => self.sum += v as i64,
            None => {},  // Skip NULL values
        }
        Ok(())
    }
    
    fn result(&self) -> Value {
        Value::Int64(self.sum)
    }
    
    fn data_type(&self) -> DataType {
        self.data_type
    }
}
```

**Handling NULLs:** Different aggregates handle NULLs differently:
- **Count**: Counts NULLs
- **Sum/Min/Max**: Ignores NULLs
- **Avg**: Ignores NULLs in both numerator and denominator

### 6.4 Min and Max Aggregates

Find minimum and maximum values:

```rust
pub struct MinAggregate {
    current: Option<i64>,
}

impl MinAggregate {
    pub fn new(_data_type: DataType) -> Self {
        MinAggregate { current: None }
    }
}

impl AggregateFunction for MinAggregate {
    fn reset(&mut self) {
        self.current = None;
    }
    
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        match value {
            Some(Value::Int64(v)) => {
                self.current = Some(match self.current {
                    Some(current) => current.min(v),
                    None => v,
                });
            }
            None => {}  // Skip NULL
        }
        Ok(())
    }
    
    fn result(&self) -> Value {
        match self.current {
            Some(v) => Value::Int64(v),
            None => Value::Int64(0),  // Or could return NULL
        }
    }
    
    fn data_type(&self) -> DataType {
        DataType::Int64
    }
}
```

### 6.5 Average Aggregate

Computes the mean of values:

```rust
pub struct AvgAggregate {
    sum: f64,
    count: i64,
}

impl AvgAggregate {
    pub fn new(data_type: DataType) -> Result<Self> {
        if data_type != DataType::Int64 && data_type != DataType::Float64 {
            return Err(DatabaseError::InvalidType("Avg requires Int64 or Float64".to_string()));
        }
        Ok(AvgAggregate { sum: 0.0, count: 0 })
    }
}

impl AggregateFunction for AvgAggregate {
    fn reset(&mut self) {
        self.sum = 0.0;
        self.count = 0;
    }
    
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        match value {
            Some(Value::Int64(v)) => {
                self.sum += v as f64;
                self.count += 1;
            }
            Some(Value::Float64(v)) => {
                self.sum += v;
                self.count += 1;
            }
            None => {}  // Skip NULL
        }
        Ok(())
    }
    
    fn result(&self) -> Value {
        if self.count == 0 {
            Value::Float64(0.0)  // Or could return NULL
        } else {
            Value::Float64(self.sum / self.count as f64)
        }
    }
    
    fn data_type(&self) -> DataType {
        DataType::Float64
    }
}
```

**Note:** Avg always returns Float64, even if input is Int64.

### 6.6 Type Safety and Validation

Aggregates must validate input types:

```rust
impl AggregateFunction for SumAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        match (value, self.data_type) {
            (Some(Value::Int64(v)), DataType::Int64) => {
                self.sum += v;
                Ok(())
            }
            (Some(Value::Int64(v)), DataType::Float64) => {
                // Type conversion needed
                self.sum += v as i64;
                Ok(())
            }
            (Some(_), _) => Err(ExecutionError::SchemaMismatch(
                "Type mismatch in Sum aggregate".to_string()
            )),
            (None, _) => Ok(()),  // NULL is ok
        }
    }
}
```

### 6.7 Testing Aggregates

**Count Test:**

```rust
#[test]
fn test_count_aggregate() {
    let mut agg = CountAggregate::new(DataType::Int64);
    
    agg.update(Some(Value::Int64(1))).unwrap();
    agg.update(Some(Value::Int64(2))).unwrap();
    agg.update(None).unwrap();  // NULL counted
    
    let result = agg.result();
    assert_eq!(result, Value::Int64(3));
}
```

**Sum Test:**

```rust
#[test]
fn test_sum_aggregate() {
    let mut agg = SumAggregate::new(DataType::Int64).unwrap();
    
    agg.update(Some(Value::Int64(10))).unwrap();
    agg.update(Some(Value::Int64(20))).unwrap();
    agg.update(None).unwrap();  // NULL skipped
    
    let result = agg.result();
    assert_eq!(result, Value::Int64(30));
}
```

**Average Test:**

```rust
#[test]
fn test_avg_aggregate() {
    let mut agg = AvgAggregate::new(DataType::Int64).unwrap();
    
    agg.update(Some(Value::Int64(10))).unwrap();
    agg.update(Some(Value::Int64(20))).unwrap();
    agg.update(None).unwrap();  // NULL skipped
    
    let result = agg.result();
    assert!(matches!(result, Value::Float64(v) if (v - 15.0).abs() < 0.01));
}
```

**Min/Max Test:**

```rust
#[test]
fn test_min_max_aggregates() {
    let values = vec![5, 2, 8, 1, 9, 3];
    
    let mut min = MinAggregate::new(DataType::Int64);
    let mut max = MaxAggregate::new(DataType::Int64);
    
    for v in values {
        min.update(Some(Value::Int64(v))).unwrap();
        max.update(Some(Value::Int64(v))).unwrap();
    }
    
    assert_eq!(min.result(), Value::Int64(1));
    assert_eq!(max.result(), Value::Int64(9));
}
```

---

## 7. GroupBy Operator

### 7.1 Purpose and Function

GroupBy organizes data into groups based on key columns and computes aggregates for each group.

**SQL Equivalent:**
```sql
SELECT department, AVG(salary) as avg_salary
FROM employees
GROUP BY department
```

**Key Operations:**

1. **Grouping**: Partition rows into groups based on group keys
2. **Aggregation**: Compute aggregate functions for each group
3. **Output**: Return one row per group

### 7.2 Group Key Design

Since `Value` doesn't implement `Hash` and `Eq`, we need a custom `GroupKey`:

```rust
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
```

**Design Considerations:**

1. **NULL Handling**: NULL values are compared as equal to other NULLs
2. **Type Discrimination**: Hash includes type tag to prevent collisions
3. **String Representation**: Float64 uses `to_bits()` for exact hashing
4. **Vec<Option<Value>>**: Supports multi-column grouping

### 7.3 GroupBy Operator Implementation

```rust
pub struct GroupBy {
    child: Box<dyn Operator>,
    group_by_columns: Vec<usize>,
    aggregate_columns: Vec<usize>,
    aggregates: Vec<Box<dyn AggregateFunction>>,
    state: OperatorState,
    output_schema: Option<HashMap<String, DataType>>,
    output_column_names: Option<Vec<String>>,
    grouped_data: Option<HashMap<GroupKey, Vec<Vec<Option<Value>>>>>,
    results_returned: bool,
}
```

**Open Implementation:**

```rust
fn open(&mut self) -> Result<()> {
    if self.state == OperatorState::Open {
        return Err(ExecutionError::OperatorAlreadyOpen);
    }
    
    self.child.open()?;
    
    let child_schema = self.child.schema()?;
    let child_columns = self.child.column_names()?;
    
    // Validate indices
    for &index in &self.group_by_columns {
        if index >= child_columns.len() {
            return Err(ExecutionError::InvalidColumnIndex {
                index,
                count: child_columns.len(),
            });
        }
    }
    
    // Validate aggregate columns length
    if self.aggregate_columns.len() != self.aggregates.len() {
        return Err(ExecutionError::Custom(
            "aggregate_columns length must match aggregates length".to_string()
        ));
    }
    
    // Build output schema
    let mut output_schema = HashMap::new();
    let mut output_columns = Vec::new();
    
    // Add group by columns
    for &index in &self.group_by_columns {
        let name = child_columns[index].clone();
        let data_type = child_schema[&name];
        output_schema.insert(name.clone(), data_type);
        output_columns.push(name);
    }
    
    // Add aggregate columns
    for (i, agg) in self.aggregates.iter().enumerate() {
        let name = format!("agg_{}", i);
        let data_type = agg.data_type();
        output_schema.insert(name.clone(), data_type);
        output_columns.push(name);
    }
    
    self.output_schema = Some(output_schema);
    self.output_columns = Some(output_columns);
    
    // Read all data and group it
    let mut grouped_data = HashMap::new();
    
    while let Some(batch) = self.child.next_batch()? {
        for row_index in 0..batch.row_count() {
            // Build group key
            let mut key_values = Vec::new();
            for &col_index in &self.group_by_columns {
                let value = batch.get(row_index, col_index)?;
                key_values.push(Some(value));
            }
            let key = GroupKey(key_values);
            
            // Get all values for this row
            let mut row_values = Vec::new();
            for col_index in 0..batch.column_count() {
                let value = batch.get(row_index, col_index)?;
                row_values.push(Some(value));
            }
            
            // Add to group
            grouped_data.entry(key).or_default().push(row_values);
        }
    }
    
    self.grouped_data = Some(grouped_data);
    self.state = OperatorState::Open;
    Ok(())
}
```

**Next Batch Implementation:**

```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    if self.state != OperatorState::Open {
        return Err(ExecutionError::OperatorNotOpen);
    }
    
    if self.results_returned {
        return Ok(None);
    }
    
    self.results_returned = true;
    
    let grouped_data = self.grouped_data.as_ref().unwrap();
    
    if grouped_data.is_empty() {
        return Ok(None);
    }
    
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
    
    let batch = create_batch_from_columns(output_columns)?;
    Ok(Some(batch))
}
```

### 7.4 Performance Considerations

**Memory Usage:**
- Current implementation loads all data into memory
- For large datasets, consider streaming approach

**Streaming GroupBy (Advanced):**

```rust
// Instead of storing all rows, store only running aggregates
struct StreamingGroup {
    key: GroupKey,
    aggregates: Vec<Box<dyn AggregateFunction>>,
}

let mut groups: HashMap<GroupKey, StreamingGroup> = HashMap::new();

// Process row by row, updating running aggregates
for batch in child {
    for row in batch {
        let key = build_key(row);
        let group = groups.entry(key).or_insert_with(|| create_group());
        for (agg, val) in group.aggregates.iter_mut().zip(values) {
            agg.update(val)?;
        }
    }
}
```

### 7.5 Testing GroupBy

**Basic GroupBy Test:**

```rust
#[test]
fn test_group_by_basic() {
    let table = create_test_table();
    let scan = Box::new(TableScan::new(table));
    
    // Group by name, count occurrences
    let mut group_by = Box::new(GroupBy::new(
        scan,
        vec![1],  // Group by name
        vec![0],  // Count id
        vec![Box::new(CountAggregate::new(DataType::Int64))],
    ));
    
    group_by.open().unwrap();
    
    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 5);  // 5 unique names
    assert_eq!(batch.column_count(), 2);  // name + count
    
    group_by.close().unwrap();
}
```

**Multiple Aggregates Test:**

```rust
#[test]
fn test_group_by_multiple_aggregates() {
    let table = create_test_table();
    let scan = Box::new(TableScan::new(table));
    
    // Group by name, compute count, sum, min, max
    let mut group_by = Box::new(GroupBy::new(
        scan,
        vec![1],          // Group by name
        vec![2, 2, 2],  // All aggregates on age column
        vec![
            Box::new(CountAggregate::new(DataType::Int64)),
            Box::new(MinAggregate::new(DataType::Float64)),
            Box::new(MaxAggregate::new(DataType::Float64)),
        ],
    ));
    
    group_by.open().unwrap();
    
    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.column_count(), 4);  // name + count + min + max
    
    group_by.close().unwrap();
}
```

**Multiple Grouping Columns Test:**

```rust
#[test]
fn test_group_by_multiple_columns() {
    let table = create_test_table();
    let scan = Box::new(TableScan::new(table));
    
    // Group by name and age
    let mut group_by = Box::new(GroupBy::new(
        scan,
        vec![1, 2],  // name and age
        vec![0],     // Count id
        vec![Box::new(CountAggregate::new(DataType::Int64))],
    ));
    
    group_by.open().unwrap();
    
    let batch = group_by.next_batch().unwrap().unwrap();
    
    // Verify column order
    let columns = group_by.column_names().unwrap();
    assert_eq!(columns, vec!["name".to_string(), "age".to_string(), "agg_0".to_string()]);
    
    group_by.close().unwrap();
}
```

---

## 8. Operator Integration Testing

### 8.1 Why Integration Tests Matter

Unit tests verify individual components work correctly. Integration tests verify they work together.

**Unit Test:**
```rust
#[test]
fn test_filter() {
    // Tests Filter operator in isolation
    let filter = create_filter();
    let batch = create_test_batch();
    let result = filter.apply(batch);
    assert!(is_correct(result));
}
```

**Integration Test:**
```rust
#[test]
fn test_scan_filter_project() {
    // Tests operators chained together
    let scan = TableScan::new(table);
    let filter = Filter::new(scan, predicate);
    let project = Project::new(filter, columns);
    
    project.open()?;
    let result = project.next_batch()?;
    assert!(is_correct(result));
}
```

### 8.2 Test Scenarios

**1. Simple Chaining:**

```rust
#[test]
fn test_scan_filter_project_basic() {
    let table = create_sales_table();
    let scan = Box::new(TableScan::new(table).with_batch_size(10));
    
    // Filter: quantity > 2
    let predicate = Arc::new(BinaryComparison::new(
        3, ComparisonOp::GreaterThan, Value::Int64(2)
    ));
    let filter = Box::new(Filter::new(scan, predicate));
    
    // Project: product, quantity
    let project = Box::new(Project::new(filter, vec![1, 3]));
    
    project.open().unwrap();
    
    let mut total_rows = 0;
    while let Some(batch) = project.next_batch().unwrap() {
        total_rows += batch.row_count();
        
        // Verify schema
        let schema = project.schema().unwrap();
        assert_eq!(schema.len(), 2);
        assert_eq!(schema.get("product"), Some(&DataType::String));
        assert_eq!(schema.get("quantity"), Some(&DataType::Int64));
    }
    
    assert_eq!(total_rows, 11);  // Expected filtered rows
    
    project.close().unwrap();
}
```

**2. Complex Pipeline:**

```rust
#[test]
fn test_scan_filter_groupby_project_complex() {
    let table = create_sales_table();
    let scan = Box::new(TableScan::new(table));
    
    // Filter: price >= 300
    let predicate = Arc::new(BinaryComparison::new(
        4, ComparisonOp::GreaterThanOrEqual, Value::Float64(300.0)
    ));
    let filter = Box::new(Filter::new(scan, predicate));
    
    // GroupBy: sum quantity by product
    let group_by = Box::new(GroupBy::new(
        filter,
        vec![1],  // product
        vec![3],  // quantity
        vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
    ));
    
    // Project: product, total_quantity
    let project = Box::new(Project::new(group_by, vec![0, 1])
        .with_aliases(vec!["product_name".to_string(), "total_quantity".to_string()]));
    
    project.open().unwrap();
    
    let batch = project.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 3);  // Laptop, Phone, Tablet
    
    project.close().unwrap();
}
```

**3. End-to-End Query:**

```rust
#[test]
fn test_end_to_end_query() {
    // SQL: SELECT product, SUM(quantity) FROM sales WHERE price > 100 GROUP BY product
    
    let table = create_sales_table();
    let scan = Box::new(TableScan::new(table));
    
    // Filter: price > 100
    let filter = Box::new(Filter::new(
        scan,
        Arc::new(BinaryComparison::new(
            4, ComparisonOp::GreaterThan, Value::Float64(100.0)
        ))
    ));
    
    // GroupBy: sum quantity by product
    let group_by = Box::new(GroupBy::new(
        filter,
        vec![1],  // product
        vec![3],  // quantity
        vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
    ));
    
    group_by.open().unwrap();
    
    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 4);  // Laptop, Phone, Tablet, Monitor
    
    // Verify sums
    let mut sums = HashMap::new();
    for i in 0..batch.row_count() {
        let product = batch.get(i, 0).unwrap();
        let sum = batch.get(i, 1).unwrap();
        
        if let (Value::String(p), Value::Int64(s)) = (product, sum) {
            sums.insert(p, s);
        }
    }
    
    assert_eq!(sums.get("Laptop"), Some(&12));
    assert_eq!(sums.get("Phone"), Some(&20));
    
    group_by.close().unwrap();
}
```

### 8.3 Schema Transformation Tests

Test that schemas transform correctly through the pipeline:

```rust
#[test]
fn test_schema_transformation_chain() {
    // Test 1: Scan schema
    {
        let table = create_sales_table();
        let mut scan = TableScan::new(table);
        scan.open().unwrap();
        let schema = scan.schema().unwrap();
        assert_eq!(schema.len(), 5);  // id, product, region, quantity, price
        scan.close().unwrap();
    }
    
    // Test 2: Filter preserves schema
    {
        let table = create_sales_table();
        let scan = Box::new(TableScan::new(table));
        let predicate = Arc::new(BinaryComparison::new(
            3, ComparisonOp::GreaterThan, Value::Int64(2)
        ));
        let mut filter = Box::new(Filter::new(scan, predicate));
        filter.open().unwrap();
        let schema = filter.schema().unwrap();
        assert_eq!(schema.len(), 5);  // Same as input
        filter.close().unwrap();
    }
    
    // Test 3: Project reduces schema
    {
        let table = create_sales_table();
        let scan = Box::new(TableScan::new(table));
        let predicate = Arc::new(BinaryComparison::new(
            3, ComparisonOp::GreaterThan, Value::Int64(2)
        ));
        let filter = Box::new(Filter::new(scan, predicate));
        let mut project = Box::new(Project::new(filter, vec![1, 3]));
        project.open().unwrap();
        let schema = project.schema().unwrap();
        assert_eq!(schema.len(), 2);  // Only product and quantity
        project.close().unwrap();
    }
}
```

### 8.4 Performance Tests

Test with larger datasets to identify performance issues:

```rust
#[test]
fn test_large_dataset_group_by() {
    let table = create_large_table(1000);  // 1000 rows
    let scan = Box::new(TableScan::new(table));
    
    // Group by category, sum value
    let group_by = Box::new(GroupBy::new(
        scan,
        vec![1],  // category
        vec![2],  // value
        vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
    ));
    
    let start = std::time::Instant::now();
    group_by.open().unwrap();
    let batch = group_by.next_batch().unwrap().unwrap();
    let duration = start.elapsed();
    
    assert_eq!(batch.row_count(), 5);  // 5 categories
    assert!(duration.as_millis() < 1000);  // Should complete in < 1 second
    
    group_by.close().unwrap();
}
```

### 8.5 Edge Case Tests

**Empty Input:**

```rust
#[test]
fn test_filter_all_rows_filtered() {
    let table = create_sales_table();
    let scan = Box::new(TableScan::new(table));
    
    // Filter that removes all rows
    let predicate = Arc::new(BinaryComparison::new(
        3, ComparisonOp::GreaterThan, Value::Int64(100)
    ));
    let filter = Box::new(Filter::new(scan, predicate));
    
    let project = Box::new(Project::new(filter, vec![1]));
    
    project.open().unwrap();
    
    let result = project.next_batch().unwrap();
    assert!(result.is_none());  // No rows left
    
    project.close().unwrap();
}
```

**Single Group:**

```rust
#[test]
fn test_group_by_single_group() {
    let table = create_sales_table();
    let scan = Box::new(TableScan::new(table));
    
    // Filter to single product
    let predicate = Arc::new(BinaryComparison::new(
        1, ComparisonOp::Equal, Value::String("Laptop".to_string())
    ));
    let filter = Box::new(Filter::new(scan, predicate));
    
    // Group by product (all same)
    let group_by = Box::new(GroupBy::new(
        filter,
        vec![1],  // product
        vec![3],  // quantity
        vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
    ));
    
    group_by.open().unwrap();
    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 1);  // Single group
    
    group_by.close().unwrap();
}
```

**Multiple Batches:**

```rust
#[test]
fn test_multiple_batches_through_pipeline() {
    let table = create_large_table(2500);
    let scan = Box::new(TableScan::new(table).with_batch_size(500));
    
    // Filter: value < 1250
    let predicate = Arc::new(BinaryComparison::new(
        2, ComparisonOp::LessThan, Value::Int64(1250)
    ));
    let filter = Box::new(Filter::new(scan, predicate));
    
    let project = Box::new(Project::new(filter, vec![0, 2]));
    
    project.open().unwrap();
    
    let mut batch_count = 0;
    let mut total_rows = 0;
    
    while let Some(batch) = project.next_batch().unwrap() {
        batch_count += 1;
        total_rows += batch.row_count();
    }
    
    assert!(batch_count > 1);  // Multiple batches
    assert_eq!(total_rows, 1249);  // Half the rows
    
    project.close().unwrap();
}
```

---

## 9. Advanced Topics

### 9.1 Predicate Pushdown

Predicate pushdown moves filters as early as possible in the pipeline to reduce data processed.

**Example:**
```rust
// SQL: SELECT name, age FROM users WHERE age > 25

// Without pushdown (BAD)
Scan(all columns) → GroupBy(department) → Filter(age > 25) → Project(name, age)

// With pushdown (GOOD)
Scan(all columns) → Filter(age > 25) → GroupBy(department) → Project(name, age)
```

**Implementation Strategy:**

1. Analyze query to identify filters
2. Determine dependencies (filter needs certain columns)
3. Move filter operator before operators that don't need filtered data
4. Ensure columns needed by filter are still available

**Benefits:**
- Reduced memory usage
- Faster processing
- Better cache utilization

### 9.2 Projection Pushdown

Similar to predicate pushdown, but for projections:

**Example:**
```rust
// SQL: SELECT name, age FROM users WHERE age > 25

// Without pushdown
Scan(all 10 columns) → Filter(age > 25) → Project(name, age)

// With pushdown
Scan(name, age) → Filter(age > 25) → Project(name, age)
```

**Implementation:**
1. Identify columns needed by query
2. Include columns needed by predicates
3. Include columns needed by grouping
4. Prune all other columns

### 9.3 Vectorized Predicate Evaluation

Current implementation evaluates predicates row-by-row. Vectorized evaluation processes entire columns:

```rust
// Row-by-row (current)
for row in 0..batch.row_count() {
    if predicate.eval(batch, row)? {
        add_row(row);
    }
}

// Vectorized (advanced)
let column = batch.get_column(predicate.column_index)?;
let mask = column.compare(predicate.op, &predicate.value)?;
let filtered_batch = batch.apply_mask(mask);
```

**Benefits:**
- CPU cache friendly
- SIMD optimization possible
- Branch prediction improvements

### 9.4 Parallel Execution

Operators can process data in parallel:

```rust
// Parallel Filter
let batches: Vec<Batch> = (0..num_threads)
    .map(|i| {
        let predicate = predicate.clone();
        let child = child.clone();
        
        thread::spawn(move || {
            // Process batches assigned to this thread
            filter_batches(child, predicate)
        })
    })
    .collect();
```

**Challenges:**
- Thread safety for shared state
- Synchronization overhead
- Load balancing
- Error propagation

### 9.5 Memory Management

**Streaming vs. Materialization:**

Current GroupBy materializes all data. Streaming approach uses less memory:

```rust
// Materializing (current)
// Reads all rows, stores them in HashMap
let mut all_rows: Vec<Row> = Vec::new();
while let Some(batch) = child.next_batch()? {
    all_rows.extend(batch.rows());
}
let groups = group_rows(all_rows);

// Streaming (advanced)
// Processes rows incrementally
let mut groups: HashMap<GroupKey, AggregateState> = HashMap::new();
while let Some(batch) = child.next_batch()? {
    for row in batch.rows() {
        let key = group_key(row);
        let agg = groups.entry(key).or_default();
        agg.update(row);
    }
}
```

**Trade-offs:**
- **Materializing**: Can sort groups, support multiple passes
- **Streaming**: Less memory, single pass only

### 9.6 Adaptive Batch Sizes

Dynamically adjust batch size based on data characteristics:

```rust
fn determine_batch_size(table_size: usize, column_count: usize) -> usize {
    // Base on cache size (e.g., 256 KB)
    let cache_size = 256 * 1024;  // bytes
    
    // Estimate row size (simplified)
    let avg_value_size = 16;  // bytes
    let row_size = column_count * avg_value_size;
    
    // Calculate optimal batch size
    let batch_size = cache_size / row_size;
    
    // Clamp to reasonable range
    batch_size.min(10000).max(100)
}
```

### 9.7 Late Materialization

Keep data in compressed format until needed:

```rust
struct CompressedColumn {
    compressed_data: Vec<u8>,
    decompression_fn: fn(&[u8]) -> Vec<Value>,
}

impl Column for CompressedColumn {
    fn get(&self, index: usize) -> Result<Value> {
        // Decompress only the needed value
        let decompressed = (self.decompression_fn)(&self.compressed_data);
        Ok(decompressed[index])
    }
}
```

**Benefits:**
- Reduced memory usage
- Better cache utilization
- Faster scans (less data to read)

---

## 10. Best Practices & Design Patterns

### 10.1 Operator Lifecycle Management

**Always follow the lifecycle:**

```rust
// Good
let mut operator = create_operator();
operator.open()?;
while let Some(batch) = operator.next_batch()? {
    process(batch);
}
operator.close()?;

// Bad - forgot to close
let mut operator = create_operator();
operator.open()?;
while let Some(batch) = operator.next_batch()? {
    process(batch);
}
// Resource leak!
```

**Use RAII pattern for automatic cleanup:**

```rust
struct OperatorGuard<'a> {
    operator: &'a mut dyn Operator,
}

impl<'a> Drop for OperatorGuard<'a> {
    fn drop(&mut self) {
        let _ = self.operator.close();
    }
}

fn execute_query<'a>(operator: &'a mut dyn Operator) -> Result<()> {
    operator.open()?;
    let _guard = OperatorGuard { operator };
    
    while let Some(batch) = operator.next_batch()? {
        process(batch);
    }
    // Automatically closed when guard goes out of scope
    Ok(())
}
```

### 10.2 Error Handling Patterns

**Provide Context:**

```rust
// Bad
Err(ExecutionError::Custom("Failed".to_string()))

// Good
Err(ExecutionError::Custom(format!(
    "Failed to group by column {}: index {} out of range (total columns: {})",
    column_name, column_index, total_columns
)))
```

**Use Error Chain:**

```rust
// When wrapping errors
self.child.open().map_err(|e| {
    ExecutionError::Custom(format!("Failed to open child operator: {}", e))
})?
```

### 10.3 Schema Validation

**Validate early:**

```rust
// Validate in open(), not during execution
fn open(&mut self) -> Result<()> {
    // Validate column indices
    for &index in &self.column_indices {
        if index >= self.child.column_count() {
            return Err(ExecutionError::InvalidColumnIndex {
                index,
                count: self.child.column_count(),
            });
        }
    }
    // ... rest of open
}

// Not during next_batch
fn next_batch(&mut self) -> Result<Option<Batch>> {
    // Don't validate here - should have been caught in open
    // ...
}
```

### 10.4 Testing Strategies

**Test Pyramid:**

```
        Integration Tests (fewer, slower)
       /              \
      /                \
   Unit Tests          End-to-End Tests
  (many, fast)         (very few, slowest)
```

**Unit Tests:**
- Test individual operators
- Fast execution
- High coverage

**Integration Tests:**
- Test operator combinations
- Medium speed
- Critical paths

**End-to-End Tests:**
- Test full queries
- Slow execution
- Validate correctness

**Property-Based Testing:**

```rust
#[quickcheck]
fn prop_filter_commutative(pred1: Predicate, pred2: Predicate, data: Vec<Row>) -> bool {
    // Filter with pred1 then pred2 should equal Filter with AND
    let result1 = filter_then_filter(data.clone(), &pred1, &pred2);
    let result2 = filter_with_and(data, &And::new(pred1, pred2));
    result1 == result2
}
```

### 10.5 Performance Optimization

**Profile Before Optimizing:**

```rust
use std::time::Instant;

fn benchmark_operator(op: &mut dyn Operator) {
    let start = Instant::now();
    op.open().unwrap();
    
    while let Some(_) = op.next_batch().unwrap() {
        // ...
    }
    
    op.close().unwrap();
    let duration = start.elapsed();
    println!("Duration: {:?}", duration);
}
```

**Measure, Don't Guess:**

```rust
// Bad
"Using Vec should be faster than HashMap because it's simpler"

// Good
"Benchmark shows Vec is 2.3x faster than HashMap for our use case with 1000 rows"
```

### 10.6 Documentation Standards

**Document Public API:**

```rust
/// Filters batches based on a predicate.
///
/// # Example
///
/// ```
/// use mini_rust_olap::execution::{Filter, TableScan, BinaryComparison, ComparisonOp};
///
/// let scan = Box::new(TableScan::new(table));
/// let predicate = Arc::new(BinaryComparison::new(
///     2,
///     ComparisonOp::GreaterThan,
///     Value::Float64(30.0),
/// ));
/// let filter = Box::new(Filter::new(scan, predicate));
/// ```
///
/// # Errors
///
/// Returns `Err(OperatorNotOpen)` if called before `open()`.
pub struct Filter { /* ... */ }
```

**Document Complex Logic:**

```rust
// Hash the group key for use in HashMap
//
// We need a custom hash implementation because Value doesn't implement Hash.
// The hash includes:
// 1. A type discriminator (1 for Int64, 2 for Float64, 3 for String)
// 2. The actual value
//
// For Float64, we use to_bits() to ensure exact hashing and avoid NaN issues.
fn hash_group_key(key: &GroupKey) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}
```

---

## 11. Learning Outcomes & Self-Assessment

### 11.1 What You've Learned

By completing Phase 4, you now understand:

1. **Vectorized Query Execution**
   - How to process data in batches for performance
   - Columnar data layout benefits
   - CPU cache optimization

2. **Operator Architecture**
   - The Operator trait and lifecycle
   - Schema management and propagation
   - Error handling patterns

3. **Query Operators**
   - TableScan: Reading data from tables
   - Filter: Predicate evaluation and filtering
   - Project: Column selection and aliasing
   - GroupBy: Hash-based grouping and aggregation

4. **Aggregate Functions**
   - Stateful aggregation design pattern
   - Implementing Count, Sum, Min, Max, Avg
   - Type safety and validation

5. **Testing**
   - Unit tests for individual operators
   - Integration tests for operator chains
   - Performance testing with large datasets

### 11.2 Self-Assessment Questions

**Conceptual Understanding:**

1. Why do we process data in batches instead of row-by-row?
2. What is the benefit of columnar storage for query execution?
3. How does the Operator lifecycle ensure proper resource management?
4. Why doesn't `Value` implement `Hash`, and how did we work around this?
5. What is the difference between schema validation in `open()` vs `next_batch()`?

**Practical Application:**

6. How would you implement a `Limit` operator that returns at most N rows?
7. How would you implement a `Sort` operator that orders results?
8. How would you implement predicate pushdown in a query optimizer?
9. How would you handle NULL values in aggregate functions differently?
10. How would you implement a `Distinct` operator that removes duplicates?

**Code Analysis:**

11. What's wrong with this code?
    ```rust
    let scan = TableScan::new(table);
    let schema = scan.schema()?;  // Get schema
    scan.open()?;
    ```

12. How would you optimize this pipeline?
    ```rust
    Scan(all columns) → Project(id, name) → Filter(age > 25)
    ```

13. What's the time complexity of this GroupBy implementation?

14. How would you test this operator comprehensively?

15. What edge cases might this operator fail on?

### 11.3 Practical Exercises

**Exercise 1: Implement Limit Operator**

```rust
pub struct Limit {
    child: Box<dyn Operator>,
    limit: usize,
    rows_returned: usize,
    // TODO: Implement
}

impl Operator for Limit {
    fn open(&mut self) -> Result<()> {
        // TODO
    }
    
    fn next_batch(&mut self) -> Result<Option<Batch>> {
        // TODO: Return at most `limit` rows
    }
    
    fn close(&mut self) -> Result<()> {
        // TODO
    }
    
    fn schema(&self) -> Result<HashMap<String, DataType>> {
        // TODO: Should match child's schema
    }
    
    fn column_names(&self) -> Result<Vec<String>> {
        // TODO: Should match child's columns
    }
    
    fn is_open(&self) -> bool {
        // TODO
    }
}
```

**Exercise 2: Implement Distinct Operator**

```rust
pub struct Distinct {
    child: Box<dyn Operator>,
    seen_values: HashSet<Vec<Value>>,
    // TODO: Implement
}

// Hint: Use HashSet to track seen rows
// Challenge: How to handle NULL values?
```

**Exercise 3: Add Variance Aggregate**

```rust
pub struct VarianceAggregate {
    sum: f64,
    sum_sq: f64,
    count: i64,
    // TODO: Implement
}

// Variance = E[X²] - E[X]²
// Where E[X] is the mean
// Hint: Track both sum and sum of squares
```

**Exercise 4: Optimize Query Pipeline**

Given this SQL:
```sql
SELECT name, SUM(quantity)
FROM sales
WHERE region = 'North'
GROUP BY name
HAVING SUM(quantity) > 100
```

Implement an optimized operator pipeline:
```rust
// TODO: Create efficient operator chain
// Hint: Consider predicate pushdown, projection pushdown
```

**Exercise 5: Implement Streaming GroupBy**

```rust
pub struct StreamingGroupBy {
    // TODO: Don't store all rows
    // Store only running aggregates per group
}

// Challenge: How to handle multiple aggregates?
// Challenge: How to handle different data types?
```

### 11.4 Next Steps

Phase 4 has built a solid foundation for query execution. The next logical steps are:

1. **SQL Parser**: Parse SQL queries into an Abstract Syntax Tree (AST)
2. **Query Planner**: Optimize query plans and choose best execution strategy
3. **More Operators**: Implement Sort, Join, Union, etc.
4. **Optimizer**: Implement predicate pushdown, projection pushdown
5. **Query Caching**: Cache query results for repeated queries

---

## 12. Appendices

### Appendix A: Code Summary

**Key Files:**

| File | Lines | Purpose |
|------|-------|---------|
| `src/execution.rs` | ~4000 | Batch, Operator trait, all operators |
| `src/aggregates.rs` | ~1100 | Aggregate function implementations |
| `tests/integration_tests.rs` | ~1000 | Integration tests for operator chaining |

**Key Types:**

```rust
// Core types
pub struct Batch { /* columnar batch of data */ }
pub trait Operator { /* query operator interface */ }

// Operators
pub struct TableScan { /* reads from table */ }
pub struct Filter { /* applies predicate */ }
pub struct Project { /* selects columns */ }
pub struct GroupBy { /* groups and aggregates */ }

// Predicates
pub trait Predicate { /* boolean expression */ }
pub struct BinaryComparison { /* column op value */ }
pub struct And { /* predicate AND predicate */ }
pub struct Or { /* predicate OR predicate */ }

// Aggregates
pub trait AggregateFunction { /* aggregate interface */ }
pub struct CountAggregate { /* counts rows */ }
pub struct SumAggregate { /* sums values */ }
pub struct MinAggregate { /* finds minimum */ }
pub struct MaxAggregate { /* finds maximum */ }
pub struct AvgAggregate { /* computes average */ }
```

### Appendix B: Performance Benchmarks

**Operator Performance (100K rows):**

| Operator | Time | Memory | Notes |
|----------|-------|---------|-------|
| TableScan | 10ms | 8 MB | Scales linearly |
| Filter | 15ms | 8 MB | Depends on selectivity |
| Project | 5ms | 2 MB | With column pruning |
| GroupBy (5 groups) | 50ms | 16 MB | Hash-based |
| GroupBy (1000 groups) | 150ms | 32 MB | More overhead |

**Query Pipeline Performance:**

```rust
// Query: SELECT name, SUM(quantity) FROM sales WHERE quantity > 5 GROUP BY name

// Pipeline: Scan → Filter → GroupBy → Project
// 100K rows, 10K groups
// Total time: 200ms
```

### Appendix C: Common Errors and Solutions

**Error 1: OperatorNotOpen**

```rust
let scan = TableScan::new(table);
scan.next_batch()?;  // Error: OperatorNotOpen
```

**Solution:** Always call `open()` before using operators.

**Error 2: SchemaNotFound**

```rust
let scan = TableScan::new(table);
let schema = scan.schema()?;  // Error: SchemaNotFound
```

**Solution:** Call `open()` before accessing schema.

**Error 3: InvalidColumnIndex**

```rust
let project = Project::new(scan, vec![0, 5, 10]);  // Error if table has < 11 columns
```

**Solution:** Validate column indices in `open()` method.

**Error 4: Memory Exhaustion**

```rust
// GroupBy with 1M groups
let group_by = GroupBy::new(scan, group_cols, agg_cols);
// Error: Out of memory
```

**Solution:** Use streaming GroupBy or limit groups.

### Appendix D: Testing Checklist

**Unit Test Checklist:**

- [ ] Test normal operation
- [ ] Test edge cases (empty, single row, large data)
- [ ] Test error conditions
- [ ] Test schema validation
- [ ] Test column name validation
- [ ] Test lifecycle (open/close)
- [ ] Test multiple batches
- [ ] Test operator not open errors
- [ ] Test operator already open errors

**Integration Test Checklist:**

- [ ] Test operator chaining
- [ ] Test schema transformations
- [ ] Test data flow through pipeline
- [ ] Test complex queries
- [ ] Test with large datasets
- [ ] Test edge cases
- [ ] Test performance

### Appendix E: Glossary

**Vectorized Execution**: Processing multiple rows simultaneously using columnar data layout.

**Batch**: A collection of rows stored in columnar format, processed as a unit.

**Operator**: A processing stage in a query pipeline that transforms data.

**Predicate**: A boolean expression that evaluates to true or false for each row.

**Aggregate**: A function that combines multiple values into a single result (SUM, AVG, COUNT).

**GroupBy**: An operation that partitions rows into groups based on key values.

**Schema**: The structure of data, including column names and types.

**Column Pruning**: Selecting only the columns needed by a query to reduce data processed.

**Predicate Pushdown**: Moving filter operations earlier in the pipeline to reduce data.

**Projection Pushdown**: Moving column selection operations earlier in the pipeline.

**Hash Join**: A join algorithm that uses hash tables for matching rows.

**Streaming**: Processing data incrementally without loading everything into memory.

---

## Conclusion

Phase 4: Query Operators Implementation represents a significant milestone in building the Mini Rust OLAP database. You've implemented:

✅ A complete query execution engine with vectorized processing
✅ Five core operators: TableScan, Filter, Project, GroupBy
✅ Five aggregate functions: Count, Sum, Min, Max, Avg
✅ Comprehensive error handling and validation
✅ Extensive testing (390 tests)

The skills you've learned - operator design, vectorized execution, aggregate functions, and testing - are fundamental to database systems. These patterns apply to many other domains:

- **Data Processing**: ETL pipelines, stream processing
- **Analytics**: Business intelligence, reporting systems
- **Search Engines**: Query processing, filtering
- **Machine Learning**: Feature extraction, data transformation

**What's Next?**

The foundation is now in place for:
1. **Phase 5**: SQL Parser and Query Planner
2. **More Operators**: Sort, Join, Union, Window functions
3. **Optimization**: Query optimization, parallel execution
4. **Persistence**: Query result caching, materialized views

Continue to the Phase 4 assessment to solidify your understanding, then move forward with building a complete database system.

Remember: Great databases are built incrementally, test-driven, and optimized based on real workloads. You've followed these principles - you're on the right track!

---

**Happy Querying!** 🚀