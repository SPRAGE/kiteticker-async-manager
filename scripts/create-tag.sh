#!/usr/bin/env bash

# Template Create Tag Script with Enhanced Major Version Branch Support
# Creates release tags ONLY from main branch after PR merges
# Works with major-version-only branching strategy

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

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_tag() {
    echo -e "${BLUE}[TAG]${NC} $1"
}

print_release() {
    echo -e "${PURPLE}[RELEASE]${NC} $1"
}

# Defaults and CLI flags
do_dry_publish=true
do_push=false
no_verify=false
remote="origin"

print_usage() {
    cat <<USAGE
Usage: $(basename "$0") [options]

Options:
  --dry-publish            Run cargo publish --dry-run validation (default)
  --skip-dry-publish       Skip cargo publish --dry-run validation
  -p, --push               After creating the tag locally, push it to the remote
  --no-verify              Use --no-verify when pushing (skips client-side hooks)
  --remote <name>          Remote name to push to (default: origin)
  -h, --help               Show this help and exit

Examples:
  $(basename "$0") --skip-dry-publish           # Local tag only, no dry-run publish
  $(basename "$0") --push --no-verify           # Local tag + push tag to origin
  $(basename "$0") --remote upstream --push     # Push to a different remote
USAGE
}

# Parse CLI args
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-publish)
            do_dry_publish=true
            shift
            ;;
        --skip-dry-publish|--no-dry-publish)
            do_dry_publish=false
            shift
            ;;
        -p|--push)
            do_push=true
            shift
            ;;
        --no-verify)
            no_verify=true
            shift
            ;;
        --remote)
            if [[ -n "${2:-}" ]]; then
                remote="$2"
                shift 2
            else
                print_error "--remote requires a value"
                exit 1
            fi
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

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

# Get current branch
current_branch=$(git branch --show-current 2>/dev/null || git rev-parse --short HEAD)

# Enforce main/master branch for tag creation
if [[ "$current_branch" != "main" && "$current_branch" != "master" ]]; then
    print_error "Tag creation must be done from main or master branch"
    print_info "Current branch: $current_branch"
    print_info "Please switch to main/master branch and pull latest changes:"
    print_info "  git checkout main && git pull origin main"
    exit 1
fi

print_info "Current branch: $current_branch ✓"

# Note: This script performs only local operations.
# Ensure your local branch is up to date before running if needed:
#   git fetch origin && git rebase origin/$current_branch

# Get version from Cargo.toml
version=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
tag="v$version"

print_tag "Target version: $version"
print_tag "Target tag: $tag"

# Parse version to determine major version info
IFS='.' read -r major minor patch <<< "$version"
major_branch="v$major"

# Check if this version represents a major version
is_major_version=false
if [[ $minor == "0" && $patch == "0" ]]; then
    is_major_version=true
    print_info "🔥 Major version detected: $version"
else
    print_info "Minor/patch version: $version (part of $major_branch series)"
fi

# Check if tag already exists
if git tag | grep -q "^$tag$"; then
    print_error "Tag $tag already exists!"
    print_info "Existing tags:"
    git tag | grep "^v" | sort -V | tail -5
    exit 1
fi

print_success "Tag $tag does not exist - proceeding with creation"

# Comprehensive validation before tag creation
print_info "🧪 Running comprehensive validation..."

# Test suite
print_info "Running test suite..."
if ! cargo test --all-features --quiet; then
    print_error "Tests failed! Cannot create release tag."
    exit 1
fi
print_success "Tests passed ✓"

# Build verification
print_info "Verifying release build..."
if ! cargo build --release --all-features --quiet; then
    print_error "Release build failed! Cannot create release tag."
    exit 1
fi
print_success "Release build successful ✓"

# Documentation test (non-fatal due to occasional cargo doc resolver panics)
print_info "Testing documentation build..."
if ! cargo doc --no-deps --all-features --quiet; then
    print_warning "Documentation build failed. Proceeding without docs validation (known cargo doc issue)."
else
    print_success "Documentation build successful ✓"
fi

if [[ "$do_dry_publish" == true ]]; then
    # Publish dry run
    print_info "Testing package publishing (dry run)..."
    if ! cargo publish --dry-run --all-features --quiet; then
        print_error "Publish dry-run failed! Cannot create release tag."
        print_info "Please fix publishing issues before creating release tag."
        exit 1
    fi
    print_success "Publish validation successful ✓"
else
    print_warning "Skipping cargo publish --dry-run validation as requested"
fi

print_success "🎉 All validation checks passed!"

# Create annotated tag with comprehensive information
print_tag "Creating annotated tag: $tag"

# Generate tag message with version context
tag_message="Release $tag

Version: $version
Branch: $current_branch
Commit: $(git rev-parse HEAD)
Date: $(date -u +"%Y-%m-%d %H:%M:%S UTC")

Version Type: "

if [[ $is_major_version == true ]]; then
    tag_message="${tag_message}Major Release 🔥
Major Version Branch: $major_branch available for long-term maintenance

Breaking Changes: This release may include breaking changes.
Please review CHANGELOG.md and migration documentation.

"
elif [[ $patch == "0" ]]; then
    tag_message="${tag_message}Minor Release ✨
New features and enhancements, backward compatible.
Part of $major_branch series.

"
else
    tag_message="${tag_message}Patch Release 🐛
Bug fixes and security updates, backward compatible.
Part of $major_branch series.

"
fi

tag_message="${tag_message}Automation: This tag triggers CI and conditional publishing to crates.io (skips if version exists)
Documentation: https://docs.rs/$(basename $(pwd))/$version
Repository: $(git config --get remote.origin.url)

Created by: $(git config user.name) <$(git config user.email)>"

# Create the annotated tag
git tag -a "$tag" -m "$tag_message"

print_success "✅ Created annotated tag: $tag"

print_release "🎉 Release tag $tag created locally!"

# Optional push
if [[ "$do_push" == true ]]; then
    print_info "Pushing tag to $remote..."
    # Validate remote exists
    if ! git remote get-url "$remote" >/dev/null 2>&1; then
        print_error "Remote '$remote' does not exist"
        exit 1
    fi
    if [[ "$no_verify" == true ]]; then
        git push --no-verify "$remote" "$tag"
    else
        git push "$remote" "$tag"
    fi
    print_success "🚀 Tag pushed to $remote successfully!"
fi
echo ""
print_info "📋 Release Summary:"
echo "  Version: $version"
echo "  Tag: $tag"
echo "  Branch: $current_branch"
echo "  Major Series: $major_branch"
if [[ $is_major_version == true ]]; then
    echo "  Type: Major Release 🔥"
else
    echo "  Type: $([ $patch == "0" ] && echo "Minor" || echo "Patch") Release"
fi
echo ""

if [[ "$do_push" != true ]]; then
    print_info "🛠 Next steps (manual push):"
    echo "  • Push the tag to GitHub to trigger CI and optional publish"
    echo "    git push $remote $tag"
    echo "  • Or push all tags at once"
    echo "    git push --tags"
    echo ""
fi
print_info "🤖 After pushing to GitHub, Actions will:"
echo "  • Run tests and validation"
echo "  • Publish to crates.io ONLY if version $version does not already exist"
echo "  • Create a GitHub Release for $tag"
echo "  • docs.rs will build documentation automatically"
echo ""

print_info "🔗 Monitor Progress:"
echo "  • GitHub Actions: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\([^/]*\/[^/]*\).*/\1/' | sed 's/\.git$//')/actions"
echo "  • Crates.io: https://crates.io/crates/$(basename $(pwd))"
echo "  • Docs.rs: https://docs.rs/$(basename $(pwd))/$version"
echo ""

if [[ $is_major_version == true ]]; then
    print_release "🎯 Major Version Branch Available:"
    echo "  The $major_branch branch is available for:"
    echo "  • Long-term maintenance and support"
    echo "  • Future minor and patch releases in $major.x.x series"
    echo "  • Backporting security fixes"
    echo "  • Community contributions to $major.x.x series"
    echo ""
fi

print_success "🎊 Release process completed successfully!"
print_info "Thank you for using automated release management!"

# Optional: Show recent tags for context
echo ""
print_info "📊 Recent release tags:"
git tag | grep "^v" | sort -V | tail -5 | while read -r recent_tag; do
    tag_date=$(git log -1 --format=%ai "$recent_tag" 2>/dev/null || echo "unknown")
    echo "  $recent_tag ($(echo $tag_date | cut -d' ' -f1))"
done
