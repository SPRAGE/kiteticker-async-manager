# Version Management Migration Guide

This guide helps you migrate from the legacy version management system to the new efficient workflow.

## 🔄 Migration Overview

### Old Workflow (Legacy)
```bash
./scripts/bump-version.sh patch     # Complex testing in script
./scripts/release.sh               # Creates PRs and branches
./scripts/create-tag.sh            # Complex tag creation
```

### New Workflow (Efficient)
```bash
just bump-patch                    # Uses pre-push hooks for testing
just release 0.1.8                 # Direct tag creation
git push origin v0.1.8             # Triggers GitHub Actions
```

## 🚀 Quick Migration

### 1. Start Using New Commands

Replace your old commands with new ones:

| Old Command | New Command | Notes |
|------------|-------------|-------|
| `./scripts/bump-version.sh patch` | `just bump-patch` | Uses pre-push hooks |
| `./scripts/bump-version.sh minor` | `just bump-minor` | Faster, cleaner |
| `./scripts/bump-version.sh major` | `just bump-major` | Major version handling |
| `./scripts/bump-version.sh 1.2.3` | `just bump-version 1.2.3` | Specific versions |
| Manual release process | `just release 0.1.8` | All-in-one workflow |

### 2. Version Information Commands

| Purpose | Command | Output |
|---------|---------|--------|
| Current version | `just version` | `0.1.7` |
| Next versions | `just next-versions` | Shows patch/minor/major |
| Simple release | `just prepare-release 0.1.8` | Just version update |

## 📋 Step-by-Step Migration

### Step 1: Install New Tools (if not done)
```bash
just install-hooks                  # Install git hooks
cargo install cargo-audit           # For security audits
```

### Step 2: Test New Workflow
```bash
# Check current version
just version

# See what versions are available
just next-versions

# Try a patch bump (safe)
just bump-patch
git status                          # See what changed
git reset --hard HEAD              # Undo if needed
```

### Step 3: First Release with New System
```bash
# Complete release workflow
just release 0.1.8                 # Bumps version, tests, commits, tags
git push origin v0.1.8             # Triggers publish

# Or step by step
just prepare-release 0.1.8         # Just update version
just pre-push                       # Verify locally
git add Cargo.toml && git commit -m 'Bump version to 0.1.8'
git tag v0.1.8 && git push origin v0.1.8
```

## 🔄 Workflow Comparison

### Legacy Workflow
```bash
# Old: Complex, slow, expensive CI
./scripts/bump-version.sh 0.1.8    # Runs tests in script
./scripts/release.sh               # Creates development branches
# Wait for PR creation and review
./scripts/create-tag.sh            # Complex tag handling
# Extensive GitHub Actions run (15-20 min, multiple jobs)
```

### New Efficient Workflow
```bash
# New: Simple, fast, cheap CI
just release 0.1.8                 # Local testing via pre-push hooks
git push origin v0.1.8             # Simple tag push
# Minimal GitHub Actions (3-5 min, single job)
```

## ⚡ Performance Improvements

| Aspect | Legacy | New | Improvement |
|--------|--------|-----|-------------|
| Local testing | Manual | Automatic (pre-push) | ✅ Consistent |
| GitHub Actions time | 15-20 min | 3-5 min | 🚀 75% faster |
| GitHub Actions jobs | 5+ jobs | 1 job | 💰 80% cost reduction |
| Developer feedback | Slow (CI) | Fast (local) | ⚡ Immediate |
| Release complexity | High | Low | 🎯 Simplified |

## 🛠️ Advanced Usage

### Major Version Releases
```bash
# Create major version (e.g., 0.x.x -> 1.0.0)
just bump-major                     # Updates to 1.0.0
git checkout -b v1                  # Create major version branch
git add Cargo.toml && git commit -m 'Bump version to 1.0.0'
git tag v1.0.0
git push origin v1 && git push origin v1.0.0
```

### Hotfix Releases
```bash
# Fix bug on existing major version
git checkout v1                     # Switch to major version branch
just bump-patch                     # Creates 1.0.1
git tag v1.0.1 && git push origin v1.0.1
```

### Development Versions
```bash
# Pre-release versions
just bump-version 1.0.0-beta.1     # Beta releases
just bump-version 1.0.0-rc.1       # Release candidates
```

## 🔧 Troubleshooting

### Pre-push hooks fail
```bash
# Debug individual checks
just fmt-check                      # Check formatting
just clippy                         # Check linting  
just test                          # Check tests
just audit                         # Check security
```

### Legacy scripts still needed
You can keep using legacy scripts alongside new ones:
```bash
./scripts/bump-version-efficient.sh patch  # New version with pre-push
./scripts/bump-version.sh patch           # Legacy version
```

### Emergency releases (bypass hooks)
```bash
git commit --no-verify              # Skip pre-commit
git push --no-verify               # Skip pre-push (NOT RECOMMENDED)
```

## 📝 Best Practices

1. **Always test locally first**: `just pre-push`
2. **Use semantic versioning**: patch/minor/major
3. **Create major branches**: For long-term support
4. **Monitor releases**: Check GitHub Actions after tagging
5. **Keep it simple**: Use `just release VERSION` for most cases

## 🎯 Next Steps

1. **Try the new workflow** with a patch release
2. **Update your documentation** to reference new commands
3. **Remove legacy scripts** when comfortable (optional)
4. **Share with team** to ensure everyone uses new workflow

For more details, see:
- `docs/EFFICIENT_WORKFLOW.md` - Complete workflow documentation
- `docs/VERSION_MANAGEMENT.md` - Updated version management guide
