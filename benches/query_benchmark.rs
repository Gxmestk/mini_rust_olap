//! Query Performance Benchmarks
//!
//! This benchmark suite measures the performance of various query operations
//! in the mini_rust_olap engine using CSV data. It helps identify performance
//! bottlenecks and track improvements over time.
//!
//! Run benchmarks with:
//!   cargo bench
//!
//! For detailed flamegraphs:
//!   cargo bench -- --profile-time <seconds>

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mini_rust_olap::catalog::Catalog;
use mini_rust_olap::ingest::load_csv_into_catalog;
use mini_rust_olap::parser::Parser;
use mini_rust_olap::planner::Planner;
use std::path::Path;

/// Execute a SQL query and return the total row count
fn execute_query(catalog: &Catalog, sql: &str) -> Result<usize, Box<dyn std::error::Error>> {
    // Parse the SQL query
    let mut parser = Parser::new(sql);
    let query = parser.parse()?;

    // Create a planner and plan the query
    let planner = Planner::new(catalog);
    let mut plan = planner.plan(&query)?;

    // Execute the query
    plan.open()?;

    let mut total_rows = 0;
    while let Some(batch) = plan.next_batch()? {
        total_rows += batch.row_count();
    }

    Ok(total_rows)
}

/// Setup catalog with test data
fn setup_catalog() -> Result<Catalog, Box<dyn std::error::Error>> {
    let mut catalog = Catalog::new();

    // Load test CSV data
    let csv_path = "tests/data/test_data.csv";
    if Path::new(csv_path).exists() {
        load_csv_into_catalog(csv_path, "employees".to_string(), &mut catalog)?;
    }

    Ok(catalog)
}

/// Benchmark SQL parsing performance
fn benchmark_sql_parsing(c: &mut Criterion) {
    let simple_query = "SELECT id, name FROM employees";
    let complex_query = "SELECT department, COUNT(*), AVG(salary) FROM employees WHERE salary > 70000 GROUP BY department";

    let mut group = c.benchmark_group("sql_parsing");

    group.bench_function("simple_query", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(simple_query));
            let _result = parser.parse();
        });
    });

    group.bench_function("complex_query", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(complex_query));
            let _result = parser.parse();
        });
    });

    group.finish();
}

/// Benchmark full table scan operations
fn benchmark_full_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_scan");

    if let Ok(catalog) = setup_catalog() {
        group.bench_function("select_all", |b| {
            let sql = "SELECT * FROM employees";
            b.iter(|| {
                let result = execute_query(black_box(&catalog), black_box(sql));
                criterion::black_box(result.ok());
            });
        });
    } else {
        println!("Warning: Test data not found, skipping full scan benchmarks");
    }

    group.finish();
}

/// Benchmark filter operations (WHERE clause)
fn benchmark_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter");

    if let Ok(catalog) = setup_catalog() {
        group.bench_function("numeric_filter", |b| {
            let sql = "SELECT * FROM employees WHERE salary > 80000";
            b.iter(|| {
                let result = execute_query(black_box(&catalog), black_box(sql));
                criterion::black_box(result.ok());
            });
        });

        group.bench_function("string_filter", |b| {
            let sql = "SELECT * FROM employees WHERE department = 'Engineering'";
            b.iter(|| {
                let result = execute_query(black_box(&catalog), black_box(sql));
                criterion::black_box(result.ok());
            });
        });
    } else {
        println!("Warning: Test data not found, skipping filter benchmarks");
    }

    group.finish();
}

/// Benchmark project operations (SELECT specific columns)
fn benchmark_project(c: &mut Criterion) {
    let mut group = c.benchmark_group("project");

    if let Ok(catalog) = setup_catalog() {
        group.bench_function("select_columns", |b| {
            let sql = "SELECT name, department, salary FROM employees";
            b.iter(|| {
                let result = execute_query(black_box(&catalog), black_box(sql));
                criterion::black_box(result.ok());
            });
        });
    } else {
        println!("Warning: Test data not found, skipping project benchmarks");
    }

    group.finish();
}

/// Benchmark aggregation operations (GROUP BY)
fn benchmark_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregation");

    if let Ok(catalog) = setup_catalog() {
        group.bench_function("group_by_count", |b| {
            let sql = "SELECT department, COUNT(*) FROM employees GROUP BY department";
            b.iter(|| {
                let result = execute_query(black_box(&catalog), black_box(sql));
                criterion::black_box(result.ok());
            });
        });

        group.bench_function("group_by_avg", |b| {
            let sql = "SELECT department, AVG(salary) FROM employees GROUP BY department";
            b.iter(|| {
                let result = execute_query(black_box(&catalog), black_box(sql));
                criterion::black_box(result.ok());
            });
        });
    } else {
        println!("Warning: Test data not found, skipping aggregation benchmarks");
    }

    group.finish();
}

/// Benchmark ORDER BY operations
fn benchmark_order_by(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_by");

    if let Ok(catalog) = setup_catalog() {
        group.bench_function("order_by_desc", |b| {
            let sql = "SELECT name, salary FROM employees ORDER BY salary DESC";
            b.iter(|| {
                let result = execute_query(black_box(&catalog), black_box(sql));
                criterion::black_box(result.ok());
            });
        });
    } else {
        println!("Warning: Test data not found, skipping order by benchmarks");
    }

    group.finish();
}

/// Benchmark full query execution (parse → plan → execute)
fn benchmark_full_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_query");

    if let Ok(catalog) = setup_catalog() {
        group.bench_function("filter_and_group", |b| {
            let sql = "SELECT department, COUNT(*) FROM employees WHERE salary > 75000 GROUP BY department";
            b.iter(|| {
                let result = execute_query(black_box(&catalog), black_box(sql));
                criterion::black_box(result.ok());
            });
        });

        group.bench_function("complex_query", |b| {
            let sql = "SELECT department, COUNT(*), AVG(salary), MAX(salary) FROM employees WHERE salary > 70000 GROUP BY department HAVING COUNT(*) >= 2 ORDER BY AVG(salary) DESC";
            b.iter(|| {
                let result = execute_query(black_box(&catalog), black_box(sql));
                criterion::black_box(result.ok());
            });
        });
    } else {
        println!("Warning: Test data not found, skipping full query benchmarks");
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_sql_parsing,
    benchmark_full_scan,
    benchmark_filter,
    benchmark_project,
    benchmark_aggregation,
    benchmark_order_by,
    benchmark_full_query
);
criterion_main!(benches);
