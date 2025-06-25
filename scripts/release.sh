#!/usr/bin/env bash

# Enhanced Release Script with Major-Version-Only Branching
# Creates development branches and pushes to GitHub for PR creation
# Usage: ./release.sh [version] or just ./release.sh to use Cargo.toml version

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

print_release() {
    echo -e "${BLUE}[RELEASE]${NC} $1"
}

print_branch() {
    echo -e "${CYAN}[BRANCH]${NC} $1"
}

print_pr() {
    echo -e "${PURPLE}[PR]${NC} $1"
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

# Get current version from Cargo.toml
current_version=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Determine target version
if [[ -n "$1" ]]; then
    target_version="$1"
    print_info "Using specified version: $target_version"
else
    target_version="$current_version"
    print_info "Using current Cargo.toml version: $target_version"
fi

# Validate version format
if [[ ! $target_version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_error "Invalid version format. Use semantic versioning (e.g., 1.2.3)"
    exit 1
fi

# Parse version components
IFS='.' read -r target_major target_minor target_patch <<< "$target_version"
IFS='.' read -r current_major current_minor current_patch <<< "$current_version"

# Determine version bump type and branching strategy
is_major_bump=false
version_bump_type=""

if [[ $target_major -gt $current_major ]]; then
    is_major_bump=true
    version_bump_type="major"
    branch_name="v${target_major}"
elif [[ $target_minor -gt $current_minor ]] || [[ $target_minor -lt $current_minor ]]; then
    version_bump_type="minor"
    # Use existing major version branch or main
    if git show-ref --verify --quiet refs/heads/v${target_major}; then
        branch_name="v${target_major}"
    else
        branch_name="main"
    fi
else
    version_bump_type="patch"
    # Use existing major version branch or main
    if git show-ref --verify --quiet refs/heads/v${target_major}; then
        branch_name="v${target_major}"
    else
        branch_name="main"
    fi
fi

print_release "Version: $current_version → $target_version ($version_bump_type)"
print_branch "Target branch: $branch_name"

# Get current branch
current_branch=$(git branch --show-current 2>/dev/null || git rev-parse --short HEAD)

# Ensure we're working from the correct base branch
base_branch="main"
if [[ $branch_name != "main" ]]; then
    if git show-ref --verify --quiet refs/heads/$branch_name; then
        base_branch="$branch_name"
        print_info "Using existing major version branch as base: $base_branch"
    else
        print_info "Major version branch doesn't exist, will create from main"
        base_branch="main"
    fi
fi

# Switch to base branch and pull latest
if [[ $current_branch != $base_branch ]]; then
    print_info "Switching to base branch: $base_branch"
    git checkout $base_branch
    git pull origin $base_branch 2>/dev/null || print_warning "Could not pull latest changes"
fi

# Create or switch to target branch
if [[ $branch_name != $base_branch ]]; then
    if git show-ref --verify --quiet refs/heads/$branch_name; then
        print_branch "Switching to existing branch: $branch_name"
        git checkout $branch_name
        # Merge latest changes from base branch
        print_info "Merging latest changes from $base_branch"
        git merge $base_branch --no-edit
    else
        print_branch "Creating new major version branch: $branch_name"
        git checkout -b $branch_name
    fi
fi

# Update version if needed
if [[ $target_version != $current_version ]]; then
    print_info "Updating version in Cargo.toml: $current_version → $target_version"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$target_version\"/" Cargo.toml
    else
        sed -i "s/^version = \".*\"/version = \"$target_version\"/" Cargo.toml
    fi
    
    # Update Cargo.lock
    cargo check > /dev/null 2>&1
    
    # Update README if version references exist
    if grep -q "$current_version" README.md 2>/dev/null; then
        print_info "Updating version references in README.md"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/$current_version/$target_version/g" README.md
        else
            sed -i "s/$current_version/$target_version/g" README.md
        fi
    fi
fi

# Comprehensive validation
print_info "🧪 Running comprehensive validation..."

# Run tests
print_info "Running test suite..."
if ! cargo test --features native; then
    print_error "Tests failed! Please fix issues before proceeding."
    exit 1
fi

# Build check
print_info "Verifying build..."
if ! cargo build --release --features native; then
    print_error "Build failed! Please fix issues before proceeding."
    exit 1
fi

# Publish dry run
print_info "Testing package publishing..."
if ! cargo publish --dry-run --features native; then
    print_error "Publish dry-run failed! Please fix issues before proceeding."
    exit 1
fi

print_info "✅ All validation checks passed!"

# Commit changes if any
if ! git diff-index --quiet HEAD --; then
    commit_message="release: prepare v$target_version

Version: $current_version → $target_version
Type: $version_bump_type
Branch: $branch_name"

    if [[ $is_major_bump == true ]]; then
        commit_message="$commit_message

🔥 MAJOR VERSION RELEASE 🔥
- Created major version branch: $branch_name
- This may include breaking changes
- Please review CHANGELOG.md thoroughly"
    fi

    git add -A
    git commit -m "$commit_message"
    print_info "Committed version changes"
fi

# Push branch to origin
print_info "Pushing branch to origin..."
git push -u origin $branch_name

print_release "🚀 Release preparation completed!"
echo ""
print_info "📋 Summary:"
echo "  Version: $current_version → $target_version"
echo "  Type: $version_bump_type"
echo "  Branch: $branch_name"
echo "  Status: Pushed to origin"
echo ""

if [[ $is_major_bump == true ]]; then
    print_release "🎯 Major Version Branch Strategy:"
    echo "  • Created major version branch: $branch_name"
    echo "  • All v${target_major}.x.x releases will use this branch"
    echo "  • Future minor/patch changes work on this branch"
    echo ""
fi

# Generate PR description
pr_title="release: v$target_version"
pr_description="## Release v$target_version

### Version Change
- **From:** $current_version
- **To:** $target_version  
- **Type:** $version_bump_type

### Branch Strategy
- **Branch:** $branch_name"

if [[ $is_major_bump == true ]]; then
    pr_description="$pr_description
- **Major Version:** Created dedicated branch for v${target_major}.x.x releases"
fi

pr_description="$pr_description

### Validation ✅
- [x] All tests pass
- [x] Build successful
- [x] Publish dry-run successful
- [x] Documentation updated

### Checklist for Reviewer
- [ ] CHANGELOG.md updated with release notes
- [ ] Documentation reflects changes"

if [[ $is_major_bump == true ]]; then
    pr_description="$pr_description
- [ ] Breaking changes documented
- [ ] Migration guide updated (if needed)"
fi

pr_description="$pr_description
- [ ] Version number is correct
- [ ] Release notes are comprehensive

### Post-Merge Actions
After merging this PR:
1. Run \`./scripts/create-tag.sh\` to create release tag
2. GitHub Actions will automatically publish to crates.io
3. Release will be created with automated release notes"

print_pr "📄 Create Pull Request:"
echo ""
echo "Title: $pr_title"
echo ""
echo "Description:"
echo "$pr_description"
echo ""

# Check if GitHub CLI is available
if command -v gh &> /dev/null; then
    echo ""
    read -p "Create PR automatically with GitHub CLI? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "$pr_description" | gh pr create --title "$pr_title" --body-file - --base main
        print_info "✅ Pull request created successfully!"
    else
        print_info "Manual PR creation required"
    fi
else
    print_info "GitHub CLI not available - create PR manually at:"
    echo "https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\([^/]*\/[^/]*\).*/\1/' | sed 's/\.git$//')/compare/main...$branch_name"
fi

print_info "🎉 Release preparation completed successfully!"
