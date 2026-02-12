#!/bin/bash

# Test script for Mini Rust OLAP REPL
# This script tests the basic functionality of the REPL

set -e  # Exit on error

echo "===================================="
echo "Testing Mini Rust OLAP REPL"
echo "===================================="
echo ""

# Define the test input commands
INPUT=$(cat <<'EOF'
SHOW TABLES
HELP
LOAD test_data.csv AS employees
SHOW TABLES
DESCRIBE employees
SELECT * FROM employees
SELECT name, department, salary FROM employees WHERE salary > 80000
SELECT department, COUNT(*) as count, AVG(salary) as avg_salary FROM employees GROUP BY department
SELECT name, salary FROM employees ORDER BY salary DESC LIMIT 3
SELECT * FROM employees WHERE name = 'Alice'
EXIT
EOF
)

# Run the REPL with the test input
echo "$INPUT" | cargo run --quiet

echo ""
echo "===================================="
echo "Test completed successfully!"
echo "===================================="
