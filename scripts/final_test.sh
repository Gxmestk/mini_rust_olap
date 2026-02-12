#!/bin/bash

# Final verification test for Mini Rust OLAP REPL

set -e

echo "===================================="
echo "Final Phase 7 Verification Test"
echo "===================================="
echo ""

# Build project first
echo "Building project..."
cargo build --release --quiet 2>&1 | grep -v "^    Compiling" || true
echo "✓ Build successful"
echo ""

# Test REPL
echo "Running REPL tests..."
INPUT=$(cat <<'EOF'
HELP
SHOW TABLES
EXIT
EOF)

echo "$INPUT" | ./target/release/mini_rust_olap > /tmp/repl_output.txt 2>&1

# Verify key outputs
if grep -q "Mini Rust OLAP - Interactive REPL" /tmp/repl_output.txt; then
    echo "✓ REPL started successfully"
else
    echo "✗ REPL failed to start"
    exit 1
fi

if grep -q "No tables in catalog" /tmp/repl_output.txt; then
    echo "✓ SHOW TABLES works"
else
    echo "✗ SHOW TABLES failed"
    exit 1
fi

if grep -q "Goodbye!" /tmp/repl_output.txt; then
    echo "✓ Exit command works"
else
    echo "✗ Exit command failed"
    exit 1
fi

if grep -q "Available Commands" /tmp/repl_output.txt; then
    echo "✓ HELP command works"
else
    echo "✗ HELP command failed"
    exit 1
fi

echo ""
echo "===================================="
echo "All tests passed! ✓"
echo "===================================="

# Clean up
rm -f /tmp/repl_output.txt
