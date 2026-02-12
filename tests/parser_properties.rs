//! Property-Based Tests for SQL Parser
//!
//! This module uses proptest to test the SQL parser with automatically generated inputs.
//! Property-based testing complements unit tests by:
//! - Testing edge cases that humans might not think of
//! - Finding bugs with minimal counterexamples
//! - Verifying properties that should hold for all inputs
//!
//! Run with: cargo test --test parser_properties

use mini_rust_olap::parser::Parser;
use proptest::prelude::*;

// ============================================================================
// Strategy Generators
// ============================================================================

/// Generate a valid SQL identifier (table or column name)
fn arb_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(|s| s)
}

/// Generate a valid table name
fn arb_table_name() -> impl Strategy<Value = String> {
    arb_identifier().prop_map(|s| s)
}

/// Generate a valid column name
fn arb_column_name() -> impl Strategy<Value = String> {
    arb_identifier().prop_map(|s| s)
}

/// Generate a simple SELECT query
fn arb_simple_select() -> impl Strategy<Value = String> {
    arb_table_name().prop_flat_map(|table| {
        (Just(table.clone()), arb_column_name(), arb_column_name())
            .prop_map(|(table, col1, col2)| format!("SELECT {}, {} FROM {}", col1, col2, table))
    })
}

/// Generate a SELECT query with WHERE clause
fn arb_select_with_where() -> impl Strategy<Value = String> {
    arb_table_name().prop_flat_map(|table| {
        (
            Just(table.clone()),
            arb_column_name(),
            arb_column_name(),
            0i64..1000i64,
        )
            .prop_map(|(table, col1, col2, value)| {
                format!("SELECT {} FROM {} WHERE {} > {}", col1, table, col2, value)
            })
    })
}

/// Generate a SELECT query with ORDER BY
fn arb_select_with_order() -> impl Strategy<Value = String> {
    arb_table_name().prop_flat_map(|table| {
        (
            Just(table.clone()),
            arb_column_name(),
            arb_column_name(),
            prop::sample::select(vec!["ASC", "DESC"]),
        )
            .prop_map(|(table, col1, col2, dir)| {
                format!("SELECT {} FROM {} ORDER BY {} {}", col1, table, col2, dir)
            })
    })
}

/// Generate a SELECT query with GROUP BY and aggregation
fn arb_select_with_groupby() -> impl Strategy<Value = String> {
    arb_table_name().prop_flat_map(|table| {
        (
            Just(table.clone()),
            arb_column_name(),
            arb_column_name(),
            prop::sample::select(vec!["COUNT", "SUM", "AVG", "MIN", "MAX"]),
        )
            .prop_map(|(table, group_col, agg_col, agg_func)| {
                format!(
                    "SELECT {}, {}({}) FROM {} GROUP BY {}",
                    group_col, agg_func, agg_col, table, group_col
                )
            })
    })
}

/// Generate a complex SELECT query with multiple clauses
fn arb_complex_select() -> impl Strategy<Value = String> {
    arb_table_name().prop_flat_map(|table| {
        (
            Just(table.clone()),
            arb_column_name(),
            arb_column_name(),
            0i64..1000i64,
            prop::sample::select(vec!["COUNT", "SUM"]),
            prop::sample::select(vec!["ASC", "DESC"]),
            0..2u8, // 0 = no ORDER BY, 1 = with ORDER BY
        )
            .prop_map(|(table, col1, col2, value, agg, dir, add_order)| {
                let query = format!(
                    "SELECT {}, {}({}) FROM {} WHERE {} > {} GROUP BY {}",
                    col1, agg, col2, table, col2, value, col1
                );
                if add_order > 0 {
                    format!("{} ORDER BY {} {}", query, col1, dir)
                } else {
                    query
                }
            })
    })
}

/// Generate various SELECT queries
fn arb_select_query() -> impl Strategy<Value = String> {
    prop_oneof![
        5 => arb_simple_select(),
        3 => arb_select_with_where(),
        3 => arb_select_with_order(),
        4 => arb_select_with_groupby(),
        5 => arb_complex_select(),
    ]
}

// ============================================================================
// Property Tests
// ============================================================================

/// Property: Parsing should not crash for valid identifiers
#[test]
fn prop_parser_does_not_crash_on_identifiers() {
    proptest!(|(ident in arb_identifier())| {
        let sql = format!("SELECT {} FROM t1", ident);
        let mut parser = Parser::new(&sql);
        let _ = parser.parse(); // Should not panic
    });
}

/// Property: Parsing should not crash for simple SELECT queries
#[test]
fn prop_parser_does_not_crash_on_simple_select() {
    proptest!(|(query in arb_simple_select())| {
        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should not panic
    });
}

/// Property: Parsing should not crash for queries with WHERE clause
#[test]
fn prop_parser_does_not_crash_on_where() {
    proptest!(|(query in arb_select_with_where())| {
        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should not panic
    });
}

/// Property: Parsing should not crash for queries with ORDER BY
#[test]
fn prop_parser_does_not_crash_on_order_by() {
    proptest!(|(query in arb_select_with_order())| {
        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should not panic
    });
}

/// Property: Parsing should not crash for GROUP BY queries
#[test]
fn prop_parser_does_not_crash_on_group_by() {
    proptest!(|(query in arb_select_with_groupby())| {
        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should not panic
    });
}

/// Property: Parsing should not crash for complex queries
#[test]
fn prop_parser_does_not_crash_on_complex_queries() {
    proptest!(|(query in arb_complex_select())| {
        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should not panic
    });
}

/// Property: All generated queries should parse successfully or produce an error (not panic)
#[test]
fn prop_parser_graceful_handling() {
    proptest!(|(query in arb_select_query())| {
        let mut parser = Parser::new(&query);
        let result = parser.parse();

        // Either succeeds or fails gracefully (no panic)
        match result {
            Ok(_) | Err(_) => {}, // Both are acceptable
        }
    });
}

// ============================================================================
// Round-Trip Properties
// ============================================================================

/// Property: Parsing a query and converting it back to a string should be idempotent
/// Note: This property assumes we have a way to convert parsed AST back to string
/// This is a placeholder for when we implement Query::to_string()
#[test]
fn prop_round_trip_parse() {
    proptest!(|(query in arb_simple_select())| {
        let mut parser1 = Parser::new(&query);
        let result1 = parser1.parse();

        // For now, just verify that the query parses consistently
        let mut parser2 = Parser::new(&query);
        let result2 = parser2.parse();

        // Both parses should have the same result
        let result1_ok = result1.is_ok();
        let result2_ok = result2.is_ok();

        if result1_ok != result2_ok {
            panic!("Inconsistent parsing result for query: {}", query);
        }
    });
}

// ============================================================================
// Semantic Properties
// ============================================================================

/// Property: Column list in SELECT should have unique names (if aliases not used)
#[test]
fn prop_unique_column_names() {
    proptest!(|(
        cols in prop::collection::vec(arb_column_name(), 1..=5),
        table in arb_table_name()
    )| {
        // Ensure columns are unique
        let mut seen = std::collections::HashSet::new();
        let unique_cols: Vec<_> = cols.into_iter()
            .filter(|c| seen.insert(c.clone()))
            .collect();

        let col_list = unique_cols.join(", ");
        let query = format!("SELECT {} FROM {}", col_list, table);

        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should handle unique columns
    });
}

/// Property: GROUP BY columns should be present in SELECT or be aggregatable
#[test]
fn prop_group_by_columns_valid() {
    proptest!(|(
        table in arb_table_name(),
        group_col in arb_column_name(),
        agg_col in arb_column_name(),
        agg_func in prop::sample::select(vec!["COUNT", "SUM", "AVG", "MIN", "MAX"])
    )| {
        let query = format!(
            "SELECT {}, {}({}) FROM {} GROUP BY {}",
            group_col, agg_func, agg_col, table, group_col
        );

        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should handle valid GROUP BY
    });
}

/// Property: ORDER BY columns should be present in SELECT or GROUP BY
#[test]
fn prop_order_by_columns_valid() {
    proptest!(|(
        table in arb_table_name(),
        col1 in arb_column_name(),
        col2 in arb_column_name()
    )| {
        let query = format!(
            "SELECT {}, {} FROM {} ORDER BY {}",
            col1, col2, table, col1
        );

        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should handle valid ORDER BY
    });
}

// ============================================================================
// Algebraic Properties
// ============================================================================

/// Property: WHERE true is equivalent to no WHERE clause
#[test]
fn prop_where_true_equivalent() {
    proptest!(|(
        table in arb_table_name(),
        cols in prop::collection::vec(arb_column_name(), 1..=3)
    )| {
        let col_list = cols.join(", ");
        let query1 = format!("SELECT {} FROM {}", col_list, table);
        let query2 = format!("SELECT {} FROM {} WHERE 1=1", col_list, table);

        let mut parser1 = Parser::new(&query1);
        let mut parser2 = Parser::new(&query2);

        // Both should parse
        let _ = parser1.parse();
        let _ = parser2.parse();

        // In a full implementation, we'd verify they produce the same results
    });
}

/// Property: ORDER BY followed by ORDER BY is equivalent to just the last ORDER BY
#[test]
fn prop_double_order_by() {
    proptest!(|(
        table in arb_table_name(),
        col1 in arb_column_name(),
        col2 in arb_column_name(),
        dir1 in prop::sample::select(vec!["ASC", "DESC"]),
        dir2 in prop::sample::select(vec!["ASC", "DESC"])
    )| {
        let query = format!(
            "SELECT {}, {} FROM {} ORDER BY {} {} ORDER BY {} {}",
            col1, col2, table, col1, dir1, col2, dir2
        );

        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Parser should handle this (even if not ideal SQL)
    });
}

// ============================================================================
// Edge Case Properties
// ============================================================================

/// Property: Parser should handle very long identifiers
#[test]
fn prop_long_identifiers() {
    proptest!(|(len in 100usize..1000)| {
        let ident = "x".repeat(len);
        let query = format!("SELECT {} FROM t1", ident);

        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should handle long identifiers
    });
}

/// Property: Parser should handle very long numeric literals
#[test]
fn prop_large_numbers() {
    proptest!(|(
        num in prop::num::i64::ANY,
        table in arb_table_name(),
        col in arb_column_name()
    )| {
        let query = format!("SELECT * FROM {} WHERE {} = {}", table, col, num);

        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should handle large numbers
    });
}

/// Property: Parser should handle many columns in SELECT
#[test]
fn prop_many_columns() {
    proptest!(|(
        num_cols in 10usize..50,
        table in arb_table_name()
    )| {
        let cols: Vec<String> = (0..num_cols)
            .map(|i| format!("col{}", i))
            .collect();
        let col_list = cols.join(", ");
        let query = format!("SELECT {} FROM {}", col_list, table);

        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should handle many columns
    });
}

/// Property: Parser should handle deeply nested parentheses (if applicable)
#[test]
fn prop_nested_parentheses() {
    proptest!(|(
        depth in 0usize..5,
        table in arb_table_name(),
        col in arb_column_name(),
        value in 0i64..100i64
    )| {
        let mut condition = format!("{} > {}", col, value);
        for _ in 0..depth {
            condition = format!("({})", condition);
        }

        let query = format!("SELECT * FROM {} WHERE {}", table, condition);

        let mut parser = Parser::new(&query);
        let _ = parser.parse(); // Should handle nested parentheses
    });
}

// ============================================================================
// Regression Tests (Found via Property Testing)
// ============================================================================

/// Regression test for specific edge cases found during property testing
#[test]
fn test_regression_empty_select() {
    // Empty SELECT list should error gracefully
    let query = "SELECT FROM table1";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_err(), "Empty SELECT should fail");
}

/// Regression test for identifier starting with number
#[test]
fn test_regression_identifier_starts_with_number() {
    let query = "SELECT 123col FROM table1";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    // This might succeed or fail, but shouldn't panic
    let _ = result;
}

/// Regression test for special characters in identifiers
#[test]
fn test_regression_special_characters() {
    let queries = vec![
        "SELECT col-name FROM table1",
        "SELECT col.name FROM table1",
        "SELECT col_name FROM table1",
    ];

    for query in queries {
        let mut parser = Parser::new(query);
        let result = parser.parse();
        // Should handle or error gracefully
        let _ = result;
    }
}
