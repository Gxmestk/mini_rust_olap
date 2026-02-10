# Phase 4: Query Operators Implementation - Learning Guide & Assessment

## Table of Contents

1. [Overview](#overview)
2. [Learning Objectives](#learning-objectives)
3. [Phase 4 Summary](#phase-4-summary)
4. [Milestone Deep Dives](#milestone-deep-dives)
5. [Key Concepts & Patterns](#key-concepts--patterns)
6. [Code Examples](#code-examples)
7. [Best Practices](#best-practices)
8. [Common Pitfalls](#common-pitfalls)
9. [Assessment](#assessment)
10. [Next Steps](#next-steps)

---

## Overview

Phase 4 of the Mini Rust OLAP database project focuses on implementing the core query execution engine. We built a vectorized query execution system with operators that can be chained together to execute complex queries efficiently.

### What We Built

- **Vectorized Execution Engine**: Columnar data processing for high performance
- **Operator Pipeline System**: Composable operators following the Iterator pattern
- **Full Query Capabilities**: Scan, Filter, Project, GroupBy with aggregation
- **Type-Safe Implementation**: Leverages Rust's type system for correctness
- **Comprehensive Testing**: 390 tests including unit, integration, and doc tests

### Why This Matters

Understanding query execution engines is fundamental to database development. This phase teaches:
- How modern databases process queries efficiently
- Design patterns for composable systems
- Vectorized vs row-oriented processing tradeoffs
- Testing complex systems thoroughly
- Rust-specific idioms for systems programming

---

## Learning Objectives

By completing Phase 4, you should be able to:

### Core Concepts
- ✅ Understand vectorized execution and its benefits over row-oriented processing
- ✅ Design and implement the Operator trait for composable query execution
- ✅ Build a type-safe, columnar data representation (Batch)
- ✅ Implement predicate evaluation for filtering data

### Implementation Skills
- ✅ Create a TableScan operator for reading data from tables
- ✅ Implement Filter operator with arbitrary predicates (AND, OR, comparisons)
- ✅ Build Project operator for column selection and renaming
- ✅ Implement GroupBy operator with multiple aggregates
- ✅ Design aggregate functions (Count, Sum, Min, Max, Avg)

### Systems Design
- ✅ Design error handling for complex operations
- ✅ Implement proper resource lifecycle (open, next_batch, close)
- ✅ Manage memory efficiently in columnar format
- ✅ Test operator pipelines comprehensively

### Rust Mastery
- ✅ Use traits and trait objects for polymorphism
- ✅ Leverage ownership and borrowing for memory safety
- ✅ Implement custom Hash and Eq for HashMap keys
- ✅ Use Arc for shared ownership in predicates
- ✅ Write clean, idiomatic Rust code with proper error handling

---

## Phase 4 Summary

### Milestones Overview

| Milestone | Focus | Lines of Code | Test Count | Status |
|-----------|-------|---------------|------------|--------|
| 4.1 | Foundation (Batch, Operator, Errors) | ~300 | 10 | ✅ Complete |
| 4.2 | TableScan Operator | ~400 | 20 | ✅ Complete |
| 4.3 | Filter Operator | ~500 | 25 | ✅ Complete |
| 4.4 | Project Operator | ~400 | 20 | ✅ Complete |
| 4.5 | Aggregate Functions | ~1,080 | 65 | ✅ Complete |
| 4.6 | GroupBy Operator | ~1,016 | 16 | ✅ Complete |
| 4.7 | Integration Tests | ~998 | 16 | ✅ Complete |

### Total Statistics

- **Total Lines Added**: ~4,700 lines
- **Total Tests**: 390 tests (310 unit + 16 integration + 15 manual + 49 doc)
- **Test Coverage**: All operators, edge cases, and error scenarios
- **Code Quality**: Zero compilation errors, zero clippy warnings
- **Commits**: 7 commits (one per milestone)

---

## Milestone Deep Dives

### Milestone 4.1: Foundation

**Goal**: Establish core data structures and interfaces for query execution.

#### Key Components

**Batch Struct**
```rust
pub struct Batch {
    columns: Vec<Arc<dyn Column>>,
    row_count: usize,
}
```
- Columnar representation: Each column is stored separately
- Arc<dyn Column> enables type erasure and shared ownership
- Efficient for operations on single columns

**Operator Trait**
```rust
pub trait Operator: Send + Sync {
    fn open(&mut self) -> Result<()>;
    fn next_batch(&mut self) -> Result<Option<Batch>>;
    fn close(&mut self) -> Result<()>;
    fn schema(&self) -> Result<HashMap<String, DataType>>;
    fn column_names(&self) -> Result<Vec<String>>;
}
```
- Lifecycle methods: open (initialize), next_batch (fetch data), close (cleanup)
- Schema methods enable validation and optimization
- Send + Sync enables multi-threaded execution

**Error Handling**
```rust
pub enum ExecutionError {
    OperatorNotOpen,
    OperatorAlreadyOpen,
    SchemaMismatch(String),
    InvalidColumnIndex { index: usize, count: usize },
    IoError(std::io::Error),
    Custom(String),
}
```
- Specific error types for different failure modes
- Implements Display and std::error::Error for ergonomics
- From impls for easy error conversion

#### Design Decisions

**Why Columnar?**
- Cache locality: Better for modern CPUs
- SIMD-friendly: Same operations on contiguous data
- Compression: Same-type data compresses better
- Pruning: Can skip entire columns

**Why Arc<dyn Column>?**
- Type erasure: Store different column types in same Vec
- Shared ownership: Multiple references without copying
- Efficient cloning: Arc cloning is cheap

### Milestone 4.2: TableScan Operator

**Goal**: Read data from tables in batches.

#### Implementation Details

```rust
pub struct TableScan {
    table: Table,
    batch_size: usize,
    columns: Option<Vec<Arc<dyn Column>>>,
    state: OperatorState,
}
```

**Key Features**
- Lazy loading: Only loads columns on open()
- Batching: Processes data in configurable batch sizes
- Column pruning: Can select specific columns

#### Code Pattern: Batch Processing

```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    if self.current_row >= self.table.row_count() {
        return Ok(None); // No more data
    }
    
    // Calculate batch range
    let end = std::cmp::min(
        self.current_row + self.batch_size,
        self.table.row_count()
    );
    
    // Extract batch data from columns
    let batch_columns = self.columns.as_ref()
        .unwrap()
        .iter()
        .map(|col| col.slice(self.current_row, end))
        .collect();
    
    self.current_row = end;
    Ok(Some(Batch::new(batch_columns)))
}
```

#### Testing Strategy

1. **Lifecycle tests**: Verify open/next_batch/close sequence
2. **Single batch tests**: Small data fits in one batch
3. **Multiple batches tests**: Large data spans multiple batches
4. **Column pruning tests**: Schema changes with column selection
5. **Edge cases**: Empty table, single row, invalid batch size

### Milestone 4.3: Filter Operator

**Goal**: Filter rows based on predicate conditions.

#### Architecture

```
Filter {
    child: Box<dyn Operator>,           // Source of data
    predicate: Arc<dyn Predicate>,      // Filter condition
    state: OperatorState,              // Lifecycle state
}
```

#### Predicate System

**Predicate Trait**
```rust
pub trait Predicate: Send + Sync + std::fmt::Debug {
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<bool>;
}
```

**Comparison Predicates**
```rust
pub struct BinaryComparison {
    column_index: usize,
    op: ComparisonOp,
    value: Value,
}

impl Predicate for BinaryComparison {
    fn eval(&self, batch: &Batch, row_index: usize) -> Result<bool> {
        let actual = batch.get(row_index, self.column_index)?;
        
        match (&self.op, &actual, &self.value) {
            (ComparisonOp::Equal, Value::Int64(a), Value::Int64(b)) => Ok(a == b),
            (ComparisonOp::GreaterThan, Value::Int64(a), Value::Int64(b)) => Ok(a > b),
            // ... other combinations
        }
    }
}
```

**Composite Predicates**
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

#### Vectorized Filtering

```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    let child_batch = self.child.next_batch()?;
    
    match child_batch {
        None => Ok(None),
        Some(batch) => {
            // Find matching rows
            let mut selected_rows = Vec::new();
            for row_index in 0..batch.row_count() {
                if self.predicate.eval(&batch, row_index)? {
                    selected_rows.push(row_index);
                }
            }
            
            // Create filtered batch
            if selected_rows.is_empty() {
                self.next_batch() // Try next batch
            } else {
                Ok(Some(batch.select(&selected_rows)))
            }
        }
    }
}
```

#### Performance Considerations

1. **Short-circuit evaluation**: AND/OR predicates stop early when result is known
2. **Index support**: Future optimization for indexed columns
3. **Predicate pushdown**: Move filters closer to data source

### Milestone 4.4: Project Operator

**Goal**: Select, reorder, and rename columns.

#### Implementation

```rust
pub struct Project {
    child: Box<dyn Operator>,
    column_indices: Vec<usize>,
    aliases: Option<Vec<String>>,
    state: OperatorState,
    output_schema: Option<HashMap<String, DataType>>,
    output_column_names: Option<Vec<String>>,
}
```

#### Key Features

**Column Selection**
```rust
fn build_output_columns(&self, batch: &Batch) -> Result<Vec<Arc<dyn Column>>> {
    self.column_indices.iter()
        .map(|&index| Ok(batch.column(index).clone()))
        .collect()
}
```

**Column Renaming**
```rust
fn with_aliases(mut self, aliases: Vec<String>) -> Self {
    self.aliases = Some(aliases);
    self
}
```

**Schema Transformation**
```rust
fn build_schema(&mut self) -> Result<()> {
    let child_schema = self.child.schema()?;
    let child_names = self.child.column_names()?;
    
    let mut output_schema = HashMap::new();
    let mut output_names = Vec::new();
    
    for (i, &index) in self.column_indices.iter().enumerate() {
        let name = self.aliases.as_ref()
            .map(|a| a[i].clone())
            .unwrap_or_else(|| child_names[index].clone());
        
        let data_type = child_schema[&child_names[index]];
        output_schema.insert(name.clone(), data_type);
        output_names.push(name);
    }
    
    self.output_schema = Some(output_schema);
    self.output_column_names = Some(output_names);
    Ok(())
}
```

#### Testing Coverage

1. **Basic projection**: Select subset of columns
2. **Column reordering**: Change column order
3. **Aliasing**: Rename columns
4. **Schema validation**: Output schema matches expectations
5. **Edge cases**: Single column, all columns, invalid indices

### Milestone 4.5: Aggregate Functions

**Goal**: Implement statistical aggregation functions.

#### Aggregate Function Trait

```rust
pub trait AggregateFunction: Send + Sync + std::fmt::Debug {
    fn update(&mut self, value: Option<Value>) -> Result<()>;
    fn result(&self) -> Value;
    fn reset(&mut self);
    fn data_type(&self) -> DataType;
}
```

#### Implemented Aggregates

**Count**
```rust
pub struct CountAggregate {
    count: i64,
    data_type: DataType,
}

impl AggregateFunction for CountAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        if value.is_some() {
            self.count += 1;
        }
        Ok(())
    }
    
    fn result(&self) -> Value {
        Value::Int64(self.count)
    }
}
```

**Sum**
```rust
pub struct SumAggregate {
    sum: f64,
    data_type: DataType,
}

impl AggregateFunction for SumAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        match value {
            Some(Value::Int64(v)) => self.sum += v as f64,
            Some(Value::Float64(v)) => self.sum += v,
            None => {} // Ignore NULL
            _ => return Err(DatabaseError::InvalidDataType),
        }
        Ok(())
    }
    
    fn result(&self) -> Value {
        match self.data_type {
            DataType::Int64 => Value::Int64(self.sum as i64),
            DataType::Float64 => Value::Float64(self.sum),
            _ => panic!("Invalid data type"),
        }
    }
}
```

**Min/Max**
```rust
pub struct MinAggregate {
    value: Option<Value>,
    data_type: DataType,
}

impl AggregateFunction for MinAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        match (value, &self.value) {
            (Some(v), None) => self.value = Some(v),
            (Some(v), Some(current)) if v < *current => self.value = Some(v),
            _ => {}
        }
        Ok(())
    }
}
```

**Average**
```rust
pub struct AvgAggregate {
    sum: f64,
    count: i64,
    data_type: DataType,
}

impl AggregateFunction for AvgAggregate {
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
            None => {}
            _ => return Err(DatabaseError::InvalidDataType),
        }
        Ok(())
    }
    
    fn result(&self) -> Value {
        if self.count == 0 {
            Value::Float64(0.0)
        } else {
            Value::Float64(self.sum / self.count as f64)
        }
    }
}
```

#### Testing Strategy

1. **Empty input**: Aggregates with no data
2. **Single value**: Edge case for min/max/avg
3. **Multiple values**: Normal operation
4. **NULL handling**: How aggregates handle NULL values
5. **Type checking**: Wrong type errors
6. **Reset functionality**: Can reuse aggregate instances

### Milestone 4.6: GroupBy Operator

**Goal**: Group rows by key columns and compute aggregates per group.

#### Architecture

```
Input: Batch → Group by key → Apply aggregates → Output Batch
```

**Key Components**

**GroupKey for HashMap**
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

**GroupBy Structure**
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

#### Implementation Strategy

**Two-Phase Execution**

1. **Open Phase (Grouping)**
```rust
fn open(&mut self) -> Result<()> {
    self.child.open()?;
    
    // Validate column indices
    // Build output schema
    // Read all data and group it
    let mut grouped_data: HashMap<GroupKey, Vec<Vec<Option<Value>>>> = HashMap::new();
    
    while let Some(batch) = self.child.next_batch()? {
        for row_index in 0..batch.row_count() {
            // Build group key from group_by_columns
            let mut key_values = Vec::new();
            for &col_index in &self.group_by_columns {
                let value = batch.get(row_index, col_index)?;
                key_values.push(Some(value));
            }
            let key = GroupKey(key_values);
            
            // Add row to group
            let mut row_values = Vec::new();
            for col_index in 0..batch.column_count() {
                row_values.push(Some(batch.get(row_index, col_index)?));
            }
            grouped_data.entry(key).or_default().push(row_values);
        }
    }
    
    self.grouped_data = Some(grouped_data);
    Ok(())
}
```

2. **Next Batch Phase (Aggregation)**
```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    if self.results_returned {
        return Ok(None);
    }
    self.results_returned = true;
    
    let grouped_data = self.grouped_data.as_ref().unwrap();
    let mut output_columns: Vec<Vec<Option<Value>>> = 
        vec![Vec::new(); group_by_cols + agg_cols];
    
    // Process each group
    for (key, rows) in grouped_data {
        // Add group by values to output
        for (i, value) in key.0.iter().enumerate() {
            output_columns[i].push(value.clone());
        }
        
        // Compute aggregates
        for (i, agg) in self.aggregates.iter_mut().enumerate() {
            agg.reset();
            let agg_col_index = self.aggregate_columns[i];
            
            for row in rows {
                agg.update(row[agg_col_index].clone())?;
            }
            
            let result = agg.result();
            output_columns[group_by_cols + i].push(Some(result));
        }
    }
    
    Ok(Some(Batch::new(convert_to_columns(output_columns))))
}
```

#### Design Considerations

**Why Two-Phase?**
- Simplicity: Easy to understand and implement
- Correctness: All data available during aggregation
- Flexibility: Can optimize later to streaming

**Memory Tradeoffs**
- Current: All data in memory (simple but limited)
- Alternative: Streaming (complex but scalable)
- Hybrid: Hash-based with overflow to disk

#### Testing Coverage

1. **Basic grouping**: Single group-by column, single aggregate
2. **Multiple groups**: Multiple distinct groups
3. **Multiple aggregates**: Count, Sum, Min, Max, Avg on same groups
4. **Multiple group-by columns**: Composite keys
5. **Empty input**: No data to group
6. **Single group**: All rows in one group
7. **Invalid indices**: Error handling for bad column indices
8. **Schema validation**: Output schema correctness

### Milestone 4.7: Integration Tests

**Goal**: Verify operators work correctly when chained together.

#### Test Scenarios

**Scenario 1: Scan → Filter → Project**
```rust
let scan = Box::new(TableScan::new(table));
let filter = Box::new(Filter::new(
    scan,
    Arc::new(BinaryComparison::new(3, ComparisonOp::GreaterThan, Value::Int64(2)))
));
let project = Box::new(Project::new(filter, vec![1, 3]));
```

**Scenario 2: Scan → GroupBy**
```rust
let scan = Box::new(TableScan::new(table));
let group_by = Box::new(GroupBy::new(
    scan,
    vec![1],  // group by product
    vec![3],  // aggregate quantity
    vec![Box::new(SumAggregate::new(DataType::Int64))]
));
```

**Scenario 3: Scan → Filter → GroupBy → Project**
```rust
let scan = Box::new(TableScan::new(table));
let filter = Box::new(Filter::new(scan, predicate));
let group_by = Box::new(GroupBy::new(filter, group_cols, agg_cols, aggs));
let project = Box::new(Project::new(group_by, output_cols).with_aliases(names));
```

#### Testing Strategy

1. **Operator chaining**: Multiple operators in sequence
2. **Schema propagation**: Correct transformations through pipeline
3. **Data correctness**: Results match expected values
4. **Large datasets**: Performance with 1000+ rows
5. **Multiple batches**: Verify batch handling across operators
6. **Edge cases**: Empty results, single rows, extreme values

#### Helper Functions

```rust
fn create_sales_table() -> Table {
    // id, product, region, quantity, price
    // 20 rows of diverse test data
}

fn create_employees_table() -> Table {
    // id, name, department, salary
    // 10 rows for HR-style queries
}

fn create_large_table(rows: usize) -> Table {
    // id, category, value, weight
    // Configurable size for performance tests
}
```

---

## Key Concepts & Patterns

### 1. Vectorized Execution

**Concept**: Process data in batches, operating on entire columns at once.

**Benefits**:
- CPU cache efficiency (contiguous memory)
- SIMD vectorization (same operations on multiple values)
- Compiler optimization (better inlining, loop unrolling)

**Tradeoffs**:
- Complex predicates (harder to vectorize)
- Memory overhead (batches need storage)
- Code complexity (more boilerplate)

### 2. Iterator Pattern in Query Engines

**Pattern**: Each operator fetches data from child, transforms it, passes to parent.

```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    let child_batch = self.child.next_batch()?;  // Get from child
    let transformed = self.transform(child_batch)?;  // Transform
    Ok(Some(transformed))  // Return to parent
}
```

**Advantages**:
- Lazy evaluation (only compute what's needed)
- Composability (easy to chain operators)
- Memory efficiency (process one batch at a time)

### 3. Type Erasure with Trait Objects

**Pattern**: Use `dyn Trait` for runtime polymorphism.

```rust
pub struct Batch {
    columns: Vec<Arc<dyn Column>>,  // Can hold any column type
}

impl Batch {
    pub fn get(&self, row: usize, col: usize) -> Result<Value> {
        self.columns[col].get(row)  // Dynamic dispatch
    }
}
```

**Use Cases**:
- Column storage (different types in same vector)
- Predicates (different comparison types)
- Aggregates (different aggregation functions)

**Tradeoffs**:
- Dynamic dispatch overhead (vtable lookup)
- No monomorphization (no type-specific optimizations)
- Boxing overhead (heap allocation)

### 4. Lifecycle Management

**Pattern**: Explicit open/process/close phases.

```rust
// 1. Initialize resources
operator.open()?;

// 2. Process data
while let Some(batch) = operator.next_batch()? {
    // Handle batch
}

// 3. Cleanup
operator.close()?;
```

**Why Not Just next_batch()?**
- Some operators need initialization (schema validation, index building)
- Some operators need cleanup (freeing memory, releasing locks)
- Explicit phases make behavior clear and predictable

### 5. Error Handling Strategy

**Pattern**: Use Result<T, E> with specific error types.

```rust
pub enum ExecutionError {
    OperatorNotOpen,
    InvalidColumnIndex { index: usize, count: usize },
    SchemaMismatch(String),
    // ...
}

impl From<std::io::Error> for ExecutionError {
    fn from(err: std::io::Error) -> Self {
        ExecutionError::IoError(err)
    }
}
```

**Best Practices**:
- Specific error types (not just generic String)
- Context information (column index, row count)
- Error propagation with `?` operator
- Implement From for common error types

### 6. Arc for Shared Ownership

**Pattern**: Use `Arc<T>` for shared, thread-safe ownership.

```rust
pub struct Predicate {
    left: Arc<dyn Predicate>,  // Can be shared
    right: Arc<dyn Predicate>,
}
```

**Use Cases**:
- Predicate composition (AND/OR share children)
- Column sharing (multiple batches reference same column)
- Thread-safe data sharing

**Alternatives**:
- `Rc<T>`: Non-thread-safe (single-threaded)
- `&T`: Borrowed (limited lifetime)
- `Box<T>`: Unique ownership (no sharing)

### 7. HashMap with Custom Keys

**Pattern**: Implement Hash and Eq for custom types.

```rust
struct GroupKey(Vec<Option<Value>>);

impl Hash for GroupKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for value in &self.0 {
            match value {
                None => 0.hash(state),
                Some(Value::Int64(i)) => (1, i).hash(state),
                // ...
            }
        }
    }
}
```

**Requirements**:
- Must implement both Hash and Eq
- Hash must be consistent with Eq (a == b → hash(a) == hash(b))
- Hash must be stable (same value → same hash)

---

## Code Examples

### Example 1: Simple Scan Query

```rust
use mini_rust_olap::execution::{Operator, TableScan};

fn query_all_sales() -> Result<()> {
    let table = Table::load_from_disk("sales.tbl")?;
    let mut scan = TableScan::new(table);
    
    scan.open()?;
    
    while let Some(batch) = scan.next_batch()? {
        for row in 0..batch.row_count() {
            let product = batch.get(row, 1)?;
            let quantity = batch.get(row, 3)?;
            println!("{:?} x {:?}", product, quantity);
        }
    }
    
    scan.close()?;
    Ok(())
}
```

### Example 2: Filtered Query

```rust
use mini_rust_olap::execution::{
    Operator, TableScan, Filter, BinaryComparison, ComparisonOp
};
use std::sync::Arc;

fn query_expensive_items() -> Result<()> {
    let table = Table::load_from_disk("sales.tbl")?;
    let scan = Box::new(TableScan::new(table));
    
    // Filter: price > 100
    let predicate = Arc::new(BinaryComparison::new(
        4,  // price column
        ComparisonOp::GreaterThan,
        Value::Float64(100.0)
    ));
    let mut filter = Filter::new(scan, predicate);
    
    filter.open()?;
    
    while let Some(batch) = filter.next_batch()? {
        for row in 0..batch.row_count() {
            let product = batch.get(row, 1)?;
            let price = batch.get(row, 4)?;
            println!("{:?} - ${:?}", product, price);
        }
    }
    
    filter.close()?;
    Ok(())
}
```

### Example 3: Complex Query with Projection

```rust
use mini_rust_olap::execution::{
    Operator, TableScan, Filter, Project, BinaryComparison, ComparisonOp
};
use std::sync::Arc;

fn query_high_value_sales() -> Result<()> {
    let table = Table::load_from_disk("sales.tbl")?;
    let scan = Box::new(TableScan::new(table));
    
    // Filter: quantity > 5
    let predicate = Arc::new(BinaryComparison::new(
        3,  // quantity column
        ComparisonOp::GreaterThan,
        Value::Int64(5)
    ));
    let filter = Box::new(Filter::new(scan, predicate));
    
    // Project: only product and quantity
    let mut project = Box::new(Project::new(
        filter,
        vec![1, 3]  // product, quantity columns
    ).with_aliases(vec![
        "Product Name".to_string(),
        "Units Sold".to_string()
    ]));
    
    project.open()?;
    
    let batch = project.next_batch()?.unwrap();
    println!("Schema: {:?}", project.schema()?);
    println!("Columns: {:?}", project.column_names()?);
    
    project.close()?;
    Ok(())
}
```

### Example 4: GroupBy Query

```rust
use mini_rust_olap::execution::{
    Operator, TableScan, GroupBy
};
use mini_rust_olap::aggregates::{CountAggregate, SumAggregate};
use mini_rust_olap::types::DataType;

fn query_sales_by_product() -> Result<()> {
    let table = Table::load_from_disk("sales.tbl")?;
    let scan = Box::new(TableScan::new(table));
    
    // Group by product, sum quantity, count rows
    let mut group_by = Box::new(GroupBy::new(
        scan,
        vec![1],  // group by product column
        vec![3, 3],  // aggregate quantity column twice
        vec![
            Box::new(CountAggregate::new(DataType::Int64)),
            Box::new(SumAggregate::new(DataType::Int64)?)
        ]
    ));
    
    group_by.open()?;
    
    let batch = group_by.next_batch()?.unwrap();
    println!("Sales by Product:");
    println!("{:<20} {:<10} {:<10}", "Product", "Count", "Total Qty");
    
    for row in 0..batch.row_count() {
        let product = batch.get(row, 0)?;
        let count = batch.get(row, 1)?;
        let sum = batch.get(row, 2)?;
        println!("{:<20} {:<10?} {:<10?}", product, count, sum);
    }
    
    group_by.close()?;
    Ok(())
}
```

### Example 5: Full Pipeline Query

```rust
use mini_rust_olap::execution::{
    Operator, TableScan, Filter, GroupBy, Project,
    BinaryComparison, ComparisonOp
};
use mini_rust_olap::aggregates::{SumAggregate};
use mini_rust_olap::types::DataType;
use std::sync::Arc;

fn query_high_value_products() -> Result<()> {
    // SQL: SELECT product, SUM(quantity) 
    //      FROM sales 
    //      WHERE price > 100 
    //      GROUP BY product
    //      HAVING SUM(quantity) > 10
    
    let table = Table::load_from_disk("sales.tbl")?;
    let scan = Box::new(TableScan::new(table));
    
    // Filter: price > 100
    let predicate = Arc::new(BinaryComparison::new(
        4,  // price column
        ComparisonOp::GreaterThan,
        Value::Float64(100.0)
    ));
    let filter = Box::new(Filter::new(scan, predicate));
    
    // GroupBy: sum quantity by product
    let group_by = Box::new(GroupBy::new(
        filter,
        vec![1],  // group by product
        vec![3],  // sum quantity
        vec![Box::new(SumAggregate::new(DataType::Int64)?)]
    ));
    
    // Project: only product and total
    let mut project = Box::new(Project::new(
        group_by,
        vec![0, 1]  // product, sum(quantity)
    ).with_aliases(vec![
        "Product".to_string(),
        "Total Quantity".to_string()
    ]));
    
    project.open()?;
    
    let batch = project.next_batch()?.unwrap();
    for row in 0..batch.row_count() {
        let product = batch.get(row, 0)?;
        let total = batch.get(row, 1)?;
        
        // HAVING clause in application code
        if let Value::Int64(t) = total {
            if t > 10 {
                println!("{:?}: {:?}", product, total);
            }
        }
    }
    
    project.close()?;
    Ok(())
}
```

---

## Best Practices

### 1. Operator Implementation

**DO**:
- ✅ Validate all inputs in `open()` method
- ✅ Return early from `next_batch()` when no data
- ✅ Clean up resources in `close()` method
- ✅ Implement schema() and column_names() after open()

**DON'T**:
- ❌ Perform expensive work in constructor
- ❌ Assume child operator is already open
- ❌ Forget to close child operators
- ❌ Return schema before opening

### 2. Error Handling

**DO**:
- ✅ Use specific error types for different conditions
- ✅ Include context information (indices, counts)
- ✅ Implement From for common error conversions
- ✅ Document error conditions in doc comments

**DON'T**:
- ❌ Use panic!() for expected errors
- ❌ Use generic String for all errors
- ❌ Ignore errors with unwrap() without reason
- ❌ Create error messages that are hard to debug

### 3. Testing

**DO**:
- ✅ Test all three lifecycle phases (open, next_batch, close)
- ✅ Test edge cases (empty, single row, maximum values)
- ✅ Test error conditions (invalid indices, wrong types)
- ✅ Use helper functions for test data

**DON'T**:
- ❌ Only test happy path
- ❌ Hardcode large test data
- ❌ Skip testing error cases
- ❌ Use magic numbers in assertions

### 4. Performance

**DO**:
- ✅ Use Arc for shared columns (avoid copying)
- ✅ Batch operations (process multiple rows together)
- ✅ Reuse aggregate instances (call reset())
- ✅ Short-circuit predicate evaluation

**DON'T**:
- ❌ Clone entire batches unnecessarily
- ❌ Allocate in tight loops
- ❌ Use HashMap when Vec would suffice
- ❌ Ignore memory pressure

### 5. Code Organization

**DO**:
- ✅ Group related operators together
- ✅ Use consistent naming conventions
- ✅ Add comprehensive documentation
- ✅ Keep functions focused and small

**DON'T**:
- ❌ Mix concerns (parsing + execution)
- ❌ Use cryptic abbreviations
- ❌ Leave undocumented public APIs
- ❌ Write functions longer than 50 lines

---

## Common Pitfalls

### Pitfall 1: Forgetting to Open Operators

**Problem**:
```rust
let mut scan = TableScan::new(table);
let batch = scan.next_batch()?;  // Error! Operator not open
```

**Solution**:
```rust
let mut scan = TableScan::new(table);
scan.open()?;
let batch = scan.next_batch()?;
```

### Pitfall 2: Incorrect Column Indexing

**Problem**:
```rust
let scan = Box::new(TableScan::new(table));
let filter = Box::new(Filter::new(
    scan,
    Arc::new(BinaryComparison::new(
        10,  // Oops! Table only has 5 columns
        ComparisonOp::Equal,
        Value::Int64(5)
    ))
));
```

**Solution**:
```rust
// Always validate column indices
let schema = table.schema();
let column_count = schema.len();
assert!(index < column_count, "Column index out of bounds");
```

### Pitfall 3: Type Mismatches in Predicates

**Problem**:
```rust
// Comparing Int64 column to String value
Arc::new(BinaryComparison::new(
    0,
    ComparisonOp::Equal,
    Value::String("test".to_string())  // Wrong type!
))
```

**Solution**:
```rust
// Ensure types match
Arc::new(BinaryComparison::new(
    0,
    ComparisonOp::Equal,
    Value::Int64(5)  // Correct type for Int64 column
))
```

### Pitfall 4: Forgetting to Close Operators

**Problem**:
```rust
let mut scan = TableScan::new(table);
scan.open()?;
while let Some(batch) = scan.next_batch()? {
    // Process batches
}
// Forgot: scan.close()?  - Resources leaked!
```

**Solution**:
```rust
let mut scan = TableScan::new(table);
scan.open()?;
while let Some(batch) = scan.next_batch()? {
    // Process batches
}
scan.close()?;  // Always clean up
```

### Pitfall 5: Cloning Instead of Sharing

**Problem**:
```rust
// Inefficient: Cloning entire column for each batch
fn next_batch(&mut self) -> Result<Option<Batch>> {
    let child_batch = self.child.next_batch()?;
    let column = child_batch.column(0).clone();  // Expensive!
    Ok(Some(Batch::new(vec![column])))
}
```

**Solution**:
```rust
// Efficient: Sharing with Arc
fn next_batch(&mut self) -> Result<Option<Batch>> {
    let child_batch = self.child.next_batch()?;
    let column = Arc::clone(&child_batch.column(0));  // Cheap!
    Ok(Some(Batch::new(vec![column])))
}
```

### Pitfall 6: Not Handling NULL Values

**Problem**:
```rust
fn update(&mut self, value: Option<Value>) -> Result<()> {
    // Ignores NULL - may not be desired behavior
    if let Some(v) = value {
        self.sum += v;
    }
    Ok(())
}
```

**Solution**:
```rust
fn update(&mut self, value: Option<Value>) -> Result<()> {
    // Document NULL handling
    match value {
        Some(Value::Int64(v)) => self.sum += v as f64,
        Some(Value::Float64(v)) => self.sum += v,
        None => {}  // NULL is ignored (documented)
        _ => return Err(DatabaseError::InvalidDataType),
    }
    Ok(())
}
```

### Pitfall 7: Infinite Loop in next_batch()

**Problem**:
```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    if filtered_batch.is_empty() {
        self.next_batch()?  // Oops! Recursion without base case
    }
    Ok(Some(filtered_batch))
}
```

**Solution**:
```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    loop {
        let batch = self.child.next_batch()?;
        let filtered = self.apply_filter(&batch)?;
        
        if !filtered.is_empty() {
            return Ok(Some(filtered));
        }
        
        if batch.is_none() {
            return Ok(None);
        }
    }
}
```

### Pitfall 8: Mutable Borrows Conflicts

**Problem**:
```rust
fn process_batch(&mut self) -> Result<()> {
    let batch = self.next_batch()?;  // &mut self borrowed
    let schema = self.schema()?;     // &self - conflict!
    // ...
}
```

**Solution**:
```rust
fn process_batch(&mut self) -> Result<()> {
    let schema = self.schema()?;  // Get schema first
    let batch = self.next_batch()?;
    // ...
}
```

---

## Assessment

### Part 1: Theory Questions

#### Section A: Basic Concepts

1. **Vectorized Execution**
   - What is vectorized execution?
   - What are its advantages over row-oriented processing?
   - What are the tradeoffs?

2. **Operator Pattern**
   - Describe the Operator trait and its methods.
   - Why do we need explicit open/close methods?
   - How does the iterator pattern apply to query execution?

3. **Columnar Storage**
   - What is columnar storage?
   - How does the Batch struct represent columnar data?
   - Why use `Arc<dyn Column>` instead of specific column types?

4. **Type Erasure**
   - What is type erasure in Rust?
   - When should you use trait objects (`dyn Trait`)?
   - What are the performance implications?

#### Section B: Implementation Details

5. **Filter Operator**
   - How does the Filter operator evaluate predicates?
   - Why use Arc<dyn Predicate> for predicate storage?
   - How do AND and OR predicates work?

6. **Project Operator**
   - What transformations does Project perform?
   - How is schema transformation implemented?
   - What's the purpose of column aliases?

7. **GroupBy Operator**
   - Why implement custom Hash and Eq for GroupKey?
   - What's the two-phase execution strategy?
   - How are aggregates computed per group?

8. **Aggregate Functions**
   - Describe the AggregateFunction trait.
   - How does each aggregate handle NULL values?
   - Why does Avg need both sum and count?

#### Section C: Design Decisions

9. **Error Handling**
   - Why have specific error types instead of generic errors?
   - What information should ExecutionError contain?
   - How do you propagate errors through the operator chain?

10. **Testing**
    - What's the difference between unit and integration tests?
    - Why test operator lifecycle separately from logic?
    - How do you test error conditions?

11. **Performance**
    - How does batch size affect performance?
    - Why use Arc for column sharing?
    - What are the memory implications of GroupBy?

12. **Future Optimizations**
    - How could you implement streaming GroupBy?
    - What is predicate pushdown and when is it useful?
    - How would you add indexes to TableScan?

---

### Part 2: Code Exercises

#### Exercise 1: Implement Average Aggregate

**Task**: Complete the AvgAggregate implementation.

```rust
pub struct AvgAggregate {
    // TODO: Add necessary fields
}

impl AggregateFunction for AvgAggregate {
    fn new(data_type: DataType) -> Result<Self> {
        // TODO: Initialize aggregate
    }
    
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        // TODO: Update running sum and count
        // Handle NULL values appropriately
    }
    
    fn result(&self) -> Value {
        // TODO: Return average (handle division by zero)
    }
    
    fn reset(&mut self) {
        // TODO: Reset to initial state
    }
    
    fn data_type(&self) -> DataType {
        // TODO: Return output data type
    }
}
```

**Requirements**:
- Handle Int64 and Float64 input types
- Return Float64 for the result
- Ignore NULL values in count
- Return 0.0 when no non-NULL values seen
- Properly handle division by zero

**Tests to Pass**:
```rust
#[test]
fn test_avg_empty() {
    let mut avg = AvgAggregate::new(DataType::Int64)?;
    avg.result() == Value::Float64(0.0);
}

#[test]
fn test_avg_single() {
    let mut avg = AvgAggregate::new(DataType::Int64)?;
    avg.update(Some(Value::Int64(10)))?;
    avg.result() == Value::Float64(10.0);
}

#[test]
fn test_avg_multiple() {
    let mut avg = AvgAggregate::new(DataType::Int64)?;
    avg.update(Some(Value::Int64(10)))?;
    avg.update(Some(Value::Int64(20)))?;
    avg.update(Some(Value::Int64(30)))?;
    avg.result() == Value::Float64(20.0);
}

#[test]
fn test_avg_with_nulls() {
    let mut avg = AvgAggregate::new(DataType::Int64)?;
    avg.update(Some(Value::Int64(10)))?;
    avg.update(None)?;
    avg.update(Some(Value::Int64(20)))?;
    avg.result() == Value::Float64(15.0);
}
```

---

#### Exercise 2: Implement Having Clause

**Task**: Add a Having clause to GroupBy operator.

```rust
pub struct GroupBy {
    // ... existing fields ...
    having_predicate: Option<Arc<dyn Predicate>>,
}

impl GroupBy {
    pub fn with_having(mut self, predicate: Arc<dyn Predicate>) -> Self {
        self.having_predicate = Some(predicate);
        self
    }
}
```

**Requirements**:
- Add having_predicate field
- Add with_having() builder method
- Filter groups in next_batch() based on predicate
- Predicate should be evaluated on aggregate results

**Example Usage**:
```rust
let group_by = GroupBy::new(
    scan,
    vec![1],
    vec![3],
    vec![Box::new(SumAggregate::new(DataType::Int64)?)]
).with_having(Arc::new(BinaryComparison::new(
    1,  // Sum aggregate column
    ComparisonOp::GreaterThan,
    Value::Int64(100)
)));
```

**Hint**: You need to compute aggregates first, then apply predicate.

---

#### Exercise 3: Implement Distinct Aggregate

**Task**: Implement a Distinct aggregate function.

```rust
pub struct DistinctAggregate {
    values: HashSet<Value>,
    data_type: DataType,
}

impl AggregateFunction for DistinctAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        // TODO: Track distinct values
    }
    
    fn result(&self) -> Value {
        // TODO: Return count of distinct values
    }
    // ... other methods
}
```

**Requirements**:
- Track distinct values using HashSet
- Return count of distinct non-NULL values
- Handle different Value types (Int64, Float64, String)
- Implement Hash for Value if needed

**Tests**:
```rust
#[test]
fn test_distinct_all_unique() {
    let mut distinct = DistinctAggregate::new(DataType::Int64)?;
    distinct.update(Some(Value::Int64(1)))?;
    distinct.update(Some(Value::Int64(2)))?;
    distinct.update(Some(Value::Int64(3)))?;
    assert_eq!(distinct.result(), Value::Int64(3));
}

#[test]
fn test_distinct_with_duplicates() {
    let mut distinct = DistinctAggregate::new(DataType::Int64)?;
    distinct.update(Some(Value::Int64(1)))?;
    distinct.update(Some(Value::Int64(1)))?;
    distinct.update(Some(Value::Int64(2)))?;
    assert_eq!(distinct.result(), Value::Int64(2));
}
```

---

#### Exercise 4: Implement Limit Operator

**Task**: Create a Limit operator that returns at most N rows.

```rust
pub struct Limit {
    child: Box<dyn Operator>,
    limit: usize,
    offset: usize,
    count: usize,
    state: OperatorState,
}

impl Operator for Limit {
    fn open(&mut self) -> Result<()> {
        // TODO: Open child and initialize state
    }
    
    fn next_batch(&mut self) -> Result<Option<Batch>> {
        // TODO: Fetch batches from child until limit reached
        // Implement offset to skip first N rows
    }
    // ... other methods
}
```

**Requirements**:
- Limit total rows returned
- Support offset (skip first N rows)
- Stop when limit reached
- Pass through unchanged batches until limit

**Example**:
```rust
let scan = Box::new(TableScan::new(table));
let limit = Box::new(Limit::new(scan, 10, 5)?);  // 10 rows starting after offset 5
```

---

#### Exercise 5: Debug a Broken Operator

**Task**: Find and fix bugs in this buggy Filter implementation.

```rust
impl Filter {
    fn next_batch(&mut self) -> Result<Option<Batch>> {
        let batch = self.child.next_batch()?;
        
        let mut selected_rows = Vec::new();
        for row_index in 0..batch.row_count() {
            if self.predicate.eval(&batch, row_index)? {
                selected_rows.push(row_index);
            }
        }
        
        if selected_rows.is_empty() {
            return self.next_batch();  // BUG 1
        }
        
        Ok(Some(batch.select(&selected_rows)))
    }
}
```

**Bugs**:
1. What happens when all rows are filtered?
2. What's wrong with the recursion?
3. What happens when child returns None?

**Fix the bugs and explain what was wrong.**

---

### Part 3: Design Challenge

#### Challenge: Implement Streaming GroupBy

**Task**: Design and implement a streaming GroupBy that doesn't require all data in memory.

**Requirements**:
- Process data in streaming fashion
- Use bounded memory regardless of input size
- Support all aggregate types (Count, Sum, Min, Max, Avg)
- Handle spilling to disk when memory limit reached

**High-Level Design**:

1. **Architecture**:
   ```
   Input → Group → In-Memory Hash Table → Aggregate Results
              ↓
            (Full) → Spill to Disk Files
                      ↓
                Merge Sort → Final Results
   ```

2. **Components**:
   - In-memory hash table with memory limit
   - Spill manager for disk I/O
   - Merge algorithm for combining spills
   - Configurable memory threshold

3. **Interface**:
   ```rust
   pub struct StreamingGroupBy {
       child: Box<dyn Operator>,
       group_by_columns: Vec<usize>,
       aggregates: Vec<Box<dyn AggregateFunction>>,
       memory_limit: usize,  // bytes
       spills: Vec<PathBuf>,
       // ... other fields
   }
   ```

**Questions to Answer**:
1. How do you track memory usage?
2. When do you spill to disk?
3. How do you merge multiple spills?
4. How do you handle multiple aggregate types?
5. What's the performance impact?

**Bonus**: Implement a basic version with one spill.

---

### Part 4: Practical Project

#### Project: Build a Simple Query Planner

**Task**: Create a query planner that generates optimized operator trees.

**Requirements**:

1. **Query Representation**:
   ```rust
   enum Query {
       Select {
           columns: Vec<Expr>,
           from: Table,
           filter: Option<Expr>,
           group_by: Option<Vec<Expr>>,
           having: Option<Expr>,
           limit: Option<usize>,
       },
   }
   
   enum Expr {
       Column(String),
       Literal(Value),
       BinaryOp(Box<Expr>, Op, Box<Expr>),
       Aggregate(String, Box<Expr>),  // "SUM", "COUNT", etc.
   }
   ```

2. **Optimizer Rules**:
   - Predicate pushdown: Move filters before joins/groupby
   - Column pruning: Remove unused columns
   - Projection pushdown: Move projections down
   - Limit pushdown: Apply limits early

3. **Planner**:
   ```rust
   struct QueryPlanner;
   
   impl QueryPlanner {
       fn plan(&self, query: Query) -> Result<Box<dyn Operator>> {
           // 1. Parse query
           // 2. Apply optimizer rules
           // 3. Generate operator tree
           // 4. Validate plan
       }
       
       fn optimize(&self, op: Box<dyn Operator>) -> Box<dyn Operator> {
           // Apply optimization rules
       }
   }
   ```

4. **Example**:
   ```rust
   let query = Query::Select {
       columns: vec![
           Expr::Column("product".to_string()),
           Expr::Aggregate("SUM".to_string(), Box::new(Expr::Column("quantity".to_string()))),
       ],
       from: Table::load("sales.tbl")?,
       filter: Some(Expr::BinaryOp(
           Box::new(Expr::Column("price".to_string())),
           Op::GreaterThan,
           Box::new(Expr::Literal(Value::Int64(100)))
       )),
       group_by: Some(vec![Expr::Column("product".to_string())]),
       having: None,
       limit: None,
   };
   
   let planner = QueryPlanner;
   let operator = planner.plan(query)?;
   ```

**Deliverables**:
1. Query and Expr enums
2. QueryPlanner with basic planning
3. At least 2 optimizer rules
4. Unit tests for planning
5. Integration tests with actual queries

---

## Assessment Solutions

### Part 1: Theory Questions

#### Section A: Basic Concepts

**1. Vectorized Execution**
- **What**: Processing data in batches, operating on entire columns at once
- **Advantages**: 
  - CPU cache efficiency (contiguous memory access)
  - SIMD vectorization (process multiple values simultaneously)
  - Compiler optimization (better inlining, loop unrolling)
- **Tradeoffs**: 
  - Complex predicates harder to vectorize
  - Memory overhead for batch storage
  - More complex code structure

**2. Operator Pattern**
- **Methods**: open() (initialize), next_batch() (fetch data), close() (cleanup), schema() (get schema), column_names() (get column names)
- **Why open/close**: Explicit resource management, validation before processing, cleanup guarantee
- **Iterator pattern**: Each operator pulls from child, transforms, passes to parent; lazy evaluation

**3. Columnar Storage**
- **What**: Store data column-wise instead of row-wise
- **Batch representation**: Vec<Arc<dyn Column>> - separate columns with shared ownership
- **Why Arc<dyn Column>**: Type erasure (different column types), shared ownership (multiple references), cheap cloning

**4. Type Erasure**
- **What**: Hiding concrete types behind trait objects at runtime
- **When to use**: Need to store different types in same collection, runtime polymorphism needed
- **Performance**: Dynamic dispatch overhead (vtable lookup), no monomorphization, heap allocation

---

### Part 2: Code Exercise Solutions

#### Exercise 1: Average Aggregate Solution

```rust
use std::cell::RefCell;
use crate::types::{DataType, Value};
use crate::error::DatabaseError;

pub struct AvgAggregate {
    sum: f64,
    count: i64,
    data_type: DataType,
}

impl AvgAggregate {
    pub fn new(data_type: DataType) -> Result<Self> {
        match data_type {
            DataType::Int64 | DataType::Float64 => Ok(AvgAggregate {
                sum: 0.0,
                count: 0,
                data_type,
            }),
            _ => Err(DatabaseError::InvalidDataType),
        }
    }
}

impl crate::aggregates::AggregateFunction for AvgAggregate {
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
            None => {}  // Ignore NULL values
            _ => return Err(DatabaseError::InvalidDataType),
        }
        Ok(())
    }
    
    fn result(&self) -> Value {
        if self.count == 0 {
            Value::Float64(0.0)
        } else {
            Value::Float64(self.sum / self.count as f64)
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
```

---

#### Exercise 2: Having Clause Solution

```rust
impl GroupBy {
    pub fn with_having(mut self, predicate: Arc<dyn Predicate>) -> Self {
        self.having_predicate = Some(predicate);
        self
    }
}

impl Operator for GroupBy {
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
            // Compute aggregates
            for (agg_index, agg) in self.aggregates.iter_mut().enumerate() {
                agg.reset();
                let agg_col_index = self.aggregate_columns[agg_index];
                
                for row in rows {
                    agg.update(row[agg_col_index].clone())?;
                }
                
                let result = agg.result();
                output_columns[group_by_col_count + agg_index].push(Some(result));
            }
            
            // Add group by values (after computing aggregates for having predicate)
            for (i, value) in key.0.iter().enumerate() {
                output_columns[i].push(value.clone());
            }
        }
        
        // Convert to batch
        let batch = self.create_batch_from_columns(output_columns.clone())?;
        
        // Apply having predicate if present
        if let Some(ref predicate) = self.having_predicate {
            let mut selected_rows = Vec::new();
            for row_index in 0..batch.row_count() {
                if predicate.eval(&batch, row_index)? {
                    selected_rows.push(row_index);
                }
            }
            
            if selected_rows.is_empty() {
                return Ok(None);
            }
            
            // Rebuild batch with selected rows
            return Ok(Some(batch.select(&selected_rows)));
        }
        
        Ok(Some(batch))
    }
    
    // Helper to create batch from columns
    fn create_batch_from_columns(&self, columns: Vec<Vec<Option<Value>>>) -> Result<Batch> {
        let mut final_columns = Vec::new();
        let child_schema = self.child.schema()?;
        let child_column_names = self.child.column_names()?;
        
        // Group by columns
        for (i, &col_index) in self.group_by_columns.iter().enumerate() {
            let col_name = &child_column_names[col_index];
            let data_type = &child_schema[col_name];
            let values = &columns[i];
            
            final_columns.push(self.create_column(data_type, values)?);
        }
        
        // Aggregate columns
        for (i, agg) in self.aggregates.iter().enumerate() {
            let data_type = agg.data_type();
            let values = &columns[group_by_col_count + i];
            final_columns.push(self.create_column(data_type, values)?);
        }
        
        Ok(Batch::new(final_columns))
    }
}
```

---

#### Exercise 3: Distinct Aggregate Solution

```rust
use std::collections::HashSet;
use crate::types::{DataType, Value};
use crate::error::DatabaseError;
use std::hash::{Hash, Hasher};

pub struct DistinctAggregate {
    values: HashSet<ValueWrapper>,
    data_type: DataType,
}

// Wrapper to make Value hashable
#[derive(Debug, Clone)]
struct ValueWrapper(Value);

impl PartialEq for ValueWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Value::Int64(a), Value::Int64(b)) => a == b,
            (Value::Float64(a), Value::Float64(b)) => a.to_bits() == b.to_bits(),
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ValueWrapper {}

impl Hash for ValueWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.0 {
            Value::Int64(i) => i.hash(state),
            Value::Float64(f) => f.to_bits().hash(state),
            Value::String(s) => s.hash(state),
        }
    }
}

impl DistinctAggregate {
    pub fn new(data_type: DataType) -> Result<Self> {
        match data_type {
            DataType::Int64 | DataType::Float64 | DataType::String => {
                Ok(DistinctAggregate {
                    values: HashSet::new(),
                    data_type,
                })
            }
            _ => Err(DatabaseError::InvalidDataType),
        }
    }
}

impl crate::aggregates::AggregateFunction for DistinctAggregate {
    fn update(&mut self, value: Option<Value>) -> Result<()> {
        if let Some(v) = value {
            self.values.insert(ValueWrapper(v));
        }
        Ok(())
    }
    
    fn result(&self) -> Value {
        Value::Int64(self.values.len() as i64)
    }
    
    fn reset(&mut self) {
        self.values.clear();
    }
    
    fn data_type(&self) -> DataType {
        DataType::Int64  // Always returns count
    }
}
```

---

#### Exercise 4: Limit Operator Solution

```rust
pub struct Limit {
    child: Box<dyn Operator>,
    limit: usize,
    offset: usize,
    count: usize,
    state: OperatorState,
}

impl Limit {
    pub fn new(child: Box<dyn Operator>, limit: usize, offset: usize) -> Result<Self> {
        Ok(Limit {
            child,
            limit,
            offset,
            count: 0,
            state: OperatorState::NotOpen,
        })
    }
}

impl Operator for Limit {
    fn open(&mut self) -> Result<()> {
        if self.state == OperatorState::Open {
            return Err(ExecutionError::OperatorAlreadyOpen);
        }
        
        self.child.open()?;
        self.count = 0;
        self.state = OperatorState::Open;
        Ok(())
    }
    
    fn next_batch(&mut self) -> Result<Option<Batch>> {
        if self.state != OperatorState::Open {
            return Err(ExecutionError::OperatorNotOpen);
        }
        
        // Already returned limit rows
        if self.count >= self.limit {
            return Ok(None);
        }
        
        let child_batch = self.child.next_batch()?;
        
        let batch = match child_batch {
            None => return Ok(None),
            Some(b) => b,
        };
        
        // Calculate how many rows we still need
        let rows_needed = self.limit - self.count;
        
        if batch.row_count() <= self.offset {
            // Skip this entire batch
            self.offset -= batch.row_count();
            return self.next_batch();  // Get next batch
        }
        
        // Build selected rows
        let mut selected_rows = Vec::new();
        for i in self.offset..batch.row_count() {
            if selected_rows.len() >= rows_needed {
                break;
            }
            selected_rows.push(i);
        }
        
        // Update offset (will be 0 after first batch with data)
        self.offset = 0;
        
        // Create filtered batch
        let filtered_batch = batch.select(&selected_rows);
        self.count += filtered_batch.row_count();
        
        Ok(Some(filtered_batch))
    }
    
    fn close(&mut self) -> Result<()> {
        self.state = OperatorState::Closed;
        self.child.close()?;
        Ok(())
    }
    
    fn schema(&self) -> Result<HashMap<String, DataType>> {
        self.child.schema()
    }
    
    fn column_names(&mut self) -> Result<Vec<String>> {
        self.child.column_names()
    }
    
    fn is_open(&self) -> bool {
        self.state == OperatorState::Open
    }
}
```

---

#### Exercise 5: Debug Broken Filter

**Bugs Found**:

1. **Infinite recursion when all rows filtered**: 
   ```rust
   if selected_rows.is_empty() {
       return self.next_batch();  // Recursion without base case
   }
   ```
   **Problem**: If every batch is empty, this recurses forever
   **Fix**: Need to check if child returned None first

2. **Potential stack overflow**:
   **Problem**: Deep recursion can cause stack overflow
   **Fix**: Use loop instead of recursion

3. **Incorrect behavior when no more data**:
   **Problem**: Doesn't handle None from child correctly
   **Fix**: Check for None before processing

**Corrected Implementation**:

```rust
impl Filter {
    fn next_batch(&mut self) -> Result<Option<Batch>> {
        loop {
            // Get batch from child
            let child_batch = self.child.next_batch()?;
            
            // If no more data, return None
            let batch = match child_batch {
                None => return Ok(None),
                Some(b) => b,
            };
            
            // Apply filter
            let mut selected_rows = Vec::new();
            for row_index in 0..batch.row_count() {
                if self.predicate.eval(&batch, row_index)? {
                    selected_rows.push(row_index);
                }
            }
            
            // If some rows match, return filtered batch
            if !selected_rows.is_empty() {
                return Ok(Some(batch.select(&selected_rows)));
            }
            
            // Otherwise, loop to get next batch
        }
    }
}
```

**What Was Wrong**:
- Original code used recursion without base case for all-empty scenario
- Loop-based approach is cleaner and avoids stack overflow
- Properly handles None from child operator

---

## Next Steps

### Phase 5: SQL Parser and Query Planner

Now that you've mastered the execution engine, the natural next step is to build higher-level interfaces:

1. **SQL Lexer**: Tokenize SQL strings
2. **SQL Parser**: Build Abstract Syntax Tree (AST)
3. **Query Planner**: Optimize execution plans
4. **Query Executor**: Orchestrate operator pipelines

### Recommended Learning Path

1. **Review Phase 4**: Ensure you understand all concepts
2. **Complete Assessments**: Work through theory questions and exercises
3. **Build Extensions**: Add new operators, optimizations, or features
4. **Study Real Systems**: Look at PostgreSQL, ClickHouse, DuckDB
5. **Experiment with Optimizations**: Predicate pushdown, projection pruning

### Advanced Topics to Explore

- **Joins**: Implement HashJoin, SortMergeJoin, NestedLoopJoin
- **Indexes**: B-tree, bitmap, hash indexes
- **Distributed Execution**: Shuffle-based aggregation, distributed joins
- **Adaptive Execution**: Change plans based on runtime statistics
- **Vectorization**: SIMD instructions, batch processing optimizations

### Resources for Further Learning

1. **Books**:
   - "Database System Concepts" by Silberschatz
   - "Readings in Database Systems" by Hellerstein
   - "Designing Data-Intensive Applications" by Kleppmann

2. **Papers**:
   - "C-Store: A Column-Oriented DBMS"
   - "MonetDB/X100: Hyper-Pipelining Query Execution"
   - "HyPer: Adaptive Indexing in Main-Memory Column-Stores"

3. **Open Source**:
   - PostgreSQL:成熟的生产级数据库
   - ClickHouse: 列式OLAP数据库
   - DuckDB: 单机分析型数据库

---

## Conclusion

Phase 4 has equipped you with a solid foundation in query execution engine implementation. You've learned:

✅ **Core Concepts**: Vectorized execution, operator pattern, columnar storage
✅ **Implementation Skills**: Building composable, type-safe operators
✅ **Testing Mastery**: Comprehensive testing of complex systems
✅ **Rust Expertise**: Traits, ownership, Arc, HashMap customization
✅ **Systems Design**: Error handling, lifecycle management, performance

### What You've Built

A fully functional query execution engine with:
- 5 core operators (Scan, Filter, Project, GroupBy, Limit)
- 5 aggregate functions (Count, Sum, Min, Max, Avg)
- 390 comprehensive tests
- Zero compilation errors or warnings
- Production-ready error handling

### What You Can Do Now

1. **Extend the system**: Add new operators, aggregates, or optimizations
2. **Build higher layers**: SQL parser, query planner, optimizer
3. **Experiment with performance**: Benchmark, profile, optimize
4. **Learn from real systems**: Study PostgreSQL, ClickHouse, DuckDB

### Final Assessment Score

Use this checklist to evaluate your understanding:

- [ ] Can explain vectorized vs row-oriented execution
- [ ] Understand the Operator trait and lifecycle
- [ ] Can implement a new operator from scratch
- [ ] Know when to use Arc vs Box vs references
- [ ] Can implement custom Hash/Eq for HashMap keys
- [ ] Understand error handling patterns in Rust
- [ ] Can write comprehensive tests for complex systems
- [ ] Can debug memory and ownership issues
- [ ] Can design and optimize query plans
- [ ] Ready to build Phase 5 (SQL Parser & Planner)

If you can check all 10 items, congratulations! You're ready to tackle advanced database engineering challenges.

---

**Good luck with your database engineering journey! 🚀**