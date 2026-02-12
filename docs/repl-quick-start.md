# Mini Rust OLAP - REPL Quick Start Guide

## 🚀 Getting Started

The Mini Rust OLAP REPL (Read-Eval-Print Loop) provides an interactive command-line interface for loading, querying, and analyzing CSV data using SQL-like queries.

### Starting the REPL

```bash
# Build the project
cargo build --release

# Run the REPL
./target/release/mini_rust_olap
```

Or use cargo directly:
```bash
cargo run --release
```

You'll see the welcome screen:
```
╔═════════════════════════════════════════════════════════╗
║     Mini Rust OLAP - Interactive REPL v0.1.0             ║
╚═════════════════════════════════════════════════════════╝

Welcome to Mini Rust OLAP! Type HELP for available commands.

olap>
```

## 📊 Loading Data

### Loading a CSV File

Create a sample CSV file `employees.csv`:
```csv
id,name,age,salary,department
1,Alice,30,75000,Engineering
2,Bob,35,82000,Sales
3,Charlie,42,95000,Engineering
4,Diana,31,68000,Marketing
5,Eve,29,72000,Engineering
```

Load it into the REPL:
```sql
olap> LOAD employees.csv AS employees
Loading CSV from 'employees.csv' as 'employees'...
✓ Loaded table 'employees' successfully.
⏱ Executed in 5.23ms
```

**Note:** CSV files are automatically parsed with type inference:
- Integers → Int64
- Decimals → Float64  
- Text → String

## 🔍 Querying Data

### Basic SELECT

Select all columns:
```sql
olap> SELECT * FROM employees
```

Select specific columns:
```sql
olap> SELECT name, salary FROM employees
```

### Filtering with WHERE

Numeric comparisons:
```sql
olap> SELECT name, salary FROM employees WHERE salary > 70000
```

String comparisons:
```sql
olap> SELECT * FROM employees WHERE department = 'Engineering'
```

### Sorting with ORDER BY

Ascending order:
```sql
olap> SELECT name, salary FROM employees ORDER BY salary ASC
```

Descending order:
```sql
olap> SELECT name, salary FROM employees ORDER BY salary DESC
```

### Limiting Results

```sql
olap> SELECT * FROM employees LIMIT 3
```

Combine with ORDER BY:
```sql
olap> SELECT name, salary FROM employees ORDER BY salary DESC LIMIT 2
```

### Grouping and Aggregation

Count rows by department:
```sql
olap> SELECT department, COUNT(*) FROM employees GROUP BY department
```

Average salary by department:
```sql
olap> SELECT department, AVG(salary) FROM employees GROUP BY department
```

Multiple aggregates:
```sql
olap> SELECT department, 
         COUNT(*) as count,
         AVG(salary) as avg_salary,
         MIN(salary) as min_salary,
         MAX(salary) as max_salary
       FROM employees 
       GROUP BY department
```

**Note:** The REPL currently shows column names as `col_0`, `col_1`, etc. Use `DESCRIBE` to see the actual column names.

## 📋 Catalog Management

### List All Tables

```sql
olap> SHOW TABLES
Tables in catalog:
  - employees
⏱ Executed in 0.05ms
```

### Describe Table Schema

```sql
olap> DESCRIBE employees

Table: employees
┌────────────────────────┬──────────┬────────────────┐
│ Column Name            │ Type     │ Description    │
├────────────────────────┼──────────┼────────────────┤
│ id                     │ Int64    │           5 rows │
│ name                   │ String   │           5 rows │
│ age                    │ Int64    │           5 rows │
│ salary                 │ Float64  │           5 rows │
│ department             │ String   │           5 rows │
└────────────────────────┴──────────┴────────────────┘
Total rows: 5
⏱ Executed in 0.47ms
```

## ⚙️ Utility Commands

### Help

```sql
olap> HELP
```

Displays all available commands and their syntax.

### Clear Screen

```sql
olap> CLEAR
```

Clears the terminal screen.

### Exit

```sql
olap> EXIT
```

Or use `QUIT`.

## 💡 Tips & Tricks

### Command History

The REPL saves your command history to `.olap_history` in your working directory. Use up/down arrows to navigate previous commands.

### Case Insensitivity

Commands are case-insensitive. These all work:
```sql
SHOW TABLES
show tables
Show Tables
```

### Error Messages

If you encounter an error, it will be displayed in a formatted box:
```
╔═════════════════════════════════════════════════════════╗
║ ❌ ERROR                                                  ║
╠═════════════════════════════════════════════════════════╣
║ Table 'nonexistent' not found in catalog                ║
╚═════════════════════════════════════════════════════════╝
```

### Performance Timing

Every command displays its execution time, helping you understand query performance:
```
⏱ Executed in 2.36ms
```

## 📝 Example Session

Here's a complete example session:

```sql
olap> LOAD employees.csv AS employees
✓ Loaded table 'employees' successfully.

olap> SHOW TABLES
Tables in catalog:
  - employees

olap> DESCRIBE employees
Table: employees
┌────────────────────────┬──────────┬────────────────┐
│ Column Name            │ Type     │ Description    │
├────────────────────────┼──────────┼────────────────┤
│ id                     │ Int64    │           5 rows │
│ name                   │ String   │           5 rows │
│ age                    │ Int64    │           5 rows │
│ salary                 │ Float64  │           5 rows │
│ department             │ String   │           5 rows │
└────────────────────────┴──────────┴────────────────┘
Total rows: 5

olap> SELECT department, COUNT(*), AVG(salary) 
         FROM employees 
         GROUP BY department
┌────────────────────┐
│ col_0   │ col_1  │ col_2    │
├────────────────────┤
│ Engineering │ 3      │ 80666.8  │
│ Marketing   │ 1      │ 68000    │
│ Sales       │ 1      │ 82000    │
└────────────────────┘

olap> SELECT name, salary 
         FROM employees 
         WHERE salary > 70000 
         ORDER BY salary DESC 
         LIMIT 3
┌────────────────────┐
│ col_0   │ col_1    │
├────────────────────┤
│ Charlie │ 95000    │
│ Bob     │ 82000    │
│ Alice   │ 75000    │
└────────────────────┘

olap> EXIT
Goodbye!
```

## 🐛 Troubleshooting

### File Not Found

If you get a file not found error:
```
❌ ERROR
Failed to load CSV: File 'data.csv' not found
```

Check:
- The file path is correct (relative to current directory)
- The file extension is `.csv`
- You have read permissions on the file

### Table Already Exists

```
❌ ERROR
Table 'employees' already exists in catalog
```

You can't load two tables with the same name. Either:
- Drop the table first (not yet implemented), or
- Use a different table name in your LOAD command

### Invalid SQL Syntax

```
❌ ERROR
Parser error: Expected FROM at line 1, column 15
```

Check:
- Your SQL syntax is correct
- Keywords are spelled correctly
- Table and column names exist
- Strings are properly quoted with single quotes

## 📚 Next Steps

- Read the [Phase 7 Learning Guide](phase7-learning-guide.md) for implementation details
- Try the [Phase 7 Assessment](phase7-assessment.md) to test your understanding
- Explore the codebase in `src/main.rs` to see how the REPL works
- Check out [Phase 6](phase6_2-learning-guide.md) for advanced query features

## 🎯 SQL Syntax Reference

### SELECT Statement

```sql
SELECT [column1, column2, ... | *]
FROM table_name
WHERE condition
GROUP BY column1, column2, ...
ORDER BY column1 [ASC | DESC], column2 [ASC | DESC], ...
LIMIT number
```

### Supported Operators

**Comparison:**
- `=` , `!=` , `<>` , `<`, `<=`, `>`, `>=`

**Logical:**
- `AND`, `OR`, `NOT`

**Arithmetic:**
- `+`, `-`, `*`, `/`

### Aggregate Functions

- `COUNT(*)` - Count all rows
- `SUM(column)` - Sum values
- `AVG(column)` - Average of values
- `MIN(column)` - Minimum value
- `MAX(column)` - Maximum value

## 🚀 Enjoy Using Mini Rust OLAP!

The REPL provides a fast, interactive way to analyze your CSV data with SQL queries. Load your data and start exploring!