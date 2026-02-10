//! # Integration Tests for Operator Chaining
//!
//! This file contains integration tests that verify multiple operators
//! work correctly when chained together in a query execution pipeline.
//!
//! ## Purpose
//!
//! These tests complement the unit tests in `src/execution.rs` by testing
//! the integration between operators rather than individual operators
//! in isolation.
//!
//! ## What We're Testing
//!
//! 1. **Operator Chaining**: Multiple operators connected in a pipeline
//! 2. **Schema Propagation**: Correct schema transformations through the pipeline
//! 3. **Data Flow**: Data correctness through multiple transformations
//! 4. **End-to-End Queries**: Realistic query scenarios
//! 5. **Performance**: Larger datasets to stress test the system

use mini_rust_olap::{
    aggregates::{AvgAggregate, CountAggregate, MaxAggregate, MinAggregate, SumAggregate},
    column::{Column, FloatColumn, IntColumn, StringColumn},
    execution::{BinaryComparison, ComparisonOp, Filter, GroupBy, Operator, Project, TableScan},
    table::Table,
    types::{DataType, Value},
};
use std::sync::Arc;

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a test sales table with the following schema:
/// - id: Int64
/// - product: String
/// - region: String
/// - quantity: Int64
/// - price: Float64
fn create_sales_table() -> Table {
    let mut table = Table::new("sales".to_string());

    // id column
    let mut id_col = IntColumn::new();
    for i in 1..=20 {
        id_col.push_value(Value::Int64(i)).unwrap();
    }
    table
        .add_column("id".to_string(), Box::new(id_col))
        .unwrap();

    // product column
    let product_data = vec![
        "Laptop", "Laptop", "Laptop", "Laptop", "Laptop", // 5
        "Phone", "Phone", "Phone", "Phone", "Phone", "Phone", // 6
        "Tablet", "Tablet", "Tablet", // 3
        "Monitor", "Monitor", // 2
        "Keyboard", "Keyboard", "Keyboard", "Keyboard", // 4
    ];
    let mut product_col = StringColumn::new();
    for p in product_data {
        product_col
            .push_value(Value::String(p.to_string()))
            .unwrap();
    }
    table
        .add_column("product".to_string(), Box::new(product_col))
        .unwrap();

    // region column
    let region_data = vec![
        "North", "South", "East", "West", "North", "South", "East", "West", "North", "South",
        "East", "West", "North", "South", "East", "West", "North", "South", "East", "West",
    ];
    let mut region_col = StringColumn::new();
    for r in region_data {
        region_col.push_value(Value::String(r.to_string())).unwrap();
    }
    table
        .add_column("region".to_string(), Box::new(region_col))
        .unwrap();

    // quantity column
    let quantity_data = vec![2, 3, 1, 4, 2, 5, 2, 3, 1, 4, 5, 2, 3, 1, 4, 5, 1, 2, 3, 4];
    let mut quantity_col = IntColumn::new();
    for q in quantity_data {
        quantity_col.push_value(Value::Int64(q)).unwrap();
    }
    table
        .add_column("quantity".to_string(), Box::new(quantity_col))
        .unwrap();

    // price column
    let price_data = vec![
        1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 500.0, 500.0, 500.0, 500.0, 500.0, 500.0, 300.0,
        300.0, 300.0, 200.0, 200.0, 50.0, 50.0, 50.0, 50.0,
    ];
    let mut price_col = FloatColumn::new();
    for p in price_data {
        price_col.push_value(Value::Float64(p)).unwrap();
    }
    table
        .add_column("price".to_string(), Box::new(price_col))
        .unwrap();

    table
}

/// Creates a test employees table with the following schema:
/// - id: Int64
/// - name: String
/// - department: String
/// - salary: Float64
fn create_employees_table() -> Table {
    let mut table = Table::new("employees".to_string());

    // id column
    let mut id_col = IntColumn::new();
    for i in 1..=10 {
        id_col.push_value(Value::Int64(i)).unwrap();
    }
    table
        .add_column("id".to_string(), Box::new(id_col))
        .unwrap();

    // name column
    let name_data = vec![
        "Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Grace", "Henry", "Ivy", "Jack",
    ];
    let mut name_col = StringColumn::new();
    for n in name_data {
        name_col.push_value(Value::String(n.to_string())).unwrap();
    }
    table
        .add_column("name".to_string(), Box::new(name_col))
        .unwrap();

    // department column
    let dept_data = vec![
        "Engineering",
        "Sales",
        "Engineering",
        "Marketing",
        "Sales",
        "Engineering",
        "Marketing",
        "Sales",
        "Marketing",
        "Engineering",
    ];
    let mut dept_col = StringColumn::new();
    for d in dept_data {
        dept_col.push_value(Value::String(d.to_string())).unwrap();
    }
    table
        .add_column("department".to_string(), Box::new(dept_col))
        .unwrap();

    // salary column
    let salary_data = vec![
        90000.0, 60000.0, 95000.0, 55000.0, 65000.0, 85000.0, 50000.0, 70000.0, 60000.0, 80000.0,
    ];
    let mut salary_col = FloatColumn::new();
    for s in salary_data {
        salary_col.push_value(Value::Float64(s)).unwrap();
    }
    table
        .add_column("salary".to_string(), Box::new(salary_col))
        .unwrap();

    table
}

/// Creates a large test table for performance testing
fn create_large_table(rows: usize) -> Table {
    let mut table = Table::new("large".to_string());

    // id column
    let mut id_col = IntColumn::new();
    for i in 1..=rows {
        id_col.push_value(Value::Int64(i as i64)).unwrap();
    }
    table
        .add_column("id".to_string(), Box::new(id_col))
        .unwrap();

    // category column (cycles through A, B, C, D, E)
    let categories = ["A", "B", "C", "D", "E"];
    let mut cat_col = StringColumn::new();
    for i in 0..rows {
        cat_col
            .push_value(Value::String(categories[i % 5].to_string()))
            .unwrap();
    }
    table
        .add_column("category".to_string(), Box::new(cat_col))
        .unwrap();

    // value column (sequential numbers)
    let mut val_col = IntColumn::new();
    for i in 1..=rows {
        val_col.push_value(Value::Int64(i as i64)).unwrap();
    }
    table
        .add_column("value".to_string(), Box::new(val_col))
        .unwrap();

    // weight column (random-ish values)
    let mut weight_col = FloatColumn::new();
    for i in 0..rows {
        let val = (i % 100) as f64 * 1.5;
        weight_col.push_value(Value::Float64(val)).unwrap();
    }
    table
        .add_column("weight".to_string(), Box::new(weight_col))
        .unwrap();

    table
}

// ============================================================================
// Test 1: Scan → Filter → Project
// ============================================================================

#[test]
fn test_scan_filter_project_basic() {
    let table = create_sales_table();

    // Step 1: Scan
    let scan = Box::new(TableScan::new(table).with_batch_size(10));

    // Step 2: Filter - only products with quantity > 2
    let predicate = Arc::new(BinaryComparison::new(
        3, // quantity column index
        ComparisonOp::GreaterThan,
        Value::Int64(2),
    ));
    let filter = Box::new(Filter::new(scan, predicate));

    // Step 3: Project - select product and quantity columns
    let project = Box::new(Project::new(filter, vec![1, 3])); // product, quantity

    // Execute
    let mut project = project;
    project.open().unwrap();

    let mut total_rows = 0;
    while let Some(batch) = project.next_batch().unwrap() {
        total_rows += batch.row_count();

        // Verify schema
        let schema = project.schema().unwrap();
        assert_eq!(schema.len(), 2);
        assert_eq!(schema.get("product"), Some(&DataType::String));
        assert_eq!(schema.get("quantity"), Some(&DataType::Int64));

        // Verify all quantities > 2
        for i in 0..batch.row_count() {
            let product = batch.get(i, 0).unwrap();
            let quantity = batch.get(i, 1).unwrap();

            match quantity {
                Value::Int64(q) => assert!(q > 2),
                _ => panic!("Expected Int64 value"),
            }

            // Verify product column
            match product {
                Value::String(_) => {} // OK
                _ => panic!("Expected String value"),
            }
        }
    }

    // Should have filtered to 11 rows (quantities > 2: 3, 4, 5, 3, 4, 5, 3, 4, 5, 3, 4)
    assert_eq!(total_rows, 11);

    project.close().unwrap();
}

#[test]
fn test_scan_filter_project_with_aliases() {
    let table = create_employees_table();

    // Step 1: Scan
    let scan = Box::new(TableScan::new(table));

    // Step 2: Filter - only Engineering department
    let predicate = Arc::new(BinaryComparison::new(
        2, // department column index
        ComparisonOp::Equal,
        Value::String("Engineering".to_string()),
    ));
    let filter = Box::new(Filter::new(scan, predicate));

    // Step 3: Project with aliases
    let project = Box::new(
        Project::new(filter, vec![1, 3]) // name, salary
            .with_aliases(vec![
                "employee_name".to_string(),
                "annual_salary".to_string(),
            ]),
    );

    // Execute
    let mut project = project;
    project.open().unwrap();

    let batch = project.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 4); // 4 engineers

    // Verify column names
    let column_names = project.column_names().unwrap();
    assert_eq!(
        column_names,
        vec!["employee_name".to_string(), "annual_salary".to_string()]
    );

    // Verify schema
    let schema = project.schema().unwrap();
    assert_eq!(schema.len(), 2);
    assert_eq!(schema.get("employee_name"), Some(&DataType::String));
    assert_eq!(schema.get("annual_salary"), Some(&DataType::Float64));

    // Verify all rows are Engineering
    for i in 0..batch.row_count() {
        let name = batch.get(i, 0).unwrap();
        let salary = batch.get(i, 1).unwrap();

        match name {
            Value::String(n) => {
                assert!(["Alice", "Charlie", "Frank", "Jack"].contains(&n.as_str()));
            }
            _ => panic!("Expected String value"),
        }

        match salary {
            Value::Float64(s) => assert!(s >= 80000.0), // All engineers earn >= 80k
            _ => panic!("Expected Float64 value"),
        }
    }

    project.close().unwrap();
}

// ============================================================================
// Test 2: Scan → GroupBy
// ============================================================================

#[test]
fn test_scan_group_by_count() {
    let table = create_sales_table();

    // Step 1: Scan
    let scan = Box::new(TableScan::new(table));

    // Step 2: GroupBy - count sales by product
    let mut group_by = Box::new(GroupBy::new(
        scan,
        vec![1], // Group by product
        vec![0], // Count id
        vec![Box::new(CountAggregate::new(DataType::Int64))],
    ));

    group_by.open().unwrap();

    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 5); // 5 products
    assert_eq!(batch.column_count(), 2);

    // Verify schema
    let schema = group_by.schema().unwrap();
    assert_eq!(schema.len(), 2);
    assert_eq!(schema.get("product"), Some(&DataType::String));
    assert_eq!(schema.get("agg_0"), Some(&DataType::Int64));

    // Verify counts
    let mut counts = std::collections::HashMap::new();
    for i in 0..batch.row_count() {
        let product = batch.get(i, 0).unwrap();
        let count = batch.get(i, 1).unwrap();

        match (product, count) {
            (Value::String(p), Value::Int64(c)) => {
                counts.insert(p.clone(), c);
            }
            _ => panic!("Expected String and Int64 values"),
        }
    }

    assert_eq!(counts.get("Laptop"), Some(&5));
    assert_eq!(counts.get("Phone"), Some(&6));
    assert_eq!(counts.get("Tablet"), Some(&3));
    assert_eq!(counts.get("Monitor"), Some(&2));
    assert_eq!(counts.get("Keyboard"), Some(&4));

    group_by.close().unwrap();
}

#[test]
fn test_scan_group_by_sum() {
    let table = create_sales_table();

    // Step 1: Scan
    let scan = Box::new(TableScan::new(table));

    // Step 2: GroupBy - sum quantity by product
    let mut group_by = Box::new(GroupBy::new(
        scan,
        vec![1], // Group by product
        vec![3], // Sum quantity
        vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
    ));

    group_by.open().unwrap();

    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 5);

    // Verify sums
    let mut sums = std::collections::HashMap::new();
    for i in 0..batch.row_count() {
        let product = batch.get(i, 0).unwrap();
        let sum = batch.get(i, 1).unwrap();

        match (product, sum) {
            (Value::String(p), Value::Int64(s)) => {
                sums.insert(p.clone(), s);
            }
            _ => panic!("Expected String and Int64 values"),
        }
    }

    assert_eq!(sums.get("Laptop"), Some(&12)); // 2+3+1+4+2
    assert_eq!(sums.get("Phone"), Some(&20)); // 5+2+3+1+4+5
    assert_eq!(sums.get("Tablet"), Some(&6)); // 2+3+1
    assert_eq!(sums.get("Monitor"), Some(&9)); // 4+5
    assert_eq!(sums.get("Keyboard"), Some(&10)); // 1+2+3+4

    group_by.close().unwrap();
}

#[test]
fn test_scan_group_by_multiple_aggregates() {
    let table = create_sales_table();

    // Step 1: Scan
    let scan = Box::new(TableScan::new(table));

    // Step 2: GroupBy - count, sum, min, max quantity by product
    let mut group_by = Box::new(GroupBy::new(
        scan,
        vec![1],          // Group by product
        vec![3, 3, 3, 3], // All aggregate on quantity
        vec![
            Box::new(CountAggregate::new(DataType::Int64)),
            Box::new(SumAggregate::new(DataType::Int64).unwrap()),
            Box::new(MinAggregate::new(DataType::Int64)),
            Box::new(MaxAggregate::new(DataType::Int64)),
        ],
    ));

    group_by.open().unwrap();

    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 5);
    assert_eq!(batch.column_count(), 5); // product + 4 aggregates

    // Verify for Laptop: 5 items, sum=12, min=1, max=4
    for i in 0..batch.row_count() {
        let product = batch.get(i, 0).unwrap();
        let count = batch.get(i, 1).unwrap();
        let sum = batch.get(i, 2).unwrap();
        let min = batch.get(i, 3).unwrap();
        let max = batch.get(i, 4).unwrap();

        if product == Value::String("Laptop".to_string()) {
            assert_eq!(count, Value::Int64(5));
            assert_eq!(sum, Value::Int64(12));
            assert_eq!(min, Value::Int64(1));
            assert_eq!(max, Value::Int64(4));
        }
    }

    group_by.close().unwrap();
}

// ============================================================================
// Test 3: Scan → Filter → GroupBy → Project
// ============================================================================

#[test]
fn test_scan_filter_groupby_project_complex() {
    let table = create_sales_table();

    // Step 1: Scan
    let scan = Box::new(TableScan::new(table));

    // Step 2: Filter - only products with price >= 300 (Laptop, Phone, Tablet)
    let predicate = Arc::new(BinaryComparison::new(
        4, // price column index
        ComparisonOp::GreaterThanOrEqual,
        Value::Float64(300.0),
    ));
    let filter = Box::new(Filter::new(scan, predicate));

    // Step 3: GroupBy - sum quantity by product
    let group_by = Box::new(GroupBy::new(
        filter,
        vec![1], // Group by product
        vec![3], // Sum quantity
        vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
    ));

    // Step 4: Project with aliases
    let project = Box::new(
        Project::new(group_by, vec![0, 1]) // product, sum
            .with_aliases(vec![
                "product_name".to_string(),
                "total_quantity".to_string(),
            ]),
    );

    // Execute
    let mut project = project;
    project.open().unwrap();

    let batch = project.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 3); // Only Laptop, Phone, Tablet

    // Verify schema and column names
    let schema = project.schema().unwrap();
    assert_eq!(schema.len(), 2);
    assert_eq!(schema.get("product_name"), Some(&DataType::String));
    assert_eq!(schema.get("total_quantity"), Some(&DataType::Int64));

    let column_names = project.column_names().unwrap();
    assert_eq!(
        column_names,
        vec!["product_name".to_string(), "total_quantity".to_string()]
    );

    // Verify sums (should exclude Monitor and Keyboard)
    let mut sums = std::collections::HashMap::new();
    for i in 0..batch.row_count() {
        let product = batch.get(i, 0).unwrap();
        let sum = batch.get(i, 1).unwrap();

        match (product, sum) {
            (Value::String(p), Value::Int64(s)) => {
                sums.insert(p.clone(), s);
            }
            _ => panic!("Expected String and Int64 values"),
        }
    }

    assert_eq!(sums.get("Laptop"), Some(&12));
    assert_eq!(sums.get("Phone"), Some(&20));
    assert_eq!(sums.get("Tablet"), Some(&6));
    assert!(!sums.contains_key("Monitor"));
    assert!(!sums.contains_key("Keyboard"));

    project.close().unwrap();
}

#[test]
fn test_scan_filter_groupby_avg() {
    let table = create_employees_table();

    // Step 1: Scan
    let scan = Box::new(TableScan::new(table));

    // Step 2: Filter - only Engineering department (using simple predicate)
    let predicate = Arc::new(BinaryComparison::new(
        2,
        ComparisonOp::Equal,
        Value::String("Engineering".to_string()),
    ));
    let filter = Box::new(Filter::new(scan, predicate));

    // Step 3: GroupBy - average salary by department
    let mut group_by = Box::new(GroupBy::new(
        filter,
        vec![2], // Group by department
        vec![3], // Average salary
        vec![Box::new(AvgAggregate::new(DataType::Float64).unwrap())],
    ));

    group_by.open().unwrap();

    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 1); // Only Engineering

    // Verify average
    for i in 0..batch.row_count() {
        let dept = batch.get(i, 0).unwrap();
        let avg = batch.get(i, 1).unwrap();

        match (dept, avg) {
            (Value::String(d), Value::Float64(a)) => {
                assert_eq!(d, "Engineering");
                // Engineering: 90000+95000+85000+80000 = 350000/4 = 87500
                assert!((a - 87500.0).abs() < 0.01);
            }
            _ => panic!("Expected String and Float64 values"),
        }
    }

    group_by.close().unwrap();
}

// ============================================================================
// Test 4: End-to-End Query Execution
// ============================================================================

#[test]
fn test_end_to_end_query_1() {
    // Query: SELECT product, SUM(quantity) FROM sales WHERE price > 100 GROUP BY product
    let table = create_sales_table();

    let scan = Box::new(TableScan::new(table));

    // Filter: price > 100 (excludes Keyboard)
    let predicate = Arc::new(BinaryComparison::new(
        4,
        ComparisonOp::GreaterThan,
        Value::Float64(100.0),
    ));
    let filter = Box::new(Filter::new(scan, predicate));

    // GroupBy: sum quantity by product
    let mut group_by = Box::new(GroupBy::new(
        filter,
        vec![1],
        vec![3],
        vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
    ));

    // Execute
    group_by.open().unwrap();

    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 4); // Laptop, Phone, Tablet, Monitor

    // Verify
    let mut sums = std::collections::HashMap::new();
    for i in 0..batch.row_count() {
        let product = batch.get(i, 0).unwrap();
        let sum = batch.get(i, 1).unwrap();

        match (product, sum) {
            (Value::String(p), Value::Int64(s)) => {
                sums.insert(p.clone(), s);
            }
            _ => panic!("Expected String and Int64 values"),
        }
    }

    assert_eq!(sums.get("Laptop"), Some(&12));
    assert_eq!(sums.get("Phone"), Some(&20));
    assert_eq!(sums.get("Tablet"), Some(&6));
    assert_eq!(sums.get("Monitor"), Some(&9));
    assert!(!sums.contains_key("Keyboard"));

    group_by.close().unwrap();
}

#[test]
fn test_end_to_end_query_2() {
    // Query: SELECT department, COUNT(*), AVG(salary) FROM employees WHERE salary > 60000 GROUP BY department
    let table = create_employees_table();

    let scan = Box::new(TableScan::new(table));

    // Filter: salary > 60000
    let predicate = Arc::new(BinaryComparison::new(
        3,
        ComparisonOp::GreaterThan,
        Value::Float64(60000.0),
    ));
    let filter = Box::new(Filter::new(scan, predicate));

    // GroupBy: count and avg salary by department
    let mut group_by = Box::new(GroupBy::new(
        filter,
        vec![2],
        vec![0, 3],
        vec![
            Box::new(CountAggregate::new(DataType::Int64)),
            Box::new(AvgAggregate::new(DataType::Float64).unwrap()),
        ],
    ));

    group_by.open().unwrap();

    let batch = group_by.next_batch().unwrap().unwrap();

    // Verify results
    let mut results = std::collections::HashMap::new();
    for i in 0..batch.row_count() {
        let dept = batch.get(i, 0).unwrap();
        let count = batch.get(i, 1).unwrap();
        let avg = batch.get(i, 2).unwrap();

        match (dept, count, avg) {
            (Value::String(d), Value::Int64(c), Value::Float64(a)) => {
                results.insert(d.clone(), (c, a));
            }
            _ => panic!("Expected String, Int64, and Float64 values"),
        }
    }

    // Engineering: 4 employees (all > 60k), avg = (90000+95000+85000+80000)/4 = 87500
    assert_eq!(results.get("Engineering"), Some(&(4, 87500.0)));
    // Marketing: only David (55000) filtered out, Grace (50000) filtered out
    // So 0 Marketing employees > 60k, so Marketing should not appear
    // Sales: Bob (60000) not > 60k, Eve (65000), Henry (70000) > 60k
    assert_eq!(results.get("Sales"), Some(&(2, 67500.0)));

    group_by.close().unwrap();
}

// ============================================================================
// Test 5: Schema Transformations
// ============================================================================

#[test]
fn test_schema_transformation_chain() {
    // Test 1: Scan schema (already tested in unit tests, but verify it works)
    {
        let table = create_sales_table();
        let mut scan = TableScan::new(table);
        scan.open().unwrap();
        let scan_schema = scan.schema().unwrap();
        assert_eq!(scan_schema.len(), 5); // id, product, region, quantity, price
        scan.close().unwrap();
    }

    // Test 2: Filter preserves schema
    {
        let table = create_sales_table();
        let scan = Box::new(TableScan::new(table));
        let predicate = Arc::new(BinaryComparison::new(
            3,
            ComparisonOp::GreaterThan,
            Value::Int64(2),
        ));
        let mut filter = Box::new(Filter::new(scan, predicate));
        filter.open().unwrap();
        let filter_schema = filter.schema().unwrap();
        assert_eq!(filter_schema.len(), 5);
        filter.close().unwrap();
    }

    // Test 3: Project reduces schema
    {
        let table = create_sales_table();
        let scan = Box::new(TableScan::new(table));
        let predicate = Arc::new(BinaryComparison::new(
            3,
            ComparisonOp::GreaterThan,
            Value::Int64(2),
        ));
        let filter = Box::new(Filter::new(scan, predicate));
        let mut project = Box::new(Project::new(filter, vec![1, 3])); // product, quantity
        project.open().unwrap();
        let project_schema = project.schema().unwrap();
        assert_eq!(project_schema.len(), 2);

        // Verify column names
        let column_names = project.column_names().unwrap();
        assert_eq!(
            column_names,
            vec!["product".to_string(), "quantity".to_string()]
        );

        // Execute and verify
        let batch = project.next_batch().unwrap().unwrap();
        assert_eq!(batch.column_count(), 2);

        project.close().unwrap();
    }
}

#[test]
fn test_schema_transformation_groupby() {
    let table = create_sales_table();

    let scan = Box::new(TableScan::new(table));

    let mut group_by = Box::new(GroupBy::new(
        scan,
        vec![1, 2], // product, region
        vec![3, 4], // quantity, price
        vec![
            Box::new(SumAggregate::new(DataType::Int64).unwrap()),
            Box::new(SumAggregate::new(DataType::Float64).unwrap()),
        ],
    ));

    // Need to open before getting schema
    group_by.open().unwrap();

    // Schema after group by
    let schema = group_by.schema().unwrap();
    assert_eq!(schema.len(), 4); // product, region, sum(quantity), sum(price)

    // Column names
    let column_names = group_by.column_names().unwrap();
    assert_eq!(
        column_names,
        vec![
            "product".to_string(),
            "region".to_string(),
            "agg_0".to_string(),
            "agg_1".to_string(),
        ]
    );
    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.column_count(), 4);

    group_by.close().unwrap();
}

// ============================================================================
// Test 6: Performance with Larger Datasets
// ============================================================================

#[test]
fn test_large_dataset_scan_filter_project() {
    let table = create_large_table(1000);

    let scan = Box::new(TableScan::new(table).with_batch_size(100));

    let predicate = Arc::new(BinaryComparison::new(
        2, // value column
        ComparisonOp::GreaterThan,
        Value::Int64(500),
    ));
    let filter = Box::new(Filter::new(scan, predicate));

    let project = Box::new(Project::new(filter, vec![0, 1, 2])); // id, category, value

    let mut project = project;
    project.open().unwrap();

    let mut total_rows = 0;
    while let Some(batch) = project.next_batch().unwrap() {
        total_rows += batch.row_count();
    }

    // Should have 500 rows > 500
    assert_eq!(total_rows, 500);

    project.close().unwrap();
}

#[test]
fn test_large_dataset_group_by() {
    let table = create_large_table(1000);

    let scan = Box::new(TableScan::new(table));

    let mut group_by = Box::new(GroupBy::new(
        scan,
        vec![1], // category
        vec![2], // value
        vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
    ));

    group_by.open().unwrap();

    let batch = group_by.next_batch().unwrap().unwrap();
    assert_eq!(batch.row_count(), 5); // 5 categories

    // Verify each category has correct sum
    let mut sums = std::collections::HashMap::new();
    for i in 0..batch.row_count() {
        let category = batch.get(i, 0).unwrap();
        let sum = batch.get(i, 1).unwrap();

        match (category, sum) {
            (Value::String(c), Value::Int64(s)) => {
                sums.insert(c.clone(), s);
            }
            _ => panic!("Expected String and Int64 values"),
        }
    }

    // Category A: values at indices 0, 5, 10, ..., 995 (200 values)
    // Sum = 1 + 6 + 11 + ... + 996 = arithmetic series
    // Sum = n/2 * (first + last) = 200/2 * (1 + 996) = 100 * 997 = 99700
    assert_eq!(sums.get("A"), Some(&99700));
    // Category B: 2 + 7 + 12 + ... + 997 = 100 * (2 + 997) = 100 * 999 = 99900
    assert_eq!(sums.get("B"), Some(&99900));
    // Category C: 3 + 8 + 13 + ... + 998 = 100 * (3 + 998) = 100 * 1001 = 100100
    assert_eq!(sums.get("C"), Some(&100100));
    // Category D: 4 + 9 + 14 + ... + 999 = 100 * (4 + 999) = 100 * 1003 = 100300
    assert_eq!(sums.get("D"), Some(&100300));
    // Category E: 5 + 10 + 15 + ... + 1000 = 100 * (5 + 1000) = 100 * 1005 = 100500
    assert_eq!(sums.get("E"), Some(&100500));

    group_by.close().unwrap();
}

// ============================================================================
// Test 7: Edge Cases
// ============================================================================

#[test]
fn test_filter_all_rows_filtered() {
    let table = create_sales_table();

    let scan = Box::new(TableScan::new(table));

    let predicate = Arc::new(BinaryComparison::new(
        3,
        ComparisonOp::GreaterThan,
        Value::Int64(100), // No quantity > 100
    ));
    let filter = Box::new(Filter::new(scan, predicate));

    let project = Box::new(Project::new(filter, vec![1]));

    let mut project = project;
    project.open().unwrap();

    let batch = project.next_batch().unwrap();
    assert!(batch.is_none());

    project.close().unwrap();
}

#[test]
fn test_group_by_single_group() {
    let table = create_sales_table();

    let scan = Box::new(TableScan::new(table));

    // Group by a constant (all rows have id > 0)
    let predicate = Arc::new(BinaryComparison::new(
        0,
        ComparisonOp::GreaterThan,
        Value::Int64(0),
    ));
    let filter = Box::new(Filter::new(scan, predicate));

    let mut group_by = Box::new(GroupBy::new(
        filter,
        vec![4], // price (but we'll filter to one value)
        vec![3], // quantity
        vec![Box::new(SumAggregate::new(DataType::Int64).unwrap())],
    ));

    group_by.open().unwrap();

    let batch = group_by.next_batch().unwrap().unwrap();
    // Should have 5 groups (one for each distinct price: 1000, 500, 300, 200, 50)
    assert_eq!(batch.row_count(), 5);

    group_by.close().unwrap();
}

#[test]
fn test_multiple_batches_through_pipeline() {
    let table = create_large_table(2500);

    let scan = Box::new(TableScan::new(table).with_batch_size(500));

    let predicate = Arc::new(BinaryComparison::new(
        2,
        ComparisonOp::LessThan,
        Value::Int64(1250),
    ));
    let filter = Box::new(Filter::new(scan, predicate));

    let project = Box::new(Project::new(filter, vec![0, 2]));

    let mut project = project;
    project.open().unwrap();

    let mut batch_count = 0;
    let mut total_rows = 0;

    while let Some(batch) = project.next_batch().unwrap() {
        batch_count += 1;
        total_rows += batch.row_count();

        // Verify each batch has correct schema
        assert_eq!(batch.column_count(), 2);
    }

    // Should have processed in multiple batches
    assert!(batch_count > 1);
    // Half the rows < 1250
    assert_eq!(total_rows, 1249);

    project.close().unwrap();
}
