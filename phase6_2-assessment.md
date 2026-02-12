# Phase 6.2 Assessment: ORDER BY, LIMIT, and OFFSET

## Overview

This assessment tests your understanding of Phase 6.2 of the Mini Rust OLAP project, which implements ORDER BY, LIMIT, and OFFSET clauses. The assessment includes multiple choice questions, true/false questions, short answer questions, code analysis, implementation challenges, and debugging exercises.

## Instructions

- **Time Limit**: 90 minutes
- **Passing Score**: 70%
- **Allowed Resources**: Access to the codebase and learning guide
- **Format**: Complete all questions in order
- **Submission**: Save your answers in a file named `phase6_2-assessment-answers.md`

---

## Part 1: Multiple Choice Questions (20 points)

Select the best answer for each question. Each question is worth 2 points.

### Question 1
What is the primary purpose of the ORDER BY clause?

A) To filter rows based on a condition
B) To sort the result set by one or more columns
C) To limit the number of rows returned
D) To aggregate data

### Question 2
Which of the following data types does the Sort operator support?

A) Int64, Float64, Boolean
B) Int64, Float64, String
C) Int64, String, Date
D) Float64, String, Boolean

### Question 3
In what order are LIMIT and OFFSET clauses processed?

A) LIMIT first, then OFFSET
B) OFFSET first, then LIMIT
C) They are processed simultaneously
D) The order doesn't matter

### Question 4
What is the default sort direction in ORDER BY?

A) DESC
B) ASC
C) RANDOM
D) NONE

### Question 5
When using ORDER BY with multiple columns, how are ties in the first column handled?

A) They are randomly ordered
B) They are discarded
C) They are sorted by the second column
D) They remain in their original order

### Question 6
What is a major limitation of the Sort operator in this implementation?

A) It only supports ascending order
B) It loads all data into memory
C) It cannot sort by multiple columns
D) It only works with Int64 data

### Question 7
Which operator is placed last in the execution plan tree?

A) TableScan
B) Filter
C) Sort
D) Limit

### Question 8
What does the `SortDirection` enum contain?

A) Ascending, Descending, Random
B) Ascending, Descending
C) Asc, Desc, None
D) Up, Down

### Question 9
How does the Limit operator improve performance?

A) By caching results
B) By stopping reading once the limit is reached
C) By sorting the data
D) By creating indexes

### Question 10
What happens when you use OFFSET without LIMIT?

A) An error is raised
B) No rows are returned
C) All rows after the offset are returned
D) Only the offset number of rows are returned

---

## Part 2: True/False Questions (10 points)

Indicate whether each statement is True or False. Each question is worth 1 point.

1. [ ] The ORDER BY clause can be used without LIMIT or OFFSET
2. [ ] The Sort operator uses external sorting for large datasets
3. [ ] NULL values are supported in sorting
4. [ ] The Limit operator always requires a LIMIT value
5. [ ] OFFSET values can be negative
6. [ ] Multi-column sorting processes columns from left to right
7. [ ] The parser adds new token types for ORDER BY, LIMIT, and OFFSET
8. [ ] The Sort operator uses a stable sorting algorithm
9. [ ] Column aliases can be used in ORDER BY
10. [ ] The execution plan order is: TableScan → Filter → GroupBy → Project → Sort → Limit

---

## Part 3: Short Answer Questions (30 points)

Answer each question concisely. Each question is worth 5 points.

### Question 1
Explain why ORDER BY should always be used with LIMIT for consistent pagination results.

### Question 2
Describe how the Sort operator handles multi-column sorting. Include the concept of precedence.

### Question 3
What are the three phases of the Sort operator's lifecycle (Open, Next Batch, Close), and what happens in each?

### Question 4
Explain the memory considerations of the Sort operator. What are the pros and cons of loading all data into memory?

### Question 5
Why is it important to combine ORDER BY with OFFSET when implementing pagination? What can happen if you use OFFSET without ORDER BY?

### Question 6
Describe how the planner resolves column names in ORDER BY to column indices. What challenges arise when ORDER BY is used with GROUP BY?

---

## Part 4: Code Analysis (20 points)

Analyze the following code snippets and answer the questions.

### Question 1 (10 points)

Consider this SQL query:
```sql
SELECT name, age, department 
FROM users 
WHERE age > 25 
ORDER BY department ASC, age DESC 
LIMIT 5 OFFSET 10;
```

Draw the execution plan tree and explain how data flows through it. For each operator, describe what it does and what the output looks like.

### Question 2 (10 points)

Analyze this Rust code from the Sort operator:
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
            _ => Equal,
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

Explain:
1. Why does it use `partial_cmp` for Float64 but `cmp` for Int64 and String?
2. What happens when `cmp != Equal`?
3. What does the function return if all columns are equal?
4. Why is the loop structured this way?

---

## Part 5: Implementation Challenge (15 points)

Implement the following feature: Add support for the `NULLS FIRST` and `NULLS LAST` options in ORDER BY.

### Requirements

1. Extend the `SortDirection` enum to include NULL position:
```rust
pub enum SortDirection {
    Ascending,
    Descending,
    // Add new variants here
}
```

2. Update the parser to recognize `NULLS FIRST` and `NULLS LAST` keywords after the column name and direction.

3. Modify the `compare_rows` function in the Sort operator to handle NULL values according to the specified NULL position.

4. Update the `OrderByItem` struct to include the NULL position.

### Tasks

1. Define the new enum structure for SortDirection (5 points)
2. Show the parser changes needed to parse `NULLS FIRST/LAST` (5 points)
3. Write the updated `compare_rows` function signature and logic (5 points)

### Notes

- Assume NULL values are now represented in the Value enum
- NULL values should be sorted before or after non-NULL values based on the option
- Maintain backward compatibility (default should be `NULLS LAST` for ASC and `NULLS FIRST` for DESC)

---

## Part 6: Debugging Challenge (15 points)

### Scenario

A developer is implementing a feature to display the top 10 most expensive products in each category. They write the following query:

```sql
SELECT category, product_name, price 
FROM products 
ORDER BY price DESC 
LIMIT 10;
```

However, this returns the 10 most expensive products overall, not per category. The developer tries to fix it with GROUP BY:

```sql
SELECT category, product_name, MAX(price) as max_price
FROM products 
GROUP BY category 
ORDER BY max_price DESC 
LIMIT 10;
```

This query compiles but doesn't work as expected. It returns only 10 rows and doesn't include the product names correctly.

### Questions

1. Why doesn't the first query return the top 10 products per category? (3 points)

2. What's wrong with the second query? Why doesn't it include product names correctly? (4 points)

3. Explain why the column mapping in ORDER BY with GROUP BY is challenging. (4 points)

4. Propose a solution to achieve the desired result (top 10 products per category) using the current implementation. You may use multiple queries or suggest a database schema change. (4 points)

---

## Part 7: Critical Thinking (20 points)

Answer the following questions with detailed explanations.

### Question 1 (10 points)

You're designing a system for a social media platform that needs to display a feed of posts. The requirements are:

- Show the most recent posts first
- Implement pagination with 20 posts per page
- Handle millions of posts
- Support filtering by user, hashtag, or date range

Design a query strategy that uses ORDER BY, LIMIT, and OFFSET effectively. Address:

1. What indexes would you create?
2. How would you handle pagination for very deep pages (e.g., page 10,000)?
3. What are the performance implications of your approach?
4. How would you optimize for the common case (first few pages)?

### Question 2 (10 points)

Compare and contrast the current in-memory Sort implementation with a hypothetical external sort implementation. Discuss:

1. When would in-memory sort be preferred?
2. When would external sort be necessary?
3. What are the trade-offs between the two approaches in terms of:
   - Memory usage
   - Performance (speed)
   - Implementation complexity
   - User experience (latency)
4. How would you design a hybrid approach that automatically chooses between in-memory and external sort based on data size?

---

## Part 8: Code Writing Challenge (20 points)

### Task: Implement a "Top N per Group" Pattern

Write a Rust function that, given a table with columns (group_id, value, name), returns the top N values per group.

### Function Signature

```rust
/// Returns the top N values per group from the given table
/// 
/// # Arguments
/// * `table` - The table to query
/// * `group_col` - The name of the column to group by
/// * `value_col` - The name of the column to order by
/// * `limit` - The number of top values to return per group
/// 
/// # Returns
/// A result containing a vector of batches with the top N per group
fn top_n_per_group(
    table: &Table,
    group_col: &str,
    value_col: &str,
    limit: usize,
) -> Result<Vec<Batch>> {
    // Your implementation here
}
```

### Requirements

1. The function should:
   - Group rows by `group_col`
   - Within each group, sort by `value_col` in descending order
   - Return only the top `limit` rows per group
   - Return results sorted by group, then by value descending

2. Handle edge cases:
   - Empty table
   - Group with fewer rows than limit
   - Invalid column names

3. Use the existing operators (TableScan, GroupBy, Sort, Limit) where possible

4. Write tests to verify correctness (at least 3 test cases)

### Example

Given this table:
```
group_id | value | name
---------|-------|--------
A        | 100   | Item1
A        | 80    | Item2
A        | 90    | Item3
B        | 50    | Item4
B        | 70    | Item5
C        | 30    | Item6
```

Calling `top_n_per_group(table, "group_id", "value", 2)` should return:
```
group_id | value | name
---------|-------|--------
A        | 100   | Item1
A        | 90    | Item3
B        | 70    | Item5
B        | 50    | Item4
C        | 30    | Item6
```

### Scoring

- Correct implementation: 10 points
- Proper error handling: 4 points
- Test cases: 4 points
- Code quality and documentation: 2 points

---

## Part 9: Advanced Topics (20 points)

Choose ONE of the following advanced topics and answer in depth.

### Option A: Window Functions

Research and explain how window functions differ from GROUP BY with ORDER BY/LIMIT for ranking tasks. Specifically:

1. Compare `ROW_NUMBER() OVER (PARTITION BY group ORDER BY value DESC)` with GROUP BY approach
2. Explain what window functions can do that GROUP BY cannot
3. Discuss the implementation challenges of window functions in the current architecture
4. Sketch a design for adding basic window function support

### Option B: Keyset Pagination

Analyze the performance issues with OFFSET-based pagination and design a keyset pagination solution:

1. Explain why OFFSET 1000000 LIMIT 10 is slow
2. Describe how keyset pagination works
3. Modify the query planner to support keyset pagination
4. Write SQL examples showing how to implement keyset pagination for:
   - Sorting by a single column (id)
   - Sorting by multiple columns (created_at, id)

### Option C: Push-down Optimization

Design and explain how to push ORDER BY down to the data source:

1. When would push-down ORDER BY be beneficial?
2. How would you modify the planner to detect push-down opportunities?
3. What constraints must be satisfied for safe push-down?
4. Provide examples of queries that can and cannot be optimized

---

## Bonus Questions (Optional, 10 points each)

### Bonus 1
Design a streaming sort algorithm that can sort an arbitrarily large dataset using only O(1) memory (aside from the output buffer). Describe the algorithm and its trade-offs.

### Bonus 2
Implement a "smart" LIMIT that, when combined with ORDER BY, can stop sorting early once it knows the top N rows won't change. Describe the approach and implement a proof of concept.

---

## Answer Key

### Part 1: Multiple Choice

1. B - To sort the result set by one or more columns
2. B - Int64, Float64, String
3. B - OFFSET first, then LIMIT
4. B - ASC
5. C - They are sorted by the second column
6. B - It loads all data into memory
7. D - Limit
8. B - Ascending, Descending
9. B - By stopping reading once the limit is reached
10. C - All rows after the offset are returned

### Part 2: True/False

1. True
2. False
3. False
4. False (LIMIT can be None)
5. False
6. True
7. True
8. True
9. False
10. True

### Part 3: Short Answers

*Note: Detailed answers would be provided in the answer key*

### Part 4: Code Analysis

*Note: Detailed answers would be provided in the answer key*

### Part 5-9

*Note: Detailed solutions would be provided in the answer key*

---

## Grading Rubric

| Section | Points | Criteria |
|---------|--------|----------|
| Part 1: Multiple Choice | 20 | 2 points per correct answer |
| Part 2: True/False | 10 | 1 point per correct answer |
| Part 3: Short Answer | 30 | 5 points per question; completeness, accuracy, clarity |
| Part 4: Code Analysis | 20 | 10 points per question; understanding of concepts |
| Part 5: Implementation | 15 | Requirements met, correct design, code quality |
| Part 6: Debugging | 15 | Problem identification, solution correctness |
| Part 7: Critical Thinking | 20 | Depth of analysis, practical considerations |
| Part 8: Code Writing | 20 | Correctness, error handling, tests, code quality |
| Part 9: Advanced Topics | 20 | Understanding, design quality, depth |
| **Total** | **170** | **Passing: 119 points (70%)** |

---

## Submission Checklist

- [ ] Answer all questions
- [ ] Include code for implementation tasks
- [ ] Write test cases where required
- [ ] Explain your reasoning
- [ ] Verify your answers against the codebase
- [ ] Save answers as `phase6_2-assessment-answers.md`

---

## Notes for Instructors

### Time Management
- Allow 90 minutes for the assessment
- Additional 30 minutes for review if needed
- Can be split into two sessions if preferred

### Difficulty Level
- Part 1-2: Beginner
- Part 3-4: Intermediate
- Part 5-7: Advanced
- Part 8-9: Expert

### Prerequisites
Students should have:
- Completed Phase 6.2 implementation
- Read the Phase 6.2 learning guide
- Understanding of SQL fundamentals
- Rust programming experience

### Learning Outcomes

After completing this assessment, students should be able to:

1. Explain how ORDER BY, LIMIT, and OFFSET work
2. Analyze and optimize queries using these clauses
3. Identify limitations and performance implications
4. Design pagination strategies
5. Debug and fix related issues
6. Implement extensions to the current system

---

**Good luck with the assessment!**