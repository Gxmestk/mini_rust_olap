//! CSV Loading Example
//!
//! This example demonstrates how to load data from CSV files
//! and perform SQL queries using the mini_rust_olap library.

use mini_rust_olap::catalog::Catalog;
use mini_rust_olap::error::Result;
use mini_rust_olap::ingest::load_csv_into_catalog;
use mini_rust_olap::parser::Parser;
use mini_rust_olap::planner::Planner;

fn main() -> Result<()> {
    println!("🚀 Mini Rust OLAP - CSV Loading Example\n");

    // Create a new catalog
    let mut catalog = Catalog::new();

    // Load CSV file into the catalog
    let csv_path = "tests/data/test_data.csv";
    println!("📂 Loading data from: {}", csv_path);
    load_csv_into_catalog(csv_path, "employees".to_string(), &mut catalog)?;
    println!("✓ Data loaded successfully\n");

    // Example 1: Basic SELECT query
    println!("🔍 Example 1: SELECT * FROM employees");
    println!("----------------------------------------");
    execute_sql_query(&catalog, "SELECT * FROM employees")?;
    println!();

    // Example 2: SELECT with specific columns
    println!("🔍 Example 2: SELECT name, department, salary FROM employees");
    println!("------------------------------------------------------------");
    execute_sql_query(&catalog, "SELECT name, department, salary FROM employees")?;
    println!();

    // Example 3: WHERE clause
    println!("🔍 Example 3: SELECT * FROM employees WHERE salary > 80000");
    println!("---------------------------------------------------------");
    execute_sql_query(&catalog, "SELECT * FROM employees WHERE salary > 80000")?;
    println!();

    // Example 4: GROUP BY with COUNT
    println!("🔍 Example 4: SELECT department, COUNT(*) FROM employees GROUP BY department");
    println!("-----------------------------------------------------------------------------");
    execute_sql_query(
        &catalog,
        "SELECT department, COUNT(*) FROM employees GROUP BY department",
    )?;
    println!();

    // Example 5: GROUP BY with multiple aggregates
    println!(
        "🔍 Example 5: SELECT department, COUNT(*), AVG(salary) FROM employees GROUP BY department"
    );
    println!(
        "---------------------------------------------------------------------------------------"
    );
    execute_sql_query(
        &catalog,
        "SELECT department, COUNT(*), AVG(salary) FROM employees GROUP BY department",
    )?;
    println!();

    // Example 6: ORDER BY with LIMIT
    println!("🔍 Example 6: SELECT name, salary FROM employees ORDER BY salary DESC LIMIT 3");
    println!("------------------------------------------------------------------------------");
    execute_sql_query(
        &catalog,
        "SELECT name, salary FROM employees ORDER BY salary DESC LIMIT 3",
    )?;
    println!();

    // Example 7: Combined WHERE and GROUP BY
    println!("🔍 Example 7: SELECT department, COUNT(*) FROM employees WHERE salary > 70000 GROUP BY department");
    println!("---------------------------------------------------------------------------------------------");
    execute_sql_query(
        &catalog,
        "SELECT department, COUNT(*) FROM employees WHERE salary > 70000 GROUP BY department",
    )?;
    println!();

    // Example 8: String filtering
    println!("🔍 Example 8: SELECT * FROM employees WHERE department = 'Engineering'");
    println!("------------------------------------------------------------------------");
    execute_sql_query(
        &catalog,
        "SELECT * FROM employees WHERE department = 'Engineering'",
    )?;
    println!();

    println!("✅ All examples completed successfully!");
    Ok(())
}

/// Execute a SQL query and print the results
fn execute_sql_query(catalog: &Catalog, sql: &str) -> Result<()> {
    println!("SQL: {}", sql);

    // Parse the SQL query
    let mut parser = Parser::new(sql);
    let query = parser.parse()?;

    // Create a planner and plan the query
    let planner = Planner::new(catalog);
    let mut plan = planner.plan(&query)?;

    // Execute the query
    plan.open()
        .map_err(|e| mini_rust_olap::error::DatabaseError::execution_error(e.to_string()))?;

    let mut all_rows: Vec<Vec<String>> = Vec::new();

    // Collect all batches
    while let Some(batch) = plan
        .next_batch()
        .map_err(|e| mini_rust_olap::error::DatabaseError::execution_error(e.to_string()))?
    {
        // Convert batch to rows
        for row_idx in 0..batch.row_count() {
            let row: Vec<String> = (0..batch.column_count())
                .map(|col_idx| {
                    batch
                        .get_as_string(row_idx, col_idx)
                        .unwrap_or_else(|_| "".to_string())
                })
                .collect();
            all_rows.push(row);
        }
    }

    // Print the results
    print_rows(&all_rows);

    Ok(())
}

/// Print rows in a formatted table
fn print_rows(rows: &[Vec<String>]) {
    if rows.is_empty() {
        println!("  (No results)");
        return;
    }

    // Calculate column widths
    let num_cols = rows[0].len();
    let mut col_widths: Vec<usize> = vec![0; num_cols];

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(cell.len());
        }
    }

    // Print separator
    let separator: String = col_widths
        .iter()
        .map(|&w| format!("+{}", "-".repeat(w + 2)))
        .collect::<Vec<_>>()
        .join("");
    println!("  {}+", separator);

    // Print each row
    for row in rows {
        let formatted: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| format!("| {:<width$} ", cell, width = col_widths[i]))
            .collect();
        println!("  {}|", formatted.join(""));
    }

    // Print separator
    println!("  {}+", separator);
    println!("  {} rows returned", rows.len());
}
