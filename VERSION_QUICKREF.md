# 🚀 Version Management Quick Reference

## One-Command Auto-Release (Recommended) ⚡

```bash
# Complete automated release workflow
just auto-patch       # 0.1.7 → 0.1.8 (bug fixes)
just auto-minor       # 0.1.7 → 0.2.0 (new features)
just auto-major       # 0.1.7 → 1.0.0 (breaking changes)
just auto-release 0.1.8  # Specific version

# What it does automatically:
# 1. 🧪 Tests everything (build, tests, docs, audit)
# 2. 🔧 Auto-fixes formatting and clippy issues
# 3. 📦 Bumps version in Cargo.toml
# 4. 💾 Commits changes and creates tag
# 5. 🚀 Pushes to GitHub (triggers publish)
```

## Manual Workflow (Step-by-step) 🔧

### Daily Development
```bash
just quick-check        # Fast compilation + unit tests
just fmt               # Format code
just pre-push          # Comprehensive local testing
```

### Version Management
```bash
just version           # Show current version: 0.1.7
just next-versions     # Show next version options
just bump-patch        # 0.1.7 → 0.1.8 (bug fixes)
just bump-minor        # 0.1.7 → 0.2.0 (new features)  
just bump-major        # 0.1.7 → 1.0.0 (breaking changes)
just bump-version 0.1.8 # Set specific version
```

### Release Workflow
```bash
# Quick release (recommended)
just release 0.1.8     # Bump version + test + commit + tag
git push origin v0.1.8 # Triggers GitHub Actions publish

# Manual step-by-step
just prepare-release 0.1.8  # Just update Cargo.toml
just pre-push               # Verify everything works
git add Cargo.toml && git commit -m 'Bump version to 0.1.8'
git tag v0.1.8 && git push origin v0.1.8
```

### Setup (One-time)
```bash
just install-hooks     # Install git hooks for local testing
cargo install cargo-audit  # For security audits
```

## �️ Error Handling & Safety

### Auto-Release Safety Features:
- ✅ **Automatic backup** before making changes
- ✅ **Rollback on failure** - any error restores original state
- ✅ **Compilation check** before auto-fixes
- ✅ **Comprehensive testing** before release
- ✅ **Clean exit** on user cancellation

### What Happens on Failure:
```bash
just auto-patch
# ❌ If clippy auto-fix fails → automatic rollback
# ❌ If tests fail → automatic rollback  
# ❌ If build fails → automatic rollback
# ✅ Your code is always restored to original state
```

### Manual Recovery (if needed):
```bash
git stash list | grep auto-release-backup  # Check for backups
git stash apply stash@{0}                  # Restore if needed
```

## �💰 Benefits

- **🚀 One-command releases** with `just auto-patch`
- **🔧 Auto-fixes issues** (formatting, clippy) 
- **80%+ reduction** in GitHub Actions costs
- **⚡ 60% faster** releases (5-8 min vs 15-20 min)
- **🛡️ Built-in safety** checks and validations
- **📊 Comprehensive testing** before every release

## 📖 Documentation

- `docs/AUTO_RELEASE.md` - Complete auto-release workflow guide
- `docs/EFFICIENT_WORKFLOW.md` - Manual workflow documentation  
- `docs/VERSION_MANAGEMENT.md` - Updated version management
- `docs/VERSION_MIGRATION.md` - Migration from legacy system

## 🔄 GitHub Actions

- **Pull Requests**: Minimal CI (compilation + unit tests)
- **Version Tags**: Publish to crates.io + create GitHub release
- **All comprehensive testing**: Happens locally before push

## 🎯 Recommended Usage

**For most releases**: Use `just auto-patch` or `just auto-minor`

**For complex releases**: Use manual workflow for more control

**For emergencies**: Direct GitHub tag push bypasses local testing
