# Phase 2 Learning Guide: Building the Storage Layer
## A Comprehensive Introduction to Table Management and Catalog Design

---

## 📚 Table of Contents

1. [Introduction](#chapter-1-introduction)
2. [Table Design Fundamentals](#chapter-2-table-design-fundamentals)
3. [HashMap and Collections in Rust](#chapter-3-hashmap-and-collections-in-rust)
4. [Advanced Trait Implementations](#chapter-4-advanced-trait-implementations)
5. [Schema Validation and Type Safety](#chapter-5-schema-validation-and-type-safety)
6. [Catalog Design Pattern](#chapter-6-catalog-design-pattern)
7. [Error Handling for Complex Types](#chapter-7-error-handling-for-complex-types)
8. [Testing Complex Data Structures](#chapter-8-testing-complex-data-structures)
9. [Integration and Modularity](#chapter-9-integration-and-modularity)
10. [Best Practices & Design Patterns](#chapter-10-best-practices--design-patterns)
11. [Learning Outcomes](#chapter-11-learning-outcomes)

---

## Chapter 1: Introduction

### 1.1 What is Phase 2?

Phase 2 implements the **Storage Layer** of Mini Rust OLAP, focusing on two core components:

1. **Table**: The fundamental data container that organizes columns
2. **Catalog**: The metadata repository that manages all tables

#### Why a Separate Storage Layer?

**Separation of Concerns:**

- **Phase 1 (Columns)**: Low-level data storage primitives
  - Individual column implementations
  - Type-specific operations
  - Basic CRUD for single columns
  
- **Phase 2 (Tables & Catalog)**: High-level data management
  - Multiple column coordination
  - Schema enforcement
  - Multi-table orchestration

### 1.2 Phase 2 Architecture

```
┌─────────────────────────────────────────┐
│         Phase 2 Storage Layer         │
├─────────────────────────────────────────┤
│  📊 Table Structure                │
│  ├─ Name identifier                 │
│  ├─ Schema (column definitions)      │
│  ├─ Column index (fast lookup)      │
│  └─ Column collection               │
├─────────────────────────────────────────┤
│  📚 Catalog System                 │
│  ├─ Table registry                  │
│  ├─ Metadata management             │
│  └─ Table operations               │
└─────────────────────────────────────────┘
           ↑              ↑
           │              │
    Phase 1 Columns    Phase 3+ Query Engine
```

### 1.3 Learning Objectives

By the end of Phase 2, you will understand:

**Rust Concepts:**
- ✅ HashMap and BTreeMap for fast lookups
- ✅ Advanced trait implementations (manual Clone)
- ✅ Ownership with complex nested types
- ✅ Iterator patterns with collections
- ✅ Builder patterns for construction
- ✅ Error handling across module boundaries
- ✅ Testing strategies for complex data structures

**Database Concepts:**
- ✅ Schema definition and validation
- ✅ Table-level operations vs column-level
- ✅ Centralized metadata management
- ✅ Catalog design patterns
- ✅ Row insertion with type inference
- ✅ Column projection and selection

### 1.4 Phase 2 Deliverables

✅ **Completed Components:**
- Table module with full CRUD operations (33 tests)
- Catalog module for table management (25 tests)
- Schema validation and integrity checks
- Manual trait implementations (Clone, Display)
- Integration tests for full workflows

---

## Chapter 2: Table Design Fundamentals

### 2.1 Understanding the Table Concept

In database terminology, a **Table** is a collection of related data organized in rows and columns. However, in our columnar OLAP database, we store data differently:

```rust
pub struct Table {
    name: String,                    // Table identifier
    column_index: HashMap<String, usize>, // Name → position mapping
    columns: Vec<Box<dyn Column>>,     // Actual column data
    schema: HashMap<String, DataType>,   // Column name → type
}
```

#### Why This Structure?

**Columnar Storage Benefits:**

1. **Efficient Queries**: Read only needed columns
2. **Compression**: Similar data types in memory blocks
3. **Caching**: Better CPU cache utilization

**Indexing HashMap:**

```rust
column_index: HashMap<String, usize>
// "name" → 0
// "age"  → 1
// "email" → 2
```

Fast O(1) column lookup by name!

### 2.2 Table Lifecycle

```rust
// 1. Creation
let mut table = Table::new("users".to_string());

// 2. Schema Definition (adding columns)
let id_col = create_column(DataType::Int64);
table.add_column("id".to_string(), id_col)?;

let name_col = create_column(DataType::String);
table.add_column("name".to_string(), name_col)?;

// 3. Data Insertion
table.add_row(vec!["1".to_string(), "Alice".to_string()])?;
table.add_row(vec!["2".to_string(), "Bob".to_string()])?;

// 4. Query
let row_count = table.row_count();  // 2
let name = table.get_value("name", 0)?;  // "Alice"
```

### 2.3 Manual Clone Implementation

The Table struct uses `Box<dyn Column>` which doesn't automatically derive Clone. We implement it manually:

```rust
impl Clone for Table {
    fn clone(&self) -> Self {
        let mut new_table = Table::new(self.name.clone());
        
        // Deep copy each column
        for (name, data_type) in &self.schema {
            let index = self.column_index.get(name).unwrap();
            let original = &self.columns[*index];
            
            // Slice to get all values
            let values = original.slice(Some(0..original.len()));
            
            // Create new column and populate
            let mut column = create_column(*data_type);
            for value in values {
                column.as_mut().push_value(value)?;
            }
            
            new_table.add_column(name.clone(), column)?;
        }
        
        new_table
    }
}
```

**Key Concepts:**

- **Deep Clone**: Each column must be cloned independently
- **Type Preservation**: Schema ensures correct column types
- **Error Propagation**: Use `?` operator for clean error handling

### 2.4 Row Insertion with Type Inference

The `add_row` method demonstrates string-to-value conversion:

```rust
pub fn add_row(&mut self, values: Vec<String>) -> Result<()> {
    // Validate count matches schema
    if values.len() != self.column_count() {
        return Err(DatabaseError::table_error(
            format!("Expected {} values, got {}", 
                    self.column_count(), values.len())
        ));
    }
    
    // Process each value
    for (index, value) in values.iter().enumerate() {
        let column = self.columns[index].as_mut();
        
        let parsed = match column.data_type() {
            DataType::Int64 => {
                value.parse::<i64>()
                    .map(Value::Int64)
                    .map_err(|_| DatabaseError::column_error(
                        format!("Invalid integer: '{}'", value)
                    ))?
            },
            DataType::Float64 => {
                value.parse::<f64>()
                    .map(Value::Float64)
                    .map_err(|_| DatabaseError::column_error(
                        format!("Invalid float: '{}'", value)
                    ))?
            },
            DataType::String => Value::String(value.clone()),
        };
        
        column.push_value(parsed)?;
    }
    
    Ok(())
}
```

**Pattern Matching with Error Handling:**

- **parse()**: Returns `Result<T, ParseIntError>`
- **map_err()**: Converts parse errors to `DatabaseError`
- **? Operator**: Propagates errors early

### 2.5 Self-Check Questions

1. Why do we need both `columns` Vec and `column_index` HashMap?
   - Answer: Vec for ordered iteration, HashMap for O(1) name lookup
   
2. What happens if you call `add_row()` with wrong number of values?
   - Answer: Returns `DatabaseError::TableError` with descriptive message
   
3. Why can't we derive `Clone` for Table?
   - Answer: `Box<dyn Column>` doesn't implement Clone automatically

---

## Chapter 3: HashMap and Collections in Rust

### 3.1 HashMap Fundamentals

Rust's `HashMap<K, V>` provides O(1) average-time lookups:

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();

// Insert
scores.insert("Alice".to_string(), 95);
scores.insert("Bob".to_string(), 87);

// Access
if let Some(score) = scores.get("Alice") {
    println!("Alice scored {}", score);
}

// Modify
scores.entry("Charlie".to_string())
    .or_insert(0);
scores.entry("Charlie".to_string())
    .and_modify(|s| *s += 10);
```

### 3.2 Entry API for Conditional Operations

The Entry API provides efficient conditional operations:

```rust
use std::collections::HashMap;

let mut table_counts: HashMap<String, usize> = HashMap::new();

// Increment count, inserting 0 if not present
table_counts.entry("users".to_string())
    .and_modify(|count| *count += 1)
    .or_insert(1);

// Pattern: ensure exists
table_counts.entry("products".to_string())
    .or_insert(0);
```

### 3.3 BTreeMap for Sorted Data

When order matters, use `BTreeMap`:

```rust
use std::collections::BTreeMap;

let mut sorted = BTreeMap::new();
sorted.insert("zebra".to_string(), 1);
sorted.insert("apple".to_string(), 2);
sorted.insert("middle".to_string(), 3);

// Iteration is sorted!
for (key, value) in &sorted {
    println!("{}: {}", key, value);
}
// Output: apple: 2, middle: 3, zebra: 1
```

### 3.4 Collection Methods in Table Module

```rust
// Get all column names
pub fn column_names(&self) -> Vec<String> {
    self.schema.keys().cloned().collect()
}

// Check if column exists
pub fn has_column(&self, name: &str) -> bool {
    self.schema.contains_key(name)
}

// Get column type
pub fn get_column_type(&self, name: &str) -> Result<DataType> {
    self.schema.get(name)
        .copied()  // Converts &DataType to DataType
        .ok_or_else(|| DatabaseError::column_error(
            format!("Column '{}' not found", name)
        ))
}
```

### 3.5 Iterator Chains for Data Processing

```rust
// Filter and transform data
let int_columns: Vec<&String> = self.schema.iter()
    .filter(|(_, dtype)| **dtype == DataType::Int64)
    .map(|(name, _)| name)
    .collect();

// Find columns matching criteria
let large_columns: Vec<&str> = self.columns.iter()
    .filter(|col| col.len() > 1000)
    .map(|col| {
        // Find name by index
        let index = self.columns.iter()
            .position(|c| std::ptr::eq(c, col))?;
        self.column_index.iter()
            .find(|(_, &idx)| idx == index)
            .map(|(name, _)| name.as_str())
    })
    .filter_map(Result::ok)
    .collect();
```

### 3.6 Self-Check Questions

1. What's the difference between HashMap and BTreeMap?
   - Answer: HashMap is O(1) but unordered; BTreeMap is O(log n) but sorted
   
2. When should you use `.copied()` with HashMap values?
   - Answer: When the value implements Copy and you need owned value instead of reference
   
3. What does `.keys().cloned().collect()` do?
   - Answer: Gets all keys (as references), clones each, collects into Vec

---

## Chapter 4: Advanced Trait Implementations

### 4.1 Manual Trait Implementations

Sometimes automatic derives don't work. Consider Table:

```rust
pub struct Table {
    name: String,
    column_index: HashMap<String, usize>,
    columns: Vec<Box<dyn Column>>,  // Problem: Column trait doesn't derive Clone
    schema: HashMap<String, DataType>,
}
```

**Solution: Implement Clone manually**

```rust
impl Clone for Table {
    fn clone(&self) -> Self {
        let mut new_table = Table::new(self.name.clone());
        
        for (name, data_type) in &self.schema {
            let index = self.column_index.get(name).unwrap();
            let original = &self.columns[*index];
            let values = original.slice(Some(0..original.len()));
            
            let mut column = create_column(*data_type);
            for i in 0..values.len() {
                let value = values.get(i).ok_or_else(|| {
                    DatabaseError::column_error(
                        format!("Failed to get value at index {}", i)
                    )
                })?;
                column.as_mut().push_value(value)?;
            }
            
            new_table.add_column(name.clone(), column)?;
        }
        
        new_table
    }
}
```

### 4.2 Display Trait for Pretty Printing

```rust
impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Table: {}", self.name)?;
        writeln!(f, "Rows: {}", self.row_count())?;
        writeln!(f, "Columns: {} ({})", 
                 self.column_count(),
                 self.column_names().join(", "))?;
        
        // Show sample data if small
        if self.row_count() > 0 && self.row_count() <= 10 {
            writeln!(f, "\nSample data:")?;
            
            // Print header
            let names: Vec<&String> = self.schema.keys().collect();
            write!(f, "|")?;
            for name in &names {
                write!(f, " {} |", name)?;
            }
            writeln!(f)?;
            
            // Print data rows
            for row in 0..self.row_count().min(5) {
                write!(f, "|")?;
                for name in &names {
                    let col = self.get_column(name).map_err(|_| std::fmt::Error)?;
                    let val = col.get(row).map_err(|_| std::fmt::Error)?;
                    write!(f, " {} |", val)?;
                }
                writeln!(f)?;
            }
        }
        
        Ok(())
    }
}
```

### 4.3 Default Trait for Convenience

```rust
impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}

// Usage
let catalog: Catalog = Catalog::default();
// or
let catalog = Catalog::new();
```

### 4.4 Trait Objects and Dynamic Dispatch

```rust
// Vec of trait objects
columns: Vec<Box<dyn Column>>

// Iterating over trait objects
for column in &self.columns {
    println!("Type: {}", column.data_type());
    println!("Length: {}", column.len());
}

// Calling trait methods
let col = table.get_column("name")?;
let value = col.get(0)?;  // Works on &dyn Column
```

**Key Points:**

- **Trait Object**: `dyn Trait` allows runtime polymorphism
- **Box**: Heap allocation needed for trait objects of unknown size
- **Dynamic Dispatch**: Method calls resolved at runtime

### 4.5 Self-Check Questions

1. Why can't we derive Clone for Table?
   - Answer: Because `Box<dyn Column>` doesn't implement Clone automatically
   
2. What's the difference between `Box<dyn Trait>` and `Box<dyn Trait + Send>`?
   - Answer: The latter requires the trait object to be thread-safe (Send + Sync)
   
3. When implementing Display trait, what does `writeln!` return?
   - Answer: `std::fmt::Result`, which is `Result<(), std::fmt::Error>`

---

## Chapter 5: Schema Validation and Type Safety

### 5.1 Understanding Database Schemas

A **schema** defines the structure of a table:

```rust
schema: HashMap<String, DataType>
// "id"    → DataType::Int64
// "name"   → DataType::String
// "score"  → DataType::Float64
```

### 5.2 Validation Strategies

**1. Duplicate Name Prevention**

```rust
pub fn add_column(&mut self, name: String, column: Box<dyn Column>) -> Result<()> {
    // Check for duplicates
    if self.schema.contains_key(&name) {
        return Err(DatabaseError::column_error(
            format!("Column '{}' already exists", name)
        ));
    }
    
    // ... rest of implementation
}
```

**2. Row Count Consistency**

```rust
pub fn add_column(&mut self, name: String, column: Box<dyn Column>) -> Result<()> {
    // ... duplicate check ...
    
    if !self.columns.is_empty() {
        let existing_rows = self.columns[0].len();
        let new_rows = column.len();
        
        if existing_rows != new_rows {
            return Err(DatabaseError::table_error(
                format!("Row count mismatch: expected {}, got {}", 
                        existing_rows, new_rows)
            ));
        }
    }
    
    // ... rest of implementation
}
```

**3. Type Consistency in Operations**

```rust
pub fn get_column_type(&self, name: &str) -> Result<DataType> {
    self.schema.get(name)
        .copied()  // DataType implements Copy
        .ok_or_else(|| DatabaseError::column_error(
            format!("Column '{}' not found in table '{}'", name, self.name)
        ))
}
```

### 5.3 Schema Evolution Considerations

**Current Implementation: Immutable Schema**

```rust
// Once added, columns cannot change type
let id_col = create_column(DataType::Int64);
table.add_column("id".to_string(), id_col)?;

// Later attempts to modify schema must create new table
```

**Future Enhancement: Schema Migration**

```rust
pub fn alter_column_type(&mut self, name: &str, new_type: DataType) 
    -> Result<()> 
{
    // 1. Validate new type is compatible
    let old_type = self.get_column_type(name)?;
    if !old_type.can_cast_to(new_type) {
        return Err(DatabaseError::column_error(
            format!("Cannot cast {} to {}", old_type, new_type)
        ));
    }
    
    // 2. Transform all values
    let index = self.column_index[name];
    let old_col = &self.columns[index];
    let values: Vec<Value> = (0..old_col.len())
        .map(|i| old_col.get(i).unwrap())
        .map(|v| v.cast_to(new_type).unwrap())
        .collect();
    
    // 3. Replace column
    let mut new_col = create_column(new_type);
    for val in values {
        new_col.as_mut().push_value(val)?;
    }
    
    // 4. Update schema
    self.schema.insert(name.to_string(), new_type);
    self.columns[index] = new_col;
    
    Ok(())
}
```

### 5.4 Self-Check Questions

1. Why check row count consistency when adding columns?
   - Answer: To ensure all columns have same number of rows for valid operations
   
2. What's the benefit of immutable schema in early implementation?
   - Answer: Simpler code, avoids complex migration logic
   
3. How would you handle schema migration in production?
   - Answer: With transactional ALTER TABLE, compatibility checks, and data validation

---

## Chapter 6: Catalog Design Pattern

### 6.1 The Catalog Concept

A **Catalog** is the central metadata repository:

```rust
pub struct Catalog {
    tables: HashMap<String, Table>,
}
```

**Responsibilities:**

1. **Table Registration**: Track all tables
2. **Name Resolution**: Prevent duplicates
3. **Retrieval**: Fast table access
4. **Lifecycle Management**: Create, drop, rename

### 6.2 Catalog Operations

```rust
impl Catalog {
    // Registration with validation
    pub fn register_table(&mut self, table: Table) -> Result<()> {
        let name = table.name().to_string();
        
        if self.tables.contains_key(&name) {
            return Err(DatabaseError::catalog_error(
                format!("Table '{}' already exists", name)
            ));
        }
        
        self.tables.insert(name, table);
        Ok(())
    }
    
    // Immutable access
    pub fn get_table(&self, name: &str) -> Result<&Table> {
        self.tables.get(name).ok_or_else(|| {
            DatabaseError::catalog_error(
                format!("Table '{}' not found", name)
            )
        })
    }
    
    // Mutable access
    pub fn get_table_mut(&mut self, name: &str) -> Result<&mut Table> {
        self.tables.get_mut(name).ok_or_else(|| {
            DatabaseError::catalog_error(
                format!("Table '{}' not found", name)
            )
        })
    }
}
```

### 6.3 Transactional Operations Pattern

```rust
impl Catalog {
    // Atomic rename
    pub fn rename_table(&mut self, old_name: &str, new_name: String) 
        -> Result<()> 
    {
        // Validation
        if !self.table_exists(old_name) {
            return Err(DatabaseError::catalog_error(
                format!("Cannot rename '{}': not found", old_name)
            ));
        }
        
        if self.table_exists(&new_name) {
            return Err(DatabaseError::catalog_error(
                format!("Cannot rename to '{}': already exists", new_name)
            ));
        }
        
        // Atomic operation
        let table = self.tables.remove(old_name).unwrap();
        self.tables.insert(new_name, table);
        
        Ok(())
    }
    
    // Conditional operations
    pub fn drop_table(&mut self, name: &str) -> Result<()> {
        self.tables.remove(name)
            .ok_or_else(|| {
                DatabaseError::catalog_error(
                    format!("Cannot drop '{}': not found", name)
                )
            })?;
        Ok(())
    }
}
```

### 6.4 Catalog Integration with Table Operations

```rust
// Example: Create table with data, register, modify
let mut catalog = Catalog::new();

// 1. Create and configure table
let mut users = Table::new("users".to_string());
let id_col = create_column(DataType::Int64);
users.add_column("id".to_string(), id_col)?;

// 2. Register in catalog
catalog.register_table(users)?;

// 3. Add data through catalog
let table = catalog.get_table_mut("users")?;
table.add_row(vec!["1".to_string()])?;
table.add_row(vec!["2".to_string()])?;

// 4. Query through catalog
let rows = catalog.get_table("users")?.row_count();
println!("Users table has {} rows", rows);
```

### 6.5 Self-Check Questions

1. Why separate Catalog from individual Table operations?
   - Answer: Centralized metadata management, cross-table operations, cleaner separation
   
2. What's the risk of allowing mutable table access through catalog?
   - Answer: Caller can modify table in ways catalog can't track (but useful for flexibility)
   
3. How would you implement table locking for multi-threaded access?
   - Answer: Use RwLock<HashMap<String, Table>> for concurrent reads, exclusive writes

---

## Chapter 7: Error Handling for Complex Types

### 7.1 Error Context and Chaining

```rust
// DatabaseError::catalog_error creates context
pub fn catalog_error(msg: impl Into<String>) -> Self {
    Self::CatalogError(msg.into())
}

// Usage with context
pub fn get_table(&self, name: &str) -> Result<&Table> {
    self.tables.get(name).ok_or_else(|| {
        DatabaseError::catalog_error(
            format!("Table '{}' not found in catalog", name)
        )
    })
}
```

### 7.2 Early Return Pattern

```rust
pub fn add_column(&mut self, name: String, column: Box<dyn Column>) -> Result<()> {
    // Validation 1: Duplicate names
    if self.schema.contains_key(&name) {
        return Err(DatabaseError::column_error(
            format!("Column '{}' already exists", name)
        ));
    }
    
    // Validation 2: Row count mismatch
    if !self.columns.is_empty() {
        let expected = self.columns[0].len();
        let actual = column.len();
        
        if expected != actual {
            return Err(DatabaseError::table_error(
                format!("Expected {} rows, got {}", expected, actual)
            ));
        }
    }
    
    // Success path
    let data_type = column.data_type();
    let index = self.columns.len();
    self.column_index.insert(name.clone(), index);
    self.columns.push(column);
    self.schema.insert(name, data_type);
    
    Ok(())
}
```

### 7.3 Error Conversion with map_err

```rust
pub fn add_row(&mut self, values: Vec<String>) -> Result<()> {
    // Validate count
    if values.len() != self.column_count() {
        return Err(DatabaseError::table_error(
            format!("Expected {} values, got {}", 
                    self.column_count(), values.len())
        ));
    }
    
    // Parse each value with error conversion
    for (index, value) in values.iter().enumerate() {
        let column = self.columns[index].as_mut();
        
        let parsed = match column.data_type() {
            DataType::Int64 => value.parse::<i64>()
                .map(Value::Int64)
                .map_err(|_| DatabaseError::column_error(
                    format!("Invalid integer: '{}'", value)
                ))?,
            DataType::Float64 => value.parse::<f64>()
                .map(Value::Float64)
                .map_err(|_| DatabaseError::column_error(
                    format!("Invalid float: '{}'", value)
                ))?,
            DataType::String => Value::String(value.clone()),
        };
        
        column.push_value(parsed)?;
    }
    
    Ok(())
}
```

### 7.4 Handling unwrap_err() Without Debug Trait

```rust
// Problem: unwrap_err() requires T: Debug
let result = catalog.get_table("nonexistent");
// result: Result<&Table, DatabaseError>
// unwrap_err() requires &Table: Debug (but Table doesn't implement Debug!)

// Solution: Use match
let error_msg = match result {
    Err(e) => format!("{}", e),  // Convert Error to String
    Ok(_) => panic!("Expected error"),
};
assert!(error_msg.contains("not found"));
```

### 7.5 Self-Check Questions

1. Why use `format!()` in error creation?
   - Answer: To create dynamic error messages with interpolated values
   
2. What's the benefit of early return pattern in validation?
   - Answer: Clear control flow, prevents deep nesting, easy to read
   
3. How would you add stack traces to DatabaseError?
   - Answer: Use `anyhow` crate with `#[source]` attribute on error variant

---

## Chapter 8: Testing Complex Data Structures

### 8.1 Unit Testing Strategies

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{Column, IntColumn};
    use crate::types::Value;
    
    #[test]
    fn test_add_column_with_validation() {
        let mut table = Table::new("test".to_string());
        
        // Add first column
        let col1 = create_column(DataType::Int64);
        assert!(table.add_column("id".to_string(), col1).is_ok());
        
        // Test duplicate
        let col2 = create_column(DataType::String);
        assert!(table.add_column("id".to_string(), col2).is_err());
    }
    
    #[test]
    fn test_row_count_mismatch_validation() {
        let mut table = Table::new("test".to_string());
        
        let mut col1 = IntColumn::new();
        let _ = col1.push_value(Value::Int64(1));
        let _ = col1.push_value(Value::Int64(2));
        table.add_column("id".to_string(), Box::new(col1)).unwrap();
        
        let mut col2 = IntColumn::new();
        let _ = col2.push_value(Value::Int64(10));
        // Only 1 row, but col1 has 2
        
        let result = table.add_column("score".to_string(), Box::new(col2));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatch"));
    }
}
```

### 8.2 Integration Testing

```rust
#[test]
fn test_catalog_with_table_operations() {
    let mut catalog = Catalog::new();
    
    // Create table with data
    let mut table = Table::new("test".to_string());
    let mut col = IntColumn::new();
    let _ = col.push_value(Value::Int64(1));
    let _ = col.push_value(Value::Int64(2));
    table.add_column("id".to_string(), Box::new(col)).unwrap();
    
    // Register
    catalog.register_table(table).unwrap();
    
    // Retrieve and verify
    let retrieved = catalog.get_table("test").unwrap();
    assert_eq!(retrieved.name(), "test");
    assert_eq!(retrieved.row_count(), 2);
    
    // Modify through catalog
    let table_mut = catalog.get_table_mut("test").unwrap();
    let _ = table_mut.add_row(vec!["3".to_string()]);
    
    // Verify modification
    let retrieved = catalog.get_table("test").unwrap();
    assert_eq!(retrieved.row_count(), 3);
}
```

### 8.3 Property-Based Testing (Optional)

```rust
#[cfg(test)]
mod prop_tests {
    use super::*;
    use crate::column::{Column, IntColumn, StringColumn};
    use crate::types::Value;
    
    // Property: Row count is consistent across columns
    #[test]
    fn test_row_count_consistency() {
        let mut table = Table::new("test".to_string());
        
        // Add columns with same row count
        for name in &["col1", "col2", "col3"] {
            let mut col = IntColumn::new();
            for i in 0..10 {
                let _ = col.push_value(Value::Int64(i));
            }
            table.add_column(name.to_string(), Box::new(col)).unwrap();
        }
        
        // All columns should have same length
        for name in table.column_names() {
            let col = table.get_column(&name).unwrap();
            assert_eq!(col.len(), table.row_count());
        }
    }
    
    // Property: Schema and columns stay synchronized
    #[test]
    fn test_schema_column_sync() {
        let mut table = Table::new("test".to_string());
        
        let col1 = create_column(DataType::Int64);
        let col2 = create_column(DataType::String);
        
        table.add_column("id".to_string(), col1).unwrap();
        table.add_column("name".to_string(), col2).unwrap();
        
        // Schema count equals columns count
        assert_eq!(table.schema.len(), table.column_count());
        
        // All schema entries have corresponding columns
        for name in table.column_names() {
            assert!(table.has_column(&name));
            assert!(table.schema.contains_key(&name));
        }
    }
}
```

### 8.4 Self-Check Questions

1. Why test row count mismatch validation?
   - Answer: To ensure data integrity across columns, prevent malformed tables
   
2. What's the difference between unit and integration tests here?
   - Answer: Unit tests test individual methods; integration tests test full workflows
   
3. How would you add performance benchmarks?
   - Answer: Use `criterion` crate with `#[bench]` attribute for timing operations

---

## Chapter 9: Integration and Modularity

### 9.1 Module Organization

```
src/
├── lib.rs          // Public API exports
├── error.rs        // Error types (shared)
├── types.rs        // Data types (shared)
├── column.rs       // Column implementations (Phase 1)
├── table.rs        // Table struct (Phase 2)
├── catalog.rs      // Catalog struct (Phase 2)
└── ...
```

### 9.2 Re-exports for Clean API

```rust
// lib.rs
pub use error::{DatabaseError, Result};
pub use types::{DataType, Value};
pub use column::{Column, IntColumn, FloatColumn, StringColumn};
pub use table::Table;
pub use catalog::Catalog;

// User can import cleanly
use mini_rust_olap::{Table, Catalog, Result, DataType};
```

### 9.3 Cross-Module Dependencies

```rust
// table.rs uses error and types
use crate::error::{DatabaseError, Result};
use crate::types::{DataType, Value};
use crate::column::{create_column, Column};

// catalog.rs uses error and table
use crate::error::{DatabaseError, Result};
use crate::Table;
```

### 9.4 Public vs Private API

```rust
// Public: Users can call
pub fn add_row(&mut self, values: Vec<String>) -> Result<()> { ... }

// Private: Internal use only
fn rebuild_column_index(&mut self) { ... }

// Public for testing, documented as such
#[cfg(test)]
pub fn internal_for_testing(&self) { ... }
```

### 9.5 Self-Check Questions

1. Why organize modules by feature rather than type?
   - Answer: Features have clearer boundaries, easier to navigate, aligns with user mental model
   
2. When should something be pub vs private?
   - Answer: Public if part of external API; private if implementation detail
   
3. How do you handle circular dependencies between modules?
   - Answer: Extract shared code to third module, use traits to abstract

---

## Chapter 10: Best Practices & Design Patterns

### 10.1 Builder Pattern for Complex Construction

```rust
// Instead of many add_column calls
let mut table = Table::new("users".to_string());
table.add_column("id".to_string(), create_column(DataType::Int64))?;
table.add_column("name".to_string(), create_column(DataType::String))?;
table.add_column("email".to_string(), create_column(DataType::String))?;

// Future: Builder pattern
let table = Table::builder()
    .name("users")
    .column("id", DataType::Int64)
    .column("name", DataType::String)
    .column("email", DataType::String)
    .build()?;
```

### 10.2 Validation Before Mutation

```rust
// Bad: Mutate then validate
pub fn add_column(&mut self, name: String, column: Box<dyn Column>) -> Result<()> {
    let index = self.columns.len();
    self.column_index.insert(name.clone(), index);
    self.columns.push(column);
    
    if self.schema.contains_key(&name) {
        return Err(...);  // Too late, already mutated!
    }
    
    self.schema.insert(name, column.data_type());
    Ok(())
}

// Good: Validate first, then mutate
pub fn add_column(&mut self, name: String, column: Box<dyn Column>) -> Result<()> {
    // All validation first
    if self.schema.contains_key(&name) { ... }
    if !self.columns.is_empty() { ... }
    
    // Only mutate after all checks pass
    let data_type = column.data_type();
    let index = self.columns.len();
    self.column_index.insert(name.clone(), index);
    self.columns.push(column);
    self.schema.insert(name, data_type);
    
    Ok(())
}
```

### 10.3 Invariant Maintenance

```rust
// Invariants that should always be true:
// 1. column_index.keys() == schema.keys()
// 2. All columns have same length
// 3. column_index maps to valid columns

pub fn validate_invariants(&self) -> bool {
    // Check 1: Index and schema match
    if self.column_index.len() != self.schema.len() {
        return false;
    }
    
    for name in self.column_index.keys() {
        if !self.schema.contains_key(name) {
            return false;
        }
    }
    
    // Check 2: Consistent row counts
    if !self.columns.is_empty() {
        let first_len = self.columns[0].len();
        for col in &self.columns[1..] {
            if col.len() != first_len {
                return false;
            }
        }
    }
    
    // Check 3: Valid indices
    for (&name, &index) in &self.column_index {
        if index >= self.columns.len() {
            return false;
        }
        if self.columns[index].data_type() != self.schema[&name] {
            return false;
        }
    }
    
    true
}

// Use in tests
#[test]
fn test_table_invariants() {
    let mut table = create_test_table();
    assert!(table.validate_invariants());
    
    table.add_row(vec!["1".to_string()]).unwrap();
    assert!(table.validate_invariants());
    
    table.drop_column("extra").unwrap();
    assert!(table.validate_invariants());
}
```

### 10.4 Documentation as Code Contract

```rust
/// Adds a column to the table.
///
/// This method validates that:
/// - No column with the same name exists
/// - The new column has the same row count as existing columns
///
/// # Arguments
///
/// * `name` - Unique identifier for the column
/// * `column` - Column data implementing the Column trait
///
/// # Returns
///
/// * `Ok(())` - Column successfully added
/// * `Err(DatabaseError)` - Validation failed
///
/// # Examples
///
/// ```ignore
/// let mut table = Table::new("users".to_string());
/// let id_col = create_column(DataType::Int64);
/// table.add_column("id".to_string(), id_col)?;
/// ```
///
/// # Errors
///
/// * `ColumnError` - If column name already exists
/// * `TableError` - If row counts don't match
pub fn add_column(&mut self, name: String, column: Box<dyn Column>) -> Result<()> {
    // ...
}
```

### 10.5 Self-Check Questions

1. Why validate before mutating instead of after?
   - Answer: Leaves system in consistent state if validation fails, no rollback needed
   
2. What's the purpose of invariant checking?
   - Answer: Catches bugs early, ensures data integrity, documents expected behavior
   
3. How can documentation serve as a contract?
   - Answer: Specifies behavior, errors, and usage; users know what to expect

---

## Chapter 11: Learning Outcomes

### 11.1 Rust Concepts Mastered

✅ **Collections and Data Structures**
- HashMap and BTreeMap for efficient lookups
- Vec for ordered storage and iteration
- Iterator patterns and chaining

✅ **Advanced Traits**
- Manual trait implementations
- Trait objects (`dyn Trait`)
- Dynamic dispatch

✅ **Error Handling**
- Contextual error messages
- Error conversion patterns
- Early return for validation

✅ **Testing Strategies**
- Unit tests for individual functions
- Integration tests for workflows
- Property-based testing (optional)

✅ **Module Design**
- Public vs private APIs
- Cross-module dependencies
- Clean re-exports

### 11.2 Database Concepts Mastered

✅ **Table Management**
- Schema definition and validation
- Row insertion with type inference
- Column projection and selection

✅ **Catalog Design**
- Centralized metadata management
- Table registration and lifecycle
- Cross-table operations

✅ **Data Integrity**
- Schema enforcement
- Row count consistency
- Type safety across operations

✅ **Columnar Storage Benefits**
- Efficient column access
- Type-specific optimization
- Analytical query preparation

### 11.3 Code Quality Achieved

✅ **Comprehensive Testing**
- 33 Table tests
- 25 Catalog tests
- Integration test coverage
- Edge case handling

✅ **Error Handling**
- Descriptive error messages
- Proper error propagation
- Contextual information

✅ **Documentation**
- Inline comments for complex logic
- Public API documentation
- Learning guides

✅ **Code Metrics**
- 1,778 lines of implementation code
- 1,015 lines of test code (57% coverage)
- Clean, readable code structure

### 11.4 Practical Skills Developed

✅ **Designing Complex Data Structures**
- Managing invariants
- Choosing appropriate collections
- Implementing traits manually

✅ **Writing Production-Ready Code**
- Validation before mutation
- Comprehensive error handling
- Test-driven development

✅ **Educational Writing**
- Explaining complex concepts
- Providing examples
- Creating learning resources

### 11.5 Self-Assessment Quiz

**Rust Knowledge:**

1. What's the difference between `HashMap` and `BTreeMap`?
2. When should you implement a trait manually vs deriving?
3. How does `Box<dyn Trait>` enable dynamic dispatch?
4. What's the Entry API for HashMaps used for?
5. Why use `?` operator for error handling?

**Database Knowledge:**

6. What's the purpose of a database schema?
7. How does columnar storage differ from row-oriented?
8. Why validate row counts across columns?
9. What's the role of a catalog in a database system?
10. How do you maintain data integrity in table operations?

**Design Knowledge:**

11. What's the builder pattern and when to use it?
12. Why validate before mutating data structures?
13. How do you maintain invariants in complex types?
14. What's the difference between unit and integration tests?
15. Why document public APIs thoroughly?

### 11.6 Next Steps

Now that you've mastered Phase 2, you're ready for:

**Phase 3: CSV Ingestion**
- Parsing CSV files
- Type inference from data
- Loading data into tables

**Phase 4: Query Operators**
- Table scanning
- Filtering and projection
- Aggregation operations

**Phase 5: SQL Parser**
- Tokenizing SQL syntax
- Building abstract syntax trees
- Converting to execution plans

---

## 📝 Appendix A: Phase 2 Code Summary

### Table Module (src/table.rs)

**Struct Definition:**
```rust
pub struct Table {
    name: String,
    column_index: HashMap<String, usize>,
    columns: Vec<Box<dyn Column>>,
    schema: HashMap<String, DataType>,
}
```

**Key Methods:**
- `new()` - Create empty table
- `add_column()` - Add column with validation
- `add_row()` - Insert data row
- `get_column()` - Retrieve column by name
- `get_value()` - Get single cell value
- `select_columns()` - Project subset of columns
- `drop_column()` - Remove column
- `row_count()` / `column_count()` - Statistics

**Traits Implemented:**
- `Clone` (manual)
- `Display`

**Test Coverage:** 33 tests

### Catalog Module (src/catalog.rs)

**Struct Definition:**
```rust
pub struct Catalog {
    tables: HashMap<String, Table>,
}
```

**Key Methods:**
- `new()` - Create empty catalog
- `register_table()` - Add table
- `get_table()` / `get_table_mut()` - Retrieve table
- `table_exists()` - Check existence
- `list_tables()` - Get all names
- `drop_table()` - Remove table
- `rename_table()` - Rename table
- `table_count()` - Statistics

**Traits Implemented:**
- `Clone` (derived)
- `Display`
- `Default`

**Test Coverage:** 25 tests

### Integration Points

**Table ↔ Column:**
- `add_column()` accepts `Box<dyn Column>`
- Uses `create_column()` factory function

**Catalog ↔ Table:**
- Manages `Table` instances
- Provides mutable access for modification

**Error Handling:**
- All methods return `Result<T, DatabaseError>`
- Contextual error messages

---

## 📚 Appendix B: Recommended Reading

### Rust Books

- **"The Rust Programming Language"** - Chapters on collections, error handling
- **"Rust by Example"** - Sections on HashMaps, traits
- **"Programming Rust"** - Advanced topics on trait objects

### Database Books

- **"Database System Concepts"** - Storage architectures
- **"Readings in Database Systems"** - Column-oriented storage
- **"Designing Data-Intensive Applications"** - Data modeling

### Online Resources

- **Rust Bookshelf**: https://rust-lang.github.io/bookshelf/
- **Rust by Example**: https://doc.rust-lang.org/rust-by-example/
- **ClickHouse Documentation**: Real-world OLAP database examples

---

## 🎓 Appendix C: Exercises and Challenges

### Exercise 1: Add Table Statistics

Implement methods to calculate statistics:
```rust
impl Table {
    pub fn min_value(&self, column_name: &str) -> Result<Value>;
    pub fn max_value(&self, column_name: &str) -> Result<Value>;
    pub fn avg_value(&self, column_name: &str) -> Result<f64>;
}
```

### Exercise 2: Add Table Export

Implement CSV export:
```rust
impl Table {
    pub fn export_to_csv(&self, path: &str) -> Result<()>;
}
```

### Exercise 3: Add Catalog Search

Implement table search:
```rust
impl Catalog {
    pub fn find_tables_with_column(&self, column_name: &str) -> Vec<String>;
    pub fn find_tables_by_size(&self, min_rows: usize) -> Vec<String>;
}
```

### Challenge: Add Table Aliases

Implement column aliases for querying:
```rust
impl Table {
    pub fn add_alias(&mut self, column: &str, alias: &str) -> Result<()>;
    pub fn resolve_name(&self, name_or_alias: &str) -> Option<&str>;
}
```

---

**End of Phase 2 Learning Guide**

Congratulations on completing Phase 2! You've built a robust storage layer with proper error handling, comprehensive testing, and clean design. You're now ready to tackle Phase 3: CSV Ingestion.

Happy Learning! 🚀