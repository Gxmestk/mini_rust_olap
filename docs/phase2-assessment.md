<file_path>
mini_rust_olap/docs/phase2-assessment.md
</file_path>
# Phase 2 Assessment: Storage Layer

## 📋 Assessment Overview

This assessment tests your understanding of Rust programming concepts and database internals covered in Phase 2 of Mini Rust OLAP (Table and Catalog implementation).

### Assessment Structure

| Part | Topic | Questions | Difficulty |
|-------|--------|------------|------------|
| 1 | Rust Fundamentals | 10 | Beginner/Intermediate |
| 2 | Table Operations | 10 | Intermediate |
| 3 | Catalog Management | 10 | Intermediate/Advanced |
| 4 | Advanced Topics | 5 | Advanced |
| **Total** | | **35** | |

### Scoring Guide

- **30-35**: Excellent understanding - ready for Phase 3
- **25-29**: Good understanding - review weak areas
- **20-24**: Fair understanding - review all topics
- **Below 20**: Needs review - revisit Phase 2 materials

---

## Part 1: Rust Fundamentals

### Q1. What is the primary advantage of using a HashMap over a Vec for lookups?

A. Lower memory usage
B. O(1) average time complexity for lookups
C. Easier to implement
D. Automatic sorting

### Q2. What does the Entry API in HashMap provide?

A. Thread-safe operations
B. Efficient conditional insert/modify operations
C. Automatic serialization
D. Faster iteration

### Q3. What is the difference between HashMap and BTreeMap?

A. HashMap is O(log n), BTreeMap is O(1)
B. HashMap is unordered, BTreeMap maintains sorted order
C. HashMap only accepts strings, BTreeMap accepts any type
D. There is no significant difference

### Q4. Why would you implement a trait manually instead of deriving it?

A. Deriving is not supported for that trait
B. You need custom behavior beyond what derive provides
C. To improve compile time
D. To reduce binary size

### Q5. What does `Box<dyn Trait>` represent?

A. A trait with multiple implementations
B. A heap-allocated trait object for runtime polymorphism
C. A compile-time generic constraint
D. A mutable trait reference

### Q6. What is dynamic dispatch?

A. Method calls resolved at compile time
B. Method calls resolved at runtime through vtable
C. Automatic error handling
D. Parallel code execution

### Q7. When should you use `.copied()` with HashMap values?

A. When the value type implements Copy
B. When you need a mutable reference
C. When the value is a String
D. When the HashMap is empty

### Q8. What does the `?` operator do in Rust?

A. Ignores errors and continues
B. Converts Option to Result
C. Returns early with error if Result is Err
D. Creates a new error value

### Q9. What is the purpose of the `Default` trait?

A. Define default configuration values
B. Enable cloning of types
C. Provide string representation
D. Handle errors automatically

### Q10. What does `self.schema.keys().cloned().collect()` do?

A. Gets all keys, clones each, collects into Vec
B. Gets all keys, removes clones, collects into Vec
C. Gets all values, clones each, collects into Vec
D. Creates a new HashMap from keys

---

## Part 2: Table Operations

### Q11. Why do we maintain both `columns` (Vec) and `column_index` (HashMap)?

A. Vec for ordered iteration, HashMap for O(1) name lookup
B. Vec for memory efficiency, HashMap for validation
C. Vec for type safety, HashMap for serialization
D. Redundancy is intentional for backup

### Q12. What does the table schema define?

A. The order of rows in the table
B. Mapping from column name to data type
C. Database connection parameters
D. Index definitions for columns

### Q13. What happens when you call `add_column()` with a duplicate name?

A. Returns Ok and merges the columns
B. Returns DatabaseError with "already exists" message
C. Silently ignores the duplicate
D. Renames the existing column

### Q14. Why must all columns have the same row count in a table?

A. To simplify the implementation
B. To ensure data integrity for valid operations
C. To enable compression
D. To match CSV file format

### Q15. What does the `add_row()` method accept as input?

A. Vec<Value>
B. Vec<String>
C. Vec<DataType>
D. HashMap<String, Value>

### Q16. How does `add_row()` convert string values to appropriate types?

A. Uses runtime type inference with pattern matching
B. Requires explicit type casting by caller
C. Converts everything to String
D. Uses schema information for conversion

### Q17. What does the `select_columns()` method do?

A. Creates a new table with only specified columns
B. Filters rows based on column values
C. Removes columns from the table
D. Sorts columns alphabetically

### Q18. Why can't we derive `Clone` for the Table struct?

A. Table is too large to clone
B. `Box<dyn Column>` doesn't implement Clone automatically
C. Clone is not supported for structs
D. It would be too slow

### Q19. What does the `validate_schema()` method check?

A. That column names are unique
B. That all columns have consistent row counts
C. That data types are valid
D. All of the above

### Q20. What is returned by `get_value(column_name, row_index)`?

A. Vec of values in the column
B. Reference to the column
C. Single Value at specified position
D. Data type of the column

---

## Part 3: Catalog Management

### Q21. What is the primary responsibility of the Catalog?

A. Execute SQL queries
B. Store and manage all tables
C. Compress data for storage
D. Handle network connections

### Q22. What does `register_table()` do if a table with the same name exists?

A. Returns Ok and replaces the table
B. Returns DatabaseError with "already exists" message
C. Merges the new table with existing one
D. Renames the existing table

### Q23. What is the difference between `get_table()` and `get_table_mut()`?

A. get_table is faster
B. get_table returns immutable reference, get_table_mut returns mutable
C. get_table is for local tables, get_table_mut for remote
D. No significant difference

### Q24. What does `drop_table()` do?

A. Removes a table from the catalog
B. Deletes the table's data from disk
C. Marks the table as hidden
D. Archives the table

### Q25. Why does `rename_table()` validate that both old and new names exist correctly?

A. To prevent accidental data loss
B. To ensure transactional integrity
C. Both A and B
D. Neither A nor B

### Q26. What does `list_tables()` return?

A. Vec of Table objects
B. Vec of table names
C. HashMap of table metadata
D. Single table object

### Q27. What does `list_tables_sorted()` provide over `list_tables()`?

A. Faster performance
B. Alphabetically sorted table names
C. Includes hidden tables
D. Returns tables with statistics

### Q28. What does the `clear()` method do?

A. Removes all tables from catalog
B. Clears data from all tables
C. Resets catalog to initial state
D. All of the above

### Q29. What is the return type of `table_exists()`?

A. Result<bool>
B. Option<&Table>
C. bool
D. &str

### Q30. Why implement both `Clone` and `Default` for Catalog?

A. Clone for copying, Default for convenience creation
B. Clone for serialization, Default for testing
C. Required by Rust compiler
D. To enable pattern matching

---

## Part 4: Advanced Topics

### Q31. What is the purpose of invariant checking in complex data structures?

A. Catch bugs early in development
B. Improve performance
C. Reduce memory usage
D. Enable serialization

### Q32. Why validate before mutating data structures rather than after?

A. It's faster
B. Leaves system in consistent state if validation fails
C. Required by Rust
D. Simpler to implement

### Q33. What is the key difference between unit and integration tests?

A. Unit tests are for public APIs, integration for private
B. Unit tests test individual functions, integration test workflows
C. Unit tests use mock data, integration use real data
D. Unit tests are faster, integration are more accurate

### Q34. What is the builder pattern used for?

A. Error handling
B. Complex object construction with many optional parameters
C. Thread synchronization
D. Memory management

### Q35. Why document public APIs thoroughly?

A. To satisfy linters
B. Specify behavior, errors, and usage as a contract
C. Increase code size
D. To make compilation faster

---

## 📊 Answer Key

### Part 1: Rust Fundamentals

| Question | Correct Answer | Explanation |
|----------|----------------|-------------|
| Q1 | **B** | HashMap provides O(1) average time complexity for lookups, much faster than Vec's O(n) |
| Q2 | **B** | Entry API enables efficient conditional insert/modify operations in a single lookup |
| Q3 | **B** | HashMap is unordered and O(1), BTreeMap maintains sorted order and is O(log n) |
| Q4 | **B** | Manual implementation allows custom behavior that derive macros cannot provide |
| Q5 | **B** | `Box<dyn Trait>` creates a heap-allocated trait object for runtime polymorphism |
| Q6 | **B** | Dynamic dispatch resolves method calls at runtime through a vtable (virtual table) |
| Q7 | **A** | `.copied()` is used when value type implements Copy to get owned value instead of reference |
| Q8 | **C** | The `?` operator returns early with error if Result is Err, otherwise unwraps Ok value |
| Q9 | **A** | Default trait defines default values for types, useful for initialization |
| Q10 | **A** | Gets all keys as references, clones each to owned values, collects into Vec<String> |

### Part 2: Table Operations

| Question | Correct Answer | Explanation |
|----------|----------------|-------------|
| Q11 | **A** | Vec maintains insertion order for iteration, HashMap provides O(1) name-based lookup |
| Q12 | **B** | Schema defines mapping from column names to their data types (Int64, Float64, String) |
| Q13 | **B** | Returns DatabaseError with descriptive "already exists" message to prevent duplicates |
| Q14 | **B** | Consistent row counts ensure data integrity for valid operations across columns |
| Q15 | **B** | `add_row()` accepts Vec<String> for convenience, then parses based on column types |
| Q16 | **A** | Uses runtime type inference with pattern matching to convert strings to appropriate types |
| Q17 | **A** | Creates a new table containing only the specified columns (column projection) |
| Q18 | **B** | `Box<dyn Column>` doesn't implement Clone automatically, so we implement it manually |
| Q19 | **B** | Validates that all columns have consistent row counts for data integrity |
| Q20 | **C** | Returns a single Value at the specified column and row position |

### Part 3: Catalog Management

| Question | Correct Answer | Explanation |
|----------|----------------|-------------|
| Q21 | **B** | Catalog stores and manages all tables as the central metadata repository |
| Q22 | **B** | Returns DatabaseError with "already exists" message to prevent naming conflicts |
| Q23 | **B** | `get_table()` returns immutable &Table, `get_table_mut()` returns mutable &mut Table |
| Q24 | **A** | Removes a table from the catalog's HashMap, freeing the table from management |
| Q25 | **C** | Validates to prevent accidental data loss (old name exists) and maintain integrity (new name doesn't exist) |
| Q26 | **B** | Returns Vec<String> containing all table names registered in the catalog |
| Q27 | **B** | Returns table names sorted alphabetically for consistent display |
| Q28 | **A** | Removes all tables from the catalog, effectively resetting it to empty |
| Q29 | **C** | Returns a simple bool indicating whether a table with given name exists |
| Q30 | **A** | Clone enables copying catalog instances; Default provides convenient creation of empty catalog |

### Part 4: Advanced Topics

| Question | Correct Answer | Explanation |
|----------|----------------|-------------|
| Q31 | **A** | Invariant checking catches bugs early and documents expected behavior of data structures |
| Q32 | **B** | Validating before mutation leaves system in consistent state if validation fails, avoiding partial changes |
| Q33 | **B** | Unit tests test individual functions/units; integration tests test how components work together |
| Q34 | **B** | Builder pattern is used for complex object construction with many optional or configurable parameters |
| Q35 | **B** | Thorough documentation serves as a contract specifying behavior, errors, and proper usage |

---

## 🎯 Scoring and Feedback

### Calculate Your Score

```
Part 1 (Rust Fundamentals):     _____ / 10
Part 2 (Table Operations):      _____ / 10
Part 3 (Catalog Management):     _____ / 10
Part 4 (Advanced Topics):        _____ / 5
------------------------------------------------
TOTAL SCORE:                      _____ / 35
```

### Interpret Your Score

#### 30-35 Points: Excellent! 🎉
- You have a strong understanding of Phase 2 concepts
- Ready to tackle Phase 3 (CSV Ingestion) confidently
- Consider reviewing Phase 2 code to solidify your knowledge

#### 25-29 Points: Good! 👍
- You understand most concepts well
- Review the questions you missed in each part
- Revisit corresponding chapters in the learning guide

#### 20-24 Points: Fair 👌
- You have basic understanding but gaps remain
- Review all Phase 2 materials systematically
- Practice with code examples and exercises

#### Below 20 Points: Needs Review 📚
- Return to Phase 2 learning guide
- Study chapters for areas where you struggled
- Run code examples and modify them to understand better

### Recommended Next Steps

1. **Review Wrong Answers**: Go back to specific questions and understand why you were wrong
2. **Read Corresponding Chapters**: Revisit learning guide chapters for missed topics
3. **Practice Code**: Modify Phase 2 code to experiment with concepts
4. **Complete Exercises**: Try the practical exercises in Chapter 11 of the learning guide
5. **Retake Assessment**: After reviewing, try the assessment again

---

## 📚 Study Resources

Based on your performance, focus on:

### Struggled with Rust Fundamentals (Part 1)?
- Review [The Rust Book](https://doc.rust-lang.org/book/) chapters on:
  - Collections (Chapter 8)
  - Smart Pointers (Chapter 15)
  - Advanced Traits (Chapter 17)
- Study HashMap and BTreeMap documentation
- Practice with trait objects and dynamic dispatch

### Struggled with Table Operations (Part 2)?
- Review source code:
  - `src/table.rs` - Understand table structure
  - Schema validation logic
  - Row insertion with type inference
- Experiment with creating and manipulating tables
- Practice with column operations

### Struggled with Catalog Management (Part 3)?
- Review source code:
  - `src/catalog.rs` - Understand catalog design
  - Table registration and lifecycle
  - Mutable vs immutable access patterns
- Build a mini application using catalog
- Practice with multiple table workflows

### Struggled with Advanced Topics (Part 4)?
- Review learning guide sections on:
  - Testing strategies
  - Design patterns
  - Best practices
- Write comprehensive tests for your own code
- Practice invariant checking

---

## 💡 Tips for Learning

1. **Code Along**: Don't just read - type out code examples yourself
2. **Experiment**: Modify code to see what breaks and why
3. **Build Something**: Apply concepts to your own small project
4. **Teach Others**: Explaining concepts solidifies understanding
5. **Practice**: Repetition builds long-term retention

---

## 🎓 Self-Reflection Questions

After completing the assessment, ask yourself:

1. Which concepts felt most natural? Why do you think that is?
2. Which concepts were most difficult? How can you approach them differently?
3. Can you explain the difference between HashMap and BTreeMap to someone else?
4. Do you understand why we can't derive Clone for Table?
5. Can you describe the purpose of a Catalog in a database system?
6. How would you modify the Table design to support schema changes?
7. What would you add to the Catalog for a production database?
8. Can you write integration tests for a scenario involving both Table and Catalog?

---

## 🚀 Preparation for Phase 3

To ensure you're ready for Phase 3 (CSV Ingestion), verify you can:

### Rust Skills
- [ ] Comfortably use HashMap and BTreeMap
- [ ] Implement traits manually when needed
- [ ] Handle errors with proper context
- [ ] Write unit and integration tests
- [ ] Use trait objects (`dyn Trait`)

### Database Skills
- [ ] Understand table schema and validation
- [ ] Create and manipulate tables
- [ ] Use catalog to manage multiple tables
- [ ] Perform table operations (insert, select, drop)
- [ ] Maintain data integrity

### Design Skills
- [ ] Think about invariants and validation
- [ ] Design clean public APIs
- [ ] Write comprehensive documentation
- [ ] Follow testing best practices

If you checked most boxes, you're ready for Phase 3! If not, review the corresponding topics before proceeding.

---

**Good luck! This assessment is designed to help you identify strengths and areas for improvement. Use it as a learning tool, not just a test!** 🦀