# Efficient Development Workflow

This document describes the efficient development workflow for `kiteticker-async-manager` that minimizes GitHub Actions usage while maintaining code quality.

## 🎯 Philosophy

**Do heavy work locally, minimal work on GitHub Actions**

- ✅ **Local**: Formatting, linting, comprehensive testing, documentation
- ✅ **GitHub**: Only smoke tests and publishing
- 💰 **Result**: Reduced GitHub Actions costs and faster feedback

## 🚀 Quick Setup

1. **Install Git hooks** (one-time setup):
   ```bash
   just install-hooks
   # or manually: ./scripts/install-hooks.sh
   ```

2. **Install required tools**:
   ```bash
   cargo install cargo-audit  # for security audits
   ```

## 📋 Development Workflow

### Daily Development

```bash
# Quick check during development
just quick-check

# Format your code (or let pre-commit hook do it)
just fmt

# Run comprehensive checks before pushing
just pre-push
```

### Before Pushing

The pre-push hook automatically runs:
- ✅ Code formatting check
- ✅ Clippy linting
- ✅ Full build
- ✅ All tests (unit, integration, doc)
- ✅ Documentation build
- ✅ Security audit
- ✅ Publish dry-run

If any check fails, the push is blocked.

### Git Hooks

- **Pre-commit**: Automatically formats code
- **Pre-push**: Runs comprehensive checks

To bypass hooks (emergency only):
```bash
git commit --no-verify
git push --no-verify
```

## 📦 Release Process

### Method 1: Using justfile
```bash
just prepare-release 0.1.8
just pre-push  # verify everything works
git add Cargo.toml && git commit -m 'Bump version to 0.1.8'
git tag v0.1.8 && git push origin v0.1.8
```

### Method 2: Using release script
```bash
./scripts/simple-release.sh 0.1.8
git push origin v0.1.8  # triggers publish
```

## 🔄 GitHub Actions

### Current Workflows

1. **`minimal-ci.yml`** - Runs on PRs only:
   - ✅ Compilation check
   - ✅ Unit tests only
   - ✅ Publish dry-run

2. **`publish.yml`** - Runs on version tags:
   - ✅ Smoke tests (compilation + unit tests)
   - ✅ Publish to crates.io
   - ✅ Create GitHub release

### Removed Workflows

The original extensive CI workflows (`ci.yml`, `release.yml`) are replaced with minimal ones to reduce costs:

- ❌ Multiple Rust versions testing
- ❌ Coverage reports
- ❌ Documentation builds
- ❌ MSRV testing
- ❌ Security audits

These are now handled locally via pre-push hooks.

## 🛠️ Available Commands

### Development
```bash
just check           # cargo check
just build           # release build
just test            # run tests
just fmt             # format code
just clippy          # run linting
just doc             # build docs
just quick-check     # fast check for development
```

### Quality Assurance
```bash
just pre-push        # comprehensive pre-push checks
just fmt-check       # check formatting without fixing
just audit           # security audit
just test-doc        # doc tests
```

### Maintenance
```bash
just clean-all       # clean everything
just install-hooks   # install git hooks
```

## 📊 Cost Savings

### Before (Old CI)
- Runs on every push/PR
- 5+ jobs per workflow
- Multiple Rust versions
- Coverage generation
- ~15-20 minutes per run

### After (New Workflow)
- PRs: 1 job, ~2-3 minutes
- Releases: 1 job, ~3-5 minutes
- **Estimated 80%+ reduction in GitHub Actions usage**

## 🔒 Security

- Pre-push hooks ensure security audits run locally
- GitHub secrets used only for crates.io publishing
- No sensitive data in workflows

## 🐛 Troubleshooting

### Pre-push hook fails
```bash
# Run individual checks to identify the issue
just fmt-check
just clippy
just test
just audit
```

### Skip hooks in emergency
```bash
git push --no-verify  # NOT RECOMMENDED
```

### Reset hooks
```bash
just install-hooks  # reinstall hooks
```

## 📝 Notes

- The old CI workflows are kept as reference but should be removed when satisfied with the new setup
- All comprehensive testing happens locally, ensuring fast feedback
- GitHub Actions are used only for the final publishing step
- Contributors must have the git hooks installed for this workflow to be effective
