# Enhanced Version Management with Major-Version-Only Branching

This document outlines the version management strategy used in this template repository.

## 🎯 Core Philosophy

**"Branch only when it matters"** - Create branches only for major versions that require long-term maintenance.

## 🌳 Branching Strategy

### Our Enhanced Approach
```
main ──────────────────────────────────────────────
     \                    \                    \
      v1 ──────────────     v2 ──────────       v3 ───
         \      \    \        \      \           \
        1.1.0  1.2.0 1.2.1   2.1.0  2.1.1      3.0.1
```

### Version Rules

- **Major Version (X.0.0)**: Creates new branch (`vX`)
- **Minor Version (x.Y.0)**: Works on existing major branch
- **Patch Version (x.y.Z)**: Works on existing major branch

## 📋 Commands

### Version Bumping
```bash
./scripts/bump-version.sh patch   # Bug fixes
./scripts/bump-version.sh minor   # New features
./scripts/bump-version.sh major   # Breaking changes
./scripts/bump-version.sh 2.1.0   # Specific version
```

### Release Process
```bash
./scripts/release.sh              # Prepare release
# Create PR, review, merge to main
./scripts/create-tag.sh           # Create release tag
```

## 🔄 Workflow Example

```bash
# Start development
./scripts/bump-version.sh 0.1.0  # On main

# First stable release
./scripts/bump-version.sh 1.0.0  # Creates v1 branch
./scripts/release.sh
# PR: v1 → main, merge
./scripts/create-tag.sh          # v1.0.0 released

# Continue v1 series
git checkout v1
./scripts/bump-version.sh 1.1.0  # Features on v1
./scripts/bump-version.sh 1.1.1  # Fixes on v1

# Breaking changes
./scripts/bump-version.sh 2.0.0  # Creates v2 branch
```

## 🎯 Benefits

- ✅ **Clean Repository**: Fewer long-lived branches
- ✅ **Strategic Branching**: Branches only when needed
- ✅ **Parallel Maintenance**: Support multiple major versions
- ✅ **Clear Lifecycle**: Each major version has dedicated branch

## 🤖 Automation

The GitHub Actions workflows automatically:
- Test all changes comprehensively
- Create releases when tags are pushed
- Publish to crates.io securely
- Generate release notes

For detailed workflow information, see the GitHub Actions files in `.github/workflows/`.
