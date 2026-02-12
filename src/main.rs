//! # Mini Rust OLAP - REPL Interface
//!
//! This is the main entry point for the Mini Rust OLAP database.
//! It provides an interactive REPL (Read-Eval-Print Loop) for:
//! - Loading CSV data
//! - Executing SQL queries
//! - Managing tables
//! - Inspecting schemas

use mini_rust_olap::catalog::Catalog;
use mini_rust_olap::error::{DatabaseError, Result};
use mini_rust_olap::execution::Batch;
use mini_rust_olap::ingest::load_csv;
use mini_rust_olap::parser::Parser;
use mini_rust_olap::planner::Planner;
use mini_rust_olap::table::Table;
use rustyline::error::ReadlineError;
use rustyline::{history::FileHistory, Editor};
use std::time::Instant;

// ============================================================================
// REPL STRUCTURE
// ============================================================================

/// Main REPL structure that holds the database state
pub struct Repl {
    /// The catalog managing all tables
    catalog: Catalog,
    /// Readline editor for command history and editing
    editor: Editor<(), FileHistory>,
    /// Whether to continue the REPL loop
    running: bool,
}

impl Repl {
    /// Creates a new REPL instance
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut editor =
            Editor::<(), FileHistory>::new().expect("Failed to initialize readline editor");

        // Load command history if available
        if let Err(_e) = editor.load_history(".olap_history") {
            // History file doesn't exist yet, that's okay
            println!("No previous history found. Starting fresh.");
        }

        Self {
            catalog: Catalog::new(),
            editor,
            running: true,
        }
    }

    /// Returns whether the REPL is still running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Runs the main REPL loop
    pub fn run(&mut self) -> Result<()> {
        self.print_welcome();

        while self.running {
            let readline = self.editor.readline("olap> ");

            match readline {
                Ok(line) => {
                    // Save to history
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let _ = self.editor.add_history_entry(line);

                    // Process the command
                    if let Err(e) = self.process_command(line) {
                        self.print_error(&e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C - continue running
                    println!("Use EXIT or QUIT to exit.");
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D - exit
                    println!("Goodbye!");
                    self.running = false;
                }
                Err(err) => {
                    // Other errors
                    eprintln!("Error reading input: {}", err);
                    self.running = false;
                }
            }
        }

        // Save history before exiting
        if let Err(e) = self.editor.save_history(".olap_history") {
            eprintln!("Warning: Failed to save history: {}", e);
        }

        Ok(())
    }

    /// Processes a single command
    pub fn process_command(&mut self, input: &str) -> Result<()> {
        let start = Instant::now();

        // Parse the command
        let result = self.execute_command(input);

        let elapsed = start.elapsed();

        // Print timing if command was successful
        if result.is_ok() {
            self.print_timing(elapsed);
        }

        result
    }

    /// Executes a parsed command
    pub fn execute_command(&mut self, input: &str) -> Result<()> {
        let upper_input = input.to_uppercase();

        // Identify command type
        if upper_input.starts_with("LOAD ") {
            self.cmd_load(input)
        } else if upper_input.starts_with("SELECT ") || upper_input.starts_with("WITH ") {
            self.cmd_select(input)
        } else if upper_input == "SHOW TABLES" || upper_input == ".TABLES" {
            self.cmd_show_tables()
        } else if upper_input.starts_with("DESCRIBE ") || upper_input.starts_with(".SCHEMA ") {
            self.cmd_describe(input)
        } else if upper_input == "EXIT" || upper_input == "QUIT" || upper_input == ".EXIT" {
            self.cmd_exit()
        } else if upper_input == "HELP" || upper_input == ".HELP" || upper_input == "?" {
            self.cmd_help()
        } else if upper_input == "CLEAR" || upper_input == ".CLEAR" {
            self.cmd_clear()
        } else {
            Err(DatabaseError::parser_error(format!(
                "Unknown command: '{}'. Type HELP for available commands.",
                input
            )))
        }
    }

    // ========================================================================
    // COMMAND IMPLEMENTATIONS
    // ========================================================================

    /// LOAD command: Load a CSV file into the catalog
    /// Syntax: LOAD <path> AS <table_name>
    pub fn cmd_load(&mut self, input: &str) -> Result<()> {
        // Parse: LOAD <path> AS <table_name>
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.len() != 4 || parts[2].to_uppercase() != "AS" {
            return Err(DatabaseError::parser_error(
                "Invalid LOAD syntax. Use: LOAD <path> AS <table_name>".to_string(),
            ));
        }

        let path = parts[1];
        let table_name = parts[3];

        // Check if table already exists
        if self.catalog.table_exists(table_name) {
            return Err(DatabaseError::catalog_error(format!(
                "Table '{}' already exists. Drop it first if you want to reload.",
                table_name
            )));
        }

        println!("Loading CSV from '{}' as '{}'...", path, table_name);

        // Load the CSV
        let table = load_csv(path, table_name.to_string())?;
        self.catalog.register_table(table)?;

        println!("✓ Loaded table '{}' successfully.", table_name);
        Ok(())
    }

    /// SELECT command: Execute a SQL query
    /// Syntax: SELECT ...
    pub fn cmd_select(&mut self, input: &str) -> Result<()> {
        // Parse the SQL query
        let mut parser = Parser::new(input);
        let query = parser.parse()?;

        // Create a planner and plan the query
        let planner = Planner::new(&self.catalog);
        let mut plan = planner.plan(&query)?;

        // Execute the query
        plan.open()
            .map_err(|e| DatabaseError::execution_error(e.to_string()))?;
        let mut all_batches: Vec<Batch> = Vec::new();

        while let Some(batch) = plan
            .next_batch()
            .map_err(|e| DatabaseError::execution_error(e.to_string()))?
        {
            all_batches.push(batch);
        }

        // Display the results
        self.print_batches(&all_batches);

        Ok(())
    }

    /// SHOW TABLES command: List all tables in the catalog
    pub fn cmd_show_tables(&self) -> Result<()> {
        let tables = self.catalog.list_tables_sorted();

        if tables.is_empty() {
            println!("No tables in catalog.");
        } else {
            println!("Tables in catalog:");
            for table_name in tables {
                println!("  - {}", table_name);
            }
        }

        Ok(())
    }

    /// DESCRIBE command: Show the schema of a table
    /// Syntax: DESCRIBE <table_name>
    pub fn cmd_describe(&self, input: &str) -> Result<()> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.len() != 2 {
            return Err(DatabaseError::parser_error(
                "Invalid DESCRIBE syntax. Use: DESCRIBE <table_name>".to_string(),
            ));
        }

        let table_name = parts[1];
        let table = self.catalog.get_table(table_name)?;

        self.print_schema(table_name, table);
        Ok(())
    }

    /// EXIT command: Exit the REPL
    pub fn cmd_exit(&mut self) -> Result<()> {
        println!("Goodbye!");
        self.running = false;
        Ok(())
    }

    /// HELP command: Display help information
    pub fn cmd_help(&self) -> Result<()> {
        println!();
        println!("Mini Rust OLAP - Available Commands:");
        println!("═══════════════════════════════════════════");
        println!();
        println!("Data Loading:");
        println!("  LOAD <path> AS <table_name>      Load a CSV file into the catalog");
        println!();
        println!("Querying:");
        println!("  SELECT <columns> FROM <table>    Execute a SQL SELECT query");
        println!("  WHERE <condition>                Add filtering conditions");
        println!("  GROUP BY <columns>               Group results");
        println!("  ORDER BY <columns> [ASC|DESC]    Sort results");
        println!("  LIMIT <n>                        Limit number of rows");
        println!();
        println!("Catalog Management:");
        println!("  SHOW TABLES                       List all tables");
        println!("  DESCRIBE <table_name>             Show table schema");
        println!();
        println!("Utility:");
        println!("  HELP or ?                         Show this help message");
        println!("  CLEAR                             Clear screen");
        println!("  EXIT or QUIT                      Exit the REPL");
        println!();
        println!("Features:");
        println!("  • Columnar storage for fast analytics");
        println!("  • SQL-like query language");
        println!("  • Automatic type inference from CSV");
        println!("  • Aggregations: COUNT, SUM, AVG, MIN, MAX");
        println!();
        Ok(())
    }

    /// CLEAR command: Clear the terminal screen
    pub fn cmd_clear(&self) -> Result<()> {
        // ANSI escape code to clear screen
        print!("\x1B[2J\x1B[1;1H");
        Ok(())
    }

    // ========================================================================
    // OUTPUT FORMATTING
    // ========================================================================

    /// Prints a welcome message
    pub fn print_welcome(&self) {
        println!();
        println!("╔═════════════════════════════════════════════════════════╗");
        println!(
            "║     Mini Rust OLAP - Interactive REPL v{}             ║",
            env!("CARGO_PKG_VERSION")
        );
        println!("╚═════════════════════════════════════════════════════════╝");
        println!();
        println!("Welcome to Mini Rust OLAP! Type HELP for available commands.");
        println!();
    }

    /// Prints a table with ASCII formatting
    pub fn print_batches(&self, batches: &[Batch]) {
        let total_rows: usize = batches.iter().map(|b| b.row_count()).sum();

        if total_rows == 0 {
            println!("Empty result set.");
            return;
        }

        // Collect all column names from the first batch
        let first_batch = &batches[0];
        let column_count = first_batch.column_count();
        let mut column_names: Vec<String> = Vec::new();
        for i in 0..column_count {
            column_names.push(format!("col_{}", i));
        }

        let mut column_widths: Vec<usize> = column_names.iter().map(|s| s.len()).collect();

        // Calculate column widths based on data
        let mut global_row_idx = 0;
        for batch in batches {
            for (col_idx, width) in column_widths
                .iter_mut()
                .enumerate()
                .take(batch.column_count())
            {
                for row_idx in 0..batch.row_count() {
                    if global_row_idx >= 100 {
                        break;
                    }
                    if let Ok(value) = batch.get(row_idx, col_idx) {
                        *width = (*width).max(value.to_string().len());
                    }
                    global_row_idx += 1;
                }
            }
        }

        // Cap width to prevent very wide tables
        for width in &mut column_widths {
            *width = (*width).min(50);
        }

        // Calculate total width
        let total_width: usize = column_widths.iter().map(|&w| w + 3).sum::<usize>() + 1;

        // Print top border
        println!("┌{}┐", "─".repeat(total_width - 2));

        // Print header
        print!("│");
        for (col_name, &width) in column_names.iter().zip(column_widths.iter()) {
            print!(" {:width$} │", col_name, width = width);
        }
        println!();

        // Print separator
        println!("├{}┤", "─".repeat(total_width - 2));

        // Print data rows (limit to 50 rows)
        let max_rows = 50;
        let mut display_rows = 0;

        for batch in batches {
            let batch_row_count = batch.row_count();
            let rows_to_show = (max_rows - display_rows).min(batch_row_count);

            for row_idx in 0..rows_to_show {
                print!("│");
                for (col_idx, width) in column_widths.iter().enumerate().take(batch.column_count())
                {
                    if let Ok(value) = batch.get(row_idx, col_idx) {
                        print!(" {:width$} │", value.to_string(), width = width);
                    } else {
                        print!(" {:width$} │", "NULL", width = width);
                    }
                }
                println!();
                display_rows += 1;

                if display_rows >= max_rows {
                    break;
                }
            }

            if display_rows >= max_rows {
                break;
            }
        }

        // Print bottom border
        println!("└{}┘", "─".repeat(total_width - 2));

        // Print row count info
        if total_rows > max_rows {
            println!("({} rows total, showing first {})", total_rows, max_rows);
        } else {
            println!(
                "({} row{})",
                total_rows,
                if total_rows == 1 { "" } else { "s" }
            );
        }
    }

    /// Prints a table's schema
    pub fn print_schema(&self, table_name: &str, table: &Table) {
        let column_names: Vec<String> = table.column_names();
        let row_count = table.row_count();

        println!();
        println!("Table: {}", table_name);
        println!("┌────────────────────────┬──────────┬────────────────┐");
        println!("│ Column Name            │ Type     │ Description    │");
        println!("├────────────────────────┼──────────┼────────────────┤");

        for col_name in &column_names {
            let col = table.get_column(col_name).unwrap();
            let type_name = format!("{:?}", col.data_type());

            println!(
                "│ {:22} │ {:8} │ {:>12} rows │",
                col_name,
                type_name,
                col.len()
            );
        }

        println!("└────────────────────────┴──────────┴────────────────┘");
        println!("Total rows: {}", row_count);
        println!();
    }

    /// Prints an error message
    pub fn print_error(&self, error: &DatabaseError) {
        println!();
        println!("╔═════════════════════════════════════════════════════════╗");
        println!("║ ❌ ERROR                                                  ║");
        println!("╠═════════════════════════════════════════════════════════╣");
        println!("║ {}", error);
        println!("╚═════════════════════════════════════════════════════════╝");
        println!();
    }

    /// Prints timing information
    pub fn print_timing(&self, elapsed: std::time::Duration) {
        let millis = elapsed.as_secs_f64() * 1000.0;
        if millis >= 1000.0 {
            println!("⏱ Executed in {:.3}s", elapsed.as_secs_f64());
        } else {
            println!("⏱ Executed in {:.2}ms", millis);
        }
    }
}

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================

fn main() -> Result<()> {
    let mut repl = Repl::new();
    repl.run()
}
