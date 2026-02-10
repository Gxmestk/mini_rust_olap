//! # Ingestion Module
//!
//! This module provides functionality for ingesting CSV files into the Mini Rust OLAP database.
//!
//! ## Key Features
//!
//! - **CSV Parsing**: Reads CSV files and parses headers and data rows
//! - **Type Inference**: Automatically detects column data types (Int64, Float64, String)
//! - **Columnar Conversion**: Transforms row-based CSV data into columnar format
//! - **Error Handling**: Graceful handling of malformed CSVs and type conversion errors
//!
//! ## Design Philosophy
//!
//! The ingestion process follows these steps:
//! 1. Read and parse CSV file
//! 2. Extract header row for column names
//! 3. Analyze sample data to infer types for each column
//! 4. Create appropriate column instances based on inferred types
//! 5. Transpose row data into columns
//! 6. Build and return a Table
//!
//! ## Type Inference Strategy
//!
//! Type inference follows a hierarchical approach:
//! - **Int64**: Try to parse as integer (no decimal point, no scientific notation)
//! - **Float64**: If not integer, try to parse as float (with decimal point or scientific notation)
//! - **String**: If neither numeric type works, treat as string
//!
//! Empty values are handled specially:
//! - They're ignored during type inference
//! - When inserting, they're represented as appropriate "null" values for the column type
//!
//! ## Usage Example
//!
//! ```ignore
//! use mini_rust_olap::ingest::load_csv;
//! use mini_rust_olap::catalog::Catalog;
//! use mini_rust_olap::error::Result;
//!
//! fn main() -> Result<()> {
//!     let mut catalog = Catalog::new();
//!
//!     // Load CSV data into a table
//!     let table = load_csv("data.csv", "users")?;
//!
//!     // Register the table in the catalog
//!     catalog.register_table(table)?;
//!
//!     Ok(())
//! }
//! ```

use crate::catalog::Catalog;
use crate::column::create_column;
use crate::error::{DatabaseError, Result};
use crate::table::Table;
use crate::types::{DataType, Value};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// ============================================================================
// TYPE INFERENCE
// ============================================================================

/// Infers the data type of a column based on a sample of values.
///
/// This function examines a sample of string values and determines the most
/// appropriate data type. The inference follows these rules:
/// - If any value cannot be parsed as a number, the type is String
/// - If all numeric values can be parsed as integers without decimal points,
///   the type is Int64
/// - Otherwise, the type is Float64
///
/// Empty strings are ignored during type inference.
///
/// # Arguments
///
/// * `values` - A slice of string values to analyze
///
/// # Returns
///
/// The inferred DataType
///
/// # Example
///
/// ```ignore
/// use mini_rust_olap::ingest::infer_column_type;
/// use mini_rust_olap::types::DataType;
///
/// // All integers -> Int64
/// let int_values = vec!["1", "2", "3"];
/// assert_eq!(infer_column_type(&int_values), DataType::Int64);
///
/// // Floats -> Float64
/// let float_values = vec!["1.5", "2.7", "3.14"];
/// assert_eq!(infer_column_type(&float_values), DataType::Float64);
/// ```
pub fn infer_column_type(values: &[String]) -> DataType {
    if values.is_empty() {
        return DataType::String;
    }

    let mut has_decimal = false;
    let mut has_non_numeric = false;

    for value in values {
        let trimmed = value.trim();

        // Skip empty values during type inference
        if trimmed.is_empty() {
            continue;
        }

        // Check if it's an integer (no decimal point, no scientific notation)
        if trimmed.parse::<i64>().is_ok() {
            continue;
        }

        // Check if it's a float
        if trimmed.parse::<f64>().is_ok() {
            has_decimal = true;
            continue;
        }

        // If we get here, it's not numeric
        has_non_numeric = true;
        break;
    }

    // Determine the type based on our analysis
    if has_non_numeric {
        DataType::String
    } else if has_decimal {
        DataType::Float64
    } else {
        DataType::Int64
    }
}

/// Parses a string value into the appropriate Value type.
///
/// This function attempts to parse a string as the specified data type,
/// falling back to string if the type conversion fails.
///
/// # Arguments
///
/// * `value` - The string value to parse
/// * `target_type` - The desired data type
///
/// # Returns
///
/// A Result containing the parsed Value or an error
fn parse_value(value: &str, target_type: DataType) -> Result<Value> {
    let trimmed = value.trim();

    // Handle empty values
    if trimmed.is_empty() {
        return match target_type {
            DataType::Int64 => Ok(Value::Int64(0)),
            DataType::Float64 => Ok(Value::Float64(0.0)),
            DataType::String => Ok(Value::String(String::new())),
        };
    }

    // Parse based on target type
    match target_type {
        DataType::Int64 => trimmed.parse::<i64>().map(Value::Int64).map_err(|_| {
            DatabaseError::type_error(format!("Failed to parse '{}' as Int64", trimmed))
        }),
        DataType::Float64 => trimmed.parse::<f64>().map(Value::Float64).map_err(|_| {
            DatabaseError::type_error(format!("Failed to parse '{}' as Float64", trimmed))
        }),
        DataType::String => Ok(Value::String(trimmed.to_string())),
    }
}

// ============================================================================
// CSV PARSING
// ============================================================================

/// Reads a CSV file and returns its headers and rows.
///
/// This function reads the entire CSV file into memory, separating the header
/// row from the data rows. It handles basic CSV parsing including quoted values.
///
/// # Arguments
///
/// * `path` - The path to the CSV file
///
/// # Returns
///
/// A tuple of (headers, rows) where headers is a Vec<String> of column names
/// and rows is a Vec<Vec<String>> of data rows
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be opened
/// - The CSV cannot be parsed
/// - The file is empty
fn read_csv_file(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    // Open the file
    let file = File::open(path).map_err(|e| {
        DatabaseError::ingestion_error(format!("Failed to open file '{}': {}", path.display(), e))
    })?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Read the header row
    let header_line = lines.next().ok_or_else(|| {
        DatabaseError::ingestion_error(format!(
            "CSV file '{}' is empty (no header row)",
            path.display()
        ))
    })?;

    let header_line = header_line.map_err(|e| {
        DatabaseError::ingestion_error(format!(
            "Failed to read header from '{}': {}",
            path.display(),
            e
        ))
    })?;

    // Parse the header using csv crate for proper handling of quoted values
    let headers = parse_csv_line(&header_line)?;

    if headers.is_empty() {
        return Err(DatabaseError::ingestion_error(format!(
            "CSV file '{}' has empty header row",
            path.display()
        )));
    }

    // Read all data rows
    let mut rows = Vec::new();
    let mut row_num = 1; // Header is row 0, so data starts at row 1

    for line_result in lines {
        row_num += 1;
        let line = line_result.map_err(|e| {
            DatabaseError::ingestion_error(format!(
                "Failed to read line {} from '{}': {}",
                row_num,
                path.display(),
                e
            ))
        })?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse the row
        let row = parse_csv_line(&line).map_err(|e| {
            DatabaseError::ingestion_error(format!(
                "Failed to parse row {} in '{}': {}",
                row_num,
                path.display(),
                e
            ))
        })?;

        rows.push(row);
    }

    Ok((headers, rows))
}

/// Parses a single CSV line into individual fields.
///
/// This function uses the csv crate's ReaderBuilder to properly handle:
/// - Quoted fields
/// - Embedded commas in quoted fields
/// - Embedded newlines (not supported in this simple version)
/// - Different quote characters
///
/// # Arguments
///
/// * `line` - The CSV line to parse
///
/// # Returns
///
/// A vector of field strings
///
/// # Errors
///
/// Returns an error if the line cannot be parsed
fn parse_csv_line(line: &str) -> Result<Vec<String>> {
    // Create a CSV reader from the line
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(line.as_bytes());

    // Get the records iterator
    let mut records = rdr.records();

    // Get the first (and should be only) record
    let record = records
        .next()
        .transpose()
        .map_err(|e| DatabaseError::ingestion_error(format!("CSV parsing error: {}", e)))?;

    match record {
        Some(record) => {
            let fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            Ok(fields)
        }
        None => Ok(Vec::new()),
    }
}

// ============================================================================
// MAIN INGESTION FUNCTION
// ============================================================================

/// Loads a CSV file and creates a Table from its contents.
///
/// This is the main entry point for CSV ingestion. It performs the following steps:
/// 1. Reads the CSV file and extracts headers and data rows
/// 2. Infers the data type for each column based on sample data
/// 3. Creates appropriate Column instances for each column
/// 4. Transposes row data into the columnar format
/// 5. Builds and returns a Table
///
/// # Arguments
///
/// * `path` - The path to the CSV file
/// * `table_name` - The name to give the created table
///
/// # Returns
///
/// A Result containing the created Table or an error
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The CSV is malformed
/// - Type inference fails
/// - Data conversion fails
///
/// # Example
///
/// ```ignore
/// use mini_rust_olap::ingest::load_csv;
///
/// // Load a CSV file
/// let table = load_csv("data/users.csv", "users")?;
///
/// println!("Loaded {} columns and {} rows",
///          table.column_count(),
///          table.row_count());
/// ```
pub fn load_csv<P: AsRef<Path>>(path: P, table_name: String) -> Result<Table> {
    let path = path.as_ref();

    // Step 1: Read the CSV file
    let (headers, rows) = read_csv_file(path)?;

    if rows.is_empty() {
        return Err(DatabaseError::ingestion_error(format!(
            "CSV file '{}' has no data rows",
            path.display()
        )));
    }

    // Step 2: Infer column types
    let column_types: Vec<DataType> = headers
        .iter()
        .enumerate()
        .map(|(col_idx, _header)| {
            // Collect sample values for this column
            let sample_values: Vec<String> = rows
                .iter()
                .filter_map(|row| {
                    if col_idx < row.len() {
                        Some(row[col_idx].clone())
                    } else {
                        None
                    }
                })
                .collect();

            // Infer the type
            infer_column_type(&sample_values)
        })
        .collect();

    // Step 3: Create the table and add columns
    let mut table = Table::new(table_name);

    for (header, data_type) in headers.iter().zip(column_types.iter()) {
        let column = create_column(*data_type);
        table.add_column(header.clone(), column)?;
    }

    // Step 4: Transpose row data into columns
    // First, collect all values by column
    let mut column_data: Vec<Vec<String>> = vec![Vec::new(); headers.len()];

    for row in &rows {
        for (col_idx, value) in row.iter().enumerate() {
            if col_idx < column_data.len() {
                column_data[col_idx].push(value.clone());
            }
        }
    }

    // Step 5: Insert values into columns
    for (col_idx, header) in headers.iter().enumerate() {
        let data_type = column_types[col_idx];

        for value_str in &column_data[col_idx] {
            // Parse the value
            let value = parse_value(value_str, data_type)?;

            // Get the column and push the value
            let column = table.get_column_mut(header)?;
            column.push_value(value)?;
        }
    }

    Ok(table)
}

/// Loads a CSV file and directly registers it in the catalog.
///
/// This is a convenience function that combines `load_csv` and catalog registration
/// into a single operation.
///
/// # Arguments
///
/// * `path` - The path to the CSV file
/// * `table_name` - The name to give the created table
/// * `catalog` - The catalog to register the table in
///
/// # Returns
///
/// A Result indicating success or failure
///
/// # Example
///
/// ```ignore
/// use mini_rust_olap::catalog::Catalog;
/// use mini_rust_olap::ingest::load_csv_into_catalog;
///
/// let mut catalog = Catalog::new();
/// load_csv_into_catalog("data/users.csv", "users", &mut catalog)?;
/// ```
pub fn load_csv_into_catalog<P: AsRef<Path>>(
    path: P,
    table_name: String,
    catalog: &mut Catalog,
) -> Result<()> {
    let table = load_csv(path, table_name.clone())?;
    catalog.register_table(table)?;
    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper function to create a temporary CSV file
    fn create_temp_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        write!(file, "{}", content).expect("Failed to write to temp file");
        file
    }

    // ------------------------------------------------------------------------
    // Type Inference Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_infer_column_type_empty() {
        let values: Vec<String> = vec![];
        assert_eq!(infer_column_type(&values), DataType::String);
    }

    #[test]
    fn test_infer_column_type_all_integers() {
        let values: Vec<String> = vec!["1", "2", "3", "100", "-50"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(infer_column_type(&values), DataType::Int64);
    }

    #[test]
    fn test_infer_column_type_all_floats() {
        let values: Vec<String> = vec!["1.5", "2.7", "3.14", "-10.5"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(infer_column_type(&values), DataType::Float64);
    }

    #[test]
    fn test_infer_column_type_mixed_numeric() {
        let values: Vec<String> = vec!["1", "2.5", "3", "4.7"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(infer_column_type(&values), DataType::Float64);
    }

    #[test]
    fn test_infer_column_type_all_strings() {
        let values: Vec<String> = vec!["hello", "world", "foo", "bar"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(infer_column_type(&values), DataType::String);
    }

    #[test]
    fn test_infer_column_type_with_empties() {
        let values: Vec<String> = vec!["1", "", "3", "", "5"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(infer_column_type(&values), DataType::Int64);
    }

    #[test]
    fn test_infer_column_type_scientific_notation() {
        let values: Vec<String> = vec!["1.5e10", "2.3e-5", "3.0E8"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        // Scientific notation should be treated as Float64
        assert_eq!(infer_column_type(&values), DataType::Float64);
    }

    // ------------------------------------------------------------------------
    // Value Parsing Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_parse_value_int() {
        let value = parse_value("42", DataType::Int64).unwrap();
        assert_eq!(value, Value::Int64(42));
    }

    #[test]
    fn test_parse_value_float() {
        let value = parse_value("12.34", DataType::Float64).unwrap();
        assert_eq!(value, Value::Float64(12.34));
    }

    #[test]
    fn test_parse_value_string() {
        let value = parse_value("hello", DataType::String).unwrap();
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_parse_value_empty_int() {
        let value = parse_value("", DataType::Int64).unwrap();
        assert_eq!(value, Value::Int64(0));
    }

    #[test]
    fn test_parse_value_empty_float() {
        let value = parse_value("", DataType::Float64).unwrap();
        assert_eq!(value, Value::Float64(0.0));
    }

    #[test]
    fn test_parse_value_empty_string() {
        let value = parse_value("", DataType::String).unwrap();
        assert_eq!(value, Value::String(String::new()));
    }

    #[test]
    fn test_parse_value_whitespace_int() {
        let value = parse_value("  42  ", DataType::Int64).unwrap();
        assert_eq!(value, Value::Int64(42));
    }

    #[test]
    fn test_parse_value_invalid_int() {
        let result = parse_value("not_a_number", DataType::Int64);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_value_negative_int() {
        let value = parse_value("-42", DataType::Int64).unwrap();
        assert_eq!(value, Value::Int64(-42));
    }

    #[test]
    fn test_parse_value_negative_float() {
        let value = parse_value("-56.78", DataType::Float64).unwrap();
        assert_eq!(value, Value::Float64(-56.78));
    }

    // ------------------------------------------------------------------------
    // CSV Parsing Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_parse_csv_line_simple() {
        let line = "id,name,age";
        let fields = parse_csv_line(line).unwrap();
        assert_eq!(fields, vec!["id", "name", "age"]);
    }

    #[test]
    fn test_parse_csv_line_with_quotes() {
        let line = "1,\"John Doe\",30";
        let fields = parse_csv_line(line).unwrap();
        assert_eq!(fields, vec!["1", "John Doe", "30"]);
    }

    #[test]
    fn test_parse_csv_line_with_embedded_comma() {
        let line = "1,\"Doe, John\",30";
        let fields = parse_csv_line(line).unwrap();
        assert_eq!(fields, vec!["1", "Doe, John", "30"]);
    }

    #[test]
    fn test_parse_csv_line_empty() {
        let line = "";
        let fields = parse_csv_line(line).unwrap();
        assert_eq!(fields.len(), 0);
    }

    // ------------------------------------------------------------------------
    // Load CSV Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_load_csv_simple() {
        let csv_content = r#"id,name,age
1,John,25
2,Jane,30
3,Bob,35"#;

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "test_table".to_string()).unwrap();

        assert_eq!(table.name(), "test_table");
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 3);

        // Check headers
        assert_eq!(table.column_names(), vec!["id", "name", "age"]);

        // Check some values
        assert_eq!(table.get_value("id", 0).unwrap(), Value::Int64(1));
        assert_eq!(
            table.get_value("name", 1).unwrap(),
            Value::String("Jane".to_string())
        );
        assert_eq!(table.get_value("age", 2).unwrap(), Value::Int64(35));
    }

    #[test]
    fn test_load_csv_with_floats() {
        let csv_content = r#"id,name,score
1,Alice,95.5
2,Bob,87.3
3,Charlie,92.7"#;

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "scores".to_string()).unwrap();

        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 3);

        // Check that score column is Float64
        let score_col = table.get_column("score").unwrap();
        assert_eq!(score_col.data_type(), DataType::Float64);

        // Check some values
        assert_eq!(table.get_value("score", 0).unwrap(), Value::Float64(95.5));
        assert_eq!(table.get_value("score", 1).unwrap(), Value::Float64(87.3));
    }

    #[test]
    fn test_load_csv_with_empty_values() {
        let csv_content = r#"id,name,email
1,John,john@example.com
2,Jane,
3,Bob,bob@example.com"#;

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "users".to_string()).unwrap();

        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 3);

        // Check that empty email is handled correctly
        let email_value = table.get_value("email", 1).unwrap();
        assert_eq!(email_value, Value::String(String::new()));
    }

    #[test]
    fn test_load_csv_with_quotes() {
        let csv_content = r#"id,full_name,description
1,"John Doe","Software Engineer"
2,"Jane Smith","Data Scientist""#;

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "employees".to_string()).unwrap();

        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 2);

        // Check that quoted values are parsed correctly
        assert_eq!(
            table.get_value("full_name", 0).unwrap(),
            Value::String("John Doe".to_string())
        );
        assert_eq!(
            table.get_value("description", 1).unwrap(),
            Value::String("Data Scientist".to_string())
        );
    }

    #[test]
    fn test_load_csv_empty_file() {
        let csv_content = "";

        let file = create_temp_csv(csv_content);
        let result = load_csv(file.path(), "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_only_header() {
        let csv_content = "id,name,age";

        let file = create_temp_csv(csv_content);
        let result = load_csv(file.path(), "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_file_not_found() {
        let result = load_csv("/nonexistent/file.csv", "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_mixed_types_becomes_string() {
        let csv_content = r#"id,value
1,100
2,text
3,200"#;

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "mixed".to_string()).unwrap();

        // Value column should be String because of the text value
        let value_col = table.get_column("value").unwrap();
        assert_eq!(value_col.data_type(), DataType::String);

        // Check values
        assert_eq!(
            table.get_value("value", 0).unwrap(),
            Value::String("100".to_string())
        );
        assert_eq!(
            table.get_value("value", 1).unwrap(),
            Value::String("text".to_string())
        );
    }

    // ------------------------------------------------------------------------
    // Load CSV Into Catalog Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_load_csv_into_catalog() {
        let csv_content = r#"id,name
1,Alice
2,Bob"#;

        let file = create_temp_csv(csv_content);
        let mut catalog = Catalog::new();

        let result = load_csv_into_catalog(file.path(), "users".to_string(), &mut catalog);
        assert!(result.is_ok());

        // Verify table was registered
        assert!(catalog.table_exists("users"));

        // Verify table data
        let table = catalog.get_table("users").unwrap();
        assert_eq!(table.row_count(), 2);
        assert_eq!(table.column_count(), 2);
    }

    #[test]
    fn test_load_csv_into_catalog_duplicate_name() {
        let csv_content = r#"id,name
1,Alice"#;

        let file = create_temp_csv(csv_content);
        let mut catalog = Catalog::new();

        // Load the same table twice
        let result1 = load_csv_into_catalog(file.path(), "users".to_string(), &mut catalog);
        assert!(result1.is_ok());

        let result2 = load_csv_into_catalog(file.path(), "users".to_string(), &mut catalog);
        assert!(result2.is_err());
    }

    // ------------------------------------------------------------------------
    // Large File Tests (Basic)
    // ------------------------------------------------------------------------

    #[test]
    fn test_load_csv_large_file() {
        let mut csv_content = "id,name,age\n".to_string();

        // Generate 1000 rows
        for i in 1..=1000 {
            csv_content.push_str(&format!("{},User{},{}\n", i, i, 20 + (i % 50)));
        }

        let file = create_temp_csv(&csv_content);
        let table = load_csv(file.path(), "large".to_string()).unwrap();

        assert_eq!(table.row_count(), 1000);
        assert_eq!(table.column_count(), 3);

        // Check first and last rows
        assert_eq!(table.get_value("id", 0).unwrap(), Value::Int64(1));
        assert_eq!(table.get_value("id", 999).unwrap(), Value::Int64(1000));
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_load_csv_single_column() {
        let csv_content = "value\n1\n2\n3";

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "single".to_string()).unwrap();

        assert_eq!(table.column_count(), 1);
        assert_eq!(table.row_count(), 3);
        assert_eq!(table.get_value("value", 1).unwrap(), Value::Int64(2));
    }

    #[test]
    fn test_load_csv_single_row() {
        let csv_content = "id,name,age\n1,John,25";

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "single".to_string()).unwrap();

        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 1);
    }

    #[test]
    fn test_load_csv_with_special_characters() {
        let csv_content = r#"id,message
1,"Hello, World!"
2,"Price: $99.99"
3,"Email: test@example.com""#;

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "special".to_string()).unwrap();

        assert_eq!(table.row_count(), 3);
        assert_eq!(
            table.get_value("message", 0).unwrap(),
            Value::String("Hello, World!".to_string())
        );
    }

    #[test]
    fn test_load_csv_type_promotion_int_to_float() {
        // All integers should stay as integers
        let csv_content = "value\n1\n2\n3";

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "ints".to_string()).unwrap();

        let col = table.get_column("value").unwrap();
        assert_eq!(col.data_type(), DataType::Int64);
    }

    #[test]
    fn test_load_csv_with_negative_numbers() {
        let csv_content = "id,value\n1,-100\n2,-200\n3,-300";

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "negative".to_string()).unwrap();

        assert_eq!(table.get_value("value", 0).unwrap(), Value::Int64(-100));
        assert_eq!(table.get_value("value", 2).unwrap(), Value::Int64(-300));
    }

    #[test]
    fn test_load_csv_with_scientific_notation() {
        let csv_content = "id,value\n1,1.5e10\n2,2.3e-5\n3,3.0E8";

        let file = create_temp_csv(csv_content);
        let table = load_csv(file.path(), "scientific".to_string()).unwrap();

        let col = table.get_column("value").unwrap();
        assert_eq!(col.data_type(), DataType::Float64);

        // The exact values might differ slightly due to floating point representation
        let value = table.get_value("value", 0).unwrap();
        if let Value::Float64(f) = value {
            assert!((f - 1.5e10).abs() < 1e5); // Allow some tolerance
        } else {
            panic!("Expected Float64");
        }
    }
}
