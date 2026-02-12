//! Simple Table Example
//!
//! This example demonstrates how to create a table programmatically
//! using the Table API and then execute SQL queries on it.
//! This shows the internal Table structure used by the OLAP engine.

use mini_rust_olap::catalog::Catalog;
use mini_rust_olap::column::IntColumn;
use mini_rust_olap::column::StringColumn;
use mini_rust_olap::error::Result;
use mini_rust_olap::parser::Parser;
use mini_rust_olap::planner::Planner;
use mini_rust_olap::table::Table;

fn main() -> Result<()> {
    println!("🚀 Mini Rust OLAP - Simple Table Example\n");

    // Create a new catalog to manage our tables
    let mut catalog = Catalog::new();

    // Create a table with employee data programmatically
    println!("📋 Creating 'employees' table...");
    let table = create_employees_table()?;

    // Register the table in the catalog
    catalog.register_table(table)?;
    println!("✓ Table created and registered\n");

    // Example 1: Simple SELECT query
    println!("🔍 Example 1: SELECT * FROM employees");
    println!("----------------------------------------");
    execute_sql_query(&catalog, "SELECT * FROM employees")?;
    println!();

    // Example 2: SELECT with specific columns
    println!("🔍 Example 2: SELECT name, department, salary FROM employees");
    println!("---------------------------------------------------------");
    execute_sql_query(&catalog, "SELECT name, department, salary FROM employees")?;
    println!();

    // Example 3: WHERE clause
    println!("🔍 Example 3: SELECT * FROM employees WHERE salary > 85000");
    println!("--------------------------------------------------------");
    execute_sql_query(&catalog, "SELECT * FROM employees WHERE salary > 85000")?;
    println!();

    // Example 4: String filtering
    println!("🔍 Example 4: SELECT * FROM employees WHERE department = 'Engineering'");
    println!("-----------------------------------------------------------------------");
    execute_sql_query(
        &catalog,
        "SELECT * FROM employees WHERE department = 'Engineering'",
    )?;
    println!();

    // Example 5: GROUP BY with COUNT
    println!("🔍 Example 5: SELECT department, COUNT(*) FROM employees GROUP BY department");
    println!("-----------------------------------------------------------------------------");
    execute_sql_query(
        &catalog,
        "SELECT department, COUNT(*) FROM employees GROUP BY department",
    )?;
    println!();

    // Example 6: GROUP BY with multiple aggregates
    println!(
        "🔍 Example 6: SELECT department, COUNT(*), AVG(salary) FROM employees GROUP BY department"
    );
    println!(
        "----------------------------------------------------------------------------------------"
    );
    execute_sql_query(
        &catalog,
        "SELECT department, COUNT(*), AVG(salary) FROM employees GROUP BY department",
    )?;
    println!();

    // Example 7: ORDER BY with LIMIT
    println!("🔍 Example 7: SELECT name, salary FROM employees ORDER BY salary DESC LIMIT 3");
    println!("------------------------------------------------------------------------------");
    execute_sql_query(
        &catalog,
        "SELECT name, salary FROM employees ORDER BY salary DESC LIMIT 3",
    )?;
    println!();

    // Example 8: Combined WHERE and GROUP BY
    println!("🔍 Example 8: SELECT department, COUNT(*) FROM employees WHERE salary > 80000 GROUP BY department");
    println!("---------------------------------------------------------------------------------------------");
    execute_sql_query(
        &catalog,
        "SELECT department, COUNT(*) FROM employees WHERE salary > 80000 GROUP BY department",
    )?;
    println!();

    println!("✅ All examples completed successfully!");
    Ok(())
}

/// Create a sample employees table with test data
fn create_employees_table() -> Result<Table> {
    let mut table = Table::new("employees".to_string());

    // Add columns (note: name is passed to add_column, not to the column constructor)
    let id_column = IntColumn::new();
    table.add_column("id".to_string(), Box::new(id_column))?;

    let name_column = StringColumn::new();
    table.add_column("name".to_string(), Box::new(name_column))?;

    let department_column = StringColumn::new();
    table.add_column("department".to_string(), Box::new(department_column))?;

    let salary_column = IntColumn::new();
    table.add_column("salary".to_string(), Box::new(salary_column))?;

    // Add sample data (values are passed as Vec<String>)
    table.add_row(vec![
        "1".to_string(),
        "Alice".to_string(),
        "Engineering".to_string(),
        "90000".to_string(),
    ])?;
    table.add_row(vec![
        "2".to_string(),
        "Bob".to_string(),
        "Marketing".to_string(),
        "75000".to_string(),
    ])?;
    table.add_row(vec![
        "3".to_string(),
        "Charlie".to_string(),
        "Engineering".to_string(),
        "95000".to_string(),
    ])?;
    table.add_row(vec![
        "4".to_string(),
        "Diana".to_string(),
        "Sales".to_string(),
        "85000".to_string(),
    ])?;
    table.add_row(vec![
        "5".to_string(),
        "Eve".to_string(),
        "Engineering".to_string(),
        "88000".to_string(),
    ])?;
    table.add_row(vec![
        "6".to_string(),
        "Frank".to_string(),
        "Marketing".to_string(),
        "72000".to_string(),
    ])?;
    table.add_row(vec![
        "7".to_string(),
        "Grace".to_string(),
        "Sales".to_string(),
        "91000".to_string(),
    ])?;
    table.add_row(vec![
        "8".to_string(),
        "Henry".to_string(),
        "Engineering".to_string(),
        "82000".to_string(),
    ])?;
    table.add_row(vec![
        "9".to_string(),
        "Iris".to_string(),
        "Sales".to_string(),
        "78000".to_string(),
    ])?;
    table.add_row(vec![
        "10".to_string(),
        "Jack".to_string(),
        "Marketing".to_string(),
        "83000".to_string(),
    ])?;

    Ok(table)
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
