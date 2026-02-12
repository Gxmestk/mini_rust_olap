# Phase 6.1 Assessment: Query Planner

## Assessment Overview

This assessment evaluates your understanding and implementation of the Mini Rust OLAP query planner (Phase 6.1). It covers query planning fundamentals, predicate evaluation, aggregate functions, operator optimization, and testing strategies.

**Duration:** 60-90 minutes  
**Passing Score:** 70% (35/50 points)  
**Format:** Multiple choice, short answer, and coding challenges

---

## Learning Objectives

After completing this assessment, you should be able to:

1. Explain the role and importance of a query planner in a database system
2. Design and implement predicate evaluation with boolean logic
3. Build aggregate functions for columnar data
4. Apply optimization techniques (column pruning, predicate pushdown)
5. Write comprehensive tests for query planner components
6. Debug and fix common issues in query planning code

---

## Section 1: Multiple Choice Questions (20 points)

### Question 1.1 (2 points)
What is the primary purpose of a query planner?

A) To parse SQL queries into an AST  
B) To generate optimized execution plans from SQL queries  
C) To execute queries against the database  
D) To store data in columnar format  

**Answer:** B

---

### Question 1.2 (2 points)
Which of the following is NOT a benefit of column pruning?

A) Reduced memory usage  
B) Faster I/O operations  
C) Increased query result accuracy  
D) Lower network bandwidth consumption  

**Answer:** C  
*Explanation: Column pruning improves performance but does not affect query result accuracy.*

---

### Question 1.3 (2 points)
In the context of predicate pushdown, why should filters be applied as early as possible?

A) To simplify the query syntax  
B) To reduce the number of rows that need to be processed downstream  
C) To improve the readability of the execution plan  
D) To ensure column order is correct  

**Answer:** B

---

### Question 1.4 (2 points)
What is the correct operator ordering for optimal query performance?

A) Project → Filter → GroupBy → TableScan  
B) Filter → GroupBy → Project → TableScan  
C) TableScan → Project → Filter → GroupBy  
D) TableScan → Filter → GroupBy → Project  

**Answer:** D  
*Explanation: Filter first to reduce rows, then GroupBy on reduced data, then Project to select columns.*

---

### Question 1.5 (2 points)
When comparing a Float64 column value with an Int64 literal, what should happen?

A) Raise a type mismatch error  
B) Convert Float64 to Int64 and compare  
C) Convert Int64 to Float64 and compare  
D) Compare using string representation  

**Answer:** C  
*Explanation: Convert the smaller precision type (Int64) to larger precision (Float64) for accurate comparison.*

---

### Question 1.6 (2 points)
Which aggregate function ignores NULL values?

A) COUNT(*)  
B) COUNT(column_name)  
C) SUM(column_name)  
D) All of the above  

**Answer:** D  
*Explanation: All aggregate functions ignore NULL values except COUNT(*), which counts all rows.*

---

### Question 1.7 (2 points)
What does the `Arc<dyn Predicate>` type represent in Rust?

A) A reference-counted pointer to a trait object  
B) A mutable reference to a predicate  
C) A boxed predicate value  
D) An array of predicates  

**Answer:** A

---

### Question 1.8 (2 points)
In the context of GROUP BY, what happens to columns not in the GROUP BY clause?

A) They are included in the result set unchanged  
B) They must be part of an aggregate function  
C) They cause a compilation error  
D) They are automatically grouped  

**Answer:** B

---

### Question 1.9 (2 points)
What is the result of `SELECT AVG(age) FROM users` if the table has no rows?

A) 0  
B) NULL  
C) Error  
D) Undefined behavior  

**Answer:** B

---

### Question 1.10 (2 points)
Which trait method must be implemented for all aggregate functions?

A) `evaluate()`  
B) `update()`, `finalize()`, and `data_type()`  
C) `open()` and `close()`  
D) `plan()` and `execute()`  

**Answer:** B

---

## Section 2: Short Answer Questions (15 points)

### Question 2.1 (3 points)
Explain the difference between predicate pushdown and column pruning. When would each be most beneficial?

**Sample Answer:**
- **Predicate pushdown**: Applying filters early in the execution plan to reduce the number of rows processed. Most beneficial when WHERE clauses eliminate a large percentage of rows.
- **Column pruning**: Only reading the columns actually needed by the query. Most beneficial when tables have many columns but queries only need a few.

---

### Question 2.2 (3 points)
Why is the `?` operator necessary when calling `AvgAggregate::new(*data_type)`? What happens if you forget it?

**Sample Answer:**
The `?` operator is necessary because `AvgAggregate::new()` returns a `Result<Self>` (i.e., `Result<AvgAggregate, DatabaseError>`), not just `AvgAggregate`. The `?` operator unwraps the Result, propagating any error upward. Without `?`, you would be boxing a `Result` instead of an `AvgAggregate`, causing a compilation error about `Result` not implementing the `AggregateFunction` trait.

---

### Question 2.3 (3 points)
Describe the execution plan structure for the following query:
```sql
SELECT department, AVG(salary) 
FROM employees 
WHERE age > 30 
GROUP BY department
```

**Sample Answer:**
```
Project(department, AVG(salary))
    ↓
GroupBy(department, [AVG(salary)])
    ↓
Filter(age > 30)
    ↓
TableScan(employees) [with column pruning: age, salary, department]
```

The plan starts with a TableScan that reads only the necessary columns (age, salary, department). A Filter removes rows where age <= 30. A GroupBy groups remaining rows by department and calculates AVG(salary). Finally, a Project selects and renames the output columns.

---

### Question 2.4 (3 points)
What is the purpose of using `Arc<dyn Predicate>` instead of `Box<dyn Predicate>` for combining predicates (AND/OR)?

**Sample Answer:**
`Arc` (Atomically Reference Counted) is used instead of `Box` because predicates need to be shared between multiple consumers. In an `And` or `Or` predicate, both the left and right sub-predicates are referenced. If we used `Box`, we would need to clone the predicates, which could be expensive. With `Arc`, we get shared ownership with reference counting, allowing multiple references to the same predicate without cloning.

---

### Question 2.5 (3 points)
Explain why the operator lifecycle (open → next_batch → close) is important for query execution operators.

**Sample Answer:**
The operator lifecycle is important because:
1. **open()**: Initializes the operator state, opens child operators, allocates resources, and validates the schema.
2. **next_batch()**: Executes the operator logic, processes data, and returns batches. Can be called multiple times.
3. **close()**: Cleans up resources, closes child operators, and ensures proper shutdown.

Following this lifecycle ensures:
- Resources are properly allocated and freed
- Operators are in the correct state before use
- Errors are caught early (e.g., calling next_batch without opening first)
- Child operators are properly chained and closed

---

## Section 3: Coding Challenges (15 points)

### Challenge 3.1: Implement NOT Predicate (5 points)

**Task:** Implement a `Not` predicate that negates another predicate.

**Requirements:**
- Implement the `Not` struct and `Predicate` trait
- The `evaluate` method should return the negation of the wrapped predicate
- Handle errors properly

**Starter Code:**
```rust
pub struct Not {
    predicate: Arc<dyn Predicate>,
}

impl Not {
    pub fn new(predicate: Arc<dyn Predicate>) -> Self {
        Self { predicate }
    }
}

impl Predicate for Not {
    // Your implementation here
}
```

**Sample Solution:**
```rust
impl Predicate for Not {
    fn evaluate(&self, batch: &Batch) -> Result<Vec<bool>> {
        let mask = self.predicate.evaluate(batch)?;
        Ok(mask.iter().map(|b| !b).collect())
    }
}
```

**Scoring:**
- 3 points: Correct implementation of evaluate()
- 1 point: Proper error handling
- 1 point: Correct negation logic

---

### Challenge 3.2: Implement COUNT(*) Aggregate (5 points)

**Task:** Implement a `CountStarAggregate` that counts all rows, including NULL values.

**Requirements:**
- Implement `CountStarAggregate` struct
- Implement `AggregateFunction` trait
- Count all rows regardless of NULL values
- Handle empty batches

**Starter Code:**
```rust
pub struct CountStarAggregate {
    count: usize,
}

impl CountStarAggregate {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl AggregateFunction for CountStarAggregate {
    // Your implementation here
}
```

**Sample Solution:**
```rust
impl AggregateFunction for CountStarAggregate {
    fn update(&mut self, batch: &Batch, _column_index: usize) -> Result<()> {
        // COUNT(*) ignores the column and counts all rows
        self.count += batch.num_rows();
        Ok(())
    }

    fn finalize(&self) -> Result<Value> {
        Ok(Value::Int64(self.count as i64))
    }

    fn data_type(&self) -> DataType {
        DataType::Int64
    }
}
```

**Scoring:**
- 2 points: Correct update() implementation
- 2 points: Correct finalize() implementation
- 1 point: Correct data_type() implementation

---

### Challenge 3.3: Fix Bug in Aggregate Creation (5 points)

**Task:** The following code has a bug. Identify and fix it.

**Buggy Code:**
```rust
fn create_aggregate(
    &self,
    name: &str,
    data_type: &DataType,
) -> PlanResult<Box<dyn AggregateFunction>> {
    match name.to_uppercase().as_str() {
        "SUM" => match data_type {
            DataType::Int64 => Ok(Box::new(SumAggregate::new(DataType::Int64))),
            DataType::Float64 => Ok(Box::new(SumAggregate::new(DataType::Float64))),
            DataType::String => Err(PlannerError::Custom(
                "SUM cannot be applied to String".to_string(),
            )),
        },
        "AVG" => match data_type {
            DataType::Int64 => Ok(Box::new(AvgAggregate::new(DataType::Int64))),
            DataType::Float64 => Ok(Box::new(AvgAggregate::new(DataType::Float64))),
            DataType::String => Err(PlannerError::Custom(
                "AVG cannot be applied to String".to_string(),
            )),
        },
        _ => Err(PlannerError::InvalidAggregateFunction(name.to_string())),
    }
}
```

**Sample Solution:**
```rust
fn create_aggregate(
    &self,
    name: &str,
    data_type: &DataType,
) -> PlanResult<Box<dyn AggregateFunction>> {
    match name.to_uppercase().as_str() {
        "SUM" => match data_type {
            DataType::Int64 => Ok(Box::new(SumAggregate::new(DataType::Int64)?)),
            DataType::Float64 => Ok(Box::new(SumAggregate::new(DataType::Float64)?)),
            DataType::String => Err(PlannerError::Custom(
                "SUM cannot be applied to String".to_string(),
            )),
        },
        "AVG" => match data_type {
            DataType::Int64 => Ok(Box::new(AvgAggregate::new(DataType::Int64)?)),
            DataType::Float64 => Ok(Box::new(AvgAggregate::new(DataType::Float64)?)),
            DataType::String => Err(PlannerError::Custom(
                "AVG cannot be applied to String".to_string(),
            )),
        },
        _ => Err(PlannerError::InvalidAggregateFunction(name.to_string())),
    }
}
```

**Explanation:** The bug is that `SumAggregate::new()` and `AvgAggregate::new()` return `Result<Self>`, not just `Self`. The `?` operator is needed to unwrap the Result.

**Scoring:**
- 3 points: Identifying the missing `?` operator
- 2 points: Explaining why it's needed (Result unwrapping)

---

## Section 4: Practical Exercises (Optional - No Points)

### Exercise 4.1: Performance Testing

**Task:** Measure the performance difference between:
1. A query without column pruning
2. The same query with column pruning

**Steps:**
1. Create a table with 10 columns and 1,000,000 rows
2. Execute `SELECT col0, col1 FROM table WHERE col2 > 100`
3. Measure execution time with and without column pruning
4. Document the performance difference

**Expected Outcome:** You should see a significant performance improvement (2-10x faster) with column pruning, as fewer bytes are read from storage.

---

### Exercise 4.2: Predicate Optimization

**Task:** Implement a predicate optimization that rewrites `NOT (NOT p)` to just `p`.

**Steps:**
1. Add an `optimize()` method to the `Predicate` trait
2. Implement the optimization for the `Not` predicate
3. Write tests to verify the optimization works
4. Benchmark the performance improvement

**Expected Outcome:** The optimized predicate tree should have fewer levels and execute faster.

---

### Exercise 4.3: Complex Group By

**Task:** Implement support for multiple GROUP BY columns and verify results.

**Steps:**
1. Create a table with columns: (department, city, salary)
2. Execute `SELECT department, city, AVG(salary) FROM employees GROUP BY department, city`
3. Verify the results are correct
4. Compare with manual calculations

**Expected Outcome:** The query should correctly group by both department and city, computing the average salary for each combination.

---

## Section 5: Evaluation Criteria

### Grading Scale

| Score Range | Grade | Performance Level |
|-------------|-------|-------------------|
| 45-50 | A+ | Excellent - Mastery demonstrated |
| 40-44 | A | Very Good - Strong understanding |
| 35-39 | B | Good - Adequate understanding |
| 30-34 | C | Satisfactory - Basic understanding |
| 0-29 | F | Needs Improvement |

### Passing Requirements

- Minimum of 35 points (70%)
- Must pass at least one coding challenge
- Must demonstrate understanding of core concepts

### Mastery Indicators

**A+ or A grade indicates:**
- Deep understanding of query planner architecture
- Ability to design and optimize execution plans
- Proficiency in Rust trait objects and error handling
- Strong debugging and problem-solving skills
- Ability to write comprehensive tests

**B grade indicates:**
- Good understanding of query planning concepts
- Ability to implement basic features correctly
- Familiarity with Rust syntax and patterns
- Basic testing skills

**C grade indicates:**
- Basic understanding of query planning
- Ability to implement simple features with guidance
- Familiarity with core concepts but struggles with complexity
- Limited testing experience

---

## Section 6: Study Guide

### Key Concepts to Review

1. **Query Planning**
   - SQL parsing and AST generation
   - Execution plan generation
   - Query optimization strategies

2. **Predicates**
   - Boolean logic (AND, OR, NOT)
   - Comparison operators
   - Type conversion and handling

3. **Aggregates**
   - COUNT, SUM, AVG, MIN, MAX
   - Handling NULL values
   - Grouping and aggregation

4. **Optimizations**
   - Column pruning
   - Predicate pushdown
   - Operator ordering

5. **Rust Concepts**
   - Trait objects (`dyn Trait`)
   - Smart pointers (`Box`, `Arc`)
   - Error handling (`Result`, `?`)
   - Ownership and borrowing

### Recommended Practice

1. Implement all examples from the Learning Guide
2. Write tests for each feature you implement
3. Debug failing tests systematically
4. Benchmark performance of different query patterns
5. Read database internals literature

### Additional Resources

- "Database Internals" by Alex Petrov
- "How Query Engines Work" by DuckDB Blog
- Rust Documentation on Trait Objects
- Wikipedia on Query Optimization

---

## Appendix: Reference Implementation

### Predicate Trait

```rust
pub trait Predicate: Send + Sync {
    /// Evaluate the predicate against a batch, returning a boolean mask
    /// where true indicates the row matches the predicate
    fn evaluate(&self, batch: &Batch) -> Result<Vec<bool>>;
}
```

### AggregateFunction Trait

```rust
pub trait AggregateFunction: Send + Sync {
    /// Update the aggregate with a new batch of data
    fn update(&mut self, batch: &Batch, column_index: usize) -> Result<()>;
    
    /// Finalize and return the aggregate result
    fn finalize(&self) -> Result<Value>;
    
    /// Get the data type of the result
    fn data_type(&self) -> DataType;
}
```

### Operator Trait

```rust
pub trait Operator: Send + Sync {
    /// Get the schema of the output
    fn schema(&self) -> &Schema;
    
    /// Open the operator for execution
    fn open(&mut self) -> Result<()>;
    
    /// Get the next batch of results
    fn next_batch(&mut self) -> Result<Batch>;
    
    /// Close the operator
    fn close(&mut self) -> Result<()>;
}
```

---

## Conclusion

This assessment covers the core concepts and practical implementation skills required for building a production-quality query planner. By successfully completing this assessment, you will have demonstrated:

- Understanding of database query optimization principles
- Proficiency in Rust programming for database systems
- Ability to design and implement complex data structures
- Skills in testing and debugging database components

Good luck! 🚀

---

**Assessment Version:** 1.0  
**Last Updated:** February 2026  
**Maintainer:** Mini Rust OLAP Project Team