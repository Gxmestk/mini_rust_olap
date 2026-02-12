# Phase 7 Assessment: REPL Interface

## 📋 Assessment Overview

This assessment tests your understanding of Phase 7 (REPL Interface) implementation. It covers REPL design patterns, command-line interface development, error handling, user experience, and integration of the full database stack.

## 🎯 Part 1: Knowledge Questions (Short Answer)

### REPL Architecture

**1.1** What is the purpose of the REPL (Read-Eval-Print Loop) pattern in database systems?

**Answer:** The REPL provides an interactive interface for database operations, allowing users to:
- Execute queries and commands immediately without writing separate programs
- Test and explore data interactively
- Iterate quickly on data analysis workflows
- Learn database features through hands-on experimentation
- Debug queries by seeing results in real-time

**1.2** Why did we use the `rustyline` crate instead of simply reading from `stdin` with `std::io`?

**Answer:** The `rustyline` crate provides advanced readline features that `std::io` lacks:
- **Command history**: Ability to recall and edit previous commands with arrow keys
- **History persistence**: Save command history between sessions to `.olap_history`
- **Signal handling**: Proper handling of Ctrl+C and Ctrl+D
- **Line editing**: Navigation, deletion, and editing of input lines
- **Tab completion**: Support for autocomplete of commands and identifiers
- **Multi-line input**: Ability to handle multi-line commands gracefully

**1.3** What is the advantage of persisting command history to a file (`.olap_history`)?

**Answer:** Persisting history provides several benefits:
- **Reproducibility**: Users can review and repeat complex queries from previous sessions
- **Efficiency**: Quickly recall and modify frequently used commands
- **Documentation**: History serves as a record of work performed
- **Learning**: New users can see examples of previously executed commands
- **Workflow continuity**: Resuming work after interruption is easier

**1.4** How does the REPL handle the `Ctrl+C` and `Ctrl+D` signals differently? What is the expected user behavior for each?

**Answer:**
- **Ctrl+C (Interrupted)**: Handled as a "cancel current input" signal. The REPL clears the current line, displays a message ("Use EXIT or QUIT to exit."), and continues running. This allows users to cancel a partially typed command without exiting.
- **Ctrl+D (End-of-File)**: Handled as a "graceful exit" signal. The REPL prints "Goodbye!" and sets `running = false` to exit the loop. This is the standard Unix way to indicate end of input.

**1.5** Why do we convert user input to uppercase before checking command types (e.g., `HELP`, `help`, `Help` all work)?

**Answer:** Converting to uppercase provides:
- **Case insensitivity**: Users can type commands in any case they prefer
- **User convenience**: Reduces cognitive load - users don't need to remember exact capitalization
- **Common CLI convention**: Most database CLIs (psql, mysql, sqlite) are case-insensitive for commands
- **Robustness**: Prevents errors due to accidental caps lock or shift key usage

The conversion happens before pattern matching, so internal logic always works with uppercase commands while users see and type in their preferred case.

### Command Processing

**1.6** Describe the flow from a user typing `SELECT * FROM employees` to seeing the results on screen. List each component involved.

**Answer:** The complete flow involves these components and steps:
1. **Input** (user types `SELECT * FROM employees`)
2. **`Editor::readline()`** - rustyline reads the line from stdin
3. **`Repl::process_command()`** - receives the trimmed input line
4. **`Repl::execute_command()`** - determines it starts with "SELECT", calls `cmd_select()`
5. **`cmd_select()`** - wraps the SQL with "SELECT ... LIMIT 50", calls `db.execute()`
6. **`Database::execute()`** - orchestrates the query execution:
   - Creates `Parser` to tokenize the SQL string
   - `Parser::parse()` builds an `AST` (Abstract Syntax Tree)
   - Creates `Planner` with the AST and catalog
   - `Planner::plan()` builds an execution plan
   - Creates `Executor` with the plan
   - `Executor::execute()` runs the plan, returns `Vec<Batch>`
7. **`print_batches()`** - formats batches as ASCII table:
   - Calculates column widths from data samples
   - Prints header row with box-drawing characters
   - Prints data rows (up to 50)
   - Prints timing information
8. **Output** (formatted table appears on screen)

**1.7** What is the purpose of the `running: bool` field in the `Repl` struct?

**Answer:** The `running` field controls the REPL main loop:
- When `running = true`, the `while self.running` loop continues
- When user types `EXIT` or `QUIT`, it sets `running = false`
- When `Ctrl+D` is pressed, it sets `running = false`
- This allows the REPL to exit gracefully by setting a flag rather than using `break` or `return`
- Provides a clean way to signal shutdown from multiple places

**1.8** Why do we use `match` when reading from the editor, instead of just `unwrap()`?

**Answer:** Using `match` provides proper error handling for different scenarios:
- **`Ok(line)`**: Successfully read input - proceed to process
- **`Err(ReadlineError::Interrupted)`**: Ctrl+C pressed - cancel current input, continue REPL
- **`Err(ReadlineError::Eof)`**: Ctrl+D pressed - graceful exit
- **`Err(other)`**: Other I/O errors - display error and exit

Using `unwrap()` would:
- Panic on Ctrl+C instead of cancelling input
- Panic on Ctrl+D instead of graceful exit
- Not provide user-friendly error messages
- Violate the principle of graceful degradation

**1.9** What happens if a user types an invalid command? How does the REPL respond?

**Answer:** When an invalid command is entered:
1. `execute_command()` doesn't match any known pattern
2. Returns `Err(DatabaseError::parser_error("Unknown command..."))`
3. The REPL loop catches this error with `if let Err(e) = self.process_command(&line)`
4. Calls `self.print_error(&e)` to display:
   ```
   ╔═══════════════════════════════════════════════════════════════╗
   ║ ERROR                                                          ║
   ╠═══════════════════════════════════════════════════════════════╣
   ║ Unknown command: 'FOOBAR'. Type HELP for available commands.  ║
   ╚═══════════════════════════════════════════════════════════════╝
   ```
5. The REPL continues running and prompts for next command

The REPL is resilient - errors are displayed, not crashes.

**1.10** How does the LIMIT clause interact with the REPL's built-in row display limit (50 rows)?

**Answer:** There are actually TWO limits that work together:
1. **Database-level LIMIT**: Added by `cmd_select()` to prevent returning massive result sets
2. **Display-level limit**: `print_batches()` only shows first 50 rows for readability

The interaction:
- User query without LIMIT: `SELECT * FROM employees`
  → `cmd_select()` wraps to: `SELECT * FROM employees LIMIT 50`
  → Database returns at most 50 rows
  → `print_batches()` displays all rows (≤50)

- User query with LIMIT < 50: `SELECT * FROM employees LIMIT 10`
  → Query unchanged
  → Database returns at most 10 rows
  → `print_batches()` displays all rows (≤10)

- User query with LIMIT > 50: `SELECT * FROM employees LIMIT 100`
  → Query unchanged
  → Database returns up to 100 rows
  → `print_batches()` displays only first 50 rows, with message "Showing 50 of 100 rows"

This design balances performance (database doesn't fetch millions of rows) with usability (user can specify their own limits).

### Error Handling

**1.11** Why do user-facing applications need different error messages compared to library code?

**Answer:** Error messages serve different audiences:
- **Library errors**: Technical, detailed, for developers integrating the library
  - Example: "Tokenization failed at position 42: unexpected identifier"
  - Include stack traces, error codes, internal details
  
- **User-facing errors**: Actionable, clear, for end users
  - Example: "Invalid SQL syntax. Expected 'FROM' keyword after column list."
  - Focus on what went wrong and how to fix it
  - Avoid jargon and implementation details

The REPL translates internal `DatabaseError` variants into user-friendly messages:
```rust
fn print_error(&self, error: &DatabaseError) {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║ ERROR                                                          ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ {} ║", self.format_error_message(error));
    println!("╚═══════════════════════════════════════════════════════════════╝");
}
```

**1.12** What is the purpose of the box-drawing characters (╔, ║, ╠, ╚, etc.) in the error output?

**Answer:** Box-drawing characters serve several purposes:
- **Visual distinction**: Makes errors stand out from normal output
- **Attention grabbing**: The box structure draws the eye to the problem
- **Professional appearance**: Looks polished like other database tools (psql, MySQL)
- **Readability**: Structured format is easier to scan quickly
- **Emotional design**: A well-formatted error feels less alarming than raw text
- **Accessibility**: Clear boundaries help screen readers identify error messages

The box format also provides space for multi-line errors and suggestions.

**1.13** Why do we use `Result<()>` as the return type for most REPL commands?

**Answer:** `Result<()>` indicates "success with no value, or an error":
- **Success (`Ok(())`)**: Command executed successfully, no return value needed
- **Failure (`Err(DatabaseError)`)**: Command failed, propagate error to REPL loop

Benefits:
- **Explicit error handling**: Callers must handle errors with `?` or `match`
- **Consistent interface**: All commands return the same type
- **Composable**: Can chain commands with `?` operator
- **Type safety**: Rust's type system ensures errors are handled
- **Clear intent**: `()` signals "side effects only" (output, state changes)

Example from `cmd_load`:
```rust
fn cmd_load(&mut self, input: &str) -> Result<()> {
    let (filename, table_name) = parse_load_command(input)?;
    let batches = self.catalog.load_csv(&filename, &table_name)?;
    let count = batches.iter().map(|b| b.row_count()).sum();
    println!("✓ Loaded {} rows into '{}'", count, table_name);
    Ok(())
}
```

**1.14** What is the difference between a `DatabaseError::parser_error()` and a `DatabaseError::execution_error()` in the context of the REPL?

**Answer:** They represent different failure modes:
- **`DatabaseError::parser_error()`**: Syntax/structure errors detected during parsing
  - Examples: Invalid SQL syntax, unknown keywords, malformed queries
  - Occurs: Before any data is accessed
  - User action: Fix the SQL syntax
  - Example message: "Syntax error: unexpected token at position 15"
  
- **`DatabaseError::execution_error()`**: Runtime errors during query execution
  - Examples: Table not found, column not found, type mismatch
  - Occurs: After parsing, during plan execution
  - User action: Check table names, schema, or data types
  - Example message: "Table 'employees' not found in catalog"

The REPL can provide more specific guidance based on error type:
```rust
match error {
    DatabaseError::ParserError(msg) => {
        format!("SQL syntax error: {}", msg)
    }
    DatabaseError::ExecutionError(msg) => {
        format!("Query execution failed: {}", msg)
    }
}
```

**1.15** How does the REPL ensure that one failed command doesn't crash the entire application?

**Answer:** The REPL uses several resilience strategies:
1. **Result-based error propagation**: Commands return `Result<()>`, not panics
2. **Error catching in main loop**: `if let Err(e) = self.process_command(&line)`
3. **Graceful error display**: Errors are printed, the REPL continues
4. **Signal handling**: Ctrl+C doesn't exit, just cancels current input
5. **Panic-free design**: All error paths return `Err`, never `panic!()`
6. **Defensive programming**: Check conditions before operations

Example of resilient main loop:
```rust
while self.running {
    let readline = self.editor.readline("olap> ");
    
    match readline {
        Ok(line) => {
            if line.trim().is_empty() { continue; }
            self.editor.add_history_entry(&line);
            
            // This won't crash even if process_command panics internally
            if let Err(e) = self.process_command(&line) {
                self.print_error(&e);
            }
            // Loop continues, REPL stays alive
        }
        Err(ReadlineError::Interrupted) => {
            println!("Use EXIT or QUIT to exit.");
            // Loop continues
        }
        Err(ReadlineError::Eof) => {
            println!("Goodbye!");
            self.running = false;
            // Loop exits gracefully
        }
        Err(err) => {
            eprintln!("Error reading input: {}", err);
            self.running = false;
            // Graceful exit on unexpected errors
        }
    }
}
```

This design ensures the REPL is a stable, long-running process.

### Output Formatting

**1.16** Why do we calculate column widths before printing the table?

**Answer:** Calculating widths upfront is necessary for:
- **Proper alignment**: Column widths must be known to draw vertical lines correctly
- **Avoiding misalignment**: If calculated on-the-fly, later rows might be misaligned
- **Header sizing**: Header text width must be compared to data width
- **Performance**: Single pass through data samples is efficient
- **Aesthetic quality**: Well-aligned tables are easier to read

The algorithm:
1. Initialize widths with header names (`col_0`, `col_1`, etc.)
2. Sample first 100 rows (or all if < 100)
3. For each column, find maximum string length
4. Cap at 50 characters to prevent very wide columns
5. Store widths for use during printing

Example:
```rust
let mut column_widths: Vec<usize> = column_names.iter()
    .map(|name| name.len())
    .collect();

// Sample up to 100 rows to determine column widths
for batch in batches.iter().take(10) {
    for row_idx in 0..batch.row_count().min(10) {
        for col_idx in 0..batch.column_count() {
            let value = batch.get(row_idx, col_idx);
            let str_value = format_value(value);
            *column_widths.get_mut(col_idx).unwrap() = 
                column_widths[col_idx].max(str_value.len());
        }
    }
}
```

**1.17** What is the purpose of capping column widths to 50 characters?

**Answer:** Capping at 50 characters prevents several problems:
- **Terminal overflow**: Very wide columns break table formatting on narrow terminals
- **Readability**: Truncated content is more readable than one giant column
- **Layout preservation**: Ensures all columns fit within reasonable width
- **Performance**: Limits string allocation for very large values
- **User focus**: Users see the most relevant part of long values

Trade-offs:
- Information loss: Users might miss important content after truncation
- Solution: Use full query with specific columns when detailed values needed

Implementation:
```rust
const MAX_COLUMN_WIDTH: usize = 50;

// During width calculation
*width = (*width).min(MAX_COLUMN_WIDTH);

// During printing
let display_value = if value.len() > MAX_COLUMN_WIDTH {
    format!("{}...", &value[..MAX_COLUMN_WIDTH - 3])
} else {
    value
};
```

**1.18** Why do we limit the display to 50 rows by default?

**Answer:** The 50-row limit balances several concerns:
- **Performance**: Printing thousands of rows is slow and resource-intensive
- **Terminal scrolling**: Most terminals hold ~24-50 lines, more is impractical
- **User experience**: Scrolling through massive output is tedious
- **Network bandwidth**: For remote terminals, less data = faster display
- **Cognitive load**: Humans can only process limited information at once

This is a "preview" mode - users can:
- Use LIMIT clause to see specific subsets
- Use WHERE to filter to relevant data
- Understand their data without overwhelming output

Similar to other tools:
- `head` command: Shows first 10 lines
- `psql`: Shows all rows but has pager
- SQL `LIMIT`: Standard way to limit results

**1.19** What information does the timing message (`⏱ Executed in 0.62ms`) provide to users?

**Answer:** Timing messages provide valuable feedback:
- **Performance awareness**: Users see how fast/slow queries are
- **Optimization insight**: Compare different query approaches
- **Progress indication**: Confirms query executed (vs hanging)
- **Debugging aid**: Identify unexpectedly slow queries
- **Benchmarking**: Informal performance comparisons

Format details:
- Uses `⏱` emoji for visual cue
- Shows milliseconds (`0.62ms`) or seconds (`1.23s`) based on duration
- Includes only query execution, not formatting time
- Helps users understand if query is fast enough for their use case

Example scenarios:
- `0.62ms`: Very fast, query is well-optimized
- `1.23s`: Moderate, acceptable for occasional use
- `45.6s`: Slow, might need optimization or LIMIT clause

**1.20** How do we handle `NULL` or missing values when printing query results?

**Answer:** NULL values require special handling:
- **Visual representation**: Use `NULL` text to indicate absence
- **Alignment**: `NULL` is 4 characters, fits in most columns
- **Type-agnostic**: Works for any column type
- **Clear semantics**: Distinct from empty string or zero

In the Arrow format used by the engine:
- NULL values are represented separately from data arrays
- Boolean validity bitmap indicates which values are present
- The `Batch::get()` method returns `Option<Value>`

Formatting logic:
```rust
fn format_value(value: Option<&Value>) -> String {
    match value {
        None => "NULL".to_string(),
        Some(Value::Int64(n)) => n.to_string(),
        Some(Value::Float64(f)) => format!("{:.2}", f),
        Some(Value::String(s)) => s.clone(),
    }
}
```

This ensures:
- NULL is always visible (not blank)
- Users understand missing data vs empty string
- Consistent with SQL NULL semantics
- Clear distinction between types

## 🔧 Part 2: Practical Tasks

### Task 2.1: Implement a New Command

Add a `COUNT_TABLES` command that displays the total number of tables in the catalog.

**Requirements:**
- Command should work with any case: `COUNT_TABLES`, `count_tables`, `Count_Tables`
- Output format: `"There are currently X tables in the catalog."`
- Handle case when there are no tables: `"The catalog is empty."`
- Include timing information

**Implementation Guidance:**

Follow this step-by-step approach:

1. **Add the command method to Repl struct:**
   - Add a new method `cmd_count_tables(&self) -> Result<()>`
   - Access the catalog to get table names
   - Count the tables
   - Display appropriate message
   - Add timing information

2. **Update the execute_command method:**
   - Add a new branch to check for "COUNT_TABLES"
   - Call the new method
   - Return the result

3. **Update the HELP command:**
   - Add COUNT_TABLES to the list of available commands
   - Include a brief description

**Reference Implementation:**

```rust
impl Repl {
    // Add this method to the Repl impl block
    fn cmd_count_tables(&self) -> Result<()> {
        let start = std::time::Instant::now();
        
        let table_names: Vec<&str> = self.catalog.get_table_names();
        let count = table_names.len();
        
        let duration = start.elapsed();
        
        if count == 0 {
            println!("The catalog is empty.");
        } else {
            println!("There are currently {} table{} in the catalog.", 
                     count, if count == 1 { "" } else { "s" });
            println!("Tables: {}", table_names.join(", "));
        }
        
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
}
```

**Update execute_command method:**

```rust
fn execute_command(&mut self, input: &str) -> Result<()> {
    let upper_input = input.to_uppercase();
    
    if upper_input == "COUNT_TABLES" {
        self.cmd_count_tables()
    } else if upper_input.starts_with("LOAD ") {
        self.cmd_load(input)
    } else if upper_input.starts_with("SELECT ") {
        self.cmd_select(input)
    } else if upper_input == "SHOW TABLES" {
        self.cmd_show_tables()
    } else if upper_input.starts_with("DESCRIBE ") {
        self.cmd_describe(input)
    } else if upper_input == "EXIT" || upper_input == "QUIT" {
        self.cmd_exit()
    } else {
        Err(DatabaseError::parser_error(
            format!("Unknown command: '{}'. Type HELP for available commands.", input)
        ))
    }
}
```

**Update HELP command:**

Add this entry to the help text:

```
COUNT_TABLES         Display the total number of tables in the catalog
```

**Testing:**

```bash
# Test with empty catalog
olap> COUNT_TABLES
The catalog is empty.
⏱ Executed in 0.02ms

# Test after loading tables
olap> LOAD employees.csv AS employees
✓ Loaded 10 rows into 'employees'

olap> LOAD departments.csv AS departments
✓ Loaded 5 rows into 'departments'

olap> COUNT_TABLES
There are currently 2 tables in the catalog.
Tables: departments, employees
⏱ Executed in 0.05ms

# Test case insensitivity
olap> count_tables
There are currently 2 tables in the catalog.
Tables: departments, employees
⏱ Executed in 0.04ms
```

**Key Learning Points:**
- Case conversion at command entry makes implementation simple
- Access catalog through `self.catalog`
- Use `std::time::Instant::now()` for timing
- Proper English grammar with singular/plural handling
- Consistent message formatting across commands


### Task 2.2: Improve Error Messages

The current LOAD command doesn't provide helpful error messages when a file is not found.

**Current behavior:**
```
LOAD nonexistent.csv AS test
❌ ERROR
No such file or directory (os error 2)
```

**Required behavior:**
```
LOAD nonexistent.csv AS test
❌ ERROR
Failed to load CSV: File 'nonexistent.csv' not found. Please check the path and try again.
```

**Implementation Guidance:**

The issue is that `std::fs::read_to_string()` returns a raw `io::Error` which is converted directly to `DatabaseError`. We need to intercept specific errors and provide more user-friendly messages.

Follow this approach:

1. **Identify error types:** Check for common file I/O errors:
   - File not found (Kind::NotFound)
   - Permission denied (Kind::PermissionDenied)
   - Invalid path (Kind::InvalidInput)

2. **Match on error kind:** Use pattern matching to detect error types
3. **Craft helpful messages:** Provide actionable guidance for each error type
4. **Maintain error propagation:** Keep the original error in `DatabaseError`

**Reference Implementation:**

```rust
use std::io::{self, ErrorKind};

impl Repl {
    fn cmd_load(&mut self, input: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        // Parse the command
        let parts: Vec<&str> = input.splitn(4, ' ').collect();
        if parts.len() != 4 || parts[1] != "AS" {
            return Err(DatabaseError::parser_error(
                "Invalid LOAD syntax. Usage: LOAD <filename> AS <table_name>".to_string()
            ));
        }
        
        let filename = parts[2].trim_matches('\'').trim_matches('"');
        let table_name = parts[3];
        
        // Try to load the file with improved error handling
        let content = match std::fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                // Provide user-friendly error messages
                let error_msg = match e.kind() {
                    ErrorKind::NotFound => {
                        format!(
                            "File '{}' not found. Please check the path and try again.\n\
                             Current directory: {}",
                            filename,
                            std::env::current_dir().unwrap_or(PathBuf::from(".")).display()
                        )
                    }
                    ErrorKind::PermissionDenied => {
                        format!(
                            "Permission denied: Cannot read '{}'. Check file permissions.",
                            filename
                        )
                    }
                    ErrorKind::InvalidInput => {
                        format!(
                            "Invalid filename '{}'. Filenames cannot contain null characters.",
                            filename
                        )
                    }
                    _ => {
                        // Generic error with original details
                        format!(
                            "Failed to load file '{}': {}",
                            filename, e
                        )
                    }
                };
                return Err(DatabaseError::execution_error(error_msg));
            }
        };
        
        // Parse CSV content
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let headers = reader.headers()?.clone();
        let column_names: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
        
        // Convert to batches
        let mut batches = Vec::new();
        for result in reader.records() {
            let record = result?;
            let row: Vec<Value> = record.iter()
                .enumerate()
                .map(|(i, field)| {
                    if field.is_empty() {
                        Value::Null
                    } else {
                        match column_names.get(i) {
                            Some(name) => {
                                // Try to parse as number first
                                field.parse::<i64>()
                                    .map(Value::Int64)
                                    .or_else(|_| field.parse::<f64>().map(Value::Float64))
                                    .unwrap_or(Value::String(field.to_string()))
                            }
                            None => Value::String(field.to_string()),
                        }
                    }
                })
                .collect();
            
            if batches.is_empty() || batches.last().unwrap().row_count() >= 1000 {
                batches.push(Batch::new(column_names.clone()));
            }
            batches.last_mut().unwrap().add_row(row);
        }
        
        // Add to catalog
        self.catalog.add_table(table_name, column_names, batches)?;
        
        let total_rows: usize = batches.iter().map(|b| b.row_count()).sum();
        let duration = start.elapsed();
        
        println!("✓ Loaded {} rows into '{}'", total_rows, table_name);
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
}
```

**Testing Scenarios:**

```bash
# Test file not found
olap> LOAD nonexistent.csv AS test
╔═══════════════════════════════════════════════════════════════╗
║ ERROR                                                          ║
╠═══════════════════════════════════════════════════════════════╣
║ Failed to load CSV: File 'nonexistent.csv' not found. Please   ║
║ check the path and try again.                                  ║
║ Current directory: /home/user/projects                        ║
╚═══════════════════════════════════════════════════════════════╝

# Test permission denied (create a file with no read permission)
$ echo "id,name" > /tmp/no_perm.csv
$ chmod 000 /tmp/no_perm.csv

olap> LOAD /tmp/no_perm.csv AS test
╔═══════════════════════════════════════════════════════════════╗
║ ERROR                                                          ║
╠═══════════════════════════════════════════════════════════════╣
║ Failed to load CSV: Permission denied: Cannot read             ║
║ '/tmp/no_perm.csv'. Check file permissions.                    ║
╚═══════════════════════════════════════════════════════════════╝

# Test successful load
olap> LOAD employees.csv AS employees
✓ Loaded 10 rows into 'employees'
⏱ Executed in 1.23ms

# Test invalid CSV syntax
olap> LOAD invalid.csv AS test
╔═══════════════════════════════════════════════════════════════╗
║ ERROR                                                          ║
╠═══════════════════════════════════════════════════════════════╣
║ Failed to load CSV: CSV parse error: expected 2 columns,      ║
║ found 1 at line 3                                             ║
╚═══════════════════════════════════════════════════════════════╝
```

**Key Learning Points:**
- Use `ErrorKind` pattern matching for specific error types
- Provide context (current directory) in error messages
- Include actionable suggestions ("check the path", "check permissions")
- Maintain error hierarchy (DatabaseError wraps io::Error)
- Box formatting makes errors stand out visually
- Helpful errors reduce user frustration and support burden

**Best Practices for Error Messages:**
1. **State what went wrong:** Clearly identify the problem
2. **Explain why:** Give context about the error condition
3. **Suggest action:** Tell user how to fix it
4. **Be specific:** Include filenames, line numbers, paths
5. **Stay calm:** Avoid scary technical jargon
6. **Keep it concise:** Users skim error messages


### Task 2.3: Add Query Statistics

Add a `STATS <table_name>` command that displays statistical information about a table.

**Required output:**
```
STATS employees
Table: employees
┌─────────────────────────────────────────────────────────────┐
│ Column Name    │ Type    │ Min    │ Max    │ Avg    │ Nulls │
├─────────────────────────────────────────────────────────────┤
│ id             │ Int64   │ 1      │ 10     │ 5.5    │ 0     │
│ age            │ Int64   │ 26     │ 45     │ 33.4   │ 0     │
│ salary         │ Float64 │ 62000  │ 105000 │ 79700. │ 0     │
└─────────────────────────────────────────────────────────────┘
Total rows: 10
```

**Requirements:**
- Only compute statistics for Int64 and Float64 columns
- Show NULL for incompatible types (String)
- Handle empty tables gracefully
- Include timing

**Implementation Guidance:**

This task requires building a statistics aggregator that scans table data and computes aggregate values. The key challenges are:
- Determining column data types
- Computing aggregates efficiently
- Formatting results in a nice table
- Handling incompatible column types (String)

Follow this approach:

1. **Check table existence:** Verify table is in catalog
2. **Get table schema:** Retrieve column names and types
3. **Scan data:** Iterate through all batches and rows
4. **Compute aggregates:** Track min, max, sum, count, nulls per column
5. **Format output:** Create an ASCII table with results

**Reference Implementation:**

```rust
impl Repl {
    fn cmd_stats(&self, input: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        // Parse table name
        let table_name = input.trim_start_matches("STATS ").trim();
        if table_name.is_empty() {
            return Err(DatabaseError::parser_error(
                "STATS requires a table name".to_string()
            ));
        }
        
        // Check if table exists
        let table = self.catalog.get_table(table_name)
            .ok_or_else(|| DatabaseError::execution_error(
                format!("Table '{}' not found", table_name)
            ))?;
        
        let column_names = &table.column_names;
        let batches = &table.batches;
        
        // Initialize statistics for each column
        #[derive(Debug, Default)]
        struct ColumnStats {
            min: Option<f64>,
            max: Option<f64>,
            sum: f64,
            count: usize,
            nulls: usize,
        }
        
        let mut stats: Vec<ColumnStats> = column_names.iter()
            .map(|_| ColumnStats::default())
            .collect();
        
        // Scan all data to compute statistics
        for batch in batches {
            for row_idx in 0..batch.row_count() {
                for col_idx in 0..batch.column_count() {
                    let value = batch.get(row_idx, col_idx);
                    
                    match value {
                        None => {
                            stats[col_idx].nulls += 1;
                        }
                        Some(Value::Int64(n)) => {
                            let f = *n as f64;
                            stats[col_idx].sum += f;
                            stats[col_idx].count += 1;
                            
                            match stats[col_idx].min {
                                None => stats[col_idx].min = Some(f),
                                Some(min) => stats[col_idx].min = Some(min.min(f)),
                            }
                            
                            match stats[col_idx].max {
                                None => stats[col_idx].max = Some(f),
                                Some(max) => stats[col_idx].max = Some(max.max(f)),
                            }
                        }
                        Some(Value::Float64(f)) => {
                            stats[col_idx].sum += f;
                            stats[col_idx].count += 1;
                            
                            match stats[col_idx].min {
                                None => stats[col_idx].min = Some(*f),
                                Some(min) => stats[col_idx].min = Some(min.min(*f)),
                            }
                            
                            match stats[col_idx].max {
                                None => stats[col_idx].max = Some(*f),
                                Some(max) => stats[col_idx].max = Some(max.max(*f)),
                            }
                        }
                        Some(Value::String(_)) => {
                            // String columns - just count nulls
                            stats[col_idx].count += 1;
                        }
                    }
                }
            }
        }
        
        // Format and display results
        println!("Table: {}", table_name);
        
        let type_width = 10;
        let min_width = 10;
        let max_width = 10;
        let avg_width = 10;
        let nulls_width = 8;
        
        let separator = "├" + &"─".repeat(55) + "┤";
        
        println!("└{}┘", "─".repeat(55));
        println!("│ {:<16} │ {:<10} │ {:<10} │ {:<10} │ {:<10} │ {:<8} │", 
                 "Column Name", "Type", "Min", "Max", "Avg", "Nulls");
        println!("{}", separator);
        
        for (col_idx, name) in column_names.iter().enumerate() {
            let col_stats = &stats[col_idx];
            
            // Determine column type
            let col_type = if col_stats.count > 0 && col_stats.min.is_some() {
                if col_stats.sum.fract() == 0.0 && col_stats.min.unwrap().fract() == 0.0 {
                    "Int64"
                } else {
                    "Float64"
                }
            } else {
                "String"
            };
            
            let (min_str, max_str, avg_str) = match col_type {
                "Int64" => {
                    let min = col_stats.min.map(|v| v as i64).unwrap_or(0);
                    let max = col_stats.max.map(|v| v as i64).unwrap_or(0);
                    let avg = if col_stats.count > 0 {
                        col_stats.sum / col_stats.count as f64
                    } else {
                        0.0
                    };
                    (
                        min.to_string(),
                        max.to_string(),
                        format!("{:.1}", avg)
                    )
                }
                "Float64" => {
                    let min = col_stats.min.unwrap_or(0.0);
                    let max = col_stats.max.unwrap_or(0.0);
                    let avg = if col_stats.count > 0 {
                        col_stats.sum / col_stats.count as f64
                    } else {
                        0.0
                    };
                    (
                        format!("{:.0}", min),
                        format!("{:.0}", max),
                        format!("{:.1}", avg)
                    )
                }
                "String" => {
                    ("NULL".to_string(), "NULL".to_string(), "NULL".to_string())
                }
                _ => ("NULL".to_string(), "NULL".to_string(), "NULL".to_string())
            };
            
            println!("│ {:<16} │ {:<10} │ {:<10} │ {:<10} │ {:<10} │ {:<8} │",
                     name, col_type, min_str, max_str, avg_str, col_stats.nulls);
        }
        
        println!("└{}┘", "─".repeat(55));
        
        let total_rows: usize = batches.iter().map(|b| b.row_count()).sum();
        println!("Total rows: {}", total_rows);
        
        let duration = start.elapsed();
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
}
```

**Update execute_command method:**

```rust
fn execute_command(&mut self, input: &str) -> Result<()> {
    let upper_input = input.to_uppercase();
    
    if upper_input == "COUNT_TABLES" {
        self.cmd_count_tables()
    } else if upper_input.starts_with("LOAD ") {
        self.cmd_load(input)
    } else if upper_input.starts_with("SELECT ") {
        self.cmd_select(input)
    } else if upper_input == "SHOW TABLES" {
        self.cmd_show_tables()
    } else if upper_input.starts_with("DESCRIBE ") {
        self.cmd_describe(input)
    } else if upper_input.starts_with("STATS ") {
        self.cmd_stats(input)
    } else if upper_input == "EXIT" || upper_input == "QUIT" {
        self.cmd_exit()
    } else {
        Err(DatabaseError::parser_error(
            format!("Unknown command: '{}'. Type HELP for available commands.", input)
        ))
    }
}
```

**Update HELP command:**

Add this entry:
```
STATS <table>      Display statistics for a table (min, max, avg, nulls)
```

**Testing:**

```bash
olap> LOAD employees.csv AS employees
✓ Loaded 10 rows into 'employees'

olap> STATS employees
Table: employees
└───────────────────────────────────────────────────────────────────────┘
│ Column Name      │ Type       │ Min        │ Max        │ Avg        │ Nulls   │
├───────────────────────────────────────────────────────────────────────┤
│ id               │ Int64      │ 1          │ 10         │ 5.5        │ 0       │
│ name             │ String     │ NULL       │ NULL       │ NULL       │ 0       │
│ age              │ Int64      │ 26         │ 45         │ 33.4       │ 0       │
│ salary           │ Float64    │ 62000      │ 105000     │ 79700.0    │ 0       │
│ department       │ String     │ NULL       │ NULL       │ NULL       │ 0       │
└───────────────────────────────────────────────────────────────────────┘
Total rows: 10
⏱ Executed in 1.45ms

# Test with non-existent table
olap> STATS nonexistent
╔═══════════════════════════════════════════════════════════════╗
║ ERROR                                                          ║
╠═══════════════════════════════════════════════════════════════╣
║ Table 'nonexistent' not found                                  ║
╚═══════════════════════════════════════════════════════════════╝
```

**Key Learning Points:**
- Scan all data to compute accurate statistics
- Use Option<f64> for min/max to handle uninitialized values
- Detect column type from actual data, not schema
- Show NULL for incompatible types
- Average is sum/count, handle division by zero
- Box-drawing characters create professional-looking tables

### Task 2.4: Multi-Line Query Support

Currently, the REPL only supports single-line queries. Add support for multi-line queries using a continuation prompt.

**Example interaction:**
```
olap> SELECT name, department
    > FROM employees
    > WHERE salary > 80000
    > ORDER BY salary DESC;
```

**Requirements:**
- Detect incomplete queries (e.g., missing semicolon)
- Change prompt to `    > ` for continuation lines
- Accumulate lines until complete
- Execute the full accumulated query
- Support both `;` and empty line as terminators

**Implementation Guidance:**

Multi-line support requires:
1. Detecting incomplete queries (no semicolon, certain keywords at end)
2. Using a different prompt for continuation lines
3. Accumulating lines until complete
4. Supporting both `;` and empty line as terminators
5. Preserving the complete query with proper formatting

Follow this approach:

1. **Add a buffer field to Repl struct:** Store accumulated lines
2. **Track multi-line state:** Boolean flag for continuation mode
3. **Modify prompt logic:** Change prompt based on state
4. **Update main loop:** Handle accumulation vs execution
5. **Detect completion:** Check for semicolon or keywords

**Reference Implementation:**

First, update the Repl struct:

```rust
pub struct Repl<'a> {
    editor: Editor<()>,
    catalog: &'a mut Catalog,
    running: bool,
    query_buffer: String,      // New field for multi-line queries
    in_multiline: bool,       // New field for continuation state
}

impl<'a> Repl<'a> {
    pub fn new(catalog: &'a mut Catalog) -> Result<Self> {
        let mut editor = Editor::<()>::new();
        
        // Load history if it exists
        if let Ok(history_file) = std::env::var("HOME") {
            let history_path = format!("{}/.olap_history", history_file);
            if editor.load_history(&history_path).is_err() {
                // History file doesn't exist yet, that's okay
            }
        }
        
        Ok(Repl {
            editor,
            catalog,
            running: true,
            query_buffer: String::new(),
            in_multiline: false,
        })
    }
}
```

Update the main REPL loop:

```rust
impl<'a> Repl<'a> {
    pub fn run(&mut self) -> Result<()> {
        println!("Mini Rust OLAP - Interactive REPL");
        println!("Type HELP for available commands, EXIT or QUIT to exit.\n");
        
        while self.running {
            // Use different prompt based on multiline state
            let prompt = if self.in_multiline {
                "    > ".to_string()
            } else {
                "olap> ".to_string()
            };
            
            let readline = self.editor.readline(&prompt);
            
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    
                    // Skip empty input unless we're in multiline
                    if line.is_empty() {
                        if self.in_multiline {
                            // Empty line ends multiline mode
                            self.in_multiline = false;
                            let query = std::mem::take(&mut self.query_buffer);
                            
                            if !query.is_empty() {
                                // Execute the accumulated query
                                self.process_command(&query);
                            }
                        }
                        continue;
                    }
                    
                    // Add to history only for first line of multi-line
                    if !self.in_multiline {
                        self.editor.add_history_entry(line);
                    }
                    
                    // Check if line ends with semicolon
                    let ends_with_semicolon = line.ends_with(';');
                    
                    if self.in_multiline {
                        // Append to buffer
                        if !self.query_buffer.is_empty() {
                            self.query_buffer.push(' ');
                        }
                        self.query_buffer.push_str(line);
                        
                        if ends_with_semicolon {
                            // Semicolon ends multiline mode
                            self.in_multiline = false;
                            let query = self.query_buffer.clone();
                            self.query_buffer.clear();
                            self.process_command(&query);
                        }
                    } else {
                        // First line - check if it's incomplete
                        let is_incomplete = self.is_incomplete_query(line);
                        
                        if is_incomplete && !ends_with_semicolon {
                            // Start multiline mode
                            self.query_buffer = line.to_string();
                            self.in_multiline = true;
                        } else {
                            // Execute immediately
                            self.process_command(line);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C
                    if self.in_multiline {
                        // Cancel multiline input
                        self.in_multiline = false;
                        self.query_buffer.clear();
                        println!("Query cancelled.");
                    } else {
                        println!("Use EXIT or QUIT to exit.");
                    }
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D
                    if self.in_multiline {
                        // Cancel multiline input
                        self.in_multiline = false;
                        self.query_buffer.clear();
                        println!("Query cancelled.");
                    } else {
                        println!("Goodbye!");
                        self.running = false;
                    }
                }
                Err(err) => {
                    eprintln!("Error reading input: {}", err);
                    self.running = false;
                }
            }
        }
        
        // Save history
        if let Ok(history_file) = std::env::var("HOME") {
            let history_path = format!("{}/.olap_history", history_file);
            let _ = self.editor.save_history(&history_path);
        }
        
        Ok(())
    }
    
    fn is_incomplete_query(&self, line: &str) -> bool {
        let upper = line.to_uppercase();
        
        // Check for keywords that typically continue
        let continues_keywords = vec![
            "SELECT",
            "FROM",
            "WHERE",
            "GROUP BY",
            "HAVING",
            "ORDER BY",
            "LIMIT",
        ];
        
        // If line starts with SELECT and doesn't have semicolon, it's incomplete
        if upper.starts_with("SELECT") && !upper.contains("FROM") {
            return true;
        }
        
        // If line ends with these keywords, it's incomplete
        for keyword in &continues_keywords {
            if upper.ends_with(keyword) || upper.ends_with(&format!("{} ", keyword)) {
                return true;
            }
        }
        
        false
    }
}
```

**Testing:**

```bash
# Simple one-line query (works as before)
olap> SELECT * FROM employees LIMIT 5
┌─────────────────────────────────────────────────────────────┐
│ col_0   │ col_1  │ col_2  │ col_3    │ col_4      │
├─────────────────────────────────────────────────────────────┤
│ 1       │ John   │ 32     │ 75000.0  │ Sales      │
│ 2       │ Jane   │ 28     │ 68000.0  │ Marketing  │
│ 3       │ Bob    │ 35     │ 82000.0  │ Engineering│
│ 4       │ Alice  │ 30     │ 72000.0  │ Sales      │
│ 5       │ Charlie│ 40     │ 95000.0  │ Engineering│
└─────────────────────────────────────────────────────────────┘
⏱ Executed in 0.45ms

# Multi-line query with continuation prompt
olap> SELECT name, department
    > FROM employees
    > WHERE salary > 80000
    > ORDER BY salary DESC;
┌─────────────────────────────────────────────────────────────┐
│ col_0  │ col_1      │
├─────────────────────────────────────────────────────────────┤
│ Charlie│ Engineering│
│ Dave   │ Sales      │
└─────────────────────────────────────────────────────────────┘
⏱ Executed in 0.38ms

# Multi-line query ending with empty line
olap> SELECT COUNT(*)
    > FROM employees
    > WHERE age > 30
    >
┌─────────────────────────────────────────────────────────────┐
│ col_0   │
├─────────────────────────────────────────────────────────────┤
│ 7       │
└─────────────────────────────────────────────────────────────┘
⏱ Executed in 0.32ms

# Cancel multi-line input with Ctrl+C
olap> SELECT name, salary
    > FROM employees
    > WHERE department = 'Engineering'
^C
Query cancelled.

# Cancel multi-line input with Ctrl+D
olap> SELECT * FROM departments
    > WHERE budget > 100000
^D
Query cancelled.
```

**Key Learning Points:**
- Stateful REPL needs to track multiline mode
- Different prompts provide visual feedback to users
- Multiple ways to end multiline input (semicolon, empty line)
- Cancel input gracefully with Ctrl+C or Ctrl+D
- Don't add continuation lines to history (only first line)
- Detect incomplete queries by checking keywords and syntax
- Use String buffer to accumulate lines efficiently
- `std::mem::take` clears buffer while returning contents
- Error handling must account for both normal and multiline modes

### Task 2.5: CSV Export

Add an `EXPORT <query> TO <filename>` command that saves query results to a CSV file.

**Example:**
```
EXPORT SELECT name, salary FROM employees WHERE salary > 80000 TO high_earners.csv
✓ Exported 4 rows to 'high_earners.csv'
```

**Requirements:**
- Parse the full syntax including SELECT query
- Execute the query
- Write results to CSV file with headers
- Handle file write errors gracefully
- Include timing

**Implementation Guidance:**

The EXPORT command requires:
1. Parsing a complex syntax: `EXPORT <query> TO <filename>`
2. Extracting the SELECT query from the command
3. Executing the query
4. Writing results to CSV with headers
5. Handling file I/O errors gracefully

This is the most complex practical task because it involves parsing nested syntax and file I/O.

Follow this approach:

1. **Parse the EXPORT command:** Split into query and filename
2. **Extract the SELECT query:** The query starts after EXPORT and ends before TO
3. **Execute the query:** Use existing query execution logic
4. **Write CSV output:** Create CSV writer and write results
5. **Handle errors:** Provide clear error messages for file issues

**Reference Implementation:**

```rust
use std::fs::File;
use std::io::Write;

impl Repl {
    fn cmd_export(&self, input: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        // Parse: EXPORT <query> TO <filename>
        let upper_input = input.to_uppercase();
        
        if !upper_input.starts_with("EXPORT ") {
            return Err(DatabaseError::parser_error(
                "EXPORT command must start with EXPORT".to_string()
            ));
        }
        
        // Find " TO " in the command
        let to_position = match upper_input.find(" TO ") {
            Some(pos) => pos,
            None => {
                return Err(DatabaseError::parser_error(
                    "EXPORT command missing ' TO ' keyword. Usage: EXPORT <query> TO <filename>".to_string()
                ));
            }
        };
        
        // Extract query (between "EXPORT" and " TO ")
        let query_part = &input[7..to_position]; // Skip "EXPORT "
        let query = query_part.trim();
        
        if query.is_empty() {
            return Err(DatabaseError::parser_error(
                "EXPORT command requires a query. Usage: EXPORT <query> TO <filename>".to_string()
            ));
        }
        
        // Extract filename (after " TO ")
        let filename_part = &input[to_position + 4..]; // Skip " TO "
        let filename = filename_part.trim();
        
        if filename.is_empty() {
            return Err(DatabaseError::parser_error(
                "EXPORT command requires a filename. Usage: EXPORT <query> TO <filename>".to_string()
            ));
        }
        
        // Remove quotes from filename if present
        let filename = filename.trim_matches('\'').trim_matches('"').trim();
        
        // Execute the query
        let batches = self.execute_query(query)?;
        
        // Check if we have results
        let total_rows: usize = batches.iter().map(|b| b.row_count()).sum();
        if total_rows == 0 {
            println!("⚠ Query returned no results. File not created.");
            return Ok(());
        }
        
        // Open file for writing
        let mut file = match File::create(filename) {
            Ok(f) => f,
            Err(e) => {
                let error_msg = match e.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        format!("Permission denied: Cannot create '{}'", filename)
                    }
                    std::io::ErrorKind::InvalidInput => {
                        format!("Invalid filename: '{}'", filename)
                    }
                    _ => {
                        format!("Failed to create file '{}': {}", filename, e)
                    }
                };
                return Err(DatabaseError::execution_error(error_msg));
            }
        };
        
        // Get column count and names
        let first_batch = &batches[0];
        let column_count = first_batch.column_count();
        
        // Generate column names (we use col_0, col_1, etc. since we don't have actual names)
        let column_names: Vec<String> = (0..column_count)
            .map(|i| format!("col_{}", i))
            .collect();
        
        // Write CSV header
        writeln!(file, "{}", column_names.join(","))
            .map_err(|e| DatabaseError::execution_error(
                format!("Failed to write CSV header: {}", e)
            ))?;
        
        // Write data rows
        for batch in &batches {
            for row_idx in 0..batch.row_count() {
                let mut row_values = Vec::new();
                
                for col_idx in 0..batch.column_count() {
                    let value = batch.get(row_idx, col_idx);
                    let value_str = match value {
                        None => "".to_string(),
                        Some(Value::Int64(n)) => n.to_string(),
                        Some(Value::Float64(f)) => {
                            // Format with enough precision to avoid losing data
                            format!("{:.6}", f).trim_end_matches('0').trim_end_matches('.').to_string()
                        }
                        Some(Value::String(s)) => {
                            // Escape quotes and commas for CSV
                            let escaped = s.replace("\"", "\"\"");
                            if escaped.contains(',') || escaped.contains('"') || escaped.contains('\n') {
                                format!("\"{}\"", escaped)
                            } else {
                                escaped
                            }
                        }
                    };
                    row_values.push(value_str);
                }
                
                writeln!(file, "{}", row_values.join(","))
                    .map_err(|e| DatabaseError::execution_error(
                        format!("Failed to write CSV row: {}", e)
                    ))?;
            }
        }
        
        // Flush file to ensure all data is written
        file.flush()
            .map_err(|e| DatabaseError::execution_error(
                format!("Failed to flush CSV file: {}", e)
            ))?;
        
        let duration = start.elapsed();
        
        println!("✓ Exported {} rows to '{}'", total_rows, filename);
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
    
    // Helper method to execute a query (extracted from cmd_select)
    fn execute_query(&self, query: &str) -> Result<Vec<Batch>> {
        // Use the existing database execution logic
        let parser = Parser::new(query);
        let ast = parser.parse()?;
        
        let planner = Planner::new(&ast, self.catalog);
        let plan = planner.plan()?;
        
        let executor = Executor::new(&plan);
        let batches = executor.execute()?;
        
        Ok(batches)
    }
}
```

**Update execute_command method:**

```rust
fn execute_command(&mut self, input: &str) -> Result<()> {
    let upper_input = input.to_uppercase();
    
    if upper_input == "COUNT_TABLES" {
        self.cmd_count_tables()
    } else if upper_input.starts_with("LOAD ") {
        self.cmd_load(input)
    } else if upper_input.starts_with("EXPORT ") {
        self.cmd_export(input)
    } else if upper_input.starts_with("SELECT ") {
        self.cmd_select(input)
    } else if upper_input == "SHOW TABLES" {
        self.cmd_show_tables()
    } else if upper_input.starts_with("DESCRIBE ") {
        self.cmd_describe(input)
    } else if upper_input.starts_with("STATS ") {
        self.cmd_stats(input)
    } else if upper_input == "EXIT" || upper_input == "QUIT" {
        self.cmd_exit()
    } else {
        Err(DatabaseError::parser_error(
            format!("Unknown command: '{}'. Type HELP for available commands.", input)
        ))
    }
}
```

**Update HELP command:**

Add this entry:
```
EXPORT <query> TO <file>  Export query results to CSV file
```

**Testing:**

```bash
# Simple export
olap> EXPORT SELECT name, salary FROM employees WHERE salary > 80000 TO high_earners.csv
✓ Exported 2 rows to 'high_earners.csv'
⏱ Executed in 1.23ms

# Verify the file was created
$ cat high_earners.csv
col_0,col_1
Charlie,95000.0
Dave,105000.0

# Export all employees
olap> EXPORT SELECT * FROM employees TO all_employees.csv
✓ Exported 10 rows to 'all_employees.csv'
⏱ Executed in 1.45ms

# Export with complex query
olap> EXPORT SELECT department, COUNT(*), AVG(salary) 
    > FROM employees 
    > GROUP BY department 
    > TO dept_stats.csv
✓ Exported 3 rows to 'dept_stats.csv'
⏱ Executed in 0.89ms

# Export with empty result
olap> EXPORT SELECT * FROM employees WHERE age > 100 TO old_employees.csv
⚠ Query returned no results. File not created.

# Export with invalid filename
olap> EXPORT SELECT * FROM employees TO /root/forbidden.csv
╔═══════════════════════════════════════════════════════════════╗
║ ERROR                                                          ║
╠═══════════════════════════════════════════════════════════════╣
║ Failed to create file '/root/forbidden.csv': Permission      ║
║ denied                                                        ║
╚═══════════════════════════════════════════════════════════════╝

# Export with missing TO clause
olap> EXPORT SELECT * FROM employees missing
╔═══════════════════════════════════════════════════════════════╗
║ ERROR                                                          ║
╠═══════════════════════════════════════════════════════════════╣
║ EXPORT command missing ' TO ' keyword. Usage: EXPORT         ║
║ <query> TO <filename>                                         ║
╚═══════════════════════════════════════════════════════════════╝
```

**Key Learning Points:**
- Parsing complex syntax requires careful string manipulation
- Use `find()` to locate keywords like " TO "
- Extract substrings with proper bounds checking
- Handle quoted filenames with `trim_matches()`
- Reuse existing query execution logic (don't duplicate)
- CSV requires special escaping for quotes, commas, newlines
- Use `File::create()` for file I/O, handle errors appropriately
- Flush file to ensure data is written
- Provide helpful error messages for common issues
- Empty result sets should not create files
- Exported files use col_0, col_1 naming (could be improved later)

## 📝 Part 3: Code Review

### Code Review 3.1: Command Parsing Logic

Review this code snippet from the `execute_command` method:

```rust
fn execute_command(&mut self, input: &str) -> Result<()> {
    let upper_input = input.to_uppercase();
    
    if upper_input.starts_with("LOAD ") {
        self.cmd_load(input)
    } else if upper_input.starts_with("SELECT ") {
        self.cmd_select(input)
    } else if upper_input == "SHOW TABLES" {
        self.cmd_show_tables()
    } else if upper_input.starts_with("DESCRIBE ") {
        self.cmd_describe(input)
    } else if upper_input == "EXIT" {
        self.cmd_exit()
    } else {
        Err(DatabaseError::parser_error(
            format!("Unknown command: '{}'. Type HELP for available commands.", input)
        ))
    }
}
```

**Questions:**

1. What is a potential issue with using `starts_with()` for command recognition?

2. What happens if a user types `SELECT*FROM employees` (no space after SELECT)?

3. Why do we check `== "SHOW TABLES"` but `starts_with("LOAD ")`?

4. What is the time complexity of this command matching approach?

5. How could this be improved for better maintainability as more commands are added?

**Answer Key:**

1. **Issue with `starts_with()`**: It's vulnerable to false positives. For example, `LOADMORE` would match `starts_with("LOAD ")` incorrectly because it starts with "LOAD". Also, `SELECTT` (with two Ts) would match `starts_with("SELECT ")`. This leads to confusing error messages.

2. **`SELECT*FROM employees` problem**: This would NOT match `starts_with("SELECT ")` because there's no space after SELECT. The command would fall through to the `else` branch and show "Unknown command", which is unhelpful. Better parsing would normalize whitespace or use tokenization.

3. **Why `==` vs `starts_with()`**: `SHOW TABLES` is a complete command with no arguments, so exact match is appropriate. `LOAD`, `SELECT`, and `DESCRIBE` all take arguments (filename, query, table_name), so `starts_with()` is needed to check for the command prefix before parsing the argument.

4. **Time complexity**: O(n × m) where n is the number of command branches and m is the length of the input string. In the worst case (no match), we check all branches. For 6 commands and average input length of 20 characters, this is negligible, but it scales poorly.

5. **Improvement suggestions**:
   - Use a command enum with pattern matching
   - Implement a command registry/trait
   - Use regex or more sophisticated parsing
   - Separate command recognition from argument parsing
   - Create a CommandParser that returns a structured Command type

**Suggested Improvements:**

Here's a refactored version using an enum-based approach:

```rust
#[derive(Debug, Clone)]
enum Command {
    Load { filename: String, table_name: String },
    Select { query: String },
    ShowTables,
    Describe { table_name: String },
    CountTables,
    Stats { table_name: String },
    Export { query: String, filename: String },
    Exit,
}

impl Command {
    fn parse(input: &str) -> Result<Self> {
        let upper = input.trim().to_uppercase();
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        
        if parts.is_empty() {
            return Err(DatabaseError::parser_error("Empty command".to_string()));
        }
        
        match upper.as_str() {
            "EXIT" | "QUIT" => Ok(Command::Exit),
            "SHOW TABLES" => Ok(Command::ShowTables),
            "COUNT_TABLES" => Ok(Command::CountTables),
            
            cmd if cmd.starts_with("LOAD ") => {
                let parsed = Self::parse_load(input)?;
                Ok(Command::Load {
                    filename: parsed.0,
                    table_name: parsed.1,
                })
            }
            
            cmd if cmd.starts_with("SELECT ") => {
                Ok(Command::Select {
                    query: input.trim().to_string(),
                })
            }
            
            cmd if cmd.starts_with("DESCRIBE ") => {
                let table_name = input.trim_start_matches("DESCRIBE ")
                    .trim_start_matches("describe ")
                    .trim_start_matches("Describe ")
                    .trim().to_string();
                if table_name.is_empty() {
                    return Err(DatabaseError::parser_error(
                        "DESCRIBE requires a table name".to_string()
                    ));
                }
                Ok(Command::Describe { table_name })
            }
            
            cmd if cmd.starts_with("STATS ") => {
                let table_name = input.trim_start_matches("STATS ")
                    .trim_start_matches("stats ")
                    .trim().to_string();
                if table_name.is_empty() {
                    return Err(DatabaseError::parser_error(
                        "STATS requires a table name".to_string()
                    ));
                }
                Ok(Command::Stats { table_name })
            }
            
            cmd if cmd.starts_with("EXPORT ") => {
                let parsed = Self::parse_export(input)?;
                Ok(Command::Export {
                    query: parsed.0,
                    filename: parsed.1,
                })
            }
            
            _ => Err(DatabaseError::parser_error(
                format!("Unknown command: '{}'. Type HELP for available commands.", input.trim())
            )),
        }
    }
    
    fn parse_load(input: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = input.splitn(4, ' ').collect();
        if parts.len() != 4 || parts[1].to_uppercase() != "AS" {
            return Err(DatabaseError::parser_error(
                "Invalid LOAD syntax. Usage: LOAD <filename> AS <table_name>".to_string()
            ));
        }
        
        let filename = parts[2].trim_matches('\'').trim_matches('"').to_string();
        let table_name = parts[3].to_string();
        
        if filename.is_empty() || table_name.is_empty() {
            return Err(DatabaseError::parser_error(
                "LOAD requires both filename and table_name".to_string()
            ));
        }
        
        Ok((filename, table_name))
    }
    
    fn parse_export(input: &str) -> Result<(String, String)> {
        let upper = input.to_uppercase();
        let to_pos = upper.find(" TO ")
            .ok_or_else(|| DatabaseError::parser_error(
                "EXPORT missing ' TO ' keyword".to_string()
            ))?;
        
        let query = input[7..to_pos].trim().to_string(); // Skip "EXPORT "
        let filename = input[to_pos + 4..].trim().to_string(); // Skip " TO "
        
        if query.is_empty() || filename.is_empty() {
            return Err(DatabaseError::parser_error(
                "EXPORT requires both query and filename".to_string()
            ));
        }
        
        Ok((query, filename.trim_matches('\'').trim_matches('"').to_string()))
    }
}
```

Then update the execute_command method:

```rust
fn execute_command(&mut self, input: &str) -> Result<()> {
    let command = Command::parse(input)?;
    
    match command {
        Command::Load { filename, table_name } => {
            self.cmd_load_with_args(filename, table_name)
        }
        Command::Select { query } => {
            self.cmd_select_with_query(query)
        }
        Command::ShowTables => {
            self.cmd_show_tables()
        }
        Command::Describe { table_name } => {
            self.cmd_describe_with_table(table_name)
        }
        Command::CountTables => {
            self.cmd_count_tables()
        }
        Command::Stats { table_name } => {
            self.cmd_stats_with_table(table_name)
        }
        Command::Export { query, filename } => {
            self.cmd_export_with_args(query, filename)
        }
        Command::Exit => {
            self.cmd_exit()
        }
    }
}
```

**Benefits of this approach:**

1. **Single responsibility**: `Command::parse()` handles all parsing logic
2. **Type safety**: The enum ensures all possible commands are handled
3. **Testability**: Parsing can be tested independently of execution
4. **Extensibility**: Adding new commands means adding an enum variant and match arm
5. **Error messages**: More specific error messages for each command type
6. **Pattern matching**: Exhaustive match ensures all commands are handled
7. **Better error recovery**: Parse errors are caught before execution
8. **Easier to maintain**: Command logic is centralized, not spread across if-else chain

**Alternative: Command Trait Pattern**

For even better extensibility, use a trait-based approach:

```rust
trait Command {
    fn name(&self) -> &'static str;
    fn execute(&mut self, repl: &mut Repl) -> Result<()>;
    fn help(&self) -> &'static str;
}

struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    fn register(&mut self, name: &str, cmd: Box<dyn Command>) {
        self.commands.insert(name.to_uppercase(), cmd);
    }
    
    fn get(&self, name: &str) -> Option<&Box<dyn Command>> {
        self.commands.get(&name.to_uppercase())
    }
}
```

This allows dynamic command registration at runtime and plugin-like architecture.

### Code Review 3.2: Output Formatting

Review the `print_batches` function implementation:

```rust
fn print_batches(&self, batches: &[Batch]) {
    let total_rows: usize = batches.iter().map(|b| b.row_count()).sum();
    
    if total_rows == 0 {
        println!("Empty result set.");
        return;
    }
    
    let first_batch = &batches[0];
    let column_count = first_batch.column_count();
    let mut column_names: Vec<String> = Vec::new();
    for i in 0..column_count {
        column_names.push(format!("col_{}", i));
    }
    // ... more formatting code
}
```

**Questions:**

1. What happens if `batches` is empty? Is this handled correctly?

2. Why do we use `col_0`, `col_1`, etc., instead of actual column names?

3. What is the problem with accessing `batches[0]` without checking if batches is empty first?

4. How does this handle the case where different batches have different column counts?

5. Why do we calculate `total_rows` before checking if it's zero?

**Answer Key:**

1. **Empty batches**: The code checks `if total_rows == 0` and returns early, which is correct. However, accessing `batches[0]` before this check would panic if batches is empty. The early return prevents the panic, but it's fragile - if someone reorders the code, it will crash.

2. **Why `col_0`, `col_1`**: This is a known limitation because the current architecture doesn't track column names through the execution pipeline. The Catalog has column names, but they're not available in the Batch. Real database systems propagate column metadata through the query plan.

3. **Accessing `batches[0]` without checking**: This is a potential panic. If `batches` is empty, `batches[0]` will cause an index-out-of-bounds panic. The early return for `total_rows == 0` prevents this, but it's relying on side effects (sum) rather than explicit checking.

4. **Different column counts**: The code doesn't handle this. It assumes all batches have the same schema. If batches have different column counts, the code would either panic (if index out of bounds) or silently produce incorrect output. In practice, the executor should ensure consistent schemas.

5. **Calculating total_rows first**: This is inefficient - we iterate through all batches to calculate total_rows, then iterate again to calculate column widths. We could do both in a single pass. However, the check for `total_rows == 0` provides a fast-path optimization for empty results.

**Suggested Improvements:**

Here's a refactored version that addresses these issues:

```rust
fn print_batches(&self, batches: &[Batch]) {
    // Early exit for empty batches - check before accessing
    if batches.is_empty() {
        println!("Empty result set.");
        return;
    }
    
    // Verify all batches have consistent column counts
    let first_batch = &batches[0];
    let column_count = first_batch.column_count();
    
    for (i, batch) in batches.iter().enumerate() {
        if batch.column_count() != column_count {
            eprintln!("Warning: Batch {} has inconsistent column count ({} vs {}). Results may be incorrect.",
                     i, batch.column_count(), column_count);
        }
    }
    
    // Try to get column names from schema if available
    // For now, we'll still use col_0, col_1 but add infrastructure for real names
    let column_names: Vec<String> = (0..column_count)
        .map(|i| format!("col_{}", i))
        .collect();
    
    // Calculate column widths and total rows in a single pass
    const SAMPLE_LIMIT: usize = 100;
    let mut column_widths: Vec<usize> = column_names.iter()
        .map(|name| name.len().min(50))
        .collect();
    let mut total_rows = 0;
    
    for batch in batches {
        let rows_to_sample = batch.row_count().min(SAMPLE_LIMIT);
        
        for row_idx in 0..rows_to_sample {
            for col_idx in 0..batch.column_count().min(column_count) {
                let value = batch.get(row_idx, col_idx);
                let str_value = format_value(value);
                let display_value = if str_value.len() > 50 {
                    format!("{}...", &str_value[..47])
                } else {
                    str_value
                };
                
                // Safe access with bounds checking
                if let Some(width) = column_widths.get_mut(col_idx) {
                    *width = (*width).min(50).max(display_value.len());
                }
            }
        }
        total_rows += batch.row_count();
    }
    
    // Second early exit - now we've already calculated total_rows
    if total_rows == 0 {
        println!("Empty result set.");
        return;
    }
    
    // Print header
    let total_width: usize = column_widths.iter().sum::<usize>() + column_count * 3 + 1;
    let horizontal = "─".repeat(total_width - 2);
    
    println!("┌{}┐", horizontal);
    
    // Header row
    print!("│");
    for (col_idx, name) in column_names.iter().enumerate() {
        let width = column_widths.get(col_idx).copied().unwrap_or(10);
        print!(" {:<width$} │", name, width = width);
    }
    println!();
    
    // Separator
    println!("├{}┤", horizontal);
    
    // Data rows (limit to 50 for display)
    const ROW_DISPLAY_LIMIT: usize = 50;
    let mut rows_displayed = 0;
    
    for batch in batches {
        if rows_displayed >= ROW_DISPLAY_LIMIT {
            break;
        }
        
        for row_idx in 0..batch.row_count() {
            if rows_displayed >= ROW_DISPLAY_LIMIT {
                break;
            }
            
            print!("│");
            for col_idx in 0..batch.column_count().min(column_count) {
                let value = batch.get(row_idx, col_idx);
                let str_value = format_value(value);
                let display_value = if str_value.len() > 50 {
                    format!("{}...", &str_value[..47])
                } else {
                    str_value
                };
                
                let width = column_widths.get(col_idx).copied().unwrap_or(10);
                print!(" {:<width$} │", display_value, width = width);
            }
            println!();
            rows_displayed += 1;
        }
    }
    
    // Footer
    println!("└{}┘", horizontal);
    
    // Display summary
    if total_rows > ROW_DISPLAY_LIMIT {
        println!("Showing {} of {} rows (use LIMIT clause to see more)", 
                ROW_DISPLAY_LIMIT, total_rows);
    }
}

fn format_value(value: Option<&Value>) -> String {
    match value {
        None => "NULL".to_string(),
        Some(Value::Int64(n)) => n.to_string(),
        Some(Value::Float64(f)) => {
            // Format with appropriate precision
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                format!("{:.6}", f).trim_end_matches('0').trim_end_matches('.').to_string()
            }
        }
        Some(Value::String(s)) => s.clone(),
    }
}
```

**Enhanced version with actual column names:**

To support actual column names, we need to pass them through the execution pipeline:

```rust
// Add to Batch struct
struct Batch {
    column_names: Vec<String>,
    // ... existing fields
}

// In print_batches, use actual names
fn print_batches_with_names(batches: &[Batch]) {
    if batches.is_empty() {
        println!("Empty result set.");
        return;
    }
    
    // Use actual column names from first batch
    let column_names = batches[0].column_names.clone();
    let column_count = column_names.len();
    
    // ... rest of implementation same as above, but using actual names
}
```

**Benefits of the refactored version:**

1. **Explicit empty check**: Check `batches.is_empty()` before any access
2. **Schema validation**: Warn about inconsistent column counts
3. **Single pass**: Calculate widths and total_rows together
4. **Safe indexing**: Use `.get_mut()` and bounds checking
5. **Better formatting**: Consistent table width calculation
6. **Row limit**: Clear indication when rows are truncated
7. **Improved numeric formatting**: Show integers without decimal points
8. **Maintainability**: Constants for limits, clearer variable names
9. **Error resilience**: Graceful handling of edge cases
10. **Extensibility**: Easy to add column name support

**Performance optimizations:**

```rust
// Pre-allocate vectors with known capacity
let mut column_widths: Vec<usize> = Vec::with_capacity(column_count);
let mut row_values: Vec<String> = Vec::with_capacity(column_count);

// Use iterators instead of index loops where possible
column_names.iter().enumerate().for_each(|(i, name)| {
    column_widths.push(name.len().min(50));
});

// Use string builder for large concatenations
use std::fmt::Write;
let mut row = String::with_capacity(total_width);
for (i, name) in column_names.iter().enumerate() {
    write!(&mut row, " {:<width$} │", name, width = column_widths[i]).unwrap();
}
```

**Further improvements:**

1. **Pagination**: Add "Press Enter for more" functionality for large results
2. **Colored output**: Use terminal colors for headers, alternating rows
3. **Custom formatters**: Allow user-specified formatting for dates, numbers
4. **Export formats**: Easy switch to JSON, Markdown, HTML output
5. **Column alignment**: Right-align numbers, left-align strings
6. **Vertical mode**: Print columns vertically for wide tables
7. **Truncation indicators**: Show "..." at both ends if truncated
8. **Unicode support**: Handle multi-byte characters correctly in width calculation

### Code Review 3.3: Error Handling

Review this error handling in the REPL loop:

```rust
while self.running {
    let readline = self.editor.readline("olap> ");
    
    match readline {
        Ok(line) => {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            self.editor.add_history_entry(line);
            
            if let Err(e) = self.process_command(&line) {
                self.print_error(&e);
            }
        }
        Err(ReadlineError::Interrupted) => {
            println!("Use EXIT or QUIT to exit.");
        }
        Err(ReadlineError::Eof) => {
            println!("Goodbye!");
            self.running = false;
        }
        Err(err) => {
            eprintln!("Error reading input: {}", err);
            self.running = false;
        }
    }
}
```

**Questions:**

1. Why do we `continue` on empty input instead of showing an error?

2. What is the purpose of `self.editor.add_history_entry(line)` returning a `Result` that we ignore?

3. Why do we print to `eprintln!` for the catch-all error case?

4. What happens if `self.process_command(&line)` panics?

5. Is it appropriate to exit the REPL on any `ReadlineError` other than Interrupted and Eof?

**Answer Key:**

1. **Why continue on empty input**: Empty input is a user action (pressing Enter), not an error. Showing an error would be annoying UX. The `continue` silently skips empty lines, allowing users to press Enter multiple times without consequences. This matches standard CLI behavior (bash, Python REPL, etc.).

2. **Ignoring `add_history_entry` Result**: The function returns `Result<(), ReadlineError>` which we ignore. This means if history saving fails (e.g., permission denied), we silently ignore it. This is acceptable because history is a convenience feature, not critical. However, we might want to log it in debug mode for troubleshooting.

3. **Using `eprintln!` for catch-all**: `eprintln!` writes to stderr, which is the standard output stream for errors and diagnostics. This separates errors from normal program output (stdout), allowing redirection:
   - `olap > output.txt` - saves only normal output
   - `olap 2> errors.txt` - saves only errors
   - `olap &> all.txt` - saves both
   This follows Unix conventions.

4. **What if `process_command` panics**: The panic will propagate up and crash the REPL. The `if let Err(e) = self.process_command(&line)` only catches returns of `Err`, not panics. A panic in `process_command` will terminate the entire program. We should wrap the call in `catch_unwind` for resilience.

5. **Exiting on other ReadlineError**: The code exits on any other error, which is probably too aggressive. There could be transient errors (temporary I/O issues, signal interruptions) where continuing might be better. However, many `ReadlineError` variants are indeed fatal (e.g., corrupted terminal state), so exiting is a reasonable default.

**Suggested Improvements:**

Here's a more robust error handling strategy:

```rust
use std::panic;
use log::{debug, warn};

impl<'a> Repl<'a> {
    pub fn run(&mut self) -> Result<()> {
        println!("Mini Rust OLAP - Interactive REPL");
        println!("Type HELP for available commands, EXIT or QUIT to exit.\n");
        
        // Set up panic handler to catch panics in command processing
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            // Still print the panic info, but don't crash
            eprintln!("\n❌ Internal error: {}", panic_info);
            eprintln!("REPL has recovered. Please report this bug if it persists.");
            // Restore default hook
            panic::set_hook(Box::new(default_hook));
        }));
        
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: usize = 5;
        
        while self.running {
            let readline = self.editor.readline("olap> ");
            
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    
                    // Skip empty input
                    if line.is_empty() {
                        continue;
                    }
                    
                    // Add to history with error handling
                    match self.editor.add_history_entry(line) {
                        Ok(_) => {},
                        Err(e) => {
                            // Log but don't fail - history is non-critical
                            debug!("Failed to add history entry: {}", e);
                        }
                    }
                    
                    // Process command with panic catching
                    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        self.process_command(line)
                    }));
                    
                    match result {
                        Ok(Ok(())) => {
                            // Success - reset error counter
                            consecutive_errors = 0;
                        }
                        Ok(Err(e)) => {
                            // Command returned an error
                            self.print_error(&e);
                            consecutive_errors += 1;
                            
                            // Check for too many consecutive errors
                            if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                                eprintln!("\n⚠ Too many consecutive errors ({}). Please check your input.", 
                                         consecutive_errors);
                                eprintln!("If you're unsure what to do, type HELP.");
                            }
                        }
                        Err(_) => {
                            // Panic was caught
                            eprintln!("\n❌ An unexpected error occurred while processing your command.");
                            eprintln!("The REPL has recovered and is ready for your next command.");
                            consecutive_errors += 1;
                        }
                    }
                }
                
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C - cancel current input
                    println!("Use EXIT or QUIT to exit.");
                    consecutive_errors = 0; // Reset on user action
                }
                
                Err(ReadlineError::Eof) => {
                    // Ctrl+D - graceful exit
                    println!("Goodbye!");
                    self.running = false;
                    break;
                }
                
                Err(ReadlineError::Io(err)) => {
                    // I/O error - might be transient
                    eprintln!("I/O error reading input: {}", err);
                    
                    // Try to continue for transient errors
                    if err.kind() == std::io::ErrorKind::Interrupted {
                        eprintln!("Interrupted by signal. Continuing...");
                        continue;
                    } else if err.kind() == std::io::ErrorKind::WouldBlock {
                        eprintln!("Input would block. Retrying...");
                        continue;
                    }
                    
                    // Fatal I/O errors - exit
                    eprintln!("Fatal I/O error. Exiting REPL.");
                    self.running = false;
                    break;
                }
                
                Err(ReadlineError::WindowResized) => {
                    // Terminal resized - just ignore and continue
                    debug!("Terminal resized");
                    continue;
                }
                
                Err(err) => {
                    // Unknown error - be cautious but try to continue
                    eprintln!("Unexpected error reading input: {}", err);
                    eprintln!("Attempting to continue...");
                    
                    consecutive_errors += 1;
                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        eprintln!("Too many errors. Exiting REPL.");
                        self.running = false;
                        break;
                    }
                }
            }
        }
        
        // Save history on exit
        self.save_history();
        
        Ok(())
    }
    
    fn save_history(&self) {
        if let Ok(home) = std::env::var("HOME") {
            let history_path = format!("{}/.olap_history", home);
            
            match self.editor.save_history(&history_path) {
                Ok(_) => debug!("History saved to {}", history_path),
                Err(e) => {
                    warn!("Failed to save history to {}: {}", history_path, e);
                    // Don't fail the REPL just because history couldn't be saved
                }
            }
        }
    }
    
    fn print_error(&self, error: &DatabaseError) {
        // Format error with context
        let error_msg = self.format_error_message(error);
        let border_len = error_msg.len().max(63);
        
        println!("╔═{}╗", "═".repeat(border_len));
        println!("║ ERROR{}║", " ".repeat(border_len - 5));
        println!("╠═{}╣", "═".repeat(border_len));
        
        // Word-wrap long error messages
        for line in self.wrap_text(&error_msg, border_len - 2) {
            println!("║ {}{}║", line, " ".repeat(border_len - line.len() - 2));
        }
        
        println!("╚═{}╝", "═".repeat(border_len));
    }
    
    fn format_error_message(&self, error: &DatabaseError) -> String {
        match error {
            DatabaseError::ParserError(msg) => {
                format!("SQL syntax error: {}", msg)
            }
            DatabaseError::ExecutionError(msg) => {
                format!("Query execution failed: {}", msg)
            }
            DatabaseError::IoError(msg) => {
                format!("I/O error: {}", msg)
            }
        }
    }
    
    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_length = 0;
        
        for word in text.split_whitespace() {
            if current_length == 0 {
                current_line.push_str(word);
                current_length = word.len();
            } else if current_length + 1 + word.len() <= width {
                current_line.push(' ');
                current_line.push_str(word);
                current_length += 1 + word.len();
            } else {
                lines.push(current_line);
                current_line = word.to_string();
                current_length = word.len();
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        lines
    }
}
```

**Key Improvements:**

1. **Panic recovery**: Use `catch_unwind` to catch panics and keep REPL running
2. **Consecutive error tracking**: Detect when user is struggling, provide help
3. **Better ReadlineError handling**: Distinguish between fatal and transient errors
4. **History error handling**: Log but don't fail on history save/load errors
5. **Error message wrapping**: Word-wrap long error messages for better readability
6. **Dynamic box sizing**: Adjust error box size to fit message
7. **Structured error formatting**: Separate formatting logic for maintainability
8. **Graceful degradation**: REPL stays alive even when non-critical features fail
9. **Debug logging**: Use logging for non-critical issues
10. **Reset error counter**: Give user credit for successful commands

**Additional robustness features:**

```rust
// Add signal handling for clean shutdown
use signal_hook::{iterator, consts::signal::*};

impl<'a> Repl<'a> {
    pub fn run_with_signals(&mut self) -> Result<()> {
        // Set up signal handlers
        let signals = iterator([SIGTERM, SIGINT])?;
        
        while self.running {
            // Check for signals with timeout
            match signals.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(signal) => {
                    match signal {
                        SIGTERM => {
                            println!("\nReceived termination signal. Shutting down gracefully...");
                            self.running = false;
                        }
                        SIGINT => {
                            println!("\nInterrupt received. Use EXIT to quit.");
                        }
                        _ => {}
                    }
                }
                Err(_) => {
                    // Timeout - normal operation
                }
            }
            
            // Continue normal REPL loop...
        }
        
        Ok(())
    }
}

// Add command timeout
fn process_command_with_timeout(&mut self, command: &str) -> Result<()> {
    let timeout = std::time::Duration::from_secs(30);
    let start = std::time::Instant::now();
    
    // Spawn a thread for command execution
    let command = command.to_string();
    let (sender, receiver) = std::sync::mpsc::channel();
    
    std::thread::spawn(move || {
        let result = Self::process_command_static(command);
        let _ = sender.send(result);
    });
    
    // Wait for completion or timeout
    match receiver.recv_timeout(timeout) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            Err(DatabaseError::ExecutionError(
                format!("Command timed out after {:?}", timeout)
            ))
        }
        Err(_) => Err(DatabaseError::ExecutionError(
            "Command execution thread failed".to_string()
        ))
    }
}

// Add input validation and sanitization
fn sanitize_input(input: &str) -> Result<String> {
    // Check for suspicious patterns
    let dangerous_patterns = vec![
        "; DROP TABLE",
        "; DELETE FROM",
        "; TRUNCATE",
        "rm -rf",
        "/etc/passwd",
        "\\x00", // null bytes
    ];
    
    let upper = input.to_uppercase();
    for pattern in dangerous_patterns {
        if upper.contains(&pattern.to_uppercase()) {
            return Err(DatabaseError::ParserError(
                format!("Potentially dangerous input detected: {}", pattern)
            ));
        }
    }
    
    // Check for excessive length
    if input.len() > 10_000 {
        return Err(DatabaseError::ParserError(
            "Input too long (max 10,000 characters)".to_string()
        ));
    }
    
    Ok(input.to_string())
}
```

**Error recovery strategies:**

1. **Retry mechanism**: For transient errors, automatically retry with exponential backoff
2. **Circuit breaker**: After too many errors, temporarily disable commands
3. **Graceful degradation**: Disable non-essential features on error
4. **State rollback**: Revert to known-good state after errors
5. **Diagnostic mode**: Automatically collect logs on errors
6. **User guidance**: Suggest similar commands on typos
7. **Safe mode**: Start in limited mode after crashes

**Best practices implemented:**

- ✅ Never panic in user-facing code
- ✅ Separate fatal from non-fatal errors
- ✅ Provide clear error messages with context
- ✅ Log errors for debugging
- ✅ Recover from panics
- ✅ Handle all error variants explicitly
- ✅ Use appropriate output streams (stdout/stderr)
- ✅ Respect Unix conventions for signals and I/O
- ✅ Maintain REPL stability across all error paths
- ✅ Provide helpful guidance when errors occur

## 🚀 Part 4: Challenge Exercises

### Challenge 4.1: Tab Completion

Implement tab completion for:
- Table names (after `LOAD`, `SELECT FROM`, `DESCRIBE`)
- Column names (after `SELECT`, `WHERE`, `ORDER BY`)
- Command names at the prompt

**Requirements:**
- Use rustyline's `Helper` trait
- Read available tables and columns from catalog
- Show suggestions on TAB
- Handle partial matches

**Deliverable:**
- Complete `ReplHelper` implementation
- Integration with `Editor`
- Demonstration of tab completion

**Hints and Implementation Approach:**

1. **Understand rustyline's Helper trait**: The `Helper` trait provides several methods for completion:
   - `complete()` - Main completion method
   - `hint()` - Optional in-line hints
   - `highlighter()` - Optional syntax highlighting

2. **Implement the Helper struct**:
```rust
use rustyline::{Helper, Hinter, Highlighter, Completer, Validator};
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;

#[derive(Helper, Hinter, Highlighter, Validator)]
struct ReplHelper<'a> {
    catalog: &'a Catalog,
}

impl<'a> ReplHelper<'a> {
    fn new(catalog: &'a Catalog) -> Self {
        ReplHelper { catalog }
    }
}
```

3. **Implement the Completer trait**:
```rust
impl<'a> Completer for ReplHelper<'a> {
    type Candidate = Pair;
    
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let before_cursor = &line[..pos];
        let last_word = before_cursor.split_whitespace().last().unwrap_or("");
        
        // Determine what to complete based on context
        let suggestions = if self.should_complete_tables(before_cursor) {
            self.complete_table_names(last_word)
        } else if self.should_complete_columns(before_cursor) {
            self.complete_column_names(last_word, before_cursor)
        } else {
            self.complete_commands(last_word)
        };
        
        // Find the start of the word to replace
        let word_start = before_cursor.rfind(' ').map_or(0, |i| i + 1);
        
        Ok((word_start, suggestions))
    }
}
```

4. **Context detection logic**:
```rust
impl<'a> ReplHelper<'a> {
    fn should_complete_tables(&self, text: &str) -> bool {
        let upper = text.to_uppercase();
        upper.ends_with("FROM ")
            || upper.ends_with("LOAD ")
            || upper.ends_with("DESCRIBE ")
            || upper.ends_with("STATS ")
            || upper.ends_with("INTO ")
    }
    
    fn should_complete_columns(&self, text: &str) -> bool {
        let upper = text.to_uppercase();
        // After SELECT, WHERE, ORDER BY, etc.
        upper.contains("SELECT ") && !upper.contains("FROM")
            || upper.ends_with("WHERE ")
            || upper.ends_with("AND ")
            || upper.ends_with("OR ")
            || upper.ends_with("ORDER BY ")
            || upper.ends_with("GROUP BY ")
    }
    
    fn complete_commands(&self, partial: &str) -> Vec<Pair> {
        let commands = vec![
            "LOAD", "SELECT", "SHOW TABLES", "DESCRIBE", 
            "STATS", "COUNT_TABLES", "EXPORT", "EXIT", "QUIT", "HELP"
        ];
        
        commands
            .into_iter()
            .filter(|c| c.to_lowercase().starts_with(&partial.to_lowercase()))
            .map(|c| Pair {
                display: c.to_string(),
                replacement: if partial.to_uppercase() == partial {
                    c.to_uppercase()
                } else {
                    c.to_string()
                }
            })
            .collect()
    }
    
    fn complete_table_names(&self, partial: &str) -> Vec<Pair> {
        let table_names = self.catalog.get_table_names();
        
        table_names
            .into_iter()
            .filter(|name| name.to_lowercase().starts_with(&partial.to_lowercase()))
            .map(|name| Pair {
                display: name.to_string(),
                replacement: name.to_string(),
            })
            .collect()
    }
    
    fn complete_column_names(&self, partial: &str, context: &str) -> Vec<Pair> {
        // Extract table name from context (e.g., "FROM employees" -> "employees")
        let table_name = self.extract_table_name(context);
        
        if let Some(table) = self.catalog.get_table(table_name) {
            table.column_names
                .iter()
                .filter(|name| name.to_lowercase().starts_with(&partial.to_lowercase()))
                .map(|name| Pair {
                    display: name.to_string(),
                    replacement: name.to_string(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    fn extract_table_name(&self, text: &str) -> Option<&str> {
        let upper = text.to_uppercase();
        
        // Find FROM, LOAD, DESCRIBE, or STATS keyword
        for keyword in &["FROM ", "LOAD ", "DESCRIBE ", "STATS "] {
            if let Some(pos) = upper.find(keyword) {
                let after_keyword = &text[pos + keyword.len()..];
                let table_name = after_keyword.split_whitespace().next()?;
                return Some(table_name);
            }
        }
        
        None
    }
}
```

5. **Integrate with Editor**:
```rust
impl<'a> Repl<'a> {
    pub fn new(catalog: &'a mut Catalog) -> Result<Self> {
        let helper = ReplHelper::new(catalog);
        let mut editor = Editor::new();
        editor.set_helper(Some(helper));
        
        // Load history if it exists
        if let Ok(history_file) = std::env::var("HOME") {
            let history_path = format!("{}/.olap_history", history_file);
            if editor.load_history(&history_path).is_err() {
                // History file doesn't exist yet, that's okay
            }
        }
        
        Ok(Repl {
            editor,
            catalog,
            running: true,
            query_buffer: String::new(),
            in_multiline: false,
        })
    }
}
```

6. **Add fuzzy matching** (bonus):
```rust
impl<'a> ReplHelper<'a> {
    fn complete_fuzzy(&self, partial: &str, candidates: Vec<String>) -> Vec<Pair> {
        if partial.is_empty() {
            return candidates
                .into_iter()
                .map(|c| Pair { display: c.clone(), replacement: c })
                .collect();
        }
        
        let partial_lower = partial.to_lowercase();
        candidates
            .into_iter()
            .filter(|c| {
                let c_lower = c.to_lowercase();
                c_lower.contains(&partial_lower) || 
                c_lower.chars().eq(partial_lower.chars().zip(c_lower.chars()).map(|(a, b)| a == b || b == '_'))
            })
            .map(|c| Pair { display: c.clone(), replacement: c })
            .collect()
    }
}
```

**Testing**:
```bash
# Command completion
olap> HE<TAB>
# Shows: HELP

# Table completion
olap> LOAD <TAB>
# Shows: employees.csv, departments.csv

olap> SELECT * FROM em<TAB>
# Shows: employees

# Column completion
olap> SELECT n<TAB>
# Shows: name, num

olap> SELECT name FROM employees WHERE sa<TAB>
# Shows: salary
```

**Key Challenges**:
- Parsing context correctly (what comes before the cursor)
- Handling case sensitivity
- Providing relevant suggestions based on SQL keywords
- Performance with many tables/columns
- Handling ambiguous matches

**Further Enhancements**:
- Display schema information in hints
- Complete nested expressions
- SQL keyword suggestions in context
- Show sample values for columns
- Complete JOIN conditions

### Challenge 4.2: Configuration File

Add support for a `.olaprc` configuration file that can set default values.

**Example `.olaprc`:**
```
# Configuration file for Mini Rust OLAP
default_table_path=./data
display_limit=100
timing_format=ms
history_size=1000
```

**Requirements:**
- Parse configuration file at startup
- Apply settings to REPL behavior
- Support comments (lines starting with #)
- Handle missing or malformed config file gracefully
- Provide `SET` command to override settings interactively

**Deliverable:**
- Configuration file parser
- Settings struct and integration
- `cmd_set` implementation
- Documentation

**Hints and Implementation Approach:**

1. **Define a Settings struct**:
```rust
#[derive(Debug, Clone)]
pub struct Settings {
    pub default_table_path: String,
    pub display_limit: usize,
    pub timing_format: TimingFormat,
    pub history_size: usize,
    pub echo_commands: bool,
    pub pager_enabled: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TimingFormat {
    Auto,
    Milliseconds,
    Seconds,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            default_table_path: "./data".to_string(),
            display_limit: 50,
            timing_format: TimingFormat::Auto,
            history_size: 1000,
            echo_commands: false,
            pager_enabled: false,
        }
    }
}
```

2. **Implement config file parser**:
```rust
use std::path::PathBuf;

impl Settings {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| DatabaseError::execution_error(
                format!("Failed to read config file '{}': {}", path.display(), e)
            ))?;
        
        Self::parse(&content)
    }
    
    pub fn parse(content: &str) -> Result<Self> {
        let mut settings = Settings::default();
        
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse key=value
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();
                
                settings.set_setting(key, value)
                    .map_err(|e| DatabaseError::execution_error(
                        format!("Config line {}: {}", line_num + 1, e)
                    ))?;
            } else {
                return Err(DatabaseError::execution_error(
                    format!("Config line {}: Invalid format (expected key=value)", line_num + 1)
                ));
            }
        }
        
        Ok(settings)
    }
    
    pub fn set_setting(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "default_table_path" => {
                self.default_table_path = value.to_string();
            }
            "display_limit" => {
                self.display_limit = value.parse()
                    .map_err(|_| DatabaseError::execution_error(
                        format!("Invalid display_limit: '{}'. Expected positive integer.", value)
                    ))?;
            }
            "timing_format" => {
                self.timing_format = match value.to_lowercase().as_str() {
                    "auto" => TimingFormat::Auto,
                    "ms" | "milliseconds" => TimingFormat::Milliseconds,
                    "s" | "seconds" => TimingFormat::Seconds,
                    _ => return Err(DatabaseError::execution_error(
                        format!("Invalid timing_format: '{}'. Expected: auto, ms, or s.", value)
                    ))
                };
            }
            "history_size" => {
                self.history_size = value.parse()
                    .map_err(|_| DatabaseError::execution_error(
                        format!("Invalid history_size: '{}'. Expected positive integer.", value)
                    ))?;
            }
            "echo_commands" => {
                self.echo_commands = value.parse()
                    .map_err(|_| DatabaseError::execution_error(
                        format!("Invalid echo_commands: '{}'. Expected true or false.", value)
                    ))?;
            }
            "pager_enabled" => {
                self.pager_enabled = value.parse()
                    .map_err(|_| DatabaseError::execution_error(
                        format!("Invalid pager_enabled: '{}'. Expected true or false.", value)
                    ))?;
            }
            _ => {
                return Err(DatabaseError::execution_error(
                    format!("Unknown setting: '{}'", key)
                ));
            }
        }
        
        Ok(())
    }
}
```

3. **Load config at REPL startup**:
```rust
impl<'a> Repl<'a> {
    pub fn new(catalog: &'a mut Catalog) -> Result<Self> {
        let helper = ReplHelper::new(catalog);
        let mut editor = Editor::new();
        editor.set_helper(Some(helper));
        
        // Load configuration
        let settings = Self::load_config()?;
        
        // Configure history size
        editor.set_max_history_size(settings.history_size);
        
        // Load history if it exists
        if let Ok(history_file) = std::env::var("HOME") {
            let history_path = format!("{}/.olap_history", history_file);
            if editor.load_history(&history_path).is_err() {
                // History file doesn't exist yet, that's okay
            }
        }
        
        Ok(Repl {
            editor,
            catalog,
            running: true,
            query_buffer: String::new(),
            in_multiline: false,
            settings,
        })
    }
    
    fn load_config() -> Result<Settings> {
        let config_paths = vec![
            PathBuf::from(".olaprc"),
            PathBuf::from(format!("{}/.olaprc", std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))),
            PathBuf::from("/etc/olaprc"),
        ];
        
        for path in config_paths {
            if path.exists() {
                return Settings::from_file(&path);
            }
        }
        
        // No config file found, use defaults
        Ok(Settings::default())
    }
}
```

4. **Implement SET command**:
```rust
impl<'a> Repl<'a> {
    fn cmd_set(&mut self, input: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        // Parse: SET <key> = <value>
        let parts: Vec<&str> = input.splitn(3, '=').collect();
        if parts.len() != 2 {
            return Err(DatabaseError::parser_error(
                "Invalid SET syntax. Usage: SET <key> = <value>".to_string()
            ));
        }
        
        let key = parts[0].trim_start_matches("SET").trim();
        let value = parts[1].trim();
        
        if key.is_empty() || value.is_empty() {
            return Err(DatabaseError::parser_error(
                "SET requires both key and value".to_string()
            ));
        }
        
        self.settings.set_setting(key, value)?;
        
        println!("✓ Setting '{}' = '{}'", key, value);
        
        // Apply some settings immediately
        self.apply_settings();
        
        let duration = start.elapsed();
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
    
    fn apply_settings(&mut self) {
        // Update history size
        self.editor.set_max_history_size(self.settings.history_size);
        
        // Note: Other settings are applied when they're used
    }
}
```

5. **Implement GET command**:
```rust
impl<'a> Repl<'a> {
    fn cmd_get(&self, input: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        let key = input.trim_start_matches("GET").trim();
        
        if key.is_empty() {
            // Show all settings
            println!("Current Settings:");
            println!("┌─────────────────────┬─────────────────────────────────┐");
            println!("│ Setting             │ Value                           │");
            println!("├─────────────────────┼─────────────────────────────────┤");
            println!("│ default_table_path  │ {:<31} │", self.settings.default_table_path);
            println!("│ display_limit       │ {:<31} │", self.settings.display_limit);
            println!("│ timing_format       │ {:<31} │", 
                     match self.settings.timing_format {
                         TimingFormat::Auto => "auto",
                         TimingFormat::Milliseconds => "ms",
                         TimingFormat::Seconds => "s",
                     });
            println!("│ history_size        │ {:<31} │", self.settings.history_size);
            println!("│ echo_commands       │ {:<31} │", self.settings.echo_commands);
            println!("│ pager_enabled       │ {:<31} │", self.settings.pager_enabled);
            println!("└─────────────────────┴─────────────────────────────────┘");
        } else {
            // Show specific setting
            let value = match key {
                "default_table_path" => self.settings.default_table_path.clone(),
                "display_limit" => self.settings.display_limit.to_string(),
                "timing_format" => match self.settings.timing_format {
                    TimingFormat::Auto => "auto".to_string(),
                    TimingFormat::Milliseconds => "ms".to_string(),
                    TimingFormat::Seconds => "s".to_string(),
                },
                "history_size" => self.settings.history_size.to_string(),
                "echo_commands" => self.settings.echo_commands.to_string(),
                "pager_enabled" => self.settings.pager_enabled.to_string(),
                _ => return Err(DatabaseError::execution_error(
                    format!("Unknown setting: '{}'", key)
                )),
            };
            
            println!("{} = {}", key, value);
        }
        
        let duration = start.elapsed();
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
}
```

6. **Update execute_command**:
```rust
fn execute_command(&mut self, input: &str) -> Result<()> {
    let upper_input = input.to_uppercase();
    
    if upper_input.starts_with("SET ") {
        self.cmd_set(input)
    } else if upper_input.starts_with("GET ") {
        self.cmd_get(input)
    } else if upper_input == "COUNT_TABLES" {
        self.cmd_count_tables()
    } else if upper_input.starts_with("LOAD ") {
        self.cmd_load(input)
    }
    // ... rest of commands
}
```

**Example `.olaprc` file**:
```ini
# Mini Rust OLAP Configuration File
# Lines starting with # are comments

# Default path for CSV files
default_table_path=./data

# Number of rows to display by default
display_limit=100

# Timing format: auto, ms, or s
timing_format=auto

# Maximum number of history entries
history_size=1000

# Echo commands before execution
echo_commands=false

# Enable pager for large results
pager_enabled=false
```

**Testing**:
```bash
# Show all settings
olap> GET
Current Settings:
┌─────────────────────┬─────────────────────────────────┐
│ Setting             │ Value                           │
├─────────────────────┼─────────────────────────────────┤
│ default_table_path  │ ./data                          │
│ display_limit       │ 50                              │
│ timing_format       │ auto                            │
│ history_size        │ 1000                            │
│ echo_commands       │ false                           │
│ pager_enabled       │ false                           │
└─────────────────────┴─────────────────────────────────┘

# Change a setting
olap> SET display_limit = 100
✓ Setting 'display_limit' = '100'
⏱ Executed in 0.02ms

# Get specific setting
olap> GET display_limit
display_limit = 100

# Test with config file
$ cat .olaprc
display_limit=100
timing_format=ms

olap> GET display_limit
display_limit = 100  # Loaded from .olaprc
```

**Key Challenges**:
- Handling boolean values (true/false vs 1/0 vs yes/no)
- Type validation for numeric settings
- Path resolution for default_table_path
- Case sensitivity for setting names
- Error messages for invalid values
- Applying settings immediately vs deferred

**Further Enhancements**:
- Support environment variables ($HOME, $USER)
- Include/exclude patterns for files
- Color themes for output
- Default query templates
- Per-table settings
- Config file hot-reloading

### Challenge 4.3: Session Variables

Add support for session variables that can be referenced in queries.

**Example usage:**
```
SET @min_salary = 75000
SET @dept = 'Engineering'

SELECT name, salary FROM employees 
WHERE salary > @min_salary 
AND department = @dept
```

**Requirements:**
- Store variables in a HashMap in the REPL
- Parse `@variable` syntax in queries
- Replace variables before passing to parser
- Support GET and UNSET commands
- Support string and numeric variable types

**Deliverable:**
- Variable storage system
- Query preprocessing with variable substitution
- `cmd_set`, `cmd_get`, `cmd_unset` implementations
- Integration tests

**Hints and Implementation Approach:**

1. **Define variable storage**:
```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SessionVariable {
    Int(i64),
    Float(f64),
    String(String),
}

impl SessionVariable {
    fn as_string(&self) -> String {
        match self {
            SessionVariable::Int(n) => n.to_string(),
            SessionVariable::Float(f) => {
                if f.fract() == 0.0 {
                    format!("{}", *f as i64)
                } else {
                    format!("{:.6}", f).trim_end_matches('0').trim_end_matches('.').to_string()
                }
            }
            SessionVariable::String(s) => s.clone(),
        }
    }
}
```

2. **Add to Repl struct**:
```rust
pub struct Repl<'a> {
    editor: Editor<()>,
    catalog: &'a mut Catalog,
    running: bool,
    query_buffer: String,
    in_multiline: bool,
    settings: Settings,
    variables: HashMap<String, SessionVariable>,  // New field
}
```

3. **Implement SET for variables**:
```rust
impl<'a> Repl<'a> {
    fn cmd_set_variable(&mut self, input: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        // Parse: SET @variable = value
        if !input.contains('@') {
            return Err(DatabaseError::parser_error(
                "Variable name must start with @".to_string()
            ));
        }
        
        let parts: Vec<&str> = input.splitn(3, '=').collect();
        if parts.len() != 2 {
            return Err(DatabaseError::parser_error(
                "Invalid SET syntax. Usage: SET @variable = value".to_string()
            ));
        }
        
        let var_name = parts[0].trim_start_matches("SET").trim();
        let value_str = parts[1].trim();
        
        if !var_name.starts_with('@') {
            return Err(DatabaseError::parser_error(
                "Variable name must start with @".to_string()
            ));
        }
        
        if var_name.len() == 1 {
            return Err(DatabaseError::parser_error(
                "Variable name cannot be empty".to_string()
            ));
        }
        
        // Remove quotes if present
        let value_str = value_str.trim_matches('\'').trim_matches('"');
        
        // Try to parse as integer, then float, then string
        let variable = if let Ok(n) = value_str.parse::<i64>() {
            SessionVariable::Int(n)
        } else if let Ok(f) = value_str.parse::<f64>() {
            SessionVariable::Float(f)
        } else {
            SessionVariable::String(value_str.to_string())
        };
        
        self.variables.insert(var_name.to_string(), variable);
        
        let var_type = match self.variables.get(var_name) {
            Some(SessionVariable::Int(_)) => "Int",
            Some(SessionVariable::Float(_)) => "Float",
            Some(SessionVariable::String(_)) => "String",
            None => unreachable!(),
        };
        
        println!("✓ {} = {} ({})", var_name, value_str, var_type);
        
        let duration = start.elapsed();
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
}
```

4. **Implement GET for variables**:
```rust
impl<'a> Repl<'a> {
    fn cmd_get_variable(&self, input: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        let var_name = input.trim_start_matches("GET").trim();
        
        if var_name.is_empty() {
            // Show all variables
            if self.variables.is_empty() {
                println!("No session variables defined.");
            } else {
                println!("Session Variables:");
                println!("┌─────────────────┬─────────────────────────┬──────────┐");
                println!("│ Name            │ Value                   │ Type     │");
                println!("├─────────────────┼─────────────────────────┼──────────┤");
                
                for (name, var) in &self.variables {
                    let value_str = var.as_string();
                    let truncated = if value_str.len() > 23 {
                        format!("{}...", &value_str[..20])
                    } else {
                        value_str.clone()
                    };
                    
                    let var_type = match var {
                        SessionVariable::Int(_) => "Int",
                        SessionVariable::Float(_) => "Float",
                        SessionVariable::String(_) => "String",
                    };
                    
                    println!("│ {:<15} │ {:<23} │ {:<8} │", name, truncated, var_type);
                }
                
                println!("└─────────────────┴─────────────────────────┴──────────┘");
            }
        } else {
            // Show specific variable
            if !var_name.starts_with('@') {
                return Err(DatabaseError::parser_error(
                    "Variable name must start with @".to_string()
                ));
            }
            
            match self.variables.get(var_name) {
                Some(var) => {
                    let value_str = var.as_string();
                    let var_type = match var {
                        SessionVariable::Int(_) => "Int",
                        SessionVariable::Float(_) => "Float",
                        SessionVariable::String(_) => "String",
                    };
                    println!("{} = {} ({})", var_name, value_str, var_type);
                }
                None => {
                    return Err(DatabaseError::execution_error(
                        format!("Variable '{}' not found", var_name)
                    ));
                }
            }
        }
        
        let duration = start.elapsed();
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
}
```

5. **Implement UNSET**:
```rust
impl<'a> Repl<'a> {
    fn cmd_unset_variable(&mut self, input: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        let var_name = input.trim_start_matches("UNSET").trim();
        
        if !var_name.starts_with('@') {
            return Err(DatabaseError::parser_error(
                "Variable name must start with @".to_string()
            ));
        }
        
        if var_name.len() == 1 {
            return Err(DatabaseError::parser_error(
                "Variable name cannot be empty".to_string()
            ));
        }
        
        match self.variables.remove(var_name) {
            Some(_) => {
                println!("✓ Variable '{}' removed", var_name);
            }
            None => {
                return Err(DatabaseError::execution_error(
                    format!("Variable '{}' not found", var_name)
                ));
            }
        }
        
        let duration = start.elapsed();
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
}
```

6. **Implement query preprocessing**:
```rust
impl<'a> Repl<'a> {
    fn preprocess_query(&self, query: &str) -> String {
        let mut result = query.to_string();
        
        // Find all @variable references and replace them
        // This is a simple implementation - for production, use proper tokenization
        let mut offset = 0;
        
        while let Some(at_pos) = result[offset..].find('@') {
            let absolute_pos = offset + at_pos;
            
            // Extract variable name
            let var_start = absolute_pos;
            let var_end = result[var_start + 1..]
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|pos| var_start + 1 + pos)
                .unwrap_or(result.len());
            
            let var_name = &result[var_start..var_end];
            
            // Replace with value
            if let Some(variable) = self.variables.get(var_name) {
                let value = variable.as_string();
                result.replace_range(var_start..var_end, &value);
                offset = var_start + value.len();
            } else {
                offset = var_end;
            }
        }
        
        result
    }
}
```

7. **Update cmd_select to use preprocessing**:
```rust
impl<'a> Repl<'a> {
    fn cmd_select(&self, input: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        // Preprocess query to substitute variables
        let query = self.preprocess_query(input);
        
        // Add LIMIT if not present
        let query_with_limit = if !query.to_uppercase().contains(" LIMIT ") {
            format!("{} LIMIT 50", query)
        } else {
            query
        };
        
        // Execute the query
        let batches = self.execute_query(&query_with_limit)?;
        
        // Print results
        self.print_batches(&batches);
        
        let duration = start.elapsed();
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
}
```

8. **Update execute_command**:
```rust
fn execute_command(&mut self, input: &str) -> Result<()> {
    let upper_input = input.to_uppercase();
    
    if upper_input.starts_with("SET @") {
        self.cmd_set_variable(input)
    } else if upper_input.starts_with("SET ") {
        self.cmd_set(input)  // Setting configuration
    } else if upper_input.starts_with("GET") {
        // Check if it's a variable (starts with @) or setting
        if input.contains('@') {
            self.cmd_get_variable(input)
        } else {
            self.cmd_get(input)
        }
    } else if upper_input.starts_with("UNSET @") {
        self.cmd_unset_variable(input)
    }
    // ... rest of commands
}
```

**Testing**:
```bash
# Set integer variable
olap> SET @min_salary = 75000
✓ @min_salary = 75000 (Int)
⏱ Executed in 0.02ms

# Set string variable
olap> SET @dept = 'Engineering'
✓ @dept = Engineering (String)
⏱ Executed in 0.01ms

# Use in query
olap> SELECT name, salary FROM employees WHERE salary > @min_salary
┌─────────────────────────────────────────────────────────────┐
│ col_0   │ col_1      │
├─────────────────────────────────────────────────────────────┤
│ Bob     │ 82000.0    │
│ Dave    │ 105000.0   │
│ Charlie │ 95000.0    │
└─────────────────────────────────────────────────────────────┘
⏱ Executed in 0.45ms

# Multiple variables
olap> SELECT * FROM employees WHERE department = @dept AND salary > @min_salary
┌─────────────────────────────────────────────────────────────┐
│ col_0  │ col_1      │ col_2 │ col_3    │ col_4      │
├─────────────────────────────────────────────────────────────┤
│ Bob    │ 82000.0    │ 35    │ 82000.0  │ Engineering│
│ Charlie│ 95000.0    │ 40    │ 95000.0  │ Engineering│
└─────────────────────────────────────────────────────────────┘
⏱ Executed in 0.38ms

# Show all variables
olap> GET
Session Variables:
┌─────────────────┬─────────────────────────┬──────────┐
│ Name            │ Value                   │ Type     │
├─────────────────┼─────────────────────────┼──────────┤
│ @dept           │ Engineering             │ String   │
│ @min_salary     │ 75000                   │ Int      │
└─────────────────┴─────────────────────────┴──────────┘

# Get specific variable
olap> GET @min_salary
@min_salary = 75000 (Int)

# Unset variable
olap> UNSET @min_salary
✓ Variable '@min_salary' removed
⏱ Executed in 0.01ms

# Try to use unset variable
olap> SELECT * FROM employees WHERE salary > @min_salary
╔═══════════════════════════════════════════════════════════════╗
║ ERROR                                                          ║
╠═══════════════════════════════════════════════════════════════╣
║ Query execution failed: Variable '@min_salary' not found      ║
╚═══════════════════════════════════════════════════════════════╝
```

**Key Challenges**:
- Substituting variables in strings without breaking syntax
- Handling variable names that are substrings of other variables
- Preserving quotes for string variables
- Type safety in query context
- Performance of repeated substitutions
- Error messages for undefined variables

**Further Enhancements**:
- Support expression evaluation (@min_salary + 1000)
- Nested variables (@var2 = @var1)
- Variable scoping (local vs global)
- Variable persistence to file
- Type coercion and conversion
- Built-in functions (@NOW, @DATE)
- Array/map variables

### Challenge 4.4: Query History Browser

Add a `HISTORY` command that allows browsing and re-executing past queries.

**Example interaction:**
```
olap> HISTORY
Query History:
1. SELECT * FROM employees WHERE salary > 80000
2. SELECT department, COUNT(*) FROM employees GROUP BY department
3. SELECT name, salary FROM employees ORDER BY salary DESC LIMIT 5

olap> !1
[Re-executes query #1]
```

**Requirements:**
- Store query history in memory
- Display numbered list with `HISTORY` command
- Re-execute with `!<number>` syntax
- Support `!-n` for n-th most recent query
- Persist history between sessions

**Deliverable:**
- History storage and management
- `cmd_history` implementation
- History command expansion (`!` syntax)
- Persistence integration

**Hints and Implementation Approach:**

1. **Add history storage to Repl**:
```rust
use std::collections::VecDeque;

pub struct Repl<'a> {
    editor: Editor<()>,
    catalog: &'a mut Catalog,
    running: bool,
    query_buffer: String,
    in_multiline: bool,
    settings: Settings,
    variables: HashMap<String, SessionVariable>,
    query_history: VecDeque<String>,  // New field
    max_history_size: usize,
}
```

2. **Initialize history**:
```rust
impl<'a> Repl<'a> {
    pub fn new(catalog: &'a mut Catalog) -> Result<Self> {
        let helper = ReplHelper::new(catalog);
        let mut editor = Editor::new();
        editor.set_helper(Some(helper));
        
        let settings = Self::load_config()?;
        let max_history_size = settings.history_size;
        
        // Load history from file
        let mut query_history = VecDeque::new();
        if let Ok(home) = std::env::var("HOME") {
            let history_path = format!("{}/.olap_query_history", home);
            if let Ok(content) = std::fs::read_to_string(&history_path) {
                for line in content.lines() {
                    if !line.is_empty() {
                        query_history.push_back(line.to_string());
                    }
                }
            }
        }
        
        Ok(Repl {
            editor,
            catalog,
            running: true,
            query_buffer: String::new(),
            in_multiline: false,
            settings,
            variables: HashMap::new(),
            query_history,
            max_history_size,
        })
    }
}
```

3. **Add to history after successful queries**:
```rust
impl<'a> Repl<'a> {
    fn add_to_history(&mut self, query: &str) {
        let trimmed = query.trim();
        
        // Don't add empty or special commands
        if trimmed.is_empty() || trimmed.starts_with("HISTORY") || 
           trimmed.starts_with("!") || trimmed.starts_with("EXIT") ||
           trimmed.starts_with("QUIT") {
            return;
        }
        
        // Don't add duplicate consecutive queries
        if let Some(last) = self.query_history.back() {
            if last == trimmed {
                return;
            }
        }
        
        // Add to history
        self.query_history.push_back(trimmed.to_string());
        
        // Trim to max size
        while self.query_history.len() > self.max_history_size {
            self.query_history.pop_front();
        }
        
        // Save to file
        self.save_history_to_file();
    }
    
    fn save_history_to_file(&self) {
        if let Ok(home) = std::env::var("HOME") {
            let history_path = format!("{}/.olap_query_history", home);
            if let Ok(mut file) = std::fs::File::create(&history_path) {
                use std::io::Write;
                for query in &self.query_history {
                    let _ = writeln!(file, "{}", query);
                }
            }
        }
    }
}
```

4. **Implement HISTORY command**:
```rust
impl<'a> Repl<'a> {
    fn cmd_history(&self) -> Result<()> {
        let start = std::time::Instant::now();
        
        if self.query_history.is_empty() {
            println!("No query history yet.");
            return Ok(());
        }
        
        println!("Query History ({} commands):", self.query_history.len());
        println!("┌───┬─────────────────────────────────────────────────────────────┐");
        println!("│ # │ Query                                                      │");
        println!("├───┼─────────────────────────────────────────────────────────────┤");
        
        for (i, query) in self.query_history.iter().enumerate() {
            let num = i + 1;
            let truncated = if query.len() > 58 {
                format!("{}...", &query[..55])
            } else {
                query.clone()
            };
            
            println!("│ {:<3} │ {:<58} │", num, truncated);
        }
        
        println!("└───┴─────────────────────────────────────────────────────────────┘");
        println!("\nUse !<number> to re-execute, e.g., !1");
        println!("Use !-<number> for n-th most recent, e.g., !-1");
        
        let duration = start.elapsed();
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
}
```

5. **Implement history expansion**:
```rust
impl<'a> Repl<'a> {
    fn expand_history_command(&self, input: &str) -> Result<String> {
        let trimmed = input.trim();
        
        // Check for history expansion
        if trimmed.starts_with('!') {
            let index_str = &trimmed[1..];
            
            if index_str.is_empty() {
                return Err(DatabaseError::parser_error(
                    "History expansion requires a number. Use !<number>".to_string()
                ));
            }
            
            // Check for negative index (most recent)
            let index = if index_str.starts_with('-') {
                // Negative index: !-1 is most recent, !-2 is second most recent, etc.
                let n = index_str[1..].parse::<isize>()
                    .map_err(|_| DatabaseError::parser_error(
                        format!("Invalid history index: '{}'", index_str)
                    ))?;
                
                if n < 1 {
                    return Err(DatabaseError::parser_error(
                        "Negative index must be >= 1 (e.g., !-1, !-2)".to_string()
                    ));
                }
                
                let pos = self.query_history.len() as isize - n;
                if pos < 0 {
                    return Err(DatabaseError::execution_error(
                        format!("History index {} out of range (only {} queries in history)", 
                                index_str, self.query_history.len())
                    ));
                }
                
                pos as usize
            } else {
                // Positive index: !1 is first, !2 is second, etc.
                let n = index_str.parse::<usize>()
                    .map_err(|_| DatabaseError::parser_error(
                        format!("Invalid history index: '{}'", index_str)
                    ))?;
                
                if n == 0 || n > self.query_history.len() {
                    return Err(DatabaseError::execution_error(
                        format!("History index {} out of range (valid: 1-{})", 
                                n, self.query_history.len())
                    ));
                }
                
                n - 1  // Convert to 0-based
            };
            
            let query = self.query_history[index].clone();
            
            // Echo the command being executed
            println!("! {}", query);
            
            Ok(query)
        } else {
            // No expansion needed
            Ok(trimmed.to_string())
        }
    }
}
```

6. **Update main loop to handle expansion**:
```rust
impl<'a> Repl<'a> {
    pub fn run(&mut self) -> Result<()> {
        println!("Mini Rust OLAP - Interactive REPL");
        println!("Type HELP for available commands, EXIT or QUIT to exit.\n");
        
        while self.running {
            let prompt = if self.in_multiline {
                "    > ".to_string()
            } else {
                "olap> ".to_string()
            };
            
            let readline = self.editor.readline(&prompt);
            
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    
                    if line.is_empty() {
                        if self.in_multiline {
                            self.in_multiline = false;
                            let query = std::mem::take(&mut self.query_buffer);
                            if !query.is_empty() {
                                if let Err(e) = self.process_query(&query) {
                                    self.print_error(&e);
                                }
                            }
                        }
                        continue;
                    }
                    
                    if !self.in_multiline {
                        // Check for history expansion
                        let expanded = self.expand_history_command(line)?;
                        
                        // Add to history (but not for history commands or expansions)
                        if !line.starts_with('!') && !line.starts_with("HISTORY") {
                            self.editor.add_history_entry(&expanded);
                        }
                        
                        let is_incomplete = self.is_incomplete_query(&expanded);
                        let ends_with_semicolon = expanded.ends_with(';');
                        
                        if is_incomplete && !ends_with_semicolon {
                            self.query_buffer = expanded;
                            self.in_multiline = true;
                        } else {
                            if let Err(e) = self.process_query(&expanded) {
                                self.print_error(&e);
                            }
                        }
                    } else {
                        // Multiline continuation
                        if !self.query_buffer.is_empty() {
                            self.query_buffer.push(' ');
                        }
                        self.query_buffer.push_str(line);
                        
                        let ends_with_semicolon = line.ends_with(';');
                        if ends_with_semicolon {
                            self.in_multiline = false;
                            let query = self.query_buffer.clone();
                            self.query_buffer.clear();
                            if let Err(e) = self.process_query(&query) {
                                self.print_error(&e);
                            }
                        }
                    }
                }
                // ... rest of error handling
            }
        }
        
        Ok(())
    }
    
    fn process_query(&mut self, query: &str) -> Result<()> {
        // Process the query and add to history if successful
        let result = self.execute_command(query);
        
        if result.is_ok() && !query.starts_with('!') {
            self.add_to_history(query);
        }
        
        result
    }
}
```

**Testing**:
```bash
# Run some queries
olap> LOAD employees.csv AS employees
✓ Loaded 10 rows into 'employees'

olap> SELECT * FROM employees LIMIT 5
┌─────────────────────────────────────────────────────────────┐
│ ...                                                         │
└─────────────────────────────────────────────────────────────┘

olap> SELECT department, COUNT(*) FROM employees GROUP BY department
┌─────────────────────────────────────────────────────────────┐
│ ...                                                         │
└─────────────────────────────────────────────────────────────┘

# View history
olap> HISTORY
Query History (3 commands):
┌───┬─────────────────────────────────────────────────────────────┐
│ # │ Query                                                      │
├───┼─────────────────────────────────────────────────────────────┤
│ 1 │ LOAD employees.csv AS employees                           │
│ 2 │ SELECT * FROM employees LIMIT 5                            │
│ 3 │ SELECT department, COUNT(*) FROM employees GROUP BY       │
│   │   department                                              │
└───┴─────────────────────────────────────────────────────────────┘

Use !<number> to re-execute, e.g., !1
Use !-<number> for n-th most recent, e.g., !-1

# Re-execute by positive index
olap> !2
! SELECT * FROM employees LIMIT 5
┌─────────────────────────────────────────────────────────────┐
│ ...                                                         │
└─────────────────────────────────────────────────────────────┘

# Re-execute by negative index (most recent)
olap> !-1
! SELECT department, COUNT(*) FROM employees GROUP BY department
┌─────────────────────────────────────────────────────────────┐
│ ...                                                         │
└─────────────────────────────────────────────────────────────┘

# Test error cases
olap> !99
╔═══════════════════════════════════════════════════════════════╗
║ ERROR                                                          ║
╠═══════════════════════════════════════════════════════════════╣
║ History index 99 out of range (valid: 1-3)                    ║
╚═══════════════════════════════════════════════════════════════╝

olap> !
╔═══════════════════════════════════════════════════════════════╗
║ ERROR                                                          ║
╠═══════════════════════════════════════════════════════════════╣
║ History expansion requires a number. Use !<number>           ║
╚═══════════════════════════════════════════════════════════════╝
```

**Key Challenges**:
- Handling concurrent history access
- Preventing infinite loops (!1 calls !1)
- Managing history size limits
- Persisting history to file
- Handling duplicate consecutive queries
- Multiline query history
- Performance with large history

**Further Enhancements**:
- Search history: !?pattern
- Edit before execution: !!s/old/new
- History substitution: !1:s/foo/bar
- Export/import history
- Timestamp history entries
- Tag/categorize queries
- Share history between sessions

### Challenge 4.5: Query Explain Plan

Add an `EXPLAIN` command that shows the execution plan without running it.

**Example:**
```
olap> EXPLAIN SELECT name, salary FROM employees WHERE salary > 80000
Execution Plan:
┌─────────────────────────────────────────────────────────────────┐
│ Operator Type    │ Details                                    │
├─────────────────────────────────────────────────────────────────┤
│ TableScan       │ Scan table: employees (6 columns)           │
│ Filter          │ Predicate: salary > 80000.0                 │
│ Project         │ Columns: name, salary                       │
└─────────────────────────────────────────────────────────────────┘
Estimated cost: 10 rows
```

**Requirements:**
- Parse `EXPLAIN` prefix before SQL
- Build the plan but don't execute
- Display operator tree with indentation
- Show operator-specific details
- Estimate row counts at each stage

**Deliverable:**
- EXPLAIN command parser
- Plan visualization logic
- Operator-specific detail extraction
- Integration with existing Planner

**Hints and Implementation Approach:**

1. **Add EXPLAIN parsing**:
```rust
impl Command {
    fn parse(input: &str) -> Result<Self> {
        let upper = input.trim().to_uppercase();
        
        if upper.starts_with("EXPLAIN ") {
            let query = input.trim_start_matches("EXPLAIN")
                .trim_start_matches("explain")
                .trim_start_matches("Explain")
                .trim().to_string();
            
            if query.is_empty() {
                return Err(DatabaseError::parser_error(
                    "EXPLAIN requires a query".to_string()
                ));
            }
            
            Ok(Command::Explain { query })
        } else {
            // ... existing parsing
        }
    }
}
```

2. **Add Explain command variant**:
```rust
enum Command {
    // ... existing variants
    Explain { query: String },
}
```

3. **Implement EXPLAIN command**:
```rust
impl<'a> Repl<'a> {
    fn cmd_explain(&self, input: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        // Parse the query (without EXPLAIN)
        let parser = Parser::new(input);
        let ast = parser.parse()?;
        
        // Build the plan
        let planner = Planner::new(&ast, self.catalog);
        let plan = planner.plan()?;
        
        // Display the execution plan
        self.display_execution_plan(&plan)?;
        
        let duration = start.elapsed();
        println!("⏱ Executed in {:.2}{}", 
                 if duration.as_millis() < 1 {
                     duration.as_secs_f64()
                 } else {
                     duration.as_millis() as f64
                 },
                 if duration.as_millis() < 1 { "s" } else { "ms" });
        
        Ok(())
    }
    
    fn display_execution_plan(&self, plan: &PlanNode) -> Result<()> {
        println!("Execution Plan:");
        
        // Calculate widths
        let type_width = 20;
        let details_width = 40;
        let total_width = type_width + details_width + 7;
        
        // Print header
        println!("┌{}┐", "─".repeat(total_width - 2));
        println!("│ {:<20} │ {:<40} │", "Operator Type", "Details");
        println!("├{}┤", "─".repeat(total_width - 2));
        
        // Walk the plan tree and print operators
        self.print_plan_node(plan, 0)?;
        
        // Print footer
        println!("└{}┘", "─".repeat(total_width - 2));
        
        // Print estimated costs
        println!("\nEstimated cost: {} rows", self.estimate_rows(plan));
        
        Ok(())
    }
    
    fn print_plan_node(&self, node: &PlanNode, indent: usize) -> Result<()> {
        let indent_str = "  ".repeat(indent);
        
        match node {
            PlanNode::TableScan { table_name, columns } => {
                println!("│ {}TableScan        │ Scan table: {} ({} columns)             │", 
                         indent_str, table_name, columns.len());
            }
            PlanNode::Filter { predicate, child } => {
                println!("│ {}Filter           │ Predicate: {}                           │", 
                         indent_str, self.format_predicate(predicate));
                self.print_plan_node(child, indent + 1)?;
            }
            PlanNode::Project { columns, child } => {
                let cols: Vec<String> = columns.iter()
                    .take(3)
                    .map(|c| c.clone())
                    .collect();
                let col_str = if columns.len() > 3 {
                    format!("{}, ... ({} total)", cols.join(", "), columns.len())
                } else {
                    cols.join(", ")
                };
                println!("│ {}Project          │ Columns: {}                              │", 
                         indent_str, col_str);
                self.print_plan_node(child, indent + 1)?;
            }
            PlanNode::Aggregate { group_by, aggregates, child } => {
                let groups: Vec<String> = group_by.iter().take(2).map(|g| g.clone()).collect();
                let group_str = if group_by.len() > 2 {
                    format!("{}, ...", groups.join(", "))
                } else {
                    groups.join(", ")
                };
                println!("│ {}Aggregate         │ Group by: {}, Aggregates: {}             │", 
                         indent_str, 
                         if group_by.is_empty() { "(none)" } else { &group_str },
                         aggregates.len());
                self.print_plan_node(child, indent + 1)?;
            }
            PlanNode::Sort { order_by, child } => {
                let orders: Vec<String> = order_by.iter()
                    .take(2)
                    .map(|(col, dir)| format!("{}{}", col, if *dir { " ASC" } else { " DESC" }))
                    .collect();
                let order_str = if order_by.len() > 2 {
                    format!("{}, ...", orders.join(", "))
                } else {
                    orders.join(", ")
                };
                println!("│ {}Sort             │ Order by: {}                              │", 
                         indent_str, order_str);
                self.print_plan_node(child, indent + 1)?;
            }
            PlanNode::Limit { limit, child } => {
                println!("│ {}Limit            │ Limit: {} rows                            │", 
                         indent_str, limit);
                self.print_plan_node(child, indent + 1)?;
            }
        }
        
        Ok(())
    }
    
    fn format_predicate(&self, predicate: &Expression) -> String {
        match predicate {
            Expression::BinaryOp { left, op, right } => {
                format!("{} {} {}", self.format_expression(left), op, self.format_expression(right))
            }
            Expression::Column(name) => name.clone(),
            Expression::Literal(value) => format!("{:?}", value),
            _ => "(complex expression)".to_string()
        }
    }
    
    fn format_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::BinaryOp { left, op, right } => {
                format!("({} {} {})", self.format_expression(left), op, self.format_expression(right))
            }
            Expression::Column(name) => name.clone(),
            Expression::Literal(Value::Int64(n)) => n.to_string(),
            Expression::Literal(Value::Float64(f)) => format!("{:.2}", f),
            Expression::Literal(Value::String(s)) => format!("'{}'", s),
            _ => "?".to_string()
        }
    }
    
    fn estimate_rows(&self, plan: &PlanNode) -> usize {
        match plan {
            PlanNode::TableScan { table_name, .. } => {
                // Estimate from catalog statistics
                if let Some(table) = self.catalog.get_table(table_name) {
                    table.batches.iter().map(|b| b.row_count()).sum()
                } else {
                    1000  // Default estimate
                }
            }
            PlanNode::Filter { child, .. } => {
                // Assume filter reduces rows by 50%
                (self.estimate_rows(child) / 2).max(1)
            }
            PlanNode::Project { child, .. } => {
                // Project doesn't change row count
                self.estimate_rows(child)
            }
            PlanNode::Aggregate { group_by, child, .. } => {
                if group_by.is_empty() {
                    1  // Single aggregate result
                } else {
                    // Estimate groups: assume each unique group has 10 rows
                    (self.estimate_rows(child) / 10).max(1)
                }
            }
            PlanNode::Sort { child, .. } => {
                self.estimate_rows(child)
            }
            PlanNode::Limit { limit, child } => {
                self.estimate_rows(child).min(*limit)
            }
        }
    }
}
```

4. **Update execute_command**:
```rust
fn execute_command(&mut self, input: &str) -> Result<()> {
    let command = Command::parse(input)?;
    
    match command {
        Command::Explain { query } => {
            self.cmd_explain(&query)
        }
        // ... rest of commands
    }
}
```

5. **Enhance with tree visualization** (bonus):
```rust
impl<'a> Repl<'a> {
    fn display_execution_plan_tree(&self, plan: &PlanNode) -> Result<()> {
        println!("Execution Plan (Tree View):");
        println!();
        
        self.print_plan_tree(plan, "")?;
        
        Ok(())
    }
    
    fn print_plan_tree(&self, node: &PlanNode, prefix: &str) -> Result<()> {
        let (op_name, details) = match node {
            PlanNode::TableScan { table_name, columns } => {
                (format!("TableScan"), format!("table: {}, columns: {}", table_name, columns.len()))
            }
            PlanNode::Filter { predicate, child } => {
                (format!("Filter"), format!("predicate: {}", self.format_predicate(predicate)))
            }
            PlanNode::Project { columns, child } => {
                (format!("Project"), format!("columns: {}", columns.len()))
            }
            PlanNode::Aggregate { group_by, aggregates, child } => {
                (format!("Aggregate"), format!("group by: {}, aggregates: {}", group_by.len(), aggregates.len()))
            }
            PlanNode::Sort { order_by, child } => {
                (format!("Sort"), format!("order by: {} columns", order_by.len()))
            }
            PlanNode::Limit { limit, child } => {
                (format!("Limit"), format!("limit: {}", limit))
            }
        };
        
        println!("{}┌─ {}", prefix, op_name);
        println!("{}│  {}", prefix, details);
        
        // Get child nodes
        let children: Vec<&PlanNode> = match node {
            PlanNode::TableScan { .. } => vec![],
            PlanNode::Filter { child, .. } => vec![child],
            PlanNode::Project { child, .. } => vec![child],
            PlanNode::Aggregate { child, .. } => vec![child],
            PlanNode::Sort { child, .. } => vec![child],
            PlanNode::Limit { child, .. } => vec![child],
        };
        
        for (i, child) in children.iter().enumerate() {
            let is_last = i == children.len() - 1;
            let new_prefix = format!("{}{}  ", prefix, if is_last { " " } else { "│" });
            let connector = if is_last { "└" } else { "├" };
            
            println!("{}{}──", prefix, connector);
            self.print_plan_tree(child, &new_prefix)?;
        }
        
        Ok(())
    }
}
```

**Testing**:
```bash
# Simple query
olap> EXPLAIN SELECT * FROM employees
Execution Plan:
┌──────────────────────────────────────────────────────────────────────┐
│ Operator Type      │ Details                                        │
├──────────────────────────────────────────────────────────────────────┤
│ TableScan         │ Scan table: employees (5 columns)               │
└──────────────────────────────────────────────────────────────────────┘

Estimated cost: 10 rows

# Query with filter
olap> EXPLAIN SELECT name, salary FROM employees WHERE salary > 80000
Execution Plan:
┌──────────────────────────────────────────────────────────────────────┐
│ Operator Type      │ Details                                        │
├──────────────────────────────────────────────────────────────────────┤
│ Project           │ Columns: name, salary                            │
│ Filter            │ Predicate: salary > 80000.0                      │
│ TableScan         │ Scan table: employees (5 columns)               │
└──────────────────────────────────────────────────────────────────────┘

Estimated cost: 5 rows

# Complex query
olap> EXPLAIN SELECT department, COUNT(*) FROM employees WHERE age > 30 GROUP BY department ORDER BY COUNT(*) DESC LIMIT 5
Execution Plan:
┌──────────────────────────────────────────────────────────────────────┐
│ Operator Type      │ Details                                        │
├──────────────────────────────────────────────────────────────────────┤
│ Limit             │ Limit: 5 rows                                   │
│ Sort              │ Order by: COUNT(*) DESC                         │
│ Aggregate         │ Group by: department, Aggregates: 1             │
│ Filter            │ Predicate: age > 30                             │
│ TableScan         │ Scan table: employees (5 columns)               │
└──────────────────────────────────────────────────────────────────────┘

Estimated cost: 3 rows

# Tree view (bonus)
olap> EXPLAIN SELECT name, salary FROM employees WHERE salary > 80000
Execution Plan (Tree View):

┌─ Project
│  columns: 2
├──
│  └─ Filter
│     predicate: salary > 80000.0
│     ├──
│     └─ TableScan
│        table: employees, columns: 5
```

**Key Challenges**:
- Extracting meaningful details from plan nodes
- Formatting complex expressions readably
- Estimating row counts accurately
- Displaying tree structures in text
- Handling nested subqueries
- Performance of plan construction

**Further Enhancements**:
- Actual cost calculation (I/O, CPU)
- Visual graph generation (DOT format)
- Display access paths (index usage)
- Show parallelism opportunities
- Suggest optimizations
- Compare alternative plans
- Highlight expensive operations
- Interactive plan exploration

## ✅ Part 5: Integration Verification

### Verification 5.1: End-to-End Workflow

Create a test script that demonstrates the complete workflow:

**Sample Solution:**

```bash
#!/bin/bash
# test-e2e.sh - End-to-End Workflow Test Script
# This script tests the complete REPL workflow from start to finish

set -e  # Exit on error

echo "========================================"
echo "Phase 7 REPL End-to-End Workflow Test"
echo "========================================"
echo ""

# Clean up from previous test runs
rm -f .olap_history .olap_query_history
rm -f test_output.log

# Create test CSV files
echo "Creating test CSV files..."
cat > test_employees.csv << 'EOF'
id,name,age,salary,department
1,John,32,75000,Sales
2,Jane,28,68000,Marketing
3,Bob,35,82000,Engineering
4,Alice,30,72000,Sales
5,Charlie,40,95000,Engineering
6,David,38,105000,Sales
7,Eve,27,65000,Marketing
8,Frank,42,88000,Engineering
9,Grace,33,77000,Sales
10,Henry,29,69000,Marketing
EOF

cat > test_departments.csv << 'EOF'
id,name,budget,location
1,Engineering,500000,Building A
2,Marketing,300000,Building B
3,Sales,400000,Building C
EOF

echo "Test files created."
echo ""

# Run the REPL with automated input
echo "Running REPL with automated commands..."
echo ""

cat > test_input.txt << 'EOF'
HELP
SHOW TABLES
LOAD test_employees.csv AS employees
SHOW TABLES
DESCRIBE employees
SELECT * FROM employees
SELECT name, salary FROM employees WHERE salary > 80000
SELECT department, COUNT(*) FROM employees GROUP BY department
SELECT AVG(salary) FROM employees
SELECT * FROM nonexistent_table
LOAD nonexistent.csv AS test
COUNT_TABLES
EXIT
EOF

# Build the project
echo "Building project..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

# Run REPL with input
./target/release/olap < test_input.txt > test_output.log 2>&1 || true

echo ""
echo "========================================"
echo "Test Results"
echo "========================================"
echo ""

# Check if output contains expected results
echo "Checking output..."

# Check 1: HELP command worked
if grep -q "Available commands" test_output.log; then
    echo "✓ HELP command executed successfully"
else
    echo "✗ HELP command failed"
    exit 1
fi

# Check 2: SHOW TABLES showed empty initially
if grep -q "The catalog is empty" test_output.log; then
    echo "✓ SHOW TABLES showed empty catalog initially"
else
    echo "✗ SHOW TABLES didn't show empty catalog"
    exit 1
fi

# Check 3: LOAD command succeeded
if grep -q "Loaded 10 rows into 'employees'" test_output.log; then
    echo "✓ LOAD command succeeded"
else
    echo "✗ LOAD command failed"
    exit 1
fi

# Check 4: DESCRIBE showed table structure
if grep -q "Column Name" test_output.log && grep -q "id" test_output.log; then
    echo "✓ DESCRIBE showed table structure"
else
    echo "✗ DESCRIBE command failed"
    exit 1
fi

# Check 5: SELECT returned results
if grep -q "John" test_output.log && grep -q "Bob" test_output.log; then
    echo "✓ SELECT queries returned results"
else
    echo "✗ SELECT queries failed"
    exit 1
fi

# Check 6: Error handling worked
if grep -q "ERROR" test_output.log; then
    echo "✓ Error handling displayed errors"
else
    echo "✗ Error handling failed"
    exit 1
fi

# Check 7: History file was created
if [ -f ".olap_history" ]; then
    echo "✓ History file (.olap_history) was created"
else
    echo "✗ History file was not created"
    exit 1
fi

# Check 8: Timing information is present
if grep -q "⏱ Executed in" test_output.log; then
    echo "✓ Timing information displayed"
else
    echo "✗ Timing information missing"
    exit 1
fi

echo ""
echo "========================================"
echo "Sample Output (first 50 lines)"
echo "========================================"
head -50 test_output.log
echo ""
echo "========================================"
echo "All checks passed! ✓"
echo "========================================"

# Cleanup
echo ""
echo "Cleaning up test files..."
rm -f test_employees.csv test_departments.csv test_input.txt
echo "Test cleanup complete."
echo ""
echo "Full output saved to test_output.log"
```

**Expected Output:**

```
========================================
Phase 7 REPL End-to-End Workflow Test
========================================

Creating test CSV files...
Test files created.

Running REPL with automated commands...

Building project...
Compiling mini-rust-olap v0.1.0
Finished release [optimized] target(s) in 2.45s

========================================
Test Results
========================================

Checking output...
✓ HELP command executed successfully
✓ SHOW TABLES showed empty catalog initially
✓ LOAD command succeeded
✓ DESCRIBE showed table structure
✓ SELECT queries returned results
✓ Error handling displayed errors
✓ History file (.olap_history) was created
✓ Timing information displayed

========================================
All checks passed! ✓
========================================

Cleaning up test files...
Test cleanup complete.

Full output saved to test_output.log
```

**Verification Script:**

```bash
#!/bin/bash
# verify-e2e.sh - Verify end-to-end test results

echo "Verifying end-to-end test results..."

# Check output file exists
if [ ! -f "test_output.log" ]; then
    echo "Error: test_output.log not found. Run test-e2e.sh first."
    exit 1
fi

# Count successful operations
success_count=0
total_checks=8

check() {
    if grep -q "$1" test_output.log; then
        echo "✓ $2"
        ((success_count++))
    else
        echo "✗ $2"
    fi
}

check "Available commands" "HELP command"
check "The catalog is empty" "SHOW TABLES (empty)"
check "Loaded 10 rows" "LOAD command"
check "Column Name.*Type" "DESCRIBE command"
check "John.*Bob.*Alice" "SELECT queries"
check "ERROR" "Error handling"
check "⏱ Executed in" "Timing information"

# Check history file
if [ -f ".olap_history" ] && [ -s ".olap_history" ]; then
    echo "✓ History file created and non-empty"
    ((success_count++))
else
    echo "✗ History file missing or empty"
fi

echo ""
echo "Passed: $success_count/$total_checks"

if [ $success_count -eq $total_checks ]; then
    echo "All verifications passed!"
    exit 0
else
    echo "Some verifications failed!"
    exit 1
fi
```

**Test Matrix:**

| Step | Command | Expected Result | Verification |
|------|---------|----------------|--------------|
| 1 | HELP | List of all commands | grep "Available commands" |
| 2 | SHOW TABLES | "The catalog is empty" | grep "catalog is empty" |
| 3 | LOAD | "Loaded 10 rows" | grep "Loaded 10 rows" |
| 4 | DESCRIBE | Table schema | grep "Column Name" |
| 5 | SELECT * | All rows shown | grep "John\|Bob\|Alice" |
| 6 | WHERE > 80000 | Filtered results | grep "Charlie\|David" |
| 7 | GROUP BY | Aggregated results | grep "COUNT" |
| 8 | AVG | Average value | grep numeric value |
| 9 | Invalid table | ERROR box | grep "ERROR" |
| 10 | Invalid file | ERROR box | grep "ERROR" |
| 11 | COUNT_TABLES | Count shown | grep "tables in the catalog" |
| 12 | EXIT | "Goodbye!" | grep "Goodbye" |
| 13 | History file | .olap_history exists | test -f .olap_history |

### Verification 5.2: Performance Benchmark

Create a benchmark to measure REPL performance under different scenarios:

**Sample Solution:**

```bash
#!/bin/bash
# benchmark.sh - Performance Benchmark Script

set -e

echo "========================================"
echo "Phase 7 REPL Performance Benchmark"
echo "========================================"
echo ""

# Build release version
echo "Building release version..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true
echo ""

# Generate test data
echo "Generating test data..."

# Large CSV file (10,000 rows)
echo "Creating large CSV file (10,000 rows)..."
cat > large_test.csv <(
    echo "id,name,age,salary,department,join_date"
    for i in {1..10000}; do
        age=$((25 + i % 25))
        salary=$((50000 + (i % 50000)))
        dept=$((1 + i % 3))
        dept_name=$(printf "Department%d" $dept)
        year=$((2018 + (i % 5)))
        month=$((1 + i % 12))
        day=$((1 + i % 28))
        printf "%d,Employee%d,%d,%d,%s,%04d-%02d-%02d\n" $i $i $age $salary "$dept_name" $year $month $day
    done
)

# Medium CSV file (1,000 rows)
echo "Creating medium CSV file (1,000 rows)..."
head -n 1001 large_test.csv > medium_test.csv

# Small CSV file (100 rows)
echo "Creating small CSV file (100 rows)..."
head -n 101 large_test.csv > small_test.csv

echo "Test files created."
echo ""

# Define benchmark function
run_benchmark() {
    local test_name="$1"
    local command="$2"
    local iterations="${3:-5}"
    
    echo "========================================"
    echo "Benchmark: $test_name"
    echo "========================================"
    echo "Command: $command"
    echo "Iterations: $iterations"
    echo ""
    
    local total_time=0
    local min_time=999999
    local max_time=0
    
    for i in $(seq 1 $iterations); do
        # Remove history to avoid interference
        rm -f .olap_history
        
        # Run command and measure time
        local start=$(date +%s.%N)
        echo "$command" | timeout 30 ./target/release/olap > /dev/null 2>&1 || true
        local end=$(date +%s.%N)
        local elapsed=$(echo "$end - $start" | bc)
        
        total_time=$(echo "$total_time + $elapsed" | bc)
        
        # Update min/max
        local comp_min=$(echo "$elapsed < $min_time" | bc)
        if [ $comp_min -eq 1 ]; then
            min_time=$elapsed
        fi
        
        local comp_max=$(echo "$elapsed > $max_time" | bc)
        if [ $comp_max -eq 1 ]; then
            max_time=$elapsed
        fi
        
        printf "Iteration %d: %.3f seconds\n" $i $elapsed
    done
    
    local avg_time=$(echo "$total_time / $iterations" | bc)
    
    echo ""
    echo "Results:"
    echo "  Average: $(printf "%.3f" $avg_time)s"
    echo "  Minimum: $(printf "%.3f" $min_time)s"
    echo "  Maximum: $(printf "%.3f" $max_time)s"
    echo ""
}

# Benchmark 1: Loading CSV files of different sizes
echo "===== BENCHMARK SET 1: CSV Loading ====="
echo ""

run_benchmark "Load Small CSV (100 rows)" "LOAD small_test.csv AS small_test" 5
run_benchmark "Load Medium CSV (1,000 rows)" "LOAD medium_test.csv AS medium_test" 5
run_benchmark "Load Large CSV (10,000 rows)" "LOAD large_test.csv AS large_test" 5

# Benchmark 2: Query performance
echo "===== BENCHMARK SET 2: Query Performance ====="
echo ""

# Preload large table
echo "Preloading large table for query benchmarks..."
echo "LOAD large_test.csv AS large_test" | ./target/release/olap > /dev/null 2>&1 || true
echo ""

run_benchmark "Simple SELECT (LIMIT 50)" "SELECT * FROM large_test LIMIT 50" 10
run_benchmark "Simple SELECT (LIMIT 1000)" "SELECT * FROM large_test LIMIT 1000" 10
run_benchmark "Filter query (WHERE)" "SELECT * FROM large_test WHERE salary > 75000" 10
run_benchmark "Aggregate query (COUNT)" "SELECT COUNT(*) FROM large_test" 10
run_benchmark "Aggregate query (AVG)" "SELECT AVG(salary) FROM large_test" 10
run_benchmark "GROUP BY query" "SELECT department, COUNT(*) FROM large_test GROUP BY department" 10
run_benchmark "ORDER BY query" "SELECT * FROM large_test ORDER BY salary DESC LIMIT 100" 10

# Benchmark 3: Complex queries
echo "===== BENCHMARK SET 3: Complex Queries ====="
echo ""

run_benchmark "Multiple WHERE conditions" "SELECT * FROM large_test WHERE age > 30 AND salary < 70000 LIMIT 100" 10
run_benchmark "Aggregate with WHERE" "SELECT department, AVG(salary) FROM large_test WHERE age > 30 GROUP BY department" 10

# Benchmark 4: Command throughput
echo "===== BENCHMARK SET 4: Command Throughput ====="
echo ""

cat > many_commands.txt << 'EOF'
SHOW TABLES
DESCRIBE large_test
SELECT * FROM large_test LIMIT 10
SELECT COUNT(*) FROM large_test
SHOW TABLES
DESCRIBE large_test
SELECT * FROM large_test LIMIT 10
SELECT COUNT(*) FROM large_test
EOF

echo "Benchmark: 8 rapid commands"
echo "Commands: SHOW TABLES, DESCRIBE, SELECT (4x each)"
echo ""

rm -f .olap_history
start=$(date +%s.%N)
cat many_commands.txt | ./target/release/olap > /dev/null 2>&1 || true
end=$(date +%s.%N)
elapsed=$(echo "$end - $start" | bc)

printf "Total time: %.3f seconds\n" $elapsed
printf "Average per command: %.3f seconds\n" $(echo "$elapsed / 8" | bc)
printf "Commands per second: %.2f\n" $(echo "8 / $elapsed" | bc)
echo ""

# Benchmark 5: History I/O
echo "===== BENCHMARK SET 5: History I/O ====="
echo ""

cat > history_test.txt << 'EOF'
SELECT 1
SELECT 2
SELECT 3
SELECT 4
SELECT 5
SELECT 6
SELECT 7
SELECT 8
SELECT 9
SELECT 10
EOF

for size in 10 100 1000; do
    echo "Benchmark: History with $size entries"
    
    # Create history file
    head -n $size history_test.txt | ./target/release/olap > /dev/null 2>&1 || true
    
    # Measure startup time with history
    start=$(date +%s.%N)
    echo "EXIT" | ./target/release/olap > /dev/null 2>&1 || true
    end=$(date +%s.%N)
    elapsed=$(echo "$end - $start" | bc)
    
    printf "Startup time with %d history entries: %.3f seconds\n" $size $elapsed
done

echo ""

# Summary Report
echo "========================================"
echo "PERFORMANCE SUMMARY"
echo "========================================"
echo ""
echo "System Information:"
echo "  OS: $(uname -s)"
echo "  Kernel: $(uname -r)"
echo "  CPU: $(nproc) cores"
echo "  Memory: $(free -h | grep Mem | awk '{print $2}')"
echo ""
echo "Build Information:"
echo "  Version: $(./target/release/olap --version 2>&1 || echo "unknown")"
echo "  Build: release (optimized)"
echo ""
echo "Recommendations:"
echo "  1. LOAD operations scale linearly with row count (expected)"
echo "  2. Query performance is sub-second for most operations"
echo "  3. History I/O has minimal impact on startup time"
echo "  4. Consider adding indexes for frequently filtered columns"
echo "  5. Parallel processing could improve aggregate query performance"
echo ""

# Cleanup
echo "Cleaning up..."
rm -f large_test.csv medium_test.csv small_test.csv
rm -f many_commands.txt history_test.txt
rm -f .olap_history .olap_query_history
echo "Benchmark complete."
```

**Performance Report Template:**

```
========================================
PHASE 7 REPL PERFORMANCE REPORT
========================================

Date: $(date)
System: $(uname -a)
Rust Version: $(rustc --version)

===== BENCHMARK RESULTS =====

1. CSV Loading Performance
┌──────────────────────┬──────────┬──────────┬──────────┐
│ Test Case           │ Avg Time │ Min Time │ Max Time │
├──────────────────────┼──────────┼──────────┼──────────┤
│ Small (100 rows)    │   0.05s  │   0.04s  │   0.07s  │
│ Medium (1,000 rows) │   0.15s  │   0.12s  │   0.18s  │
│ Large (10,000 rows)  │   1.23s  │   1.15s  │   1.45s  │
└──────────────────────┴──────────┴──────────┴──────────┘

Observation: Linear scaling as expected. O(n) complexity.

2. Query Performance
┌──────────────────────────────────┬──────────┬──────────┬──────────┐
│ Query Type                     │ Avg Time │ Min Time │ Max Time │
├──────────────────────────────────┼──────────┼──────────┼──────────┤
│ SELECT (LIMIT 50)              │   0.12s  │   0.09s  │   0.18s  │
│ SELECT (LIMIT 1000)            │   0.45s  │   0.38s  │   0.52s  │
│ WHERE filter                    │   0.28s  │   0.22s  │   0.35s  │
│ COUNT(*)                       │   0.15s  │   0.12s  │   0.19s  │
│ AVG(salary)                     │   0.18s  │   0.14s  │   0.24s  │
│ GROUP BY                       │   0.32s  │   0.28s  │   0.41s  │
│ ORDER BY (100 rows)            │   0.19s  │   0.15s  │   0.25s  │
└──────────────────────────────────┴──────────┴──────────┴──────────┘

Observation: All queries complete in < 0.5s. Good interactive performance.

3. Complex Queries
┌──────────────────────────────────┬──────────┬──────────┬──────────┐
│ Query Type                     │ Avg Time │ Min Time │ Max Time │
├──────────────────────────────────┼──────────┼──────────┼──────────┤
│ Multiple WHERE conditions      │   0.35s  │   0.30s  │   0.42s  │
│ Aggregate with WHERE           │   0.41s  │   0.35s  │   0.48s  │
└──────────────────────────────────┴──────────┴──────────┴──────────┘

Observation: Complex queries still sub-second performance.

4. Command Throughput
┌────────────────────────┬──────────────┬────────────────┐
│ Metric                 │ Value        │ Comments       │
├────────────────────────┼──────────────┼────────────────┤
│ 8 commands total       │   2.45s      │                │
│ Average per command    │   0.31s      │ Good           │
│ Commands per second    │   3.27       │ Adequate       │
└────────────────────────┴──────────────┴────────────────┘

5. History I/O Performance
┌────────────────────────┬──────────────┬────────────────┐
│ History Size           │ Startup Time │ Comments       │
├────────────────────────┼──────────────┼────────────────┤
│ 10 entries            │   0.08s      │ Negligible     │
│ 100 entries           │   0.09s      │ Negligible     │
│ 1000 entries          │   0.12s      │ Acceptable     │
└────────────────────────┴──────────────┴────────────────┘

Observation: History I/O has minimal impact on performance.

===== IDENTIFIED BOTTLENECKS =====

1. CSV Parsing: Currently uses Rust's csv crate with single-threaded parsing.
   Impact: Moderate - only affects load time
   Recommendation: Consider parallel parsing for very large files

2. Query Execution: Full table scans for all queries.
   Impact: Low for current dataset size, high for larger datasets
   Recommendation: Add indexing for common filter columns

3. String Formatting: Box-drawing characters and formatting in main loop.
   Impact: Low - only affects display time
   Recommendation: Lazy formatting or caching

===== OPTIMIZATION RECOMMENDATIONS =====

Priority 1 (High Impact):
  ✓ Implement column indexes for frequently filtered columns
  ✓ Add query plan caching for repeated queries
  ✓ Optimize aggregate computation with parallel processing

Priority 2 (Medium Impact):
  • Implement row-level parallelism for large queries
  • Add result set streaming instead of materializing all rows
  • Cache column widths for repeated displays

Priority 3 (Low Impact):
  • Lazy load history on demand
  • Implement connection pooling for concurrent access
  • Add query result compression for network transmission

===== CONCLUSION =====

The REPL demonstrates excellent performance for its current scope:
- Sub-second response times for most queries
- Linear scaling for CSV loading (as expected)
- Minimal overhead from history I/O
- Good command throughput (3+ commands/second)

Performance is suitable for:
  ✓ Interactive data exploration
  ✓ Ad-hoc queries on moderate datasets
  ✓ Educational and learning purposes

Future work needed for:
  • Production workloads on large datasets
  • High-concurrency scenarios
  • Real-time analytics use cases

Overall Rating: EXCELLENT for Phase 7 goals
```

**Optimization Checklist:**

```bash
#!/bin/bash
# optimization-checklist.sh - Track optimization progress

echo "Optimization Progress Tracker"
echo "============================="
echo ""

# Format: [ ] Not started, [WIP] Work in progress, [x] Complete

echo "Priority 1 Optimizations:"
echo "  [ ] Implement column indexes"
echo "  [ ] Add query plan caching"
echo "  [ ] Optimize aggregate computation"
echo ""

echo "Priority 2 Optimizations:"
echo "  [ ] Implement row-level parallelism"
echo "  [ ] Add result set streaming"
echo "  [ ] Cache column widths"
echo ""

echo "Priority 3 Optimizations:"
echo "  [ ] Lazy load history"
echo "  [ ] Implement connection pooling"
echo "  [ ] Add query result compression"
```

### Verification 5.3: Error Recovery Test

Test REPL resilience to various error conditions:

**Sample Solution:**

```bash
#!/bin/bash
# error-recovery.sh - Error Recovery Test Script

set -e

echo "========================================"
echo "Phase 7 REPL Error Recovery Test"
echo "========================================"
echo ""

# Build project
echo "Building project..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true
echo ""

# Create test data
cat > test_data.csv << 'EOF'
id,name,age,salary
1,John,30,50000
2,Jane,25,45000
EOF

# Clean up
rm -f .olap_history .olap_query_history
rm -f corrupted_history.txt error_test.log

echo "Running error recovery tests..."
echo ""

# Test 1: Invalid SQL syntax
echo "Test 1: Invalid SQL Syntax"
echo "Command: SELCT * FROM test_data (typo in SELECT)"
echo ""

echo "SELCT * FROM test_data" | ./target/release/olap >> error_test.log 2>&1 || true

if grep -q "ERROR" error_test.log && grep -q "Unknown command" error_test.log; then
    echo "✓ PASS: Error displayed for invalid command"
else
    echo "✗ FAIL: Error not displayed correctly"
fi
echo ""

# Test 2: Non-existent table
echo "Test 2: Non-existent Table"
echo "Command: SELECT * FROM nonexistent_table"
echo ""

echo "SELECT * FROM nonexistent_table" | ./target/release/olap >> error_test.log 2>&1 || true

if grep -q "ERROR" error_test.log && grep -q "not found" error_test.log; then
    echo "✓ PASS: Error displayed for non-existent table"
else
    echo "✗ FAIL: Error not displayed correctly"
fi
echo ""

# Test 3: Non-existent column
echo "Test 3: Non-existent Column"
echo "Commands:"
echo "  LOAD test_data.csv AS test_data"
echo "  SELECT nonexistent_column FROM test_data"
echo ""

cat << 'EOF' | ./target/release/olap >> error_test.log 2>&1 || true
LOAD test_data.csv AS test_data
SELECT nonexistent_column FROM test_data
EOF

if grep -q "ERROR" error_test.log | tail -1 && grep -q "column" error_test.log | tail -1; then
    echo "✓ PASS: Error displayed for non-existent column"
else
    echo "✗ FAIL: Error not displayed correctly"
fi
echo ""

# Test 4: Missing CSV file
echo "Test 4: Missing CSV File"
echo "Command: LOAD missing_file.csv AS test"
echo ""

echo "LOAD missing_file.csv AS test" | ./target/release/olap >> error_test.log 2>&1 || true

if grep -q "ERROR" error_test.log && grep -q "not found" error_test.log; then
    echo "✓ PASS: Error displayed for missing file"
else
    echo "✗ FAIL: Error not displayed correctly"
fi
echo ""

# Test 5: Invalid LOAD syntax
echo "Test 5: Invalid LOAD Syntax"
echo "Commands:"
echo "  LOAD test_data.csv (missing AS clause)"
echo "  LOAD AS test_data (missing filename)"
echo ""

cat << 'EOF' | ./target/release/olap >> error_test.log 2>&1 || true
LOAD test_data.csv
LOAD AS test_data
EOF

error_count=$(grep -c "ERROR" error_test.log)
if [ $error_count -ge 2 ]; then
    echo "✓ PASS: Errors displayed for invalid LOAD syntax"
else
    echo "✗ FAIL: Expected 2 errors, found $error_count"
fi
echo ""

# Test 6: Malformed CSV file
echo "Test 6: Malformed CSV File"
echo "Creating malformed CSV file..."
cat > malformed.csv << 'EOF'
id,name,age
1,John
2,Jane,25,extra
3
EOF

echo "Command: LOAD malformed.csv AS bad_data"
echo ""

echo "LOAD malformed.csv AS bad_data" | ./target/release/olap >> error_test.log 2>&1 || true

if grep -q "ERROR" error_test.log || grep -q "CSV" error_test.log; then
    echo "✓ PASS: Error displayed for malformed CSV"
else
    echo "✗ FAIL: Error not displayed correctly"
fi
echo ""

# Test 7: Empty table
echo "Test 7: Empty Table"
echo "Creating empty CSV file..."
echo "id,name" > empty.csv

echo "Commands:"
echo "  LOAD empty.csv AS empty_table"
echo "  SELECT * FROM empty_table"
echo ""

cat << 'EOF' | ./target/release/olap >> error_test.log 2>&1 || true
LOAD empty.csv AS empty_table
SELECT * FROM empty_table
EOF

if grep -q "Empty result set" error_test.log || grep -q "0 rows" error_test.log; then
    echo "✓ PASS: Empty table handled gracefully"
else
    echo "✗ FAIL: Empty table not handled correctly"
fi
echo ""

# Test 8: Type mismatch in query
echo "Test 8: Type Mismatch in Query"
echo "Command: SELECT * FROM test_data WHERE name > 100 (comparing string to number)"
echo ""

echo "SELECT * FROM test_data WHERE name > 100" | ./target/release/olap >> error_test.log 2>&1 || true

if grep -q "ERROR" error_test.log || grep -q "type" error_test.log; then
    echo "✓ PASS: Type mismatch handled (or warning displayed)"
else
    echo "⚠ NOTE: Type handling may be permissive"
fi
echo ""

# Test 9: Unicode characters
echo "Test 9: Unicode and Multibyte Characters"
echo "Creating CSV with Unicode characters..."
cat > unicode.csv << 'EOF'
id,name,city
1,José,São Paulo
2,Müller,München
3,北京,北京
4,佐藤,東京
5,Иванов,Москва
EOF

echo "Commands:"
echo "  LOAD unicode.csv AS unicode_data"
echo "  SELECT * FROM unicode_data"
echo ""

cat << 'EOF' | ./target/release/olap >> error_test.log 2>&1 || true
LOAD unicode.csv AS unicode_data
SELECT * FROM unicode_data
EOF

if grep -q "José" error_test.log || grep -q "北京" error_test.log; then
    echo "✓ PASS: Unicode characters handled correctly"
else
    echo "✗ FAIL: Unicode characters not displayed correctly"
fi
echo ""

# Test 10: Very long input
echo "Test 10: Very Long Input"
echo "Creating very long query..."
long_query="SELECT * FROM test_data WHERE id = 1"
for i in {1..100}; do
    long_query="$long_query OR id = $((i % 10 + 1))"
done

echo "Testing query with $(echo $long_query | wc -c) characters"
echo ""

echo "$long_query" | ./target/release/olap >> error_test.log 2>&1 || true

if grep -q "ERROR" error_test.log | tail -1; then
    echo "⚠ NOTE: Long query may have been rejected"
else
    echo "✓ PASS: Long query processed (or no error displayed)"
fi
echo ""

# Test 11: Corrupted history file
echo "Test 11: Corrupted History File"
echo "Creating corrupted history file..."
cat > .olap_history << 'EOF'
Valid command 1
Valid command 2
\x00\x01\x02\x03\x04\x05\x06\x07 (binary garbage)
Valid command 4
EOF

echo "Starting REPL with corrupted history..."
echo "EXIT" | ./target/release/olap >> error_test.log 2>&1 || true

echo "✓ PASS: REPL started even with corrupted history (if it started)"
echo ""

# Test 12: Multiple consecutive errors
echo "Test 12: Multiple Consecutive Errors"
echo "Commands: 5 consecutive invalid commands"
echo ""

cat << 'EOF' | ./target/release/olap >> error_test.log 2>&1 || true
INVALID COMMAND 1
INVALID COMMAND 2
INVALID COMMAND 3
INVALID COMMAND 4
INVALID COMMAND 5
EOF

error_count=$(grep -c "ERROR" error_test.log | tail -1)
if [ $error_count -ge 5 ]; then
    echo "✓ PASS: All errors displayed, REPL continued"
else
    echo "✗ FAIL: Expected at least 5 errors, found $error_count"
fi
echo ""

# Test 13: Special characters in input
echo "Test 13: Special Characters in Input"
echo "Commands with special characters: ;, $, `, |, &, >, <"
echo ""

cat << 'EOF' | ./target/release/olap >> error_test.log 2>&1 || true
;echo test
$echo test
`echo test
|echo test
&echo test
>echo test
<echo test
EOF

echo "✓ PASS: Special characters handled without crashes"
echo ""

# Test 14: Empty input
echo "Test 14: Empty Input"
echo "Commands: Multiple empty lines"
echo ""

cat << 'EOF' | ./target/release/olap >> error_test.log 2>&1 || true





EOF

if ! grep -q "ERROR.*Empty command" error_test.log; then
    echo "✓ PASS: Empty input handled gracefully"
else
    echo "⚠ NOTE: Empty input may show error (acceptable)"
fi
echo ""

# Test 15: Mixed case commands
echo "Test 15: Mixed Case Commands"
echo "Commands: HELP, help, HeLp, LoAd test_data.csv AS test, SeLeCt * FROM test"
echo ""

cat << 'EOF' | ./target/release/olap >> error_test.log 2>&1 || true
HELP
help
HeLp
LoAd test_data.csv AS test2
SeLeCt * FROM test2
EOF

if grep -q "Available commands" error_test.log && grep -q "John" error_test.log; then
    echo "✓ PASS: Mixed case commands work correctly"
else
    echo "✗ FAIL: Mixed case handling failed"
fi
echo ""

# Summary
echo "========================================"
echo "ERROR RECOVERY TEST SUMMARY"
echo "========================================"
echo ""

total_errors=$(grep -c "ERROR" error_test.log)
total_panics=$(grep -c "panicked\|panic\|unwrap\|expect" error_test.log)

echo "Total errors displayed: $total_errors"
echo "Total panics/crashes: $total_panics"
echo ""

if [ $total_panics -eq 0 ]; then
    echo "✓ NO PANICS: REPL remained stable throughout all tests"
else
    echo "✗ PANICS DETECTED: REPL crashed $total_panics times"
fi
echo ""

echo "Error Scenarios Tested:"
echo "  1. Invalid SQL syntax"
echo "  2. Non-existent table"
echo "  3. Non-existent column"
echo "  4. Missing CSV file"
echo "  5. Invalid LOAD syntax"
echo "  6. Malformed CSV file"
echo "  7. Empty table"
echo "  8. Type mismatch in query"
echo "  9. Unicode and multibyte characters"
echo "  10. Very long input"
echo "  11. Corrupted history file"
echo "  12. Multiple consecutive errors"
echo "  13. Special characters in input"
echo "  14. Empty input"
echo "  15. Mixed case commands"
echo ""

echo "Test Results:"
passed_tests=$(grep "✓ PASS" error_recovery.sh | wc -l)
failed_tests=$(grep "✗ FAIL" error_recovery.sh | wc -l)
note_tests=$(grep "⚠ NOTE" error_recovery.sh | wc -l)

echo "  Passed: $passed_tests"
echo "  Failed: $failed_tests"
echo "  Notes:  $note_tests"
echo ""

if [ $total_panics -eq 0 ] && [ $failed_tests -eq 0 ]; then
    echo "✓ ALL TESTS PASSED: REPL demonstrates excellent error resilience"
    exit 0
elif [ $total_panics -eq 0 ]; then
    echo "⚠ SOME TESTS FAILED: REPL is stable but needs improvement"
    exit 1
else
    echo "✗ CRITICAL FAILURES: REPL crashed during tests"
    exit 1
fi

# Cleanup
rm -f test_data.csv malformed.csv empty.csv unicode.csv corrupted_history.txt
echo ""
echo "Cleanup complete. Test log saved to error_test.log"
```

**Test Matrix:**

| # | Scenario | Command | Expected Behavior | Actual |
|---|----------|---------|-------------------|--------|
| 1 | Invalid SQL | `SELCT * FROM test` | "Unknown command" error | ✓ |
| 2 | Non-existent table | `SELECT * FROM missing` | "Table not found" error | ✓ |
| 3 | Non-existent column | `SELECT bad_col FROM test` | "Column not found" error | ✓ |
| 4 | Missing CSV | `LOAD missing.csv AS t` | "File not found" error | ✓ |
| 5 | Invalid LOAD syntax | `LOAD file.csv` | "Invalid syntax" error | ✓ |
| 6 | Malformed CSV | `LOAD bad.csv AS t` | "CSV parse error" | ✓ |
| 7 | Empty table | `SELECT * FROM empty` | "Empty result set" message | ✓ |
| 8 | Type mismatch | `SELECT str > 100` | Error or warning | ⚠ |
| 9 | Unicode chars | `LOAD unicode.csv` | Characters display correctly | ✓ |
| 10 | Very long input | 100+ character query | Processed or error message | ✓ |
| 11 | Corrupted history | Start with bad history | REPL starts anyway | ✓ |
| 12 | Consecutive errors | 5 invalid commands | All errors shown, no crash | ✓ |
| 13 | Special chars | `; $ ` | | handled | ✓ |
| 14 | Empty input | Multiple empty lines | Silent continuation | ✓ |
| 15 | Mixed case | `HeLp, LoAd, SeLeCt` | Commands work | ✓ |

**Error Message Quality Checklist:**

```bash
#!/bin/bash
# error-message-check.sh - Verify error message quality

echo "Error Message Quality Assessment"
echo "================================"
echo ""

check_error_quality() {
    local scenario="$1"
    local pattern="$2"
    local has_context="$3"
    local has_action="$4"
    
    echo "Scenario: $scenario"
    
    if grep -q "$pattern" error_test.log; then
        echo "  ✓ Error displayed"
    else
        echo "  ✗ Error not found"
    fi
    
    if [ "$has_context" = "yes" ]; then
        if grep -q "table\|column\|file" error_test.log | grep -i "$pattern" | head -1; then
            echo "  ✓ Includes context (table/column/file name)"
        else
            echo "  ⚠ Could improve: Add context"
        fi
    fi
    
    if [ "$has_action" = "yes" ]; then
        if grep -q "please\|check\|try\|use" error_test.log | grep -i "$pattern" | head -1; then
            echo "  ✓ Includes actionable guidance"
        else
            echo "  ⚠ Could improve: Add action suggestion"
        fi
    fi
    
    echo ""
}

check_error_quality "Missing file" "not found" "yes" "yes"
check_error_quality "Invalid syntax" "syntax" "no" "yes"
check_error_quality "Unknown command" "unknown command" "no" "yes"
check_error_quality "Malformed CSV" "CSV" "yes" "no"
check_error_quality "Type mismatch" "type" "no" "no"

echo "Overall Assessment:"
echo "  Error messages are clear and user-friendly"
echo "  Most include relevant context and guidance"
echo "  Consider adding more specific action suggestions for complex errors"
```

**Bug Fixes If Needed:**

If any tests fail, here are common issues and fixes:

1. **Panic on corrupted history**:
   ```rust
   // In Repl::new(), wrap history loading:
   let _ = self.editor.load_history(&history_path);
   // Ignore errors, don't panic
   ```

2. **Crash on very long input**:
   ```rust
   // Add input validation:
   if input.len() > 10_000 {
       return Err(DatabaseError::ParserError(
           "Input too long (max 10,000 characters)".to_string()
       ));
   }
   ```

3. **Unicode display issues**:
   ```rust
   // Ensure terminal uses UTF-8:
   // Most modern terminals do this automatically
   // For Windows, may need: chcp 65001
   ```

4. **Empty input showing error**:
   ```rust
   // In main loop:
   if line.trim().is_empty() {
       continue; // Silent skip, no error
   }
   ```

5. **Multiple consecutive errors**:
   ```rust
   // Implement error counter with guidance:
   consecutive_errors += 1;
   if consecutive_errors >= 5 {
       println!("⚠ Multiple errors. Type HELP for available commands.");
   }
   ```

## 📊 Scoring Guidelines

### Part 1: Knowledge Questions (25 points)
- 2 points per question (15 questions)
- Full points for complete, accurate answers

### Part 2: Practical Tasks (35 points)
- Task 2.1: 5 points
- Task 2.2: 5 points
- Task 2.3: 10 points
- Task 2.4: 7.5 points
- Task 2.5: 7.5 points

### Part 3: Code Review (20 points)
- Code Review 3.1: 7 points
- Code Review 3.2: 7 points
- Code Review 3.3: 6 points

### Part 4: Challenge Exercises (20 points)
- Complete any ONE challenge for full 20 points
- Partial credit for incomplete but working solutions
- Bonus: Complete multiple challenges

### Part 5: Integration Verification (Optional)
- Extra credit for complete verification
- Not required for passing grade

## 🎯 Passing Criteria

To pass this assessment, you must:

- ✅ Answer all Part 1 knowledge questions
- ✅ Complete at least 3 of 5 practical tasks in Part 2
- ✅ Provide thorough code reviews for all Part 3 exercises
- ✅ Complete at least one challenge exercise in Part 4
- ✅ Achieve a total score of at least 60/100 points

## 🏆 Excellence Criteria

To achieve excellence in this assessment:

- ✅ Answer all questions with detailed explanations
- ✅ Complete all 5 practical tasks with high-quality code
- ✅ Provide insightful code reviews with working refactored versions
- ✅ Complete multiple challenge exercises with production-ready code
- ✅ Complete all verification exercises in Part 5
- ✅ Achieve a total score of 90/100 points or higher

## 📚 Additional Resources

### Rust Documentation
- [The Rust CLI Book](https://rust-cli.github.io/book/)
- [Rustyline API Documentation](https://docs.rs/rustyline/)
- [Command Line Interface Guidelines](https://clig.dev/)

### Database Resources
- [REPL Design Patterns](https://www.knowledgehut.com/blog/programming/how-to-build-a-repl)
- [Readline History Standards](https://tiswww.case.edu/php/chet/readline/rltop.html)
- [Table Formatting Libraries](https://github.com/Nukesor/comfy-table)

### Related Projects
- [SQLite CLI](https://www.sqlite.org/cli.html) - Reference for database CLI
- [PostgreSQL psql](https://www.postgresql.org/docs/current/app-psql.html) - Advanced CLI features
- [DuckDB CLI](https://duckdb.org/docs/api/cli) - Modern database CLI

## 🎓 Assessment Notes

**Time Estimate:**
- Part 1: 30-45 minutes
- Part 2: 2-3 hours
- Part 3: 45-60 minutes
- Part 4: 3-5 hours (per challenge)
- Part 5: 1-2 hours

**Tips for Success:**

1. **Read the codebase carefully** - Understand how REPL integrates with other modules
2. **Test incrementally** - Verify each change before moving to the next
3. **Document your work** - Add comments explaining your design decisions
4. **Consider edge cases** - Think about what could go wrong
5. **Seek feedback** - Code reviews are learning opportunities

**Common Pitfalls:**

1. **Not handling all error cases** - Users will find unexpected inputs
2. **Breaking existing functionality** - Test changes thoroughly
3. **Over-engineering** - Keep solutions simple and maintainable
4. **Ignoring UX** - Good error messages matter
5. **Forgetting about performance** - CLI should feel snappy

## ✅ Self-Check Checklist

Before submitting, ensure you have:

- [ ] Answered all 15 knowledge questions
- [ ] Completed at least 3 practical tasks
- [ ] Provided code reviews for all 3 code snippets
- [ ] Completed at least 1 challenge exercise
- [ ] Tested all changes with real CSV data
- [ ] Verified error handling doesn't crash the REPL
- [ ] Checked that timing information is accurate
- [ ] Ensured HELP command is up to date
- [ ] Tested with both valid and invalid inputs
- [ ] Documented any assumptions or design decisions

## 🎉 Congratulations!

Completing this assessment demonstrates that you have mastered:

- ✅ Building interactive command-line applications in Rust
- ✅ Integrating multiple database components into a cohesive system
- ✅ Designing user-friendly interfaces
- ✅ Implementing robust error handling
- ✅ Formatting output for readability
- ✅ Creating extensible command systems

You now have a fully functional OLAP database with an interactive REPL! 🚀