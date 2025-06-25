# Setup Guide

This guide walks you through setting up a new project using this template.

## 🚀 Quick Setup

### 1. Create Repository from Template

1. **Click "Use this template"** button on GitHub
2. **Name your repository** (e.g., `my-awesome-rust-project`)
3. **Clone your new repository**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/YOUR_PROJECT_NAME.git
   cd YOUR_PROJECT_NAME
   ```

### 2. Run Setup Script

```bash
# Make setup script executable and run it
chmod +x scripts/setup-template.sh
./scripts/setup-template.sh
```

### 3. Customize Project

#### Update Cargo.toml
```toml
[package]
name = "your-project-name"
version = "0.1.0"
authors = ["Your Name <your.email@example.com>"]
description = "Your project description"
repository = "https://github.com/YOUR_USERNAME/YOUR_PROJECT_NAME"
# ... other fields
```

#### Update README.md
Replace the template README with your project-specific information.

#### Add Your Code
Replace the template code in `src/` with your actual implementation.

### 4. Configure GitHub Repository

#### Required Secret
1. Go to **Settings → Secrets and variables → Actions**
2. Add `CARGO_REGISTRY_TOKEN`:
   - Get token from [crates.io/settings/tokens](https://crates.io/settings/tokens)
   - Add as repository secret

#### Branch Protection (Recommended)
1. Go to **Settings → Branches**
2. Add protection rule for `main`:
   - ✅ Require pull request reviews before merging
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging

#### Environment Protection (Optional)
1. Go to **Settings → Environments**
2. Create environment named `crates-io`
3. Add protection rules and environment-specific secrets

## 🎯 First Release

### 1. Create Initial Version
```bash
# Create your first version (this will create v1 branch for 1.x.x series)
./scripts/bump-version.sh 1.0.0
```

### 2. Prepare Release
```bash
./scripts/release.sh
```

### 3. Create Pull Request
- Push the branch: `git push -u origin v1`
- Create PR from `v1` → `main`
- Review and merge

### 4. Create Release Tag
```bash
# After PR is merged to main
git checkout main
git pull origin main
./scripts/create-tag.sh
```

This will trigger the automated release workflow!

## 🔧 Development Workflow

### Working on Major Version Series

```bash
# Switch to major version branch for development
git checkout v1

# Add features (minor version)
./scripts/bump-version.sh minor  # 1.0.0 → 1.1.0

# Fix bugs (patch version)
./scripts/bump-version.sh patch  # 1.1.0 → 1.1.1

# Prepare releases
./scripts/release.sh
# Create PR: v1 → main, merge, then tag
```

### Creating New Major Version

```bash
# When you need breaking changes
./scripts/bump-version.sh major  # Creates v2 branch for 2.x.x series

# Now you have:
# - v1 branch: maintaining 1.x.x
# - v2 branch: developing 2.x.x  
# - main: coordination and releases
```

## 🛠️ Customization

### Adding Dependencies
```bash
# Add to Cargo.toml
cargo add serde --features derive
cargo add tokio --features full
```

### Adding Features
```toml
# In Cargo.toml
[features]
default = []
async = ["tokio"]
serialization = ["serde"]
```

### Updating Workflows
Modify `.github/workflows/` files for your specific needs:
- Add target platforms
- Configure additional checks
- Customize release process

## 📚 Next Steps

1. **Read the documentation**: Check `docs/VERSION_MANAGEMENT.md`
2. **Explore examples**: Look at `examples/` directory
3. **Write tests**: Add to `tests/` directory
4. **Update documentation**: Keep README and docs current
5. **Start coding**: Replace template code with your implementation

## ❓ Troubleshooting

### Common Issues

**Scripts not executable:**
```bash
chmod +x scripts/*.sh
```

**Version conflicts:**
```bash
# Check current version
grep "^version" Cargo.toml

# Check existing tags
git tag -l
```

**CI failures:**
- Check that all tests pass locally
- Verify formatting: `cargo fmt --check`
- Run clippy: `cargo clippy --all-targets --all-features`

**Release failures:**
- Ensure `CARGO_REGISTRY_TOKEN` is set correctly
- Check crates.io for naming conflicts
- Verify package manifest is complete

### Getting Help

1. Check existing issues in the template repository
2. Review GitHub Actions logs for detailed error information
3. Verify all required secrets and settings are configured
4. Test scripts locally before pushing

## 🎉 You're Ready!

With this setup, you have:
- ✅ Automated version management
- ✅ Major-version-only branching strategy
- ✅ Comprehensive CI/CD pipeline
- ✅ Secure release automation
- ✅ Professional project structure

Happy coding! 🚀
