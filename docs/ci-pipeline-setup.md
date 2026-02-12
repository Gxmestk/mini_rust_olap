# CI/CD Pipeline Setup Guide

## 📋 Table of Contents

1. [Introduction](#1-introduction)
2. [Pipeline Overview](#2-pipeline-overview)
3. [Pre-commit Hook](#3-pre-commit-hook)
4. [Pre-push Hook](#4-pre-push-hook)
5. [Installation](#5-installation)
6. [Usage](#6-usage)
7. [What's Checked](#7-whats-checked)
8. [Troubleshooting](#8-troubleshooting)
9. [Best Practices](#9-best-practices)
10. [Advanced Configuration](#10-advanced-configuration)

---

## 1. Introduction

### What is a CI/CD Pipeline?

A **CI/CD (Continuous Integration/Continuous Deployment)** pipeline automatically validates your code before it's committed or pushed to remote repositories. Unlike GitHub Actions or other external CI systems, our pipeline runs **locally** using git hooks, providing instant feedback without waiting for remote builds.

### Why Local Git Hooks?

**Benefits:**
- **Instant Feedback**: Get immediate feedback on your changes
- **No External Dependencies**: Runs without GitHub Actions, Jenkins, etc.
- **Faster Development**: Fail fast, iterate quickly
- **Privacy**: All checks happen on your machine
- **Cost-Effective**: No build minutes or server costs

**Trade-offs:**
- **Local Only**: Doesn't replace remote CI for team workflows
- **Manual Setup**: Each developer must install hooks
- **No Build Artifacts**: Doesn't create deployable artifacts

### Our Pipeline Philosophy

Mini Rust OLAP's CI pipeline follows **Rust Standard Best Practices**:

```
┌─────────────────────────────────────────────────────────┐
│        Mini Rust OLAP CI Pipeline                 │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────┐    ┌─────────────┐          │
│  │   Commit    │    │    Push     │          │
│  └──────┬──────┘    └──────┬──────┘          │
│         │                    │                    │
│         ▼                    ▼                    │
│  ┌────────────────────────────────────────┐         │
│  │     Pre-commit Hook                   │         │
│  │  - Format check                     │         │
│  │  - Clippy linting                    │         │
│  │  - Compilation check                 │         │
│  │  - Documentation check                │         │
│  │  - Unit tests                       │         │
│  │  - Integration tests                │         │
│  └──────────────┬─────────────────────────┘         │
│                 │                                    │
│                 ▼                                    │
│  ┌────────────────────────────────────────┐         │
│  │     Pre-push Hook                      │         │
│  │  - All pre-commit checks             │         │
│  │  - Release mode compilation           │         │
│  │  - Release mode tests               │         │
│  │  - Generated files check            │         │
│  │  - Code coverage (optional)        │         │
│  │  - README examples check            │         │
│  │  - TODO/FIXME check               │         │
│  └──────────────┬─────────────────────────┘         │
│                 │                                    │
│                 ▼                                    │
│  ┌────────────────────────────────────────┐         │
│  │     Push to GitHub                 │         │
│  └────────────────────────────────────────┘         │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Pipeline Overview

### Pipeline Components

| Component | File | Purpose | Trigger |
|-----------|------|---------|----------|
| **Pre-commit Hook** | `.githooks/pre-commit` | Before `git commit` |
| **Pre-push Hook** | `.githooks/pre-push` | Before `git push` |
| **Setup Script** | `scripts/setup-ci-hooks.sh` | Install/uninstall hooks |

### Check Hierarchy

```
Pre-commit (Fast Checks)
├── Format check        (cargo fmt)
├── Clippy linting    (cargo clippy)
├── Compile check      (cargo check)
├── Documentation      (cargo doc)
├── Unit tests        (cargo test --lib)
└── Integration tests (cargo test --test)
   ↓
Pre-push (Comprehensive Checks)
├── All pre-commit checks
├── Release build      (cargo check --release)
├── Release tests      (cargo test --release)
├── Generated files    (Cargo.lock, target/)
├── Code coverage     (cargo tarpaulin)
├── README examples    (extract & validate)
└── TODO/FIXME       (grep for comments)
   ↓
Push to Remote
```

### Tools Used

| Tool | Purpose | Command |
|-------|---------|----------|
| **cargo fmt** | Code formatting | `cargo fmt --all -- --check` |
| **cargo clippy** | Rust linter | `cargo clippy --all-targets` |
| **cargo check** | Compilation check | `cargo check --all-targets` |
| **cargo doc** | Documentation check | `cargo doc --no-deps` |
| **cargo test** | Run tests | `cargo test --lib --test` |
| **cargo-tarpaulin** | Code coverage | `cargo tarpaulin` |
| **git** | Version control | Various git commands |

---

## 3. Pre-commit Hook

### What It Does

The pre-commit hook runs **before** you create a commit. It performs fast checks to catch common issues early.

### Checks Performed

#### 1. Formatting Check

```bash
cargo fmt --all -- --check
```

**Purpose**: Ensures code follows Rust's standard formatting style.

**What Happens on Failure**:
- The hook runs `cargo fmt --all` to auto-format your code
- Shows you which files were formatted
- Prevents commit until you review and recommit

**Example Output**:
```
========================================
1. Formatting Check (cargo fmt)
========================================
✗ Code is not properly formatted

Run 'cargo fmt' to auto-format your code:

Diff in src/column.rs at line 42:
-    let mut ids = Vec::new();
+    let mut ids = Vec::new();
```

#### 2. Clippy Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

**Purpose**: Catches common Rust mistakes and suggests improvements.

**What Happens on Failure**:
- Shows clippy warnings and suggestions
- Provides command to fix issues
- Prevents commit until warnings are resolved

**Common Clippy Warnings**:
- Unnecessary allocations
- Unused variables
- Inefficient patterns
- Potential bugs

**Example Output**:
```
========================================
2. Clippy Linting
========================================
✗ Clippy found issues

warning: this expression borrows a local value here...
   --> src/column.rs:127:9
    |
127 |         .map(|v| Value::Int64(*v))
    |              ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: consider removing the dereference
    |
    = note: `Value` implements the `Copy` trait

Please fix the clippy warnings above:
 cargo clippy --all-targets --all-features
```

#### 3. Compile Check

```bash
cargo check --all-targets
```

**Purpose**: Verifies code compiles without producing binaries.

**What Happens on Failure**:
- Shows compilation errors
- Provides line numbers and context
- Prevents commit until code compiles

**Example Output**:
```
========================================
3. Compile Check (cargo check)
========================================
✗ Code does not compile

error[E0382]: use of moved value: `column`
   --> src/table.rs:45:13
    |
45  |         let data = column.as_vec();
    |                    ^^^^^^ value moved here
46  |         println!("{:?}", data);
    |                         ---- value used here
    |
help: consider cloning the value before using it
    |
    = note: value moved here
   --> src/table.rs:43:9
    |
43  |     let column = create_column(DataType::Int64);
    |         ------- value moved here
```

#### 4. Documentation Check

```bash
cargo doc --no-deps --document-private-items
```

**Purpose**: Ensures documentation compiles without warnings.

**What Happens on Failure**:
- Shows documentation warnings
- Links to problematic documentation
- Prevents commit until docs are clean

**Example Output**:
```
========================================
4. Documentation Check (cargo doc)
========================================
✗ Documentation has issues

warning: unused struct: `IntColumn`
 --> src/column.rs:10:1
  |
10 | pub struct IntColumn {
  | ^^^^^^^^^^^^
  |
  = note: `IntColumn` has public visibility but is never used
```

#### 5. Unit Tests

```bash
cargo test --lib --quiet
```

**Purpose**: Runs unit tests (in `src/` files).

**What Happens on Failure**:
- Shows which tests failed
- Displays assertion errors
- Prevents commit until tests pass

**Example Output**:
```
========================================
5. Unit Tests (cargo test --lib)
========================================
✗ Unit tests failed

failures:

---- tests::test_column_get_out_of_bounds stdout ----
thread 'tests::test_column_get_out_of_bounds' panicked at src/column.rs:152:9:
assertion `left == right` failed
  left: "Column error: Index 0 out of bounds (len: 0)"
 right: "Expected error for empty column"
```

#### 6. Integration Tests

```bash
cargo test --test --quiet
```

**Purpose**: Runs integration tests (in `tests/` directory).

**What Happens on Failure**:
- Shows which tests failed
- Displays assertion errors
- Prevents commit until tests pass

**Example Output**:
```
========================================
6. Integration Tests (cargo test --test)
========================================
✗ Integration tests failed

failures:

---- manual_query::test_manual_sum_aggregation stdout ----
thread 'manual_query::test_manual_sum_aggregation' panicked at tests/manual_query.rs:45:9:
assertion `left == right` failed
  left: 180000
 right: 50000
```

### Pre-commit Success

When all checks pass:

```
========================================
Pre-commit Checks: All Passed!
========================================

✓ Code is properly formatted
✓ No clippy warnings found
✓ Code compiles successfully
✓ Documentation compiles successfully
✓ All unit tests passed
✓ All integration tests passed

Changes staged for commit:
 M  src/column.rs
 M  tests/manual_query.rs
```

---

## 4. Pre-push Hook

### What It Does

The pre-push hook runs **before** you push to a remote repository. It performs all pre-commit checks plus additional validations to ensure only high-quality code is pushed.

### Additional Checks Beyond Pre-commit

#### 7. Release Mode Compilation

```bash
cargo check --all-targets --release
```

**Purpose**: Verifies code compiles with release optimizations.

**Why This Matters**:
- Release mode enables optimizations that might expose different bugs
- Ensures code will build successfully when creating releases
- Catches issues only visible in optimized builds

**Example Issues Caught**:
- Debug assertions that panic in release
- Unused code not eliminated in debug mode
- Integer overflow in release builds
- Dead code that disappears with optimizations

#### 8. Release Mode Tests

```bash
cargo test --all --release --quiet
```

**Purpose**: Runs all tests with release optimizations.

**Why This Matters**:
- Tests might behave differently in release mode
- Optimizations can change behavior subtly
- Ensures correctness before deploying

**Example Issues Caught**:
- Race conditions more likely in optimized builds
- Floating point precision changes
- Memory layout differences affecting unsafe code
- Timing-dependent bugs

#### 9. Generated Files Check

```bash
# Check Cargo.lock
git diff --name-only --exit-code Cargo.lock

# Check target/ directory
git diff --name-only --exit-code target/
```

**Purpose**: Ensures generated files are properly managed.

**What's Checked**:
- **Cargo.lock**: Uncommitted dependency lock file changes
- **target/**:** Uncommitted build artifacts

**Why This Matters**:
- `Cargo.lock` changes should be committed with dependency updates
- `target/` should be in `.gitignore` (build artifacts)
- Prevents committing build artifacts to version control

**Example Output**:
```
========================================
9. Generated Files Check
========================================
✓ Cargo.lock is up to date
✓ No uncommitted changes in generated directories
```

#### 10. Code Coverage (Optional)

```bash
cargo tarpaulin --lib -- --test --out Html --output-dir target/tarpaulin-report --line -- --avoid-cfg-tarpaulin
```

**Purpose**: Measures how much of the code is tested.

**Why This Matters**:
- Identifies untested code
- Tracks improvement over time
- Provides confidence in code quality

**Coverage Thresholds**:
- **70%+**: Good coverage for a learning project
- **80%+**: Production-ready level
- **90%+**: Excellent coverage

**Example Output**:
```
========================================
10. Code Coverage Check (Optional)
========================================
ℹ Running code coverage check...

|| Tested/Uncovered || ||
===============================
|                83.3%            |
|===============================|
```

**Note**: Coverage check is informational, not blocking. It won't prevent push if below threshold.

#### 11. README Examples Check

**Purpose**: Ensures code examples in README.md are present and valid.

**What's Checked**:
- Presence of code blocks marked with ```rust
- Count of examples found
- Syntactic validity (basic check)

**Why This Matters**:
- Examples should work for new users
- Documentation should match actual code
- Prevents outdated examples

**Example Output**:
```
========================================
11. README Examples Check
========================================
ℹ Checking README.md for code examples...
ℹ Found 15 Rust code examples in README.md
✓ README.md examples present
```

#### 12. Documentation Completeness

**Purpose**: Verifies essential documentation files exist.

**What's Checked**:
- `README.md` present?
- `docs/references/progress.md` present (recommended)?
- `docs/` directory exists?
- `docs/phase1-learning-guide.md` exists?

**Why This Matters**:
- Ensures project is well-documented
- Tracks educational resources
- Maintains project standards

**Example Output**:
```
========================================
12. Documentation Completeness
========================================
✓ Core documentation files present
```

#### 13. TODO/FIXME Check

```bash
git diff --cached --name-only --diff-filter=AM | xargs grep -h "TODO\|FIXME" 2>/dev/null | wc -l
```

**Purpose**: Warns about TODOs and FIXMEs in staged files.

**Why This Matters**:
- TODOs might indicate incomplete work
- Encourages cleaning up before pushing
- Improves code review experience

**Example Output**:
```
========================================
13. TODO/FIXME Check
========================================
⚠ Found 2 TODO/FIXME comment(s) in staged files

Please review these before pushing:

src/column.rs:45:10:// TODO: Add column compression
src/table.rs:78:5:// FIXME: Optimize this function

ℹ This is a warning, not blocking
```

### Pre-push Success

When all checks pass:

```
========================================
Pre-push Checks: All Passed!
========================================

✓ Formatting: Proper
✓ Clippy: No warnings
✓ Compilation: Debug & Release modes
✓ Documentation: Compiles
✓ Tests: Debug & Release modes (all passing)
✓ Generated files: Clean
✓ Coverage: Checked (if available)
✓ Examples: Present in README
✓ Documentation: Core files present
✓ TODO/FIXME: None in staged files

Summary of checks:
  ✓ Formatting: Proper
  ✓ Clippy: No warnings
  ✓ Compilation: Debug & Release modes
  ✓ Documentation: Compiles
  ✓ Tests: Debug & Release modes (all passing)
  ✓ Generated files: Clean
  ✓ Coverage: Checked (if available)
  ✓ Examples: Present in README
  ✓ Documentation: Core files present

Files being pushed:
 M  src/column.rs
 M  tests/manual_query.rs

Remote repository:
origin  git@github.com:Gxmestk/mini_rust_olap.git (push)

Pushing to remote...
```

---

## 5. Installation

### Prerequisites

```bash
# Check you're in a git repository
git rev-parse --git-dir

# Check hooks directory exists (created automatically)
ls -la .githooks/

# Check setup script exists
ls -la scripts/setup-ci-hooks.sh
```

### Installation Steps

#### Step 1: Run Setup Script

```bash
# Navigate to project root
cd mini_rust_olap

# Run setup script
./scripts/setup-ci-hooks.sh

# Or with bash explicitly
bash scripts/setup-ci-hooks.sh
```

**Expected Output**:
```
========================================
Installing Git Hooks
========================================

========================================
Installing: pre-commit hook
========================================
ℹ Installing pre-commit hook...

Backup created: .git/hooks/pre-commit.backup.20240103_143052
✓ Installed: pre-commit hook

========================================
Installing: pre-push hook
========================================
ℹ Installing pre-push hook...

Backup created: .git/hooks/pre-push.backup.20240103_143053
✓ Installed: pre-push hook

ℹ Making source hooks executable...
✓ Made executable: .githooks/pre-commit
✓ Made executable: .githooks/pre-push

ℹ Verifying installation...
✓ All hooks installed successfully!

Installed hooks:
 - pre-commit: Runs before commits
 - pre-push: Runs before pushes

What they check:
 - Code formatting (cargo fmt)
 - Linting (cargo clippy)
 - Compilation (cargo check)
 - Documentation (cargo doc)
 - Tests (unit + integration)
 - Generated files (Cargo.lock, target/)
 - TODO/FIXME comments
```

#### Step 2: Verify Installation

```bash
# Check hook status
./scripts/setup-ci-hooks.sh --check
```

**Expected Output**:
```
========================================
Git Hooks Status
========================================

✓ pre-commit hook: Installed
  Location: .git/hooks/pre-commit
✓ pre-push hook: Installed
  Location: .git/hooks/pre-push

✓ Source pre-commit: Found
✓ Source pre-push: Found
```

### Manual Installation (Alternative)

If setup script doesn't work for you:

```bash
# Copy pre-commit hook
cp .githooks/pre-commit .git/hooks/pre-commit

# Copy pre-push hook
cp .githooks/pre-push .git/hooks/pre-push

# Make executable
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push

# Verify
ls -la .git/hooks/ | grep -E "(pre-commit|pre-push)"
```

### Uninstallation

```bash
# Remove all hooks
./scripts/setup-ci-hooks.sh --uninstall

# Or manually
rm .git/hooks/pre-commit .git/hooks/pre-push

# Remove backups if desired
rm .git/hooks/*.backup.*
```

---

## 6. Usage

### Normal Development Workflow

```
1. Make code changes
   ↓
2. Stage changes (git add)
   ↓
3. Attempt commit (git commit)
   ↓
4. Pre-commit hook runs automatically
   ↓
5. Fix any issues reported
   ↓
6. Commit succeeds
   ↓
7. Make more changes or push
   ↓
8. Attempt push (git push)
   ↓
9. Pre-push hook runs automatically
   ↓
10. Fix any issues reported
    ↓
11. Push succeeds
```

### Using Hooks

#### Committing (Pre-commit)

```bash
# Stage your changes
git add src/column.rs tests/column_tests.rs

# Try to commit
git commit -m "Add column compression feature"

# If checks pass:
# [main abc1234] Add column compression feature
#  2 files changed, 45 insertions(+), 12 deletions(-)

# If checks fail:
# ========================================
# 1. Formatting Check (cargo fmt)
# ========================================
# ✗ Code is not properly formatted
# ...
# Hook exited with code 1, commit aborted
```

#### Pushing (Pre-push)

```bash
# Commit your changes
git commit -m "Update column tests"

# Try to push
git push origin main

# If checks pass:
# Enumerating objects: 7, done.
# Counting objects: 100% (7/7), done.
# Delta compression using up to 8 threads
# Compressing objects: 100% (7/7), done.
# Writing objects: 100% (7/7), 8.42 KiB | 8.42 KiB/s, done.
# Total 7 (delta 3), reused 0 (delta 0), pack-reused 0 (delta 0)
# To github.com:Gxmestk/mini_rust_olap.git
#   abc1234..def5678  main -> main
# Updating abc1234..def5678
# Fast-forward
# github.com:Gxmestk/mini_rust_olap.git -> abc1234..def5678

# If checks fail:
# ========================================
# 6. Integration Tests (Debug Mode)
# ========================================
# ✗ Integration tests failed
# ...
# Hook exited with code 1, push aborted
```

### Bypassing Hooks (Not Recommended)

**WARNING**: Only bypass hooks if you know what you're doing!

```bash
# Commit without hooks
git commit --no-verify -m "WIP: Force commit"

# Push without hooks
git push --no-verify origin main
```

**When to Bypass**:
- Experimental work not ready for hooks
- Fixing hook configuration itself
- Emergency fixes to critical issues
- Temporary workarounds during refactoring

---

## 7. What's Checked

### Pre-commit Checks Summary

| Check | Tool | Time | Blocking |
|-------|------|------|----------|
| Formatting | `cargo fmt` | <1s | ✅ Yes |
| Clippy Linting | `cargo clippy` | 5-10s | ✅ Yes |
| Compilation | `cargo check` | 10-30s | ✅ Yes |
| Documentation | `cargo doc` | 5-15s | ✅ Yes |
| Unit Tests | `cargo test --lib` | 5-20s | ✅ Yes |
| Integration Tests | `cargo test --test` | 10-30s | ✅ Yes |

**Total Time**: ~30-105 seconds

### Pre-push Checks Summary

| Check | Tool | Time | Blocking |
|-------|------|------|----------|
| All Pre-commit Checks | varies | ~30-105s | ✅ Yes |
| Release Compilation | `cargo check --release` | 30-90s | ✅ Yes |
| Release Tests | `cargo test --release` | 30-90s | ✅ Yes |
| Generated Files | `git diff` | <1s | ✅ Yes |
| Code Coverage | `cargo tarpaulin` | 20-40s | ⚠️ No (info) |
| README Examples | `grep` | <1s | ⚠️ No (info) |
| Documentation Files | `ls` | <1s | ⚠️ No (info) |
| TODO/FIXME | `grep` | <1s | ⚠️ No (warning) |

**Total Time**: ~110-325 seconds

### Check Frequency

| Event | Checks Run | Total Time |
|--------|-------------|------------|
| **Commit** | Pre-commit (6 checks) | 30-105s |
| **Push** | Pre-commit + Pre-push (13 checks) | 110-325s |

**Example**:
- 10 commits/day = ~10-17 minutes/day
- 3 pushes/day = ~5-16 minutes/day

---

## 8. Troubleshooting

### Issue: Hook Not Running

**Symptom**: Changes commit/push without hook execution.

**Possible Causes**:
1. Hook file not executable
2. Hook in wrong location
3. Git configuration disabled hooks

**Solutions**:

```bash
# 1. Check if executable
ls -l .git/hooks/pre-commit
# Should show: -rwxr-xr-x (executable)

# If not executable:
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push

# 2. Check location
ls -la .git/hooks/ | grep pre
# Should show both pre-commit and pre-push

# 3. Check if hooks enabled
git config --get core.hooksPath
# Should show: .git/hooks

# If hooksPath is set incorrectly:
git config core.hooksPath .git/hooks
```

### Issue: Formatting Check Fails

**Symptom**: `cargo fmt` reports unformatted code but formatting doesn't help.

**Solutions**:

```bash
# Auto-format all code
cargo fmt --all

# Verify formatting
cargo fmt --all -- --check

# Check if files are tracked by git
git status

# Format and stage changes
cargo fmt --all
git add -u

# Try commit again
git commit -m "Fix formatting"
```

### Issue: Clippy Warnings

**Symptom**: Clippy reports warnings but you can't fix them.

**Solutions**:

```bash
# Get detailed clippy output
cargo clippy --all-targets --all-features -D warnings

# Common fixes:

# 1. Unnecessary clone
# Before:
let data = column.as_vec().clone();
# After:
let data = column.as_vec();

# 2. Unnecessary allocation
# Before:
let s = format!("Value: {:?}", v);
# After:
let s = format!("Value: {v:?}");

# 3. Iteration methods
# Before:
for i in 0..vec.len() {
    println!("{}", vec[i]);
}
# After:
for (i, item) in vec.iter().enumerate() {
    println!("{}: {}", i, item);
}
```

### Issue: Test Failures

**Symptom**: Tests fail but you're not sure why.

**Solutions**:

```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_column_push_value

# Run tests in release mode
cargo test --release

# Run with logging
RUST_BACKTRACE=1 cargo test

# Run only failing tests
cargo test -- --fail-fast

# Run with nextest cache disabled
cargo clean
cargo test
```

### Issue: Permission Denied

**Symptom**: `permission denied: .git/hooks/pre-commit`

**Solutions**:

```bash
# Make executable
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push
chmod +x .githooks/pre-commit
chmod +x .githooks/pre-push
chmod +x scripts/setup-ci-hooks.sh

# Check ownership
ls -l .git/hooks/

# If wrong owner:
sudo chown $USER:$USER .git/hooks/pre-commit
```

### Issue: Hook Too Slow

**Symptom**: Hooks take too long to run.

**Solutions**:

```bash
# 1. Skip documentation check (pre-commit)
# Edit .githooks/pre-commit
# Comment out cargo doc check

# 2. Skip release mode tests (pre-push)
# Edit .githooks/pre-push
# Comment out cargo test --release

# 3. Reduce parallelism
# Edit .githooks
# Use cargo test --jobs 1

# 4. Incremental testing
# Only run tests for changed files
# (requires advanced setup)
```

---

## 9. Best Practices

### 1. Run Tests Locally First

```bash
# Always run tests before committing
cargo test

# This gives you faster feedback than hooks
# Hooks are your last line of defense, not first
```

### 2. Fix Issues Incrementally

```bash
# Don't try to fix everything at once

# 1. Fix formatting
cargo fmt --all
git add -u

# 2. Commit
git commit -m "Fix formatting"

# 3. Fix clippy warnings
# Edit code
cargo clippy

# 4. Commit
git commit -m "Fix clippy warnings"

# 5. Run tests
cargo test

# 6. Commit
git commit -m "Fix test failures"
```

### 3. Keep Commits Small

```bash
# Small commits make hooks faster
# and make debugging easier

# Bad: One giant commit
git add .
git commit -m "Huge feature implementation"

# Good: Multiple small commits
git add src/column.rs
git commit -m "Add column compression"
git add src/column.rs
git commit -m "Add column statistics"
git add src/column.rs
git commit -m "Optimize column operations"
```

### 4. Write Meaningful Commit Messages

```bash
# Good commit messages help with debugging
# They're shown when hooks fail

# Bad:
git commit -m "fix stuff"
git commit -m "update"

# Good:
git commit -m "Fix: Handle empty columns in get() method"
git commit -m "Add: Vectorized sum aggregation"
git commit -m "Refactor: Simplify column trait"
git commit -m "Test: Add edge case for negative indices"
```

### 5. Use Git Stash for Experimental Work

```bash
# Stash uncommitted work
git stash push

# Work on something else
git checkout -b new-feature

# When done, switch back
git checkout main

# Restore stash
git stash pop

# Hooks will run on commit of stashed changes
```

### 6. Review Hook Output

```bash
# Always read hook output carefully
# It often contains helpful hints

# Example from clippy:
# help: consider removing the dereference
# = note: `Value` implements the `Copy` trait

# Example from compiler:
# help: consider cloning the value before using it
```

### 7. Update Dependencies Carefully

```bash
# Update dependencies
cargo update some-crate

# This changes Cargo.lock
# Pre-push will warn you

# Test with new dependencies
cargo test

# Update Cargo.lock together with Cargo.toml
git add Cargo.toml Cargo.lock
git commit -m "Update dependencies"
```

---

## 10. Advanced Configuration

### Customizing Hook Behavior

#### Skip Coverage Check

Edit `.githooks/pre-push`:

```bash
# Skip coverage check
SKIP_COVERAGE=true

if [ "$SKIP_COVERAGE" != "true" ] && command -v cargo-tarpaulin &> /dev/null; then
    print_info "Running code coverage check..."
    # ... coverage check code
fi
```

**Usage**:
```bash
# Skip coverage for this push
SKIP_COVERAGE=true git push
```

#### Adjust Clippy Strictness

Edit `.githooks/pre-commit`:

```bash
# Use stricter clippy flags
CLIPPY_FLAGS="--all-targets --all-features -W clippy::all"

# Or use warnings only
CLIPPY_FLAGS="--all-targets --all-features -D warnings"
```

#### Custom Test Command

Edit `.githooks/pre-commit`:

```bash
# Run tests with specific features
cargo test --features "csv,serde" --lib

# Or run only specific test files
cargo test --lib --test column
```

### Adding Custom Checks

#### Check for Large Files

```bash
# Add to pre-commit hook
check_large_files() {
    print_header "Large File Check"
    
    MAX_SIZE=$((10 * 1024 * 1024))  # 10MB
    
    git diff --cached --name-only --diff-filter=AM | while read -r file; do
        size=$(stat -f%s "$file" 2>/dev/null)
        if [ "$size" -gt "$MAX_SIZE" ]; then
            print_warning "Large file staged: $file ($size bytes)"
            echo "Consider splitting into smaller files"
        fi
    done
}
```

#### Check for Sensitive Data

```bash
# Add to pre-commit hook
check_sensitive_data() {
    print_header "Sensitive Data Check"
    
    # Check for common patterns
    git diff --cached --text --diff-filter=AM | grep -i "password\|api_key\|secret"
    
    if [ $? -eq 0 ]; then
        print_error "Potential sensitive data found in staged changes"
        echo "Please review before committing"
        exit 1
    fi
}
```

### Performance Optimization

#### Parallel Test Execution

```bash
# Run tests in parallel
# Edit pre-commit hook
cargo test --lib --jobs 4 --test-threads=4
```

#### Incremental Testing

```bash
# Only test affected modules
# (Requires setting up workspace structure)
cargo test --lib --package mini_rust_olap
```

#### Caching Build Artifacts

```bash
# Keep target/ directory between runs
# Add to pre-commit:
export CARGO_TARGET_DIR=/path/to/shared/target

# First run creates cache
# Subsequent runs are faster
```

### Team Workflows

#### Shared Hooks Repository

For team projects, consider:

```bash
# Store hooks in separate repository
# Team members install via:
git clone https://github.com/org/shared-hooks.git
./shared-hooks/install.sh
```

#### Per-Developer Hooks

Allow developers to override:

```bash
# Check for local override
if [ -f .git/hooks.local/pre-commit ]; then
    echo "Using local pre-commit hook"
    exec .git/hooks.local/pre-commit "$@"
fi
```

### Hook Maintenance

#### Updating Hooks

```bash
# When updating hooks:
# 1. Pull latest changes
git pull origin main

# 2. Reinstall hooks
./scripts/setup-ci-hooks.sh --force

# 3. Verify installation
./scripts/setup-ci-hooks.sh --check
```

#### Adding New Checks

```bash
# Add new check to pre-commit
# 1. Edit .githooks/pre-commit
# 2. Add check in appropriate section
# 3. Test locally
git commit --no-verify -m "Test new hook"

# 4. Commit updated hook
git add .githooks/pre-commit
git commit -m "Add new check to pre-commit hook"
```

---

## 🎓 Summary

### Key Takeaways

1. **Local CI is Fast**: Get instant feedback without waiting for remote builds
2. **Rust Standards**: Use `cargo fmt`, `clippy`, `cargo test` for consistency
3. **Fail Fast**: Hooks catch issues early, preventing broken code from being committed
4. **Incremental**: Fix issues one at a time, not all at once
5. **Meaningful Commits**: Write clear messages to help with debugging

### Next Steps

1. **Install Hooks**: Run `./scripts/setup-ci-hooks.sh`
2. **Test Hooks**: Make a change and try to commit
3. **Read Output**: Understand what each check does
4. **Customize**: Add checks specific to your workflow
5. **Share with Team**: Document hooks for team members

### Resources

- [Git Hooks Documentation](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks.html)
- [Cargo Commands](https://doc.rust-lang.org/cargo/commands/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Code Coverage](https://github.com/rust-lang/cargo-tarpaulin)

---

**Happy coding with confidence!** Your CI pipeline ensures code quality every step of the way! 🚀