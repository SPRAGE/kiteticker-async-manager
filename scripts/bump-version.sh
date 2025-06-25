#!/usr/bin/env bash

# Modified Version Bump Script with Major-Version-Only Branching
# Usage: ./bump-version.sh [patch|minor|major|VERSION]
# Only major versions create branches, minor/patch versions work on existing branches

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_major() {
    echo -e "${BLUE}[MAJOR]${NC} $1"
}

print_branch() {
    echo -e "${CYAN}[BRANCH]${NC} $1"
}

print_version() {
    echo -e "${PURPLE}[VERSION]${NC} $1"
}

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_error "Not in a git repository"
    exit 1
fi

# Check if working directory is clean
if ! git diff-index --quiet HEAD --; then
    print_error "Working directory is not clean. Please commit or stash changes first."
    exit 1
fi

# Run comprehensive tests before making any changes
print_info "🧪 Running comprehensive test suite before version bump..."
echo ""

# Test 1: Check code formatting (if rustfmt is available)
if command -v rustfmt > /dev/null 2>&1; then
    print_info "Checking code formatting..."
    if ! cargo fmt --all -- --check > /dev/null 2>&1; then
        print_warning "Code formatting issues detected. Run 'cargo fmt' to fix."
        # Don't fail on formatting issues, just warn
    else
        print_info "✅ Code formatting looks good"
    fi
else
    print_info "⏭️  Skipping format check (rustfmt not available)"
fi

# Test 2: Run clippy for code quality
print_info "Running clippy for code quality checks..."
if ! cargo clippy --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
    print_error "❌ Clippy checks failed! Please fix code quality issues first."
    echo ""
    print_info "Run 'cargo clippy --all-targets --all-features' to see issues"
    exit 1
else
    print_info "✅ Clippy checks passed"
fi

# Test 3: Run unit tests
print_info "Running unit tests..."
if ! cargo test --lib --all-features > /dev/null 2>&1; then
    print_error "❌ Unit tests failed!"
    echo ""
    print_info "Run 'cargo test --lib --all-features' to see failing tests"
    exit 1
else
    print_info "✅ Unit tests passed"
fi

# Test 4: Run integration tests (if any exist)
print_info "Checking for integration tests..."
if [ -d "tests" ] && [ "$(find tests -name '*.rs' | wc -l)" -gt 0 ]; then
    print_info "Running integration tests..."
    if ! cargo test --test '*' --all-features > /dev/null 2>&1; then
        print_error "❌ Integration tests failed!"
        echo ""
        print_info "Run 'cargo test --test '*' --all-features' to see failing tests"
        exit 1
    else
        print_info "✅ Integration tests passed"
    fi
else
    print_info "⏭️  No integration tests found (tests/ directory empty or missing)"
fi

# Test 5: Run doc tests
print_info "Running documentation tests..."
if ! cargo test --doc --all-features > /dev/null 2>&1; then
    print_error "❌ Documentation tests failed!"
    echo ""
    print_info "Run 'cargo test --doc --all-features' to see failing tests"
    exit 1
else
    print_info "✅ Documentation tests passed"
fi

# Test 6: Test with minimal features
print_info "Testing with minimal features..."
if ! cargo test --no-default-features > /dev/null 2>&1; then
    print_error "❌ Minimal feature tests failed!"
    echo ""
    print_info "Run 'cargo test --no-default-features' to see failing tests"
    exit 1
else
    print_info "✅ Minimal feature tests passed"
fi

# Test 7: Build check
print_info "Running build check..."
if ! cargo build --release > /dev/null 2>&1; then
    print_error "❌ Release build failed!"
    echo ""
    print_info "Run 'cargo build --release' to see build errors"
    exit 1
else
    print_info "✅ Release build successful"
fi

# Test 8: Check examples compile
print_info "Checking example compilation..."
if ! cargo check --examples > /dev/null 2>&1; then
    print_error "❌ Example compilation failed!"
    echo ""
    print_info "Run 'cargo check --examples' to see compilation errors"
    exit 1
else
    print_info "✅ Examples compile successfully"
fi

print_info "🎉 All tests passed! Proceeding with version bump..."
echo ""

# Get current branch
current_branch=$(git branch --show-current 2>/dev/null || git rev-parse --short HEAD)

# Get current version from Cargo.toml
current_version=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
print_info "Current version: $current_version"
print_info "Current branch: $current_branch"

# Parse version components
IFS='.' read -r major minor patch <<< "$current_version"

# Variables to track version bump type
is_major_bump=false
version_bump_type=""
branch_required=false

# Determine new version
case "$1" in
    "patch")
        new_patch=$((patch + 1))
        new_version="$major.$minor.$new_patch"
        version_bump_type="patch"
        ;;
    "minor")
        new_minor=$((minor + 1))
        new_version="$major.$new_minor.0"
        version_bump_type="minor"
        ;;
    "major")
        new_major=$((major + 1))
        new_version="$new_major.0.0"
        version_bump_type="major"
        is_major_bump=true
        branch_required=true
        ;;
    "")
        print_error "Please specify version bump type: patch, minor, major, or specific version"
        print_info "Usage: $0 [patch|minor|major|VERSION]"
        exit 1
        ;;
    *)
        # Check if it's a valid semantic version
        if [[ $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            new_version="$1"
            # Determine what type of bump this is
            new_major=$(echo "$1" | cut -d'.' -f1)
            new_minor=$(echo "$1" | cut -d'.' -f2)
            new_patch=$(echo "$1" | cut -d'.' -f3)
            
            if [[ $new_major -gt $major ]]; then
                is_major_bump=true
                version_bump_type="major"
                branch_required=true
            elif [[ $new_minor -gt $minor ]] || [[ $new_minor -lt $minor ]]; then
                version_bump_type="minor"
            else
                version_bump_type="patch"
            fi
        else
            print_error "Invalid version format. Use semantic versioning (e.g., 1.2.3)"
            exit 1
        fi
        ;;
esac

print_version "Version bump: $current_version → $new_version ($version_bump_type)"

# Determine target branch based on version change
if [[ $is_major_bump == true ]]; then
    target_branch="v${new_major}"
    print_major "Major version change detected - will create/use branch: $target_branch"
else
    # For minor/patch, stay on current major version branch or main
    current_major_branch="v${major}"
    if git show-ref --verify --quiet refs/heads/$current_major_branch; then
        target_branch="$current_major_branch"
        print_info "Using existing major version branch: $target_branch"
    else
        target_branch="main"
        print_info "No major version branch exists, using: $target_branch"
    fi
fi

# Create or switch to target branch
if [[ $target_branch != $current_branch ]]; then
    if git show-ref --verify --quiet refs/heads/$target_branch; then
        print_branch "Switching to existing branch: $target_branch"
        git checkout $target_branch
        git pull origin $target_branch 2>/dev/null || print_warning "Could not pull from origin (branch may not exist remotely)"
    else
        print_branch "Creating new branch: $target_branch"
        git checkout -b $target_branch
    fi
else
    print_info "Already on target branch: $target_branch"
fi

# Update version in Cargo.toml
print_info "Updating Cargo.toml version to $new_version"
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
else
    # Linux and others
    sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
fi

# Update Cargo.lock
print_info "Updating Cargo.lock"
cargo check > /dev/null 2>&1

# Update version references in README.md if they exist
if grep -q "$current_version" README.md 2>/dev/null; then
    print_info "Updating version references in README.md"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/$current_version/$new_version/g" README.md
    else
        sed -i "s/$current_version/$new_version/g" README.md
    fi
fi

# Final validation after version update
print_info "🔍 Running final validation with new version..."
if ! cargo check > /dev/null 2>&1; then
    print_error "❌ Final validation failed! Version update broke something."
    print_info "Reverting changes..."
    git checkout -- Cargo.toml Cargo.lock README.md 2>/dev/null || true
    exit 1
else
    print_info "✅ Final validation passed"
fi

# Commit changes
commit_message="chore: bump version to $new_version

Version bump: $current_version → $new_version
Type: $version_bump_type
Branch: $target_branch

✅ All tests passed:
- Code quality (clippy)
- Unit tests
- Integration tests
- Documentation tests
- Minimal features test
- Release build
- Example compilation"

if [[ $is_major_bump == true ]]; then
    commit_message="$commit_message

⚠️  MAJOR VERSION CHANGE ⚠️
This may include breaking changes.
Please review CHANGELOG.md and update migration documentation."
fi

git add Cargo.toml Cargo.lock
if grep -q "$new_version" README.md 2>/dev/null; then
    git add README.md
fi

git commit -m "$commit_message"

print_info "✅ Version bump completed successfully!"
echo ""
print_version "Summary:"
echo "  Current version: $current_version → $new_version"
echo "  Bump type: $version_bump_type"
echo "  Branch: $target_branch"
echo ""

if [[ $is_major_bump == true ]]; then
    print_major "🎯 Major Version Branch Strategy:"
    echo "  • Created/switched to major version branch: $target_branch"
    echo "  • All future minor/patch releases for v${new_major}.x.x will use this branch"
    echo "  • Push this branch to enable team collaboration"
    echo ""
fi

print_info "Next steps:"
echo "  1. Review changes and update CHANGELOG.md"
if [[ $is_major_bump == true ]]; then
    echo "  2. Document any breaking changes"
    echo "  3. Update migration guide (if needed)"
fi
echo "  4. Push branch: git push -u origin $target_branch"
echo "  5. Create PR to merge into main when ready"
echo "  6. After merge, run: ./scripts/create-tag.sh"
