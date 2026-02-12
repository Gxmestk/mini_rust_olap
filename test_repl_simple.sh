#!/bin/bash

# Simple test script for Mini Rust OLAP REPL
# This script tests GROUP BY functionality without complex aggregates

set -e  # Exit on error

echo "===================================="
echo "Testing Mini Rust OLAP REPL - Simple Test"
echo "===================================="
echo ""

# Define the test input commands
INPUT=$(cat <<'EOF'
LOAD test_data.csv AS employees
SHOW TABLES
SELECT * FROM employees
SELECT name, department FROM employees
SELECT department FROM employees GROUP BY department
SELECT name, salary FROM employees WHERE salary > 70000 ORDER BY salary ASC
SELECT COUNT(*) FROM employees
EXIT
EOF
)

# Run the REPL with the test input
echo "$INPUT" | cargo run --quiet

echo ""
echo "===================================="
echo "Test completed!"
echo "===================================="
