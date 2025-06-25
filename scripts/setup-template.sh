#!/usr/bin/env bash

# GitHub Template Setup Script
# Initializes the template for a new project with major-version-only branching

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

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

print_setup() {
    echo -e "${BLUE}[SETUP]${NC} $1"
}

print_template() {
    echo -e "${PURPLE}[TEMPLATE]${NC} $1"
}

echo ""
echo "🚀 GitHub Template Setup for Enhanced Version Management"
echo "======================================================"
echo ""

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_error "Not in a git repository. Please run this script in your project's git repository."
    exit 1
fi

# Get project information
project_name=$(basename $(pwd))
print_info "Project detected: $project_name"

# Check for Cargo.toml
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Cargo.toml not found. This template is designed for Rust projects."
    print_info "Please create a Cargo.toml file first or use 'cargo init' to initialize a Rust project."
    exit 1
fi

print_setup "Setting up GitHub template files..."

# Make template scripts executable
print_info "Making scripts executable..."
chmod +x scripts/*-template.sh 2>/dev/null || print_warning "No template scripts found to make executable"

# Copy template scripts to replace existing ones (with backup)
if [[ -f "scripts/bump-version-template.sh" ]]; then
    print_info "Installing enhanced bump-version script..."
    [[ -f "scripts/bump-version.sh" ]] && cp scripts/bump-version.sh scripts/bump-version.sh.backup
    cp scripts/bump-version-template.sh scripts/bump-version.sh
    chmod +x scripts/bump-version.sh
fi

if [[ -f "scripts/release-template.sh" ]]; then
    print_info "Installing enhanced release script..."
    [[ -f "scripts/release.sh" ]] && cp scripts/release.sh scripts/release.sh.backup
    cp scripts/release-template.sh scripts/release.sh
    chmod +x scripts/release.sh
fi

if [[ -f "scripts/create-tag-template.sh" ]]; then
    print_info "Installing enhanced create-tag script..."
    [[ -f "scripts/create-tag.sh" ]] && cp scripts/create-tag.sh scripts/create-tag.sh.backup
    cp scripts/create-tag-template.sh scripts/create-tag.sh
    chmod +x scripts/create-tag.sh
fi

# Copy template workflows
if [[ -f ".github/workflows/ci-template.yml" ]]; then
    print_info "Installing enhanced CI workflow..."
    [[ -f ".github/workflows/ci.yml" ]] && cp .github/workflows/ci.yml .github/workflows/ci.yml.backup
    cp .github/workflows/ci-template.yml .github/workflows/ci.yml
fi

if [[ -f ".github/workflows/release-template.yml" ]]; then
    print_info "Installing enhanced release workflow..."
    [[ -f ".github/workflows/release.yml" ]] && cp .github/workflows/release.yml .github/workflows/release.yml.backup
    cp .github/workflows/release-template.yml .github/workflows/release.yml
fi

print_success "✅ Template files installed successfully!"

# Get current version from Cargo.toml
current_version=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/' || echo "0.1.0")
print_info "Current version in Cargo.toml: $current_version"

# Parse version to determine if major version branch exists
IFS='.' read -r major minor patch <<< "$current_version"
major_branch="v$major"

echo ""
print_setup "🎯 Repository Configuration Recommendations"
echo ""

print_info "1. Branch Protection Rules:"
echo "   Set up branch protection for 'main' in GitHub Settings:"
echo "   • Require pull request reviews before merging"
echo "   • Require status checks to pass before merging"
echo "   • Require branches to be up to date before merging"
echo ""

print_info "2. Repository Secrets:"
echo "   Add the following secret in GitHub Settings → Secrets and variables → Actions:"
echo "   • CARGO_REGISTRY_TOKEN: Your crates.io API token"
echo ""

print_info "3. Environment Protection (Optional but Recommended):"
echo "   Create a 'crates-io' environment in GitHub Settings for additional security"
echo ""

# Check if major version branch should be created
if [[ $major -gt 0 ]]; then
    echo ""
    print_template "🌳 Major Version Branch Setup"
    echo ""
    
    if git show-ref --verify --quiet refs/heads/$major_branch; then
        print_info "Major version branch '$major_branch' already exists ✓"
    else
        print_warning "Major version branch '$major_branch' does not exist"
        echo ""
        read -p "Create major version branch '$major_branch' now? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_info "Creating major version branch: $major_branch"
            git checkout -b $major_branch
            git push -u origin $major_branch
            git checkout main
            print_success "✅ Created and pushed major version branch: $major_branch"
        else
            print_info "You can create it later with: git checkout -b $major_branch && git push -u origin $major_branch"
        fi
    fi
fi

echo ""
print_template "📋 Next Steps"
echo ""

print_info "1. Review and customize your project:"
echo "   • Update Cargo.toml with your project details"
echo "   • Update README.md with your project information"
echo "   • Review and customize GitHub issue/PR templates"
echo ""

print_info "2. Test the version management system:"
echo "   • ./scripts/bump-version.sh patch    (for bug fixes)"
echo "   • ./scripts/bump-version.sh minor    (for new features)"
echo "   • ./scripts/bump-version.sh major    (for breaking changes)"
echo ""

print_info "3. Set up your first release:"
echo "   • ./scripts/release.sh               (prepare release)"
echo "   • Create PR and merge to main"
echo "   • ./scripts/create-tag.sh            (create release tag)"
echo ""

print_info "4. Configure GitHub repository settings:"
echo "   • Add CARGO_REGISTRY_TOKEN secret"
echo "   • Set up branch protection rules"
echo "   • Configure environment protection (optional)"
echo ""

print_template "📚 Documentation"
echo ""
print_info "Complete documentation available in:"
echo "   • docs/TEMPLATE_README.md           (Complete template guide)"
echo "   • docs/VERSION_MANAGEMENT_TEMPLATE.md (Detailed workflow guide)"
echo "   • .github/ISSUE_TEMPLATE/           (Issue templates)"
echo "   • .github/pull_request_template.md  (PR template)"
echo ""

print_template "🎯 Major Version Branching Strategy"
echo ""
print_info "This template uses major-version-only branching:"
echo "   • Major versions (X.0.0): Create dedicated branches (v1, v2, v3)"
echo "   • Minor versions (x.Y.0): Work on existing major branch"
echo "   • Patch versions (x.y.Z): Work on existing major branch"
echo ""
print_info "Benefits:"
echo "   ✅ Cleaner repository with fewer long-lived branches"
echo "   ✅ Clear maintenance path for major versions"
echo "   ✅ Simplified workflow for minor/patch releases"
echo "   ✅ Parallel development on multiple major versions"
echo ""

echo ""
print_success "🎉 GitHub Template Setup Complete!"
echo ""
print_template "Your repository is now configured with:"
echo "   ✅ Enhanced version management scripts"
echo "   ✅ Automated CI/CD workflows"
echo "   ✅ Major-version-only branching strategy"
echo "   ✅ Comprehensive GitHub templates"
echo "   ✅ Security-first release process"
echo ""

print_info "Ready to start building with automated version management! 🚀"

# Optional: Show current git status
echo ""
print_info "📊 Current Repository Status:"
git status --porcelain | head -10 || echo "   Working directory clean"
echo ""

print_template "Happy coding and automated releasing! 🎊"
