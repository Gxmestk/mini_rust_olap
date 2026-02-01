//! # Manual Query Integration Tests
//!
//! This file contains integration tests that demonstrate manual query operations
//! on our columnar storage without using SQL or a query planner.
//!
//! ## Purpose
//!
//! These tests verify that the fundamental columnar storage system works correctly.
//! Before building a complex SQL parser and query planner, we need to ensure that
//! the basic operations (sum, filter, etc.) work as expected.
//!
//! ## What We're Testing
//!
//! 1. **Column Creation**: Creating columns of different types
//! 2. **Data Insertion**: Populating columns with test data
//! 3. **Manual Aggregation**: Summing values without SQL
//! 4. **Manual Filtering**: Filtering rows without SQL
//! 5. **Result Verification**: Checking that results match expectations
//!
//! ## Why Manual Tests?
//!
//! Testing queries manually (without SQL) helps us:
//! - Understand the data flow
//! - Validate the core columnar storage design
//! - Build confidence before adding complexity
//! - Serve as documentation for expected behavior

use mini_olap_database::{Column, FloatColumn, IntColumn, StringColumn, Value};

/// Test 1: Simple column creation and data insertion
///
/// This test demonstrates the basic workflow of creating columns
/// and inserting data into them.
#[test]
fn test_basic_column_operations() {
    // Create an integer column for user IDs
    let mut user_ids = IntColumn::new();

    // Create a string column for names
    let mut user_names = StringColumn::new();

    // Create a float column for ages
    let mut user_ages = FloatColumn::new();

    // Insert some test data
    // Row 1
    user_ids.push_value(Value::Int64(1)).unwrap();
    user_names
        .push_value(Value::String("Alice".to_string()))
        .unwrap();
    user_ages.push_value(Value::Float64(25.0)).unwrap();

    // Row 2
    user_ids.push_value(Value::Int64(2)).unwrap();
    user_names
        .push_value(Value::String("Bob".to_string()))
        .unwrap();
    user_ages.push_value(Value::Float64(30.0)).unwrap();

    // Row 3
    user_ids.push_value(Value::Int64(3)).unwrap();
    user_names
        .push_value(Value::String("Charlie".to_string()))
        .unwrap();
    user_ages.push_value(Value::Float64(35.0)).unwrap();

    // Verify the data was inserted correctly
    assert_eq!(user_ids.len(), 3);
    assert_eq!(user_names.len(), 3);
    assert_eq!(user_ages.len(), 3);

    // Verify individual values
    assert_eq!(user_ids.get(0).unwrap(), Value::Int64(1));
    assert_eq!(user_names.get(1).unwrap(), Value::String("Bob".to_string()));
    assert_eq!(user_ages.get(2).unwrap(), Value::Float64(35.0));
}

/// Test 2: Manual SUM aggregation
///
/// This test demonstrates how to manually aggregate (sum) values
/// from a column. This is equivalent to the SQL: `SELECT SUM(salary) FROM employees`
#[test]
fn test_manual_sum_aggregation() {
    let mut salaries = IntColumn::new();

    // Insert salary data
    salaries.push_value(Value::Int64(50000)).unwrap();
    salaries.push_value(Value::Int64(60000)).unwrap();
    salaries.push_value(Value::Int64(70000)).unwrap();
    salaries.push_value(Value::Int64(55000)).unwrap();
    salaries.push_value(Value::Int64(65000)).unwrap();

    // Manually sum the values
    let mut sum = 0i64;
    for i in 0..salaries.len() {
        if let Value::Int64(salary) = salaries.get(i).unwrap() {
            sum += salary;
        }
    }

    // Verify the sum is correct
    let expected_sum = 50000 + 60000 + 70000 + 55000 + 65000;
    assert_eq!(
        sum, expected_sum,
        "Sum of salaries should be {}",
        expected_sum
    );

    println!("✓ Manual SUM: {} (expected {})", sum, expected_sum);
}

/// Test 3: Manual AVG aggregation
///
/// This test demonstrates how to manually calculate the average
/// of values in a column. This is equivalent to the SQL: `SELECT AVG(age) FROM users`
#[test]
fn test_manual_avg_aggregation() {
    let mut ages = FloatColumn::new();

    // Insert age data
    ages.push_value(Value::Float64(25.0)).unwrap();
    ages.push_value(Value::Float64(30.0)).unwrap();
    ages.push_value(Value::Float64(35.0)).unwrap();
    ages.push_value(Value::Float64(40.0)).unwrap();
    ages.push_value(Value::Float64(45.0)).unwrap();

    // Manually calculate the average
    let mut sum = 0.0;
    let count = ages.len();

    for i in 0..count {
        if let Value::Float64(age) = ages.get(i).unwrap() {
            sum += age;
        }
    }

    let avg = sum / count as f64;
    let expected_avg = (25.0 + 30.0 + 35.0 + 40.0 + 45.0) / 5.0;

    // Use approximate comparison for floating point
    assert!(
        (avg - expected_avg).abs() < 0.0001,
        "Average should be approximately {}",
        expected_avg
    );

    println!("✓ Manual AVG: {:.2} (expected {:.2})", avg, expected_avg);
}

/// Test 4: Manual COUNT aggregation
///
/// This test demonstrates how to manually count the number of rows.
/// This is equivalent to the SQL: `SELECT COUNT(*) FROM users`
#[test]
fn test_manual_count_aggregation() {
    let mut names = StringColumn::new();

    // Insert name data
    names
        .push_value(Value::String("Alice".to_string()))
        .unwrap();
    names.push_value(Value::String("Bob".to_string())).unwrap();
    names
        .push_value(Value::String("Charlie".to_string()))
        .unwrap();

    // Count the rows
    let count = names.len();

    assert_eq!(count, 3, "Should have 3 rows");

    println!("✓ Manual COUNT: {}", count);
}

/// Test 5: Manual MIN aggregation
///
/// This test demonstrates how to manually find the minimum value.
/// This is equivalent to the SQL: `SELECT MIN(price) FROM products`
#[test]
fn test_manual_min_aggregation() {
    let mut prices = FloatColumn::new();

    // Insert price data
    prices.push_value(Value::Float64(19.99)).unwrap();
    prices.push_value(Value::Float64(9.99)).unwrap();
    prices.push_value(Value::Float64(29.99)).unwrap();
    prices.push_value(Value::Float64(14.99)).unwrap();

    // Find the minimum
    let mut min = f64::MAX;
    for i in 0..prices.len() {
        if let Value::Float64(price) = prices.get(i).unwrap() {
            if price < min {
                min = price;
            }
        }
    }

    assert_eq!(min, 9.99, "Minimum price should be 9.99");

    println!("✓ Manual MIN: {:.2}", min);
}

/// Test 6: Manual MAX aggregation
///
/// This test demonstrates how to manually find the maximum value.
/// This is equivalent to the SQL: `SELECT MAX(score) FROM students`
#[test]
fn test_manual_max_aggregation() {
    let mut scores = IntColumn::new();

    // Insert score data
    scores.push_value(Value::Int64(85)).unwrap();
    scores.push_value(Value::Int64(92)).unwrap();
    scores.push_value(Value::Int64(78)).unwrap();
    scores.push_value(Value::Int64(95)).unwrap();

    // Find the maximum
    let mut max = i64::MIN;
    for i in 0..scores.len() {
        if let Value::Int64(score) = scores.get(i).unwrap() {
            if score > max {
                max = score;
            }
        }
    }

    assert_eq!(max, 95, "Maximum score should be 95");

    println!("✓ Manual MAX: {}", max);
}

/// Test 7: Manual filtering (WHERE clause)
///
/// This test demonstrates how to manually filter rows based on a condition.
/// This is equivalent to the SQL: `SELECT * FROM users WHERE age > 30`
#[test]
fn test_manual_filter() {
    let mut user_ids = IntColumn::new();
    let mut user_ages = FloatColumn::new();

    // Insert test data
    for (id, age) in [(1, 25), (2, 35), (3, 28), (4, 42), (5, 31)] {
        user_ids.push_value(Value::Int64(id)).unwrap();
        user_ages.push_value(Value::Float64(age as f64)).unwrap();
    }

    // Filter: Find users older than 30
    let mut filtered_ids = Vec::new();
    let mut filtered_ages = Vec::new();

    for i in 0..user_ids.len() {
        if let Value::Float64(age) = user_ages.get(i).unwrap() {
            if age > 30.0 {
                if let Value::Int64(id) = user_ids.get(i).unwrap() {
                    filtered_ids.push(id);
                    filtered_ages.push(age);
                }
            }
        }
    }

    // Verify the filter worked
    assert_eq!(filtered_ids.len(), 3, "Should have 3 users older than 30");
    assert_eq!(
        filtered_ids,
        vec![2, 4, 5],
        "Filtered IDs should be [2, 4, 5]"
    );
    assert_eq!(
        filtered_ages,
        vec![35.0, 42.0, 31.0],
        "Filtered ages should be [35.0, 42.0, 31.0]"
    );

    println!(
        "✓ Manual FILTER: Found {} users older than 30",
        filtered_ids.len()
    );
}

/// Test 8: Manual filtering with string comparison
///
/// This test demonstrates filtering based on string values.
/// This is equivalent to the SQL: `SELECT * FROM users WHERE city = 'New York'`
#[test]
fn test_manual_string_filter() {
    let mut user_ids = IntColumn::new();
    let mut cities = StringColumn::new();

    // Insert test data
    for (id, city) in [
        (1, "New York"),
        (2, "Los Angeles"),
        (3, "New York"),
        (4, "Chicago"),
        (5, "New York"),
    ] {
        user_ids.push_value(Value::Int64(id)).unwrap();
        cities.push_value(Value::String(city.to_string())).unwrap();
    }

    // Filter: Find users in New York
    let mut ny_ids = Vec::new();
    let target_city = "New York";

    for i in 0..user_ids.len() {
        if let Value::String(city) = cities.get(i).unwrap() {
            if city == target_city {
                if let Value::Int64(id) = user_ids.get(i).unwrap() {
                    ny_ids.push(id);
                }
            }
        }
    }

    // Verify the filter worked
    assert_eq!(ny_ids.len(), 3, "Should have 3 users in New York");
    assert_eq!(ny_ids, vec![1, 3, 5], "NY user IDs should be [1, 3, 5]");

    println!(
        "✓ Manual STRING FILTER: Found {} users in {}",
        ny_ids.len(),
        target_city
    );
}

/// Test 9: Manual filtering with AND condition
///
/// This test demonstrates filtering with multiple conditions (AND).
/// This is equivalent to the SQL: `SELECT * FROM products WHERE price < 50 AND category = 'Electronics'`
#[test]
fn test_manual_filter_and() {
    let mut product_ids = IntColumn::new();
    let mut prices = FloatColumn::new();
    let mut categories = StringColumn::new();

    // Insert test data
    for (id, price, category) in [
        (1, 29.99, "Electronics"),
        (2, 49.99, "Books"),
        (3, 39.99, "Electronics"),
        (4, 59.99, "Electronics"),
        (5, 19.99, "Books"),
    ] {
        product_ids.push_value(Value::Int64(id)).unwrap();
        prices.push_value(Value::Float64(price)).unwrap();
        categories
            .push_value(Value::String(category.to_string()))
            .unwrap();
    }

    // Filter: Find Electronics products under $50
    let mut matching_ids = Vec::new();

    for i in 0..product_ids.len() {
        let price = prices.get(i).unwrap();
        let category = categories.get(i).unwrap();

        if let (Value::Float64(p), Value::String(c)) = (price, category) {
            if p < 50.0 && c == "Electronics" {
                if let Value::Int64(id) = product_ids.get(i).unwrap() {
                    matching_ids.push(id);
                }
            }
        }
    }

    // Verify the filter worked
    assert_eq!(
        matching_ids.len(),
        2,
        "Should have 2 Electronics products under $50"
    );
    assert_eq!(matching_ids, vec![1, 3], "Matching IDs should be [1, 3]");

    println!(
        "✓ Manual FILTER AND: Found {} Electronics products under $50",
        matching_ids.len()
    );
}

/// Test 10: Manual projection (SELECT specific columns)
///
/// This test demonstrates selecting only specific columns.
/// This is equivalent to the SQL: `SELECT name, email FROM users`
#[test]
fn test_manual_projection() {
    let mut user_ids = IntColumn::new();
    let mut user_names = StringColumn::new();
    let mut user_emails = StringColumn::new();

    // Insert test data
    for (id, name, email) in [
        (1, "Alice", "alice@example.com"),
        (2, "Bob", "bob@example.com"),
        (3, "Charlie", "charlie@example.com"),
    ] {
        user_ids.push_value(Value::Int64(id)).unwrap();
        user_names
            .push_value(Value::String(name.to_string()))
            .unwrap();
        user_emails
            .push_value(Value::String(email.to_string()))
            .unwrap();
    }

    // Project: Only select names and emails (not IDs)
    let mut selected_names = Vec::new();
    let mut selected_emails = Vec::new();

    for i in 0..user_ids.len() {
        let name = user_names.get(i).unwrap();
        let email = user_emails.get(i).unwrap();

        if let (Value::String(n), Value::String(e)) = (name, email) {
            selected_names.push(n);
            selected_emails.push(e);
        }
    }

    // Verify the projection worked
    assert_eq!(selected_names.len(), 3, "Should have 3 names");
    assert_eq!(
        selected_names,
        vec!["Alice", "Bob", "Charlie"],
        "Names should be ['Alice', 'Bob', 'Charlie']"
    );
    assert_eq!(
        selected_emails,
        vec![
            "alice@example.com",
            "bob@example.com",
            "charlie@example.com"
        ],
        "Emails should match"
    );

    println!(
        "✓ Manual PROJECTION: Selected {} name/email pairs",
        selected_names.len()
    );
}

/// Test 11: Manual GROUP BY aggregation
///
/// This test demonstrates grouping and aggregating by a key column.
/// This is equivalent to the SQL: `SELECT city, COUNT(*) FROM users GROUP BY city`
#[test]
fn test_manual_group_by_count() {
    let mut cities = StringColumn::new();

    // Insert test data with multiple users in the same cities
    for city in [
        "New York",
        "Los Angeles",
        "New York",
        "Chicago",
        "New York",
        "Los Angeles",
    ] {
        cities.push_value(Value::String(city.to_string())).unwrap();
    }

    // Group by city and count
    use std::collections::HashMap;
    let mut city_counts: HashMap<String, i64> = HashMap::new();

    for i in 0..cities.len() {
        if let Value::String(city) = cities.get(i).unwrap() {
            *city_counts.entry(city.clone()).or_insert(0) += 1;
        }
    }

    // Verify the aggregation worked
    assert_eq!(
        city_counts.get("New York"),
        Some(&3),
        "New York should have 3 users"
    );
    assert_eq!(
        city_counts.get("Los Angeles"),
        Some(&2),
        "Los Angeles should have 2 users"
    );
    assert_eq!(
        city_counts.get("Chicago"),
        Some(&1),
        "Chicago should have 1 user"
    );

    println!("✓ Manual GROUP BY COUNT:");
    for (city, count) in city_counts.iter() {
        println!("  - {}: {}", city, count);
    }
}

/// Test 12: Manual GROUP BY SUM aggregation
///
/// This test demonstrates grouping and summing values by a key column.
/// This is equivalent to the SQL: `SELECT department, SUM(salary) FROM employees GROUP BY department`
#[test]
fn test_manual_group_by_sum() {
    let mut departments = StringColumn::new();
    let mut salaries = IntColumn::new();

    // Insert test data
    for (dept, salary) in [
        ("Engineering", 100000),
        ("Sales", 50000),
        ("Engineering", 120000),
        ("Marketing", 60000),
        ("Engineering", 90000),
        ("Sales", 70000),
    ] {
        departments
            .push_value(Value::String(dept.to_string()))
            .unwrap();
        salaries.push_value(Value::Int64(salary)).unwrap();
    }

    // Group by department and sum salaries
    use std::collections::HashMap;
    let mut dept_salaries: HashMap<String, i64> = HashMap::new();

    for i in 0..departments.len() {
        let dept = departments.get(i).unwrap();
        let salary = salaries.get(i).unwrap();

        if let (Value::String(d), Value::Int64(s)) = (dept, salary) {
            *dept_salaries.entry(d).or_insert(0) += s;
        }
    }

    // Verify the aggregation worked
    assert_eq!(
        dept_salaries.get("Engineering"),
        Some(&310000),
        "Engineering total salary should be 310,000"
    );
    assert_eq!(
        dept_salaries.get("Sales"),
        Some(&120000),
        "Sales total salary should be 120,000"
    );
    assert_eq!(
        dept_salaries.get("Marketing"),
        Some(&60000),
        "Marketing total salary should be 60,000"
    );

    println!("✓ Manual GROUP BY SUM:");
    for (dept, total) in dept_salaries.iter() {
        println!("  - {}: ${}", dept, total);
    }
}

/// Test 13: Complex manual query (multi-step)
///
/// This test demonstrates a more complex query that combines
/// filtering, projection, and aggregation.
///
/// Equivalent to SQL:
/// ```sql
/// SELECT AVG(salary)
/// FROM employees
/// WHERE department = 'Engineering' AND salary > 80000
/// ```
#[test]
fn test_manual_complex_query() {
    let mut departments = StringColumn::new();
    let mut salaries = FloatColumn::new();

    // Insert test data
    for (dept, salary) in [
        ("Engineering", 100000.0),
        ("Sales", 50000.0),
        ("Engineering", 120000.0),
        ("Marketing", 60000.0),
        ("Engineering", 85000.0),
        ("Sales", 75000.0),
        ("Engineering", 95000.0),
    ] {
        departments
            .push_value(Value::String(dept.to_string()))
            .unwrap();
        salaries.push_value(Value::Float64(salary)).unwrap();
    }

    // Complex query: Filter Engineering employees with salary > 80k, then calculate average
    let mut matching_salaries = Vec::new();

    for i in 0..departments.len() {
        let dept = departments.get(i).unwrap();
        let salary = salaries.get(i).unwrap();

        if let (Value::String(d), Value::Float64(s)) = (dept, salary) {
            if d == "Engineering" && s > 80000.0 {
                matching_salaries.push(s);
            }
        }
    }

    // Calculate average of matching salaries
    let sum: f64 = matching_salaries.iter().sum();
    let avg = sum / matching_salaries.len() as f64;
    let expected_avg = (100000.0 + 120000.0 + 85000.0 + 95000.0) / 4.0;

    // Verify results
    assert_eq!(
        matching_salaries.len(),
        4,
        "Should have 4 Engineering employees earning > 80k"
    );
    assert!(
        (avg - expected_avg).abs() < 0.01,
        "Average should be approximately {:.2}",
        expected_avg
    );

    println!("✓ Manual COMPLEX QUERY:");
    println!("  - Found {} matching employees", matching_salaries.len());
    println!("  - Average salary: ${:.2}", avg);
}

/// Test 14: Edge case - Empty column operations
///
/// This test ensures our operations handle empty columns gracefully.
#[test]
fn test_empty_column_operations() {
    let empty_column = IntColumn::new();

    // Sum of empty column should be 0
    let mut sum = 0i64;
    for i in 0..empty_column.len() {
        if let Value::Int64(v) = empty_column.get(i).unwrap() {
            sum += v;
        }
    }
    assert_eq!(sum, 0, "Sum of empty column should be 0");

    // Count of empty column should be 0
    assert_eq!(empty_column.len(), 0, "Empty column should have 0 rows");

    println!("✓ Empty column operations work correctly");
}

/// Test 15: Edge case - Single row operations
///
/// This test ensures our operations work correctly with single-row columns.
#[test]
fn test_single_row_operations() {
    let mut single_column = FloatColumn::new();
    single_column.push_value(Value::Float64(42.5)).unwrap();

    // Sum should equal the single value
    let mut sum = 0.0;
    for i in 0..single_column.len() {
        if let Value::Float64(v) = single_column.get(i).unwrap() {
            sum += v;
        }
    }
    assert_eq!(sum, 42.5, "Sum of single value should be the value itself");

    // Min should equal the single value
    let mut min = f64::MAX;
    for i in 0..single_column.len() {
        if let Value::Float64(v) = single_column.get(i).unwrap() {
            if v < min {
                min = v;
            }
        }
    }
    assert_eq!(min, 42.5, "Min of single value should be the value itself");

    // Max should equal the single value
    let mut max = f64::MIN;
    for i in 0..single_column.len() {
        if let Value::Float64(v) = single_column.get(i).unwrap() {
            if v > max {
                max = v;
            }
        }
    }
    assert_eq!(max, 42.5, "Max of single value should be the value itself");

    println!("✓ Single row operations work correctly");
}
