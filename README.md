# 🚀 Rust Project Template with Enhanced Version Management

**A comprehensive, reusable GitHub template for Rust projects with major-version-only branching strategy and automated release workflows.**

[![CI](https://github.com/YOUR_USERNAME/rust-project-template/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/rust-project-template/actions)
[![Template](https://img.shields.io/badge/template-rust--project-blue)](https://github.com/YOUR_USERNAME/rust-project-template/generate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ What's This Template?

This is a production-ready GitHub template that provides:

- **🎯 Major-Version-Only Branching**: Clean repository management with strategic branching
- **🤖 Automated CI/CD**: Complete GitHub Actions workflows for testing and releasing
- **🔧 Enhanced Scripts**: Intelligent version management with branch detection
- **🛡️ Security-First**: Comprehensive validation before any release
- **📋 Template Ready**: Fully configured GitHub issue and PR templates

## 🚀 Quick Start

### 1. Use This Template

1. **Click the "Use this template" button** above
2. **Create your new repository** with your project name
3. **Clone your new repository**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/YOUR_PROJECT_NAME.git
   cd YOUR_PROJECT_NAME
   ```

### 2. Initialize Your Project

```bash
# Make setup script executable and run it
chmod +x scripts/setup-template.sh
./scripts/setup-template.sh
```

### 3. Customize for Your Project

```bash
# Update Cargo.toml with your project details
vim Cargo.toml

# Update README.md with your project information  
vim README.md

# Add your project code
```

### 4. Configure GitHub Repository

1. **Add Repository Secret**: Go to Settings → Secrets and variables → Actions
   - Add `CARGO_REGISTRY_TOKEN` with your crates.io API token
2. **Set Branch Protection**: Settings → Branches → Add protection rule for `main`
3. **Optional**: Create `crates-io` environment for additional security

### 5. Start Version Management

```bash
# Create your first version
./scripts/bump-version.sh 1.0.0  # Creates v1 major version branch

# Prepare release
./scripts/release.sh

# Create PR, review, merge to main, then:
./scripts/create-tag.sh  # Triggers automated publishing
```

## 🌳 Major-Version-Only Branching Strategy

### The Innovation: Strategic Branching

Instead of creating branches for every version, we only branch for major versions:

```bash
# ❌ Traditional: Too many branches
main ─────────────────────────────────
     \     \     \     \     \     \
    v1.0.0 v1.1.0 v1.2.0 v2.0.0 v2.1.0 v2.2.0

# ✅ Enhanced: Strategic branching  
main ──────────────────────────────────
     \                    \
      v1 ──────────────     v2 ──────────
         \      \    \        \      \
        1.1.0  1.2.0 1.2.1   2.1.0  2.1.1
```

### Branching Rules

- **Major Versions (X.0.0)**: Create dedicated branch (`v1`, `v2`, `v3`)
- **Minor Versions (x.Y.0)**: Work on existing major branch  
- **Patch Versions (x.y.Z)**: Work on existing major branch

## 📋 Version Management Commands

### Basic Version Bumping

```bash
# Patch version (bug fixes) - works on current major branch
./scripts/bump-version.sh patch

# Minor version (new features) - works on current major branch
./scripts/bump-version.sh minor

# Major version (breaking changes) - creates new major branch
./scripts/bump-version.sh major

# Specific version with intelligent branch detection
./scripts/bump-version.sh 2.1.0
```

### Release Workflow

```bash
# Prepare release with comprehensive validation
./scripts/release.sh [version]

# This will:
# ✅ Create/switch to appropriate branch
# ✅ Update version in Cargo.toml
# ✅ Run comprehensive tests
# ✅ Validate security and publishing
# ✅ Push branch and help create PR

# After PR review and merge to main:
./scripts/create-tag.sh

# This triggers automated:
# 🤖 GitHub Actions testing
# 📦 Publishing to crates.io
# 🎉 GitHub release creation
```

## 🤖 Automated Workflows

### CI/CD Features

- **🧪 Multi-Rust Testing**: Stable, beta, nightly
- **🔍 Code Quality**: fmt, clippy, security audits
- **📚 Documentation**: Build docs, check broken links
- **🕰️ MSRV Validation**: Minimum Supported Rust Version
- **📦 Publish Validation**: Dry-run before release
- **🛡️ Security Scanning**: Automated vulnerability checks

### Release Automation

- **🔍 Pre-release Validation**: Comprehensive testing
- **🏷️ Intelligent Tagging**: Automatic tag creation
- **📦 Automated Publishing**: Secure crates.io publishing
- **🎉 GitHub Releases**: Auto-generated release notes
- **📧 Notifications**: Success/failure notifications

## 📁 Template Structure

```
📂 .github/
├── 📁 ISSUE_TEMPLATE/          # Bug reports, features, questions
├── 📁 workflows/
│   ├── 🔄 ci.yml              # Enhanced CI with security
│   └── 🚀 release.yml         # Automated release pipeline
└── 📋 pull_request_template.md

📂 scripts/
├── 🔧 setup-template.sh       # One-command initialization
├── ⬆️ bump-version.sh          # Smart version bumping
├── 📦 release.sh               # Release preparation
└── 🏷️ create-tag.sh            # Secure tag creation

📂 src/                         # Your Rust code goes here
├── lib.rs                      # Main library file
└── main.rs                     # Optional binary

📂 docs/                        # Template documentation
└── examples/                   # Usage examples
```

## 🎯 Real-World Usage Example

```bash
# Project lifecycle demonstration:

# 1. Start development
./scripts/bump-version.sh 0.1.0  # Early development on main
./scripts/bump-version.sh 0.2.0  # Add features

# 2. First stable release  
./scripts/bump-version.sh 1.0.0  # Creates v1 branch
./scripts/release.sh             # Prepare release
# Create PR: v1 → main, review, merge
./scripts/create-tag.sh          # Release v1.0.0

# 3. Continue v1 development
git checkout v1
./scripts/bump-version.sh 1.1.0  # New features on v1 branch
./scripts/bump-version.sh 1.1.1  # Bug fixes on v1 branch

# 4. Breaking changes needed
./scripts/bump-version.sh 2.0.0  # Creates v2 branch

# 5. Parallel maintenance
# v1 branch: 1.x.x maintenance releases
# v2 branch: 2.x.x active development
# Both maintained simultaneously!
```

## 🔧 Configuration

### Required Repository Secrets

```bash
CARGO_REGISTRY_TOKEN    # Your crates.io API token for publishing
```

### Recommended Branch Protection

For `main` branch:
- ✅ Require pull request reviews before merging
- ✅ Require status checks to pass before merging  
- ✅ Require branches to be up to date before merging
- ✅ Include administrators

### Optional Environment Protection

Create `crates-io` environment for additional security:
- Required reviewers for releases
- Deployment protection rules
- Environment-specific secrets

## 🛠️ Customization

### For Your Project

1. **Update Metadata**: Modify `Cargo.toml` with your project details
2. **Add Your Code**: Replace template code with your implementation
3. **Configure Features**: Adjust feature flags and dependencies
4. **Customize Workflows**: Modify GitHub Actions for your needs
5. **Update Documentation**: Replace template docs with your project info

### For Other Languages

While optimized for Rust, core concepts work for any language:
- Replace `Cargo.toml` version management with `package.json`, `pyproject.toml`, etc.
- Adjust CI commands for your language toolchain
- Modify publishing steps for your package registry

## 🎉 Benefits

### ✅ For Developers
- **Clear Context**: Always know which version you're working on
- **Faster Development**: No branches for minor changes
- **Easy Maintenance**: Simple backporting to older versions
- **Parallel Work**: Multiple major versions simultaneously

### ✅ For Projects  
- **Clean History**: Organized and readable repository
- **Scalable**: Works for long-term support requirements
- **Maintainable**: Easy version lifecycle tracking
- **Team Friendly**: Clear collaboration conventions

### ✅ For Users
- **Predictable Releases**: Clear version stability expectations
- **Long-term Support**: Security updates for older versions
- **Clear Migration**: Obvious upgrade paths between versions

## 📚 Documentation

- **[Setup Guide](docs/SETUP.md)**: Detailed setup instructions
- **[Version Management](docs/VERSION_MANAGEMENT.md)**: Complete workflow guide
- **[Contributing](docs/CONTRIBUTING.md)**: How to contribute to projects using this template
- **[Examples](examples/)**: Real-world usage examples

## 🚨 When to Use This Template

### ✅ Perfect For:
- Libraries with API stability commitments
- Projects with long-term support requirements
- Teams managing multiple major versions
- Professional/production Rust projects

### ❌ Consider Alternatives For:
- Rapid prototype development
- Single-developer hobby projects
- Projects with frequent breaking changes
- Short-lived experimental projects

## 🤝 Contributing

Contributions to improve this template are welcome!

1. Fork this repository
2. Create a feature branch: `git checkout -b feature/amazing-improvement`
3. Make your changes
4. Test with a real project
5. Submit a pull request

## 📄 License

This template is provided under the [MIT License](LICENSE). Projects created from this template can use any license.

---

## 🎊 Ready to Start?

1. **[Use this template](https://github.com/YOUR_USERNAME/rust-project-template/generate)** to create your repository
2. **Run the setup script**: `./scripts/setup-template.sh`
3. **Configure your repository** with secrets and protection rules
4. **Start building** with automated version management!

**🚀 Happy coding and automated releasing!**

---

**Made with ❤️ for the Rust community**
