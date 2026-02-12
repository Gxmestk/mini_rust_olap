# Phase 6.2 Learning Guide: ORDER BY, LIMIT, and OFFSET

## Overview

Phase 6.2 extends the Mini Rust OLAP query engine with advanced query features that are essential for practical database usage. This phase implements three critical SQL clauses:

- **ORDER BY**: Sort query results by one or more columns
- **LIMIT**: Restrict the number of rows returned
- **OFFSET**: Skip a specified number of rows before returning results

These features work together to enable pagination, ranked results, and controlled result sets.

## Learning Objectives

By the end of this guide, you will understand:

1. How ORDER BY sorts results with multiple columns and directions
2. How LIMIT and OFFSET implement pagination
3. The execution engine's Sort and Limit operators
4. How the planner integrates these features into query plans
5. The parser's approach to handling these clauses
6. Best practices and limitations of these features

## Table of Contents

1. [ORDER BY Clause](#1-order-by-clause)
2. [LIMIT Clause](#2-limit-clause)
3. [OFFSET Clause](#3-offset-clause)
4. [Combined Usage](#4-combined-usage)
5. [Parser Implementation](#5-parser-implementation)
6. [Execution Engine](#6-execution-engine)
7. [Query Planning](#7-query-planning)
8. [Code Examples](#8-code-examples)
9. [Best Practices](#9-best-practices)
10. [Known Limitations](#10-known-limitations)

---

## 1. ORDER BY Clause

### Purpose

The ORDER BY clause sorts the result set of a query by one or more columns. You can sort in ascending (ASC) or descending (DESC) order.

### Basic Syntax

```sql
SELECT column1, column2, ...
FROM table_name
ORDER BY column1 [ASC|DESC], column2 [ASC|DESC], ...
```

### How It Works

#### Single Column Sorting

When sorting by a single column, the Sort operator:
1. Reads all data from the child operator into memory
2. Sorts the rows based on the specified column
3. Returns the sorted rows in batches

**Example:**
```sql
SELECT name, age FROM users ORDER BY age DESC;
```

This sorts users by age in descending order (oldest first).

#### Multiple Column Sorting

When sorting by multiple columns, the Sort operator applies sorting precedence:
- First sorts by the primary column
- Then by secondary columns within groups of equal primary values

**Example:**
```sql
SELECT name, age, department FROM users 
ORDER BY department ASC, age DESC;
```

This sorts users by department (A-Z), then within each department by age (oldest first).

### Sort Directions

- **ASC (Ascending)**: Default, sorts from lowest to highest
  - Numbers: 1, 2, 3, ...
  - Strings: A, B, C, ... (lexicographic)
  - Floats: 1.0, 1.5, 2.0, ...
  
- **DESC (Descending)**: Sorts from highest to lowest
  - Numbers: 100, 50, 25, ...
  - Strings: Z, Y, X, ...
  - Floats: 10.0, 5.0, 2.5, ...

### Supported Data Types

The Sort operator supports three data types:
- **Int64**: Integer comparison
- **Float64**: Floating-point comparison
- **String**: Lexicographic (alphabetical) comparison

**Note:** NULL values are not supported in this implementation (no Value::Null variant exists).

---

## 2. LIMIT Clause

### Purpose

The LIMIT clause restricts the number of rows returned by a query. It's commonly used for:
- Pagination (showing 10 results at a time)
- Fetching top N results
- Performance optimization (reducing result set size)

### Basic Syntax

```sql
SELECT column1, column2, ...
FROM table_name
LIMIT n
```

Where `n` is the maximum number of rows to return.

### How It Works

The Limit operator:
1. Starts reading from the child operator
2. Skips OFFSET rows (if specified)
3. Returns up to LIMIT rows
4. Stops reading once the limit is reached

**Example:**
```sql
SELECT name FROM users LIMIT 5;
```

Returns only the first 5 users from the result set.

### Performance Benefits

LIMIT provides significant performance benefits:
- **Early Termination**: The operator stops reading once the limit is reached
- **Reduced Memory**: Less data needs to be stored in memory
- **Faster Queries**: Especially useful with large tables

---

## 3. OFFSET Clause

### Purpose

The OFFSET clause skips a specified number of rows before returning results. It's typically used with LIMIT for pagination.

### Basic Syntax

```sql
SELECT column1, column2, ...
FROM table_name
OFFSET n
```

Where `n` is the number of rows to skip.

### How It Works

The Offset functionality:
1. Starts reading from the child operator
2. Skips the first OFFSET rows
3. Returns all remaining rows (unless LIMIT is specified)

**Example:**
```sql
SELECT name FROM users OFFSET 10;
```

Skips the first 10 users and returns all remaining users.

### Important Notes

- OFFSET is always processed before LIMIT
- OFFSET without LIMIT returns all rows after the skipped ones
- OFFSET values must be non-negative

---

## 4. Combined Usage

### Pagination Pattern

The most common pattern is combining ORDER BY, LIMIT, and OFFSET for pagination:

```sql
SELECT name, email 
FROM users 
ORDER BY created_at DESC 
LIMIT 10 OFFSET 20;
```

This query:
1. Sorts users by creation date (newest first)
2. Skips the first 20 results (pages 1 and 2)
3. Returns the next 10 results (page 3)

### Common Use Cases

#### Top N Results
```sql
SELECT product_name, price 
FROM products 
ORDER BY price DESC 
LIMIT 5;
```
Returns the 5 most expensive products.

#### Pagination (Page N)
```sql
-- Page 1: results 1-10
SELECT * FROM items ORDER BY id LIMIT 10 OFFSET 0;

-- Page 2: results 11-20
SELECT * FROM items ORDER BY id LIMIT 10 OFFSET 10;

-- Page 3: results 21-30
SELECT * FROM items ORDER BY id LIMIT 10 OFFSET 20;
```

#### Skip-and-Fetch Pattern
```sql
SELECT * FROM logs 
ORDER BY timestamp DESC 
OFFSET 100 LIMIT 50;
```
Skips the most recent 100 logs and returns the next 50.

### Execution Order

When combined, these clauses execute in this order:
1. **FROM/WHERE/GROUP BY/HAVING**: Filter and aggregate data
2. **ORDER BY**: Sort the results
3. **OFFSET**: Skip the specified number of rows
4. **LIMIT**: Restrict the number of rows returned

**Why this order?**
- Sorting before pagination ensures consistent results
- OFFSET is applied after sorting so you skip the correct rows
- LIMIT is applied last to control the final result size

---

## 5. Parser Implementation

### New Token Types

The parser was extended with new token types:

```rust
Order,   // ORDER keyword
By,      // BY keyword
Asc,     // ASC keyword
Desc,    // DESC keyword
Limit,   // LIMIT keyword
Offset,  // OFFSET keyword
```

### AST Changes

The `SelectStatement` struct was extended:

```rust
pub struct SelectStatement {
    pub select_items: Vec<SelectItem>,
    pub from: String,
    pub where_clause: Option<Expression>,
    pub group_by: Option<Vec<String>>,
    pub order_by: Option<Vec<OrderByItem>>,  // New
    pub limit: Option<usize>,                 // New
    pub offset: Option<usize>,                // New
}
```

### OrderByItem Struct

Represents a single column in the ORDER BY clause:

```rust
pub struct OrderByItem {
    pub column: String,
    pub direction: SortDirection,  // Asc or Desc
}
```

### Parsing Logic

#### ORDER BY Parsing

The parser looks for the `ORDER BY` keywords after the WHERE and GROUP BY clauses:

```rust
if self.current_token() == &Token::Order {
    self.consume(Token::Order)?;
    self.consume(Token::By)?;
    
    let mut order_by_items = Vec::new();
    
    // Parse first column
    order_by_items.push(self.parse_order_by_item()?);
    
    // Parse additional columns separated by commas
    while self.current_token() == &Token::Comma {
        self.consume(Token::Comma)?;
        order_by_items.push(self.parse_order_by_item()?);
    }
    
    stmt.order_by = Some(order_by_items);
}
```

#### LIMIT Parsing

```rust
if self.current_token() == &Token::Limit {
    self.consume(Token::Limit)?;
    stmt.limit = Some(self.parse_number_literal()?);
}
```

#### OFFSET Parsing

```rust
if self.current_token() == &Token::Offset {
    self.consume(Token::Offset)?;
    stmt.offset = Some(self.parse_number_literal()?);
}
```

### parse_order_by_item Method

Parses a single ORDER BY specification:

```rust
fn parse_order_by_item(&mut self) -> Result<OrderByItem> {
    // Parse column name
    let column = self.parse_identifier()?;
    
    // Parse direction (default is Asc)
    let direction = match self.current_token() {
        Token::Asc => {
            self.consume(Token::Asc)?;
            SortDirection::Ascending
        }
        Token::Desc => {
            self.consume(Token::Desc)?;
            SortDirection::Descending
        }
        _ => SortDirection::Ascending,  // Default
    };
    
    Ok(OrderByItem { column, direction })
}
```

---

## 6. Execution Engine

### Sort Operator

#### Structure

```rust
pub struct Sort {
    child: Box<dyn Operator>,
    sort_columns: Vec<usize>,
    sort_directions: Vec<SortDirection>,
    state: OperatorState,
    sorted_data: Option<Vec<Row>>,
    current_row: usize,
    batch_size: usize,
}
```

#### How It Works

1. **Open Phase**:
   - Calls `child.open()` to open the child operator
   - Reads all data from the child into memory
   - Sorts the entire dataset in memory
   - Stores sorted data in `sorted_data`

2. **Next Batch Phase**:
   - Returns rows from `sorted_data` in batches
   - Starts at `current_row = 0`
   - Returns up to `batch_size` rows per batch
   - Increments `current_row` after each batch
   - Returns `None` when all rows have been returned

3. **Close Phase**:
   - Clears `sorted_data` to free memory
   - Calls `child.close()` to close the child operator

#### Sorting Implementation

The Sort operator uses a multi-column comparison:

```rust
fn compare_rows(&self, row_a: &Row, row_b: &Row) -> Ordering {
    for (col_idx, direction) in self.sort_columns.iter().zip(&self.sort_directions) {
        let val_a = &row_a.values[*col_idx];
        let val_b = &row_b.values[*col_idx];
        
        let cmp = match (val_a, val_b) {
            (Value::Int64(a), Value::Int64(b)) => a.cmp(b),
            (Value::Float64(a), Value::Float64(b)) => {
                a.partial_cmp(b).unwrap_or(Equal)
            }
            (Value::String(a), Value::String(b)) => a.cmp(b),
            _ => Equal,  // Should not happen with proper schema
        };
        
        if cmp != Equal {
            return match direction {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            };
        }
    }
    Equal
}
```

#### Memory Considerations

**Pros:**
- Simple implementation
- Fast multi-column sorting
- Stable results

**Cons:**
- Requires loading all data into memory
- Not suitable for very large datasets
- Potential memory exhaustion with large tables

**Future Improvements:**
- External sort for datasets larger than memory
- Push-down sorting (sort at the source if possible)
- Streaming sort for single-column sorts

### Limit Operator

#### Structure

```rust
pub struct Limit {
    child: Box<dyn Operator>,
    limit: usize,
    offset: usize,
    rows_returned: usize,
    rows_skipped: usize,
    state: OperatorState,
}
```

#### How It Works

1. **Open Phase**:
   - Calls `child.open()` to open the child operator
   - Resets counters (`rows_returned = 0`, `rows_skipped = 0`)

2. **Next Batch Phase**:
   - Reads batches from the child
   - Skips rows until `rows_skipped` reaches `offset`
   - Returns rows until `rows_returned` reaches `limit`
   - Stops reading once limit is reached
   - Returns `None` when limit is reached or child is exhausted

3. **Close Phase**:
   - Calls `child.close()` to close the child operator

#### Implementation

```rust
fn next_batch(&mut self) -> Result<Option<Batch>> {
    if self.rows_returned >= self.limit {
        return Ok(None);  // Limit reached
    }
    
    let mut result_rows = Vec::new();
    
    while result_rows.len() < self.batch_size && self.rows_returned < self.limit {
        if let Some(mut batch) = self.child.next_batch()? {
            let batch_rows = batch.row_count();
            
            for row_idx in 0..batch_rows {
                if self.rows_skipped < self.offset {
                    self.rows_skipped += 1;
                } else if self.rows_returned < self.limit {
                    // Add row to result
                    result_rows.push(extract_row(&batch, row_idx));
                    self.rows_returned += 1;
                } else {
                    break;
                }
            }
        } else {
            break;
        }
    }
    
    if result_rows.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Batch::from_rows(result_rows)))
    }
}
```

### Batch Helper Methods

Two new methods were added to the Batch struct:

#### skip_rows()

```rust
fn skip_rows(&mut self, n: usize) -> Vec<Value> {
    self.values.drain(0..n.min(self.row_count())).collect()
}
```

Skips the first `n` rows and returns the removed values.

#### take_rows()

```rust
fn take_rows(&mut self, n: usize) -> Batch {
    let count = n.min(self.row_count());
    let remaining = self.values.drain(count..).collect();
    let taken = self.values.drain(..).collect();
    
    self.values = remaining;
    Batch { values: taken }
}
```

Takes the first `n` rows and returns them as a new batch.

---

## 7. Query Planning

### Operator Ordering

The planner builds the operator tree in this order (from leaf to root):

1. **TableScan**: Read data from the table
2. **Filter**: Apply WHERE clause
3. **GroupBy**: Apply GROUP BY and aggregates
4. **Project**: Apply SELECT column selection
5. **Sort**: Apply ORDER BY
6. **Limit**: Apply LIMIT and OFFSET

### Planning ORDER BY

The planner's `plan_select()` method handles ORDER BY:

```rust
// Start with the base plan
let mut plan = self.build_base_plan(stmt)?;

// Apply WHERE clause
if let Some(where_expr) = stmt.where_clause {
    plan = self.plan_filter(plan, where_expr)?;
}

// Apply GROUP BY
if let Some(group_by) = stmt.group_by {
    plan = self.plan_group_by(plan, stmt, group_by)?;
}

// Apply ORDER BY
if let Some(order_by_items) = stmt.order_by {
    plan = self.plan_order_by(plan, order_by_items, &output_schema)?;
}

// Apply LIMIT and OFFSET
if stmt.limit.is_some() || stmt.offset.is_some() {
    plan = Box::new(Limit::new(
        plan,
        stmt.limit,
        stmt.offset.unwrap_or(0),
    ));
}
```

### Column Index Mapping

A key challenge is mapping column names from ORDER BY to column indices in the output schema.

#### Simple Queries

For queries without GROUP BY:

```sql
SELECT name, age FROM users ORDER BY age DESC
```

The output schema is `[name, age]`, so `age` maps to index 1.

#### Queries with GROUP BY

For queries with GROUP BY, the output schema changes:

```sql
SELECT department, COUNT(*) as cnt 
FROM users 
GROUP BY department 
ORDER BY cnt DESC
```

The output schema is `[department, cnt]`, so `cnt` maps to index 1.

The planner handles this by:
1. Building the output schema as it processes the query
2. Using this schema to resolve ORDER BY column names
3. Mapping column names to their indices in the final output

### plan_order_by Method

```rust
fn plan_order_by(
    &self,
    plan: Box<dyn Operator>,
    order_by_items: Vec<OrderByItem>,
    output_schema: &[Column],
) -> Result<Box<dyn Operator>> {
    let mut sort_columns = Vec::new();
    let mut sort_directions = Vec::new();
    
    for item in order_by_items {
        // Find column index in output schema
        let col_idx = output_schema
            .iter()
            .position(|col| col.name == item.column)
            .ok_or_else(|| PlannerError::ColumnNotFound(item.column.clone()))?;
        
        sort_columns.push(col_idx);
        sort_directions.push(item.direction);
    }
    
    Ok(Box::new(Sort::new(plan, sort_columns, sort_directions)))
}
```

---

## 8. Code Examples

### Example 1: Simple ORDER BY

```sql
SELECT name, age 
FROM users 
ORDER BY age DESC;
```

**Execution Plan:**
```
TableScan("users")
  ↓
Project([name, age])
  ↓
Sort([1], [Descending])
```

**Result:**
```
name    | age
--------|-----
Alice   | 65
Bob     | 42
Charlie | 35
David   | 28
```

### Example 2: Multi-Column ORDER BY

```sql
SELECT name, age, department 
FROM users 
ORDER BY department ASC, age DESC;
```

**Execution Plan:**
```
TableScan("users")
  ↓
Project([name, age, department])
  ↓
Sort([2, 1], [Ascending, Descending])
```

**Result:**
```
name    | age | department
--------|-----|------------
Alice   | 65  | Engineering
Bob     | 42  | Engineering
Charlie | 35  | Engineering
David   | 50  | HR
Eve     | 30  | HR
```

### Example 3: LIMIT

```sql
SELECT name, age 
FROM users 
ORDER BY age DESC 
LIMIT 3;
```

**Execution Plan:**
```
TableScan("users")
  ↓
Project([name, age])
  ↓
Sort([1], [Descending])
  ↓
Limit(Some(3), 0)
```

**Result:**
```
name    | age
--------|-----
Alice   | 65
Bob     | 42
Charlie | 35
```

### Example 4: OFFSET

```sql
SELECT name, age 
FROM users 
ORDER BY age DESC 
OFFSET 2;
```

**Execution Plan:**
```
TableScan("users")
  ↓
Project([name, age])
  ↓
Sort([1], [Descending])
  ↓
Limit(None, 2)
```

**Result:**
```
name    | age
--------|-----
Charlie | 35
David   | 28
Eve     | 25
```

### Example 5: LIMIT + OFFSET (Pagination)

```sql
SELECT name, age 
FROM users 
ORDER BY age DESC 
LIMIT 2 OFFSET 2;
```

**Execution Plan:**
```
TableScan("users")
  ↓
Project([name, age])
  ↓
Sort([1], [Descending])
  ↓
Limit(Some(2), 2)
```

**Result:**
```
name    | age
--------|-----
Charlie | 35
David   | 28
```

### Example 6: ORDER BY with GROUP BY

```sql
SELECT department, COUNT(*) as employee_count 
FROM users 
GROUP BY department 
ORDER BY employee_count DESC;
```

**Execution Plan:**
```
TableScan("users")
  ↓
Project([department])
  ↓
GroupBy([department], [COUNT(*)])
  ↓
Project([department, COUNT(*)])
  ↓
Sort([1], [Descending])
```

**Result:**
```
department  | employee_count
------------|---------------
Engineering | 15
HR          | 8
Sales       | 5
```

### Example 7: Complex Query

```sql
SELECT department, AVG(salary) as avg_salary 
FROM employees 
WHERE hire_date > '2020-01-01' 
GROUP BY department 
ORDER BY avg_salary DESC 
LIMIT 5;
```

**Execution Plan:**
```
TableScan("employees")
  ↓
Filter(hire_date > '2020-01-01')
  ↓
GroupBy([department], [AVG(salary)])
  ↓
Project([department, AVG(salary)])
  ↓
Sort([1], [Descending])
  ↓
Limit(Some(5), 0)
```

---

## 9. Best Practices

### ORDER BY Best Practices

1. **Always use ORDER BY with LIMIT**
   ```sql
   -- Good
   SELECT * FROM items ORDER BY price DESC LIMIT 10;
   
   -- Bad (may return unsorted results in some databases)
   SELECT * FROM items ORDER BY price DESC;
   ```

2. **Index columns used in ORDER BY** (when indexes are implemented)
   ```sql
   -- If there's an index on created_at, this is efficient
   SELECT * FROM logs ORDER BY created_at DESC LIMIT 100;
   ```

3. **Be specific about sort direction**
   ```sql
   -- Clear
   SELECT * FROM users ORDER BY name DESC;
   
   -- Less clear (relies on default)
   SELECT * FROM users ORDER BY name;
   ```

4. **Avoid sorting on computed expressions** (not supported yet)
   ```sql
   -- This won't work yet
   SELECT * FROM users ORDER BY age * 2 DESC;
   ```

### LIMIT Best Practices

1. **Use LIMIT for performance**
   ```sql
   -- Good for quick checks
   SELECT * FROM large_table LIMIT 10;
   ```

2. **Combine with ORDER BY for consistent results**
   ```sql
   -- Consistent results
   SELECT * FROM products ORDER BY id LIMIT 10;
   
   -- Unpredictable results
   SELECT * FROM products LIMIT 10;
   ```

3. **Use sensible LIMIT values**
   ```sql
   -- Good for pagination
   SELECT * FROM items ORDER BY id LIMIT 20;
   
   -- May cause performance issues
   SELECT * FROM items ORDER BY id LIMIT 10000;
   ```

### OFFSET Best Practices

1. **Always combine ORDER BY with OFFSET**
   ```sql
   -- Good
   SELECT * FROM users ORDER BY id OFFSET 100 LIMIT 10;
   
   -- Bad (unpredictable results)
   SELECT * FROM users OFFSET 100 LIMIT 10;
   ```

2. **Be careful with large OFFSET values**
   ```sql
   -- May be slow (skips 100,000 rows)
   SELECT * FROM logs ORDER BY id OFFSET 100000 LIMIT 10;
   
   -- Consider keyset pagination instead (future enhancement)
   SELECT * FROM logs WHERE id > 100000 ORDER BY id LIMIT 10;
   ```

3. **Use consistent pagination logic**
   ```sql
   -- Page 1
   SELECT * FROM items ORDER BY id LIMIT 10 OFFSET 0;
   
   -- Page 2
   SELECT * FROM items ORDER BY id LIMIT 10 OFFSET 10;
   
   -- Page 3
   SELECT * FROM items ORDER BY id LIMIT 10 OFFSET 20;
   ```

### Performance Considerations

1. **Sort before LIMIT**
   ```sql
   -- Efficient: Sort all rows, then take top N
   SELECT * FROM items ORDER BY price DESC LIMIT 10;
   
   -- Less efficient (not supported): Find top N without full sort
   ```

2. **Push down WHERE filters**
   ```sql
   -- Good: Filter first, then sort
   SELECT * FROM items 
   WHERE category = 'books' 
   ORDER BY price DESC 
   LIMIT 10;
   ```

3. **Avoid unnecessary sorting**
   ```sql
   -- Good: Only sort what you need
   SELECT id, name FROM items ORDER BY price DESC LIMIT 10;
   
   -- Less efficient: Sort all columns
   SELECT * FROM items ORDER BY price DESC LIMIT 10;
   ```

---

## 10. Known Limitations

### ORDER BY Limitations

1. **No support for column aliases**
   ```sql
   -- Doesn't work
   SELECT name AS n FROM users ORDER BY n DESC;
   
   -- Workaround: Use the original column name
   SELECT name FROM users ORDER BY name DESC;
   ```

2. **No support for expressions**
   ```sql
   -- Doesn't work
   SELECT * FROM users ORDER BY age * 2 DESC;
   
   -- Workaround: Use a subquery (not supported yet)
   ```

3. **No support for ordinal positions**
   ```sql
   -- Doesn't work
   SELECT name, age FROM users ORDER BY 2 DESC;
   ```

4. **No NULL handling**
   - NULL values are not supported in the Value enum
   - All columns are assumed to have values

5. **ORDER BY with GROUP BY complexity**
   - Column mapping can be complex when using aggregates
   - Currently doesn't support ordering by aggregate aliases

### LIMIT/OFFSET Limitations

1. **No support for LIMIT without ORDER BY in some contexts**
   ```sql
   -- May return unpredictable results
   SELECT * FROM items LIMIT 10;
   ```

2. **No support for negative values**
   ```sql
   -- Doesn't work
   SELECT * FROM items LIMIT -10;
   ```

3. **Performance issues with large OFFSET**
   - OFFSET must read and skip all preceding rows
   - OFFSET 1,000,000 is slow even with LIMIT 1

4. **No total count in pagination**
   ```sql
   -- Not supported: Get total count
   SELECT COUNT(*) FROM items;
   
   -- Workaround: Run a separate query
   ```

### Memory Limitations

1. **Sort loads all data into memory**
   - Not suitable for very large tables
   - May cause Out of Memory errors

2. **No external sorting**
   - Cannot sort datasets larger than available memory
   - Future enhancement needed

### Data Type Limitations

1. **Only three data types supported**
   - Int64, Float64, String
   - No Boolean, Date, Timestamp, etc.

2. **No type conversion in sorting**
   - Cannot sort mixed types
   - Cannot convert string to number for sorting

### Future Enhancements

1. **ORDER BY Improvements**
   - Support for column aliases
   - Support for expressions
   - Support for ordinal positions
   - NULL handling (NULLS FIRST/LAST)

2. **Pagination Improvements**
   - Keyset pagination for large datasets
   - Total count queries
   - Cursor-based pagination

3. **Performance Improvements**
   - External sorting for large datasets
   - Push-down sorting to data source
   - Index-aware sorting

4. **Additional Features**
   - DISTINCT with ORDER BY
   - Window functions (future)
   - Multiple ORDER BY in subqueries

---

## Summary

Phase 6.2 successfully implements ORDER BY, LIMIT, and OFFSET clauses, providing essential query capabilities for practical database usage. The implementation:

- **Extends the parser** to recognize new keywords and construct appropriate AST nodes
- **Adds new operators** (Sort and Limit) to the execution engine
- **Integrates with the planner** to build proper query plans
- **Maintains test coverage** with 361 passing unit tests
- **Follows architectural patterns** established in previous phases

These features enable pagination, ranked results, and controlled result sets - capabilities that are fundamental to real-world database applications. While there are some limitations, the implementation provides a solid foundation for future enhancements.

## Next Steps

After completing Phase 6.2, consider:

1. **Phase 6.3**: Implement HAVING clause for filtering grouped results
2. **Performance optimization**: Implement external sorting for large datasets
3. **Index support**: Add indexes to optimize ORDER BY queries
4. **Advanced features**: Support column aliases, expressions in ORDER BY
5. **Testing**: Add more comprehensive edge case tests

Continue to Phase 6.2 Assessment to test your understanding!