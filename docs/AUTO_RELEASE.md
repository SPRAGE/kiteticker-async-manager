# 🚀 Auto-Release Workflow

The ultimate one-command release solution for `kiteticker-async-manager`.

## 🎯 What It Does

The auto-release script (`scripts/auto-release.sh`) is a comprehensive solution that:

1. **🧪 Tests Everything** - Runs all tests, builds, and checks
2. **🔧 Fixes Issues** - Auto-fixes formatting and clippy issues where possible
3. **📦 Bumps Version** - Updates Cargo.toml with new version
4. **💾 Commits Changes** - Commits fixes and version bump
5. **🏷️ Creates Tag** - Creates and pushes version tag
6. **🚀 Triggers Publish** - GitHub Actions automatically publishes to crates.io

## ⚡ Quick Usage

### Using Just Commands (Recommended)
```bash
# Patch release (bug fixes): 0.1.7 → 0.1.8
just auto-patch

# Minor release (new features): 0.1.7 → 0.2.0  
just auto-minor

# Major release (breaking changes): 0.1.7 → 1.0.0
just auto-major

# Specific version: 0.1.7 → 0.1.8
just auto-release 0.1.8
```

### Direct Script Usage
```bash
# Patch release
./scripts/auto-release.sh patch

# Minor release
./scripts/auto-release.sh minor

# Major release  
./scripts/auto-release.sh major

# Specific version
./scripts/auto-release.sh 0.1.8
```

## 🔄 What Happens Step by Step

### 1. Prerequisites Check ✅
- Verifies git repository
- Checks for Cargo.toml
- Ensures clean working directory
- Warns if not on main/master/v* branch

### 2. Auto-Fix Phase 🔧
```bash
# Automatically fixes:
cargo fmt                          # Code formatting
cargo clippy --fix                 # Auto-fixable clippy issues
```

### 3. Comprehensive Testing 🧪
```bash
cargo build --all-features         # Full build
cargo test --all-features          # All tests
cargo test --doc --all-features    # Documentation tests
cargo doc --no-deps --all-features # Documentation build
cargo audit                        # Security audit (if installed)
cargo publish --dry-run            # Publish readiness
```

### 4. Version Management 📦
- Updates `version = "..."` in Cargo.toml
- Follows semantic versioning rules
- Validates version format

### 5. Git Operations 💾
```bash
git add .                          # Stage auto-fixes
git commit -m "Auto-fix..."        # Commit fixes
git add Cargo.toml                 # Stage version bump
git commit -m "Bump version..."    # Commit version
git tag v0.1.8                     # Create version tag
git push origin main               # Push commits
git push origin v0.1.8            # Push tag (triggers publish)
```

### 6. GitHub Actions Trigger 🚀
- Minimal smoke tests (~3-5 minutes)
- Automatic publish to crates.io
- GitHub release creation

## 🛡️ Safety Features

### Validation Checks
- **Clean workspace** - Prevents accidental commits
- **Branch validation** - Warns for non-standard branches
- **Version format** - Validates semantic versioning
- **Test failures** - Stops on any test failure
- **Manual confirmation** - Asks before proceeding

### Auto-Fix Capabilities
- **Formatting** - `cargo fmt` fixes all style issues
- **Clippy** - Auto-fixes safe clippy suggestions
- **Import optimization** - Removes unused imports
- **Code style** - Standardizes code patterns

### Rollback Safety
If the script fails at any point:
- No changes are committed to git
- You can fix issues and run again
- Working directory remains clean

## 📊 Comparison with Other Workflows

| Workflow | Commands | Time | Auto-fixes | Safety |
|----------|----------|------|------------|--------|
| **Auto-release** | 1 | ~5 min | ✅ Yes | 🛡️ High |
| Manual process | 8-10 | ~15 min | ❌ No | ⚠️ Manual |
| Legacy scripts | 3-4 | ~10 min | ❌ No | ⚠️ Medium |

## 🎯 Best Practices

### When to Use Auto-Release
- ✅ **Regular releases** - Patch and minor versions
- ✅ **Clean codebase** - When you trust auto-fixes
- ✅ **Time pressure** - Fast, reliable releases
- ✅ **Team consistency** - Standardized process

### When to Use Manual Process
- ⚠️ **Major releases** - When you want more control
- ⚠️ **Complex changes** - Manual review needed
- ⚠️ **First time** - Learning the codebase
- ⚠️ **Custom workflows** - Special requirements

## 🔧 Troubleshooting

### Common Issues

#### "Working directory is not clean"
```bash
git status                         # See what's changed
git add . && git commit -m "WIP"   # Commit changes
# or
git stash                          # Stash changes
```

#### "Clippy checks failed"
```bash
cargo clippy --all-features        # See detailed issues
# Fix manually, then run auto-release again
```

#### "Tests failed"
```bash
cargo test --all-features          # See failing tests
# Fix tests, then run auto-release again
```

#### "Security audit failed"
```bash
cargo audit                        # See vulnerabilities
cargo update                       # Update dependencies
# or manually address security issues
```

### Emergency Bypass
If you need to release urgently despite issues:
```bash
# Use manual process instead
just prepare-release 0.1.8
git add Cargo.toml && git commit -m 'Bump version to 0.1.8'
git tag v0.1.8 && git push origin v0.1.8
```

## 📈 Performance Benefits

### Time Savings
- **Old process**: ~15-20 minutes (manual + CI)
- **Auto-release**: ~5-8 minutes (local + CI)
- **Savings**: 60%+ faster releases

### Error Reduction
- **Automatic fixes** prevent common issues
- **Comprehensive testing** catches problems early
- **Standardized process** reduces human error

### Developer Experience
- **One command** vs multiple manual steps
- **Immediate feedback** from local testing
- **Consistent results** across team members

## 🔗 Integration with Other Tools

### Works With
- ✅ **Git hooks** - Pre-commit and pre-push hooks
- ✅ **GitHub Actions** - Triggers minimal CI/CD
- ✅ **Cargo tools** - fmt, clippy, audit, etc.
- ✅ **Just commands** - Integrates with justfile

### Replaces
- ❌ **Manual testing** - Automated comprehensive testing
- ❌ **Manual fixes** - Auto-fixes formatting and clippy
- ❌ **Complex scripts** - Single command replaces multiple
- ❌ **Error-prone process** - Standardized and validated

## 📝 Example Session

```bash
$ just auto-patch
🚀 KiteTicker Async Manager - Auto Release Script
=================================================

[VERSION] Current version: 0.1.7
[VERSION] Target version: 0.1.8

🤔 Proceed with release v0.1.8? (y/N): y

[STEP] Checking prerequisites...
[SUCCESS] Prerequisites check passed

[STEP] Checking and fixing code formatting...
[SUCCESS] Code formatting is already correct

[STEP] Running clippy checks and auto-fixes...
[FIX] Auto-fixed 3 clippy suggestions
[SUCCESS] All clippy checks passed

[STEP] Running comprehensive test suite...
[INFO] Building project...
[SUCCESS] Build successful
[INFO] Running tests...
[SUCCESS] All tests passed
[INFO] Running documentation tests...
[SUCCESS] Documentation tests passed
[INFO] Verifying documentation builds...
[SUCCESS] Documentation builds successfully
[INFO] Running security audit...
[SUCCESS] Security audit passed
[INFO] Testing publish readiness...
[SUCCESS] Package is ready for publishing

[STEP] Updating version...
[SUCCESS] Version updated to 0.1.8

[STEP] Creating release v0.1.8...
[INFO] Committing auto-fixes...
[INFO] Committing version bump...
[INFO] Creating version tag...
[INFO] Pushing to GitHub...
[SUCCESS] Release v0.1.8 pushed to GitHub!

🎉 Release v0.1.8 completed successfully!

What happens next:
  📦 GitHub Actions will run smoke tests
  🚀 Package will be published to crates.io
  📝 GitHub release will be created

Monitor progress at:
  https://github.com/SPRAGE/kiteticker-async-manager/actions

[SUCCESS] Auto-release script completed! 🚀
```

## 🚀 Get Started

1. **First time setup**:
   ```bash
   just install-hooks              # Install git hooks
   cargo install cargo-audit       # Install security audit tool
   ```

2. **Make your first auto-release**:
   ```bash
   just auto-patch                 # Safe patch release
   ```

3. **Monitor the results** on GitHub Actions

That's it! You now have a one-command release workflow that handles everything automatically while maintaining high quality and safety standards.
