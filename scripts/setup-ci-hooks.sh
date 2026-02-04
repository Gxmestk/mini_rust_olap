#!/usr/bin/env bash
#
# CI Hooks Setup Script for Mini Rust OLAP
#
# This script installs git hooks for comprehensive Rust standard checks:
# - Pre-commit: Runs before commits (formatting, linting, tests)
# - Pre-push: Runs before pushes (all checks + additional validations)
#
# Usage:
#   ./setup-ci-hooks.sh              # Install hooks
#   ./setup-ci-hooks.sh --uninstall  # Remove hooks
#   ./setup-ci-hooks.sh --check     # Check if hooks are installed
#   ./setup-ci-hooks.sh --force     # Force reinstall hooks
#   ./setup-ci-hooks.sh --dry-run   # Show what would be done
#
# Author: Mini Rust OLAP Team
# License: MIT
#

set -e

# ============================================================================
# CONFIGURATION
# ============================================================================

# Script directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
GITHOOKS_DIR="$PROJECT_ROOT/.githooks"
GIT_HOOKS_DIR=".git/hooks"

# Hook files
PRE_COMMIT_FILE="$GITHOOKS_DIR/pre-commit"
PRE_PUSH_FILE="$GITHOOKS_DIR/pre-push"

# Target hook locations
TARGET_PRE_COMMIT="$GIT_HOOKS_DIR/pre-commit"
TARGET_PRE_PUSH="$GIT_HOOKS_DIR/pre-push"

# ============================================================================
# COLOR OUTPUT
# ============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print functions
print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ $1${NC}"
}

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

# Check if running in project root
check_in_project_root() {
    if [ ! -f "Cargo.toml" ] && [ ! -f "src/lib.rs" ]; then
        print_error "Not in project root directory"
        print_info "Current directory: $(pwd)"
        print_info "Expected directory containing: Cargo.toml and src/lib.rs"
        exit 1
    fi
}

# Check if hooks directory exists
check_hooks_directory() {
    if [ ! -d "$GITHOOKS_DIR" ]; then
        print_error "Git hooks directory not found: $GITHOOKS_DIR"
        print_info "Please ensure .githooks directory exists in project root"
        exit 1
    fi
}

# Check if hook files exist
check_hook_files() {
    local missing_files=()

    if [ ! -f "$PRE_COMMIT_FILE" ]; then
        missing_files+=("pre-commit")
    fi

    if [ ! -f "$PRE_PUSH_FILE" ]; then
        missing_files+=("pre-push")
    fi

    if [ ${#missing_files[@]} -gt 0 ]; then
        print_error "Missing hook file(s): ${missing_files[*]}"
        exit 1
    fi
}

# Check if hooks are already installed
check_hooks_installed() {
    local installed=true

    if [ ! -f "$TARGET_PRE_COMMIT" ]; then
        installed=false
    fi

    if [ ! -f "$TARGET_PRE_PUSH" ]; then
        installed=false
    fi

    if [ "$installed" = true ]; then
        return 0
    else
        return 1
    fi
}

# Make file executable
make_executable() {
    local file="$1"

    if [ -f "$file" ]; then
        chmod +x "$file"
        print_success "Made executable: $file"
    else
        print_error "File not found: $file"
        return 1
    fi
}

# Install hook
install_hook() {
    local source_file="$1"
    local target_file="$2"
    local hook_name="$3"

    print_info "Installing $hook_name..."

    # Check if target hook already exists
    if [ -f "$target_file" ]; then
        print_warning "$hook_name already exists"
        print_info "Backing up existing hook..."

        # Create backup
        local backup_file="${target_file}.backup.$(date +%Y%m%d_%H%M%S)"
        cp "$target_file" "$backup_file"
        print_success "Backup created: $backup_file"
    fi

    # Copy hook
    cp "$source_file" "$target_file"

    # Make executable
    chmod +x "$target_file"

    print_success "Installed: $hook_name"
}

# Remove hook
uninstall_hook() {
    local target_file="$1"
    local hook_name="$2"

    if [ -f "$target_file" ]; then
        rm "$target_file"
        print_success "Removed: $hook_name"
    else
        print_warning "$hook_name not installed"
    fi
}

# ============================================================================
# INSTALLATION
# ============================================================================

install_hooks() {
    print_header "Installing Git Hooks"
    echo ""

    # Check we're in project root
    check_in_project_root

    # Check hooks directory exists
    check_hooks_directory

    # Check hook files exist
    check_hook_files

    # Check if already installed
    if [ "$FORCE_INSTALL" = false ] && check_hooks_installed; then
        print_warning "Hooks are already installed"
        print_info "Use --force to reinstall"
        exit 0
    fi

    # Install pre-commit hook
    install_hook "$PRE_COMMIT_FILE" "$TARGET_PRE_COMMIT" "pre-commit hook"
    echo ""

    # Install pre-push hook
    install_hook "$PRE_PUSH_FILE" "$TARGET_PRE_PUSH" "pre-push hook"
    echo ""

    # Make source hooks executable
    print_info "Making source hooks executable..."
    make_executable "$PRE_COMMIT_FILE"
    make_executable "$PRE_PUSH_FILE"
    echo ""

    # Verify installation
    print_info "Verifying installation..."

    if [ -f "$TARGET_PRE_COMMIT" ] && [ -f "$TARGET_PRE_PUSH" ]; then
        print_success "All hooks installed successfully!"
        echo ""
        print_info "Installed hooks:"
        echo "  - pre-commit: Runs before commits"
        echo "  - pre-push: Runs before pushes"
        echo ""
        print_info "What they check:"
        echo "  - Code formatting (cargo fmt)"
        echo "  - Linting (cargo clippy)"
        echo "  - Compilation (cargo check)"
        echo "  - Documentation (cargo doc)"
        echo "  - Tests (unit + integration)"
        echo "  - Generated files (Cargo.lock, target/)"
        echo "  - TODO/FIXME comments"
        echo ""
    else
        print_error "Installation verification failed"
        exit 1
    fi
}

# ============================================================================
# UNINSTALLATION
# ============================================================================

uninstall_hooks() {
    print_header "Uninstalling Git Hooks"
    echo ""

    # Uninstall pre-commit hook
    uninstall_hook "$TARGET_PRE_COMMIT" "pre-commit hook"
    echo ""

    # Uninstall pre-push hook
    uninstall_hook "$TARGET_PRE_PUSH" "pre-push hook"
    echo ""

    print_success "All hooks uninstalled"
    echo ""
    print_info "Backup files (if any) remain in $GIT_HOOKS_DIR"
    echo ""
}

# ============================================================================
# STATUS CHECK
# ============================================================================

check_status() {
    print_header "Git Hooks Status"
    echo ""

    if [ -f "$TARGET_PRE_COMMIT" ]; then
        print_success "pre-commit hook: Installed"
        print_info "  Location: $TARGET_PRE_COMMIT"
    else
        print_warning "pre-commit hook: Not installed"
    fi
    echo ""

    if [ -f "$TARGET_PRE_PUSH" ]; then
        print_success "pre-push hook: Installed"
        print_info "  Location: $TARGET_PRE_PUSH"
    else
        print_warning "pre-push hook: Not installed"
    fi
    echo ""

    # Check if source hooks exist
    if [ -f "$PRE_COMMIT_FILE" ]; then
        print_success "Source pre-commit: Found"
    else
        print_warning "Source pre-commit: Not found"
    fi

    if [ -f "$PRE_PUSH_FILE" ]; then
        print_success "Source pre-push: Found"
    else
        print_warning "Source pre-push: Not found"
    fi
    echo ""
}

# ============================================================================
# DRY RUN
# ============================================================================

dry_run() {
    print_header "Dry Run Mode"
    echo ""
    print_info "This shows what would be installed without making changes"
    echo ""

    print_info "Pre-commit hook:"
    if [ -f "$PRE_COMMIT_FILE" ]; then
        print_success "  Source found: $PRE_COMMIT_FILE"
        print_info "  Would install to: $TARGET_PRE_COMMIT"
    else
        print_error "  Source not found: $PRE_COMMIT_FILE"
    fi
    echo ""

    print_info "Pre-push hook:"
    if [ -f "$PRE_PUSH_FILE" ]; then
        print_success "  Source found: $PRE_PUSH_FILE"
        print_info "  Would install to: $TARGET_PRE_PUSH"
    else
        print_error "  Source not found: $PRE_PUSH_FILE"
    fi
    echo ""
}

# ============================================================================
# USAGE INFORMATION
# ============================================================================

show_usage() {
    print_header "CI Hooks Setup Script"
    echo ""
    echo "This script installs git hooks for comprehensive Rust checks"
    echo ""
    echo "Usage:"
    echo "  $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --install (default)  Install git hooks"
    echo "  --uninstall            Remove git hooks"
    echo "  --check               Check if hooks are installed"
    echo "  --force               Force reinstall hooks"
    echo "  --dry-run             Show what would be installed"
    echo "  -h, --help            Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                      # Install hooks"
    echo "  $0 --check            # Check status"
    echo "  $0 --uninstall         # Remove hooks"
    echo "  $0 --force            # Force reinstall"
    echo ""
    echo "What the hooks do:"
    echo "  Pre-commit: Runs formatting, linting, compilation, tests before commits"
    echo "  Pre-push:   Runs all checks plus additional validations before pushes"
    echo ""
    echo "Hook checks:"
    echo "  ✓ Code formatting (cargo fmt)"
    echo "  ✓ Linting (cargo clippy)"
    echo "  ✓ Compilation (cargo check)"
    echo "  ✓ Documentation (cargo doc)"
    echo "  ✓ Tests (cargo test)"
    echo "  ✓ Generated files (Cargo.lock, target/)"
    echo "  ✓ TODO/FIXME comments"
    echo ""
}

# ============================================================================
# MAIN
# ============================================================================

# Default action
ACTION="install"
FORCE_INSTALL=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --install)
            ACTION="install"
            shift
            ;;
        --uninstall)
            ACTION="uninstall"
            shift
            ;;
        --check)
            ACTION="check"
            shift
            ;;
        --force)
            FORCE_INSTALL=true
            shift
            ;;
        --dry-run)
            ACTION="dry-run"
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo ""
            show_usage
            exit 1
            ;;
    esac
done

# Change to project root
cd "$PROJECT_ROOT" || {
    print_error "Failed to change to project root: $PROJECT_ROOT"
    exit 1
}

# Execute action
case "$ACTION" in
    install)
        install_hooks
        ;;
    uninstall)
        uninstall_hooks
        ;;
    check)
        check_status
        ;;
    dry-run)
        dry_run
        ;;
    *)
        print_error "Unknown action: $ACTION"
        exit 1
        ;;
esac

exit 0
