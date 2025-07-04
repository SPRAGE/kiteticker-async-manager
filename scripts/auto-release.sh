#!/usr/bin/env bash

# All-in-One Release Script for kiteticker-async-manager
# Tests, fixes issues, bumps version, and pushes to GitHub
# Usage: ./scripts/auto-release.sh [patch|minor|major|VERSION]

set -e

# Global variables to track script state
BACKUP_CREATED=false
CHANGES_MADE=false

# Cleanup function for unexpected exits
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ] && [ "$CHANGES_MADE" = true ]; then
        echo ""
        print_warning "Script exited unexpectedly (code: $exit_code)"
        rollback_changes
    fi
}

# Set up signal traps for cleanup
trap cleanup EXIT INT TERM

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_step() {
    echo -e "${BOLD}${BLUE}[STEP]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_success() {
    echo -e "${BOLD}${GREEN}[SUCCESS]${NC} $1"
}

print_version() {
    echo -e "${PURPLE}[VERSION]${NC} $1"
}

print_fix() {
    echo -e "${CYAN}[FIX]${NC} $1"
}

# Usage information
show_usage() {
    echo "Usage: $0 [patch|minor|major|VERSION]"
    echo ""
    echo "Examples:"
    echo "  $0 patch      # Increment patch version (bug fixes)"
    echo "  $0 minor      # Increment minor version (new features)"
    echo "  $0 major      # Increment major version (breaking changes)"
    echo "  $0 0.1.8      # Set specific version"
    echo ""
    echo "This script will:"
    echo "  1. 🧪 Run comprehensive tests"
    echo "  2. 🔧 Auto-fix clippy issues where possible"
    echo "  3. 📦 Bump version in Cargo.toml"
    echo "  4. 💾 Commit changes"
    echo "  5. 🏷️  Create and push version tag"
    echo "  6. 🚀 Trigger GitHub Actions publish"
    echo ""
    echo "Current version: $(get_current_version)"
}

# Check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_error "Not in a git repository"
        exit 1
    fi
    
    # Check if we're in the right directory (has Cargo.toml)
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found in current directory"
        exit 1
    fi
    
    # Check if working directory is clean
    if [ -n "$(git status --porcelain)" ]; then
        print_warning "Working directory has uncommitted changes"
        print_info "The script will create auto-fixes and then commit everything together"
        print_info "Uncommitted files:"
        git status --porcelain
        echo
        read -p "Continue with auto-release? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Aborting release"
            exit 1
        fi
        create_backup  # Create backup since we have existing changes
    fi
    
    # Check if we're on a reasonable branch
    CURRENT_BRANCH=$(git branch --show-current)
    if [[ "$CURRENT_BRANCH" != "main" && "$CURRENT_BRANCH" != "master" && ! "$CURRENT_BRANCH" =~ ^v[0-9]+$ ]]; then
        print_warning "You're on branch '$CURRENT_BRANCH' (not main/master/v*)"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Aborting release"
            exit 1
        fi
    fi
    
    print_success "Prerequisites check passed"
}

# Get current version from Cargo.toml
get_current_version() {
    grep "^version" Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Parse semantic version
parse_version() {
    local version="$1"
    echo "$version" | sed -E 's/([0-9]+)\.([0-9]+)\.([0-9]+).*/\1 \2 \3/'
}

# Validate version format
validate_version() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        print_error "Invalid version format: $version"
        print_info "Expected format: MAJOR.MINOR.PATCH (e.g., 1.2.3)"
        exit 1
    fi
}

# Increment version
increment_version() {
    local current_version="$1"
    local bump_type="$2"
    
    read -r major minor patch <<< "$(parse_version "$current_version")"
    
    case "$bump_type" in
        "patch")
            patch=$((patch + 1))
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        *)
            print_error "Invalid bump type: $bump_type"
            exit 1
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Update version in Cargo.toml
update_cargo_version() {
    local new_version="$1"
    print_info "Updating Cargo.toml version to $new_version"
    
    # Use sed to update the version
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    fi
}

# Create a backup of current state for rollback
create_backup() {
    print_info "Creating backup of current state..."
    git stash push -u -m "auto-release-backup-$(date +%s)" > /dev/null 2>&1 || true
    BACKUP_CREATED=true
}

# Rollback changes if something fails
rollback_changes() {
    if [ "$BACKUP_CREATED" = true ]; then
        print_warning "Rolling back changes..."
        git reset --hard HEAD > /dev/null 2>&1 || true
        git stash pop > /dev/null 2>&1 || true
        print_info "Changes rolled back to original state"
    fi
}

# Run and fix formatting
run_formatting() {
    print_step "Checking and fixing code formatting..."
    
    if ! cargo fmt --check > /dev/null 2>&1; then
        print_fix "Formatting issues found, auto-fixing..."
        if ! cargo fmt; then
            print_error "❌ Formatting failed!"
            rollback_changes
            exit 1
        fi
        print_success "Code formatted successfully"
        CHANGES_MADE=true
    else
        print_success "Code formatting is already correct"
    fi
}

# Run and fix clippy issues  
run_clippy_fixes() {
    print_step "Running clippy checks and auto-fixes..."
    
    # First, try to auto-fix what we can
    print_info "Attempting to auto-fix clippy suggestions..."
    
    # Run clippy --fix with proper error handling
    if cargo clippy --fix --allow-dirty --allow-staged --all-features 2>/dev/null; then
        print_success "Auto-fixed clippy suggestions"
        CHANGES_MADE=true
    else
        # Check if the failure was due to compilation errors vs no fixes available
        if ! cargo check --all-features > /dev/null 2>&1; then
            print_error "❌ Code has compilation errors that prevent clippy auto-fix!"
            print_info "Please fix compilation errors first and try again"
            rollback_changes
            exit 1
        else
            print_info "No auto-fixable clippy suggestions found"
        fi
    fi
    
    # Now check if there are any remaining issues
    print_info "Checking for remaining clippy issues..."
    if ! cargo clippy --all-features -- -D warnings 2>/dev/null; then
        print_error "❌ Clippy checks failed with remaining issues!"
        print_info ""
        print_info "Some clippy issues require manual fixing. Please address them and run again."
        print_info "Common fixes:"
        print_info "  - Remove unused imports/variables"
        print_info "  - Fix deprecated function calls"  
        print_info "  - Address performance suggestions"
        print_info "  - Fix compiler warnings"
        print_info ""
        print_info "Run 'cargo clippy --all-features' to see detailed issues"
        rollback_changes
        exit 1
    fi
    
    print_success "All clippy checks passed"
}

# Run comprehensive tests
run_tests() {
    print_step "Running comprehensive test suite..."
    
    # Build first
    print_info "Building project..."
    if ! cargo build --all-features; then
        print_error "❌ Build failed!"
        rollback_changes
        exit 1
    fi
    print_success "Build successful"
    
    # Unit and integration tests
    print_info "Running tests..."
    if ! cargo test --all-features; then
        print_error "❌ Tests failed!"
        print_info "Please fix failing tests before proceeding"
        rollback_changes
        exit 1
    fi
    print_success "All tests passed"
    
    # Doc tests
    print_info "Running documentation tests..."
    if ! cargo test --doc --all-features; then
        print_error "❌ Documentation tests failed!"
        print_info "Please fix documentation issues before proceeding"
        rollback_changes
        exit 1
    fi
    print_success "Documentation tests passed"
    
    # Security audit (if available)
    if command -v cargo-audit > /dev/null 2>&1; then
        print_info "Running security audit..."
        if ! cargo audit; then
            print_error "❌ Security audit failed!"
            print_info "Please address security vulnerabilities before proceeding"
            rollback_changes
            exit 1
        fi
        print_success "Security audit passed"
    else
        print_warning "cargo-audit not installed, skipping security audit"
    fi
    
    # Publish dry run
    print_info "Testing publish readiness..."
    if ! cargo publish --dry-run --all-features > /dev/null 2>&1; then
        print_error "❌ Publish dry run failed!"
        print_info "Package is not ready for publishing"
        rollback_changes
        exit 1
    fi
    print_success "Package is ready for publishing"
    
    print_success "All tests and checks passed"
}

# Create and push release
create_release() {
    local new_version="$1"
    
    print_step "Creating release v$new_version..."
    
    # Create tag (changes already committed in main function)
    print_info "Creating version tag..."
    git tag "v$new_version"
    
    # Push changes and tag
    print_info "Pushing to GitHub..."
    git push origin "$(git branch --show-current)"
    git push origin "v$new_version"
    
    print_success "Release v$new_version pushed to GitHub!"
}

# Main execution
main() {
    local bump_type="$1"
    
    echo "🚀 KiteTicker Async Manager - Auto Release Script"
    echo "================================================="
    echo ""
    
    if [ -z "$bump_type" ]; then
        show_usage
        exit 1
    fi
    
    # Show current status
    local current_version
    current_version=$(get_current_version)
    print_version "Current version: $current_version"
    
    # Determine new version
    local new_version
    case "$bump_type" in
        "patch"|"minor"|"major")
            new_version=$(increment_version "$current_version" "$bump_type")
            ;;
        *)
            # Assume it's a specific version
            new_version="$bump_type"
            validate_version "$new_version"
            ;;
    esac
    
    print_version "Target version: $new_version"
    echo ""
    
    # Confirmation
    read -p "🤔 Proceed with release v$new_version? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Release cancelled"
        exit 0
    fi
    
    echo ""
    
    # Execute all steps
    check_prerequisites
    run_formatting
    run_clippy_fixes
    run_tests
    
    # Update version
    print_step "Updating version..."
    update_cargo_version "$new_version"
    CHANGES_MADE=true
    print_success "Version updated to $new_version"
    
    # Commit any changes made during the process
    if [ "$CHANGES_MADE" = true ] || [ -n "$(git status --porcelain)" ]; then
        print_step "Committing changes..."
        git add -A
        git commit -m "Auto-release v$new_version: format, clippy fixes, and version bump"
        print_success "Changes committed"
    fi
    
    # Create and push release
    create_release "$new_version"
    
    # Final success message
    echo ""
    echo "🎉 Release v$new_version completed successfully!"
    echo ""
    echo "What happens next:"
    echo "  📦 GitHub Actions will run smoke tests"
    echo "  🚀 Package will be published to crates.io"
    echo "  📝 GitHub release will be created"
    echo ""
    echo "Monitor progress at:"
    echo "  https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\([^/]*\/[^/]*\).*/\1/' | sed 's/\.git$//')/actions"
    echo ""
    print_success "Auto-release script completed! 🚀"
}

# Run main function with all arguments
main "$@"
