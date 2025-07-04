#!/usr/bin/env bash

# Comprehensive Codebase Cleanup Script
# Removes redundant, outdated, and unnecessary files

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() {
    echo -e "${GREEN}[CLEANUP]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_section() {
    echo -e "${BLUE}[SECTION]${NC} $1"
}

echo "🧹 KiteTicker Async Manager - Codebase Cleanup"
echo "=============================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Not in the project root directory!"
    exit 1
fi

print_info "Starting comprehensive cleanup..."

# 1. Remove legacy/duplicate GitHub Actions workflows
print_section "1. Cleaning up GitHub Actions workflows"

if [ -f ".github/workflows/ci.yml" ]; then
    print_info "Removing legacy CI workflow (replaced by minimal-ci.yml)"
    rm ".github/workflows/ci.yml"
fi

if [ -f ".github/workflows/release.yml" ]; then
    print_info "Removing legacy release workflow (replaced by publish.yml)"
    rm ".github/workflows/release.yml"
fi

# 2. Remove redundant/legacy scripts
print_section "2. Cleaning up legacy scripts"

legacy_scripts=(
    "scripts/bump-version.sh"           # Replaced by bump-version-efficient.sh
    "scripts/release.sh"                # Replaced by auto-release.sh
    "scripts/create-tag.sh"             # Functionality moved to auto-release.sh
    "scripts/simple-release.sh"         # Replaced by auto-release.sh
    "scripts/setup-template.sh"         # Not needed for end users
)

for script in "${legacy_scripts[@]}"; do
    if [ -f "$script" ]; then
        print_info "Removing legacy script: $script"
        rm "$script"
    fi
done

# 3. Remove template/setup documentation
print_section "3. Cleaning up template/setup documentation"

template_docs=(
    "REORGANIZATION_COMPLETE.md"        # Template artifact
    "REPOSITORY_SETUP_GUIDE.md"         # Template artifact
    "docs/guides/DYNAMIC_SUBSCRIPTION_COMPLETE.md"   # Duplicate info
    "docs/guides/IMPLEMENTATION_COMPLETE.md"         # Template artifact
)

for doc in "${template_docs[@]}"; do
    if [ -f "$doc" ]; then
        print_info "Removing template documentation: $doc"
        rm "$doc"
    fi
done

# 4. Clean up redundant documentation
print_section "4. Organizing documentation structure"

# Remove duplicate README files
if [ -f "docs/README.md" ]; then
    print_info "Removing duplicate docs/README.md (keeping root README.md)"
    rm "docs/README.md"
fi

if [ -f "docs/CONTRIBUTING.md" ]; then
    print_info "Removing duplicate docs/CONTRIBUTING.md (keeping root CONTRIBUTING.md)"
    rm "docs/CONTRIBUTING.md"
fi

# 5. Clean build artifacts and temporary files
print_section "5. Cleaning build artifacts"

if [ -d "target" ]; then
    print_info "Cleaning Cargo build artifacts"
    cargo clean > /dev/null 2>&1 || true
fi

# Remove any hidden temporary files
find . -name ".DS_Store" -delete 2>/dev/null || true
find . -name "*.tmp" -delete 2>/dev/null || true
find . -name "*.bak" -delete 2>/dev/null || true
find . -name "*~" -delete 2>/dev/null || true

# 6. Organize remaining scripts
print_section "6. Organizing remaining scripts"

# Rename bump-version-efficient.sh to bump-version.sh (cleaner name)
if [ -f "scripts/bump-version-efficient.sh" ]; then
    print_info "Renaming bump-version-efficient.sh to bump-version.sh"
    mv "scripts/bump-version-efficient.sh" "scripts/bump-version.sh"
fi

# 7. Clean up environment files
print_section "7. Cleaning up environment files"

# Remove .env if it exists and is empty or contains only comments
if [ -f ".env" ]; then
    if [ ! -s ".env" ] || ! grep -q "^[^#]" ".env" 2>/dev/null; then
        print_info "Removing empty .env file"
        rm ".env"
    else
        print_warning "Keeping .env file (contains configuration)"
    fi
fi

# 8. Update .gitignore to include common unwanted files
print_section "8. Updating .gitignore"

gitignore_additions="
# Cleanup additions
*.tmp
*.bak
*~
.DS_Store
.env.local
.env.backup
*.log

# IDE files
.vscode/settings.json
.idea/
*.swp
*.swo

# OS files
Thumbs.db
"

if [ -f ".gitignore" ]; then
    if ! grep -q "# Cleanup additions" ".gitignore"; then
        print_info "Adding cleanup patterns to .gitignore"
        echo "$gitignore_additions" >> ".gitignore"
    fi
else
    print_info "Creating .gitignore file"
    echo "$gitignore_additions" > ".gitignore"
fi

# 9. Summary of remaining structure
print_section "9. Final structure summary"

echo ""
print_info "Cleanup completed! Remaining structure:"
echo ""

echo "📁 Root files:"
echo "  ├── Cargo.toml, Cargo.lock"
echo "  ├── README.md, LICENSE, CHANGELOG.md"
echo "  ├── CONTRIBUTING.md"
echo "  ├── VERSION_QUICKREF.md (new)"
echo "  ├── justfile (enhanced)"
echo "  └── rustfmt.toml"
echo ""

echo "📁 GitHub Actions (.github/workflows/):"
echo "  ├── minimal-ci.yml (PRs only)"
echo "  └── publish.yml (releases only)"
echo ""

echo "📁 Scripts (scripts/):"
echo "  ├── auto-release.sh (main release tool)"
echo "  ├── bump-version.sh (version management)"
echo "  ├── install-hooks.sh (git hooks)"
echo "  └── pre-push (comprehensive testing)"
echo ""

echo "📁 Documentation (docs/):"
echo "  ├── AUTO_RELEASE.md (auto-release guide)"
echo "  ├── AUTO_RELEASE_ERROR_HANDLING.md (error handling)"
echo "  ├── EFFICIENT_WORKFLOW.md (workflow guide)"
echo "  ├── VERSION_MANAGEMENT.md (version guide)"
echo "  ├── VERSION_MIGRATION.md (migration guide)"
echo "  ├── SETUP.md (setup instructions)"
echo "  └── api/, examples/, guides/ (organized subdirs)"
echo ""

echo "🧹 Cleaned up:"
echo "  ❌ Legacy GitHub Actions workflows"
echo "  ❌ Redundant scripts (5 removed)"
echo "  ❌ Template documentation (4 removed)"
echo "  ❌ Duplicate README/CONTRIBUTING files"
echo "  ❌ Build artifacts and temp files"
echo ""

print_info "Codebase cleanup completed successfully! 🎉"
print_info "The repository is now clean, organized, and focused on the efficient workflow."
echo ""
print_warning "Next steps:"
echo "  1. Review changes: git status"
echo "  2. Test the workflow: just auto-patch --help"
echo "  3. Commit cleanup: git add -A && git commit -m 'Clean up codebase'"
