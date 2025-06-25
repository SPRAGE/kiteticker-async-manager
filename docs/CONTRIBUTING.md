# Contributing Guide

Thank you for your interest in contributing to projects using this template!

## 🎯 Understanding the Template

This project uses the **Enhanced Version Management** template with major-version-only branching:

- **Major versions (X.0.0)**: Create dedicated branches (`v1`, `v2`, `v3`)
- **Minor versions (x.Y.0)**: Work on existing major branches
- **Patch versions (x.y.Z)**: Work on existing major branches

## 🔄 Contribution Workflow

### 1. Understanding Branches

Before contributing, understand the branch structure:

```bash
# List all branches
git branch -a

# Current branches might include:
# main        - Integration and release coordination
# v1          - Version 1.x.x maintenance and development
# v2          - Version 2.x.x active development
```

### 2. Choosing the Right Branch

**For bug fixes:**
- Target the **oldest affected major version branch**
- We'll help backport to newer versions if needed

**For new features:**
- Target the **current active major version branch**
- Usually the highest numbered version branch

**For breaking changes:**
- Discuss in an issue first
- May require creating a new major version

### 3. Development Process

#### Fork and Clone
```bash
# Fork the repository on GitHub
git clone https://github.com/YOUR_USERNAME/PROJECT_NAME.git
cd PROJECT_NAME
```

#### Set Up Development Environment
```bash
# Install dependencies
cargo build

# Run tests to ensure everything works
cargo test --all-features

# Check formatting and linting
cargo fmt --check
cargo clippy --all-targets --all-features
```

#### Create Feature Branch
```bash
# Switch to appropriate base branch
git checkout v2  # or whichever major version you're targeting

# Create feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes:
git checkout -b fix/issue-number-description
```

#### Make Changes
- Write code following the project's style
- Add tests for new functionality
- Update documentation as needed
- Follow commit message conventions

#### Test Your Changes
```bash
# Run full test suite
cargo test --all-features

# Test with minimal features
cargo test --no-default-features

# Check formatting
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Test documentation
cargo doc --no-deps --all-features
```

## 📝 Code Standards

### Rust Code Style
- Follow official Rust formatting: `cargo fmt`
- Address all clippy warnings: `cargo clippy`
- Use meaningful variable and function names
- Add documentation for public APIs
- Write tests for new functionality

### Commit Messages
Use conventional commit format:
```
type(scope): description

- feat: new feature
- fix: bug fix
- docs: documentation changes
- style: formatting changes
- refactor: code refactoring
- test: adding or updating tests
- chore: maintenance tasks
```

Examples:
```
feat(auth): add OAuth2 authentication support
fix(orders): handle missing order ID in response
docs(readme): update installation instructions
```

### Documentation
- Update README.md for user-facing changes
- Add docstring comments for new public functions
- Update CHANGELOG.md for notable changes
- Include examples for new features

## 🔍 Pull Request Process

### 1. Before Submitting
- [ ] All tests pass locally
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for notable changes)

### 2. Creating the Pull Request

**Target Branch:**
- Target the appropriate major version branch (e.g., `v2`)
- NOT the `main` branch (used only for releases)

**PR Title:**
Use descriptive titles following conventional commit format:
```
feat(orders): add order modification functionality
fix(auth): resolve token refresh race condition
```

**PR Description:**
Use the provided template and fill out all sections:
- Description of changes
- Type of change
- Testing performed
- Breaking changes (if any)
- Related issues

### 3. Review Process
- Maintainers will review your PR
- Address feedback promptly
- Keep PR focused and reasonably sized
- Be patient - reviews take time

### 4. After Approval
- Maintainers will merge to the target major version branch
- Changes will be included in the next release from that branch
- You'll be credited in the release notes

## 🎯 Contribution Types

### Bug Reports
- Use the bug report template
- Include minimal reproduction case
- Specify affected versions
- Provide environment details

### Feature Requests  
- Use the feature request template
- Explain the use case clearly
- Consider backward compatibility
- Discuss implementation approach

### Documentation Improvements
- Fix typos and grammar
- Add missing documentation
- Improve examples
- Update outdated information

### Code Contributions
- Bug fixes
- New features
- Performance improvements
- Refactoring
- Test improvements

## 🔧 Development Tips

### Working with Major Version Branches

```bash
# See what's different between major versions
git diff v1..v2

# Cherry-pick fix from one version to another
git checkout v1
git cherry-pick <commit-hash>

# Check which major version includes a change
git branch --contains <commit-hash>
```

### Testing Across Versions

```bash
# Test current branch
cargo test --all-features

# Test specific features
cargo test --features "feature1,feature2"

# Test minimal configuration
cargo test --no-default-features
```

### Understanding Release Process

The maintainers handle releases using:
1. `./scripts/release.sh` - Prepare release
2. PR from version branch → main
3. `./scripts/create-tag.sh` - Create release tag
4. Automated publishing via GitHub Actions

## ❓ Getting Help

### Communication Channels
- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, general discussion
- **Pull Requests**: Code review and collaboration

### Questions About:
- **Template Usage**: Ask in the template repository
- **Project-Specific**: Ask in the project repository  
- **Version Management**: Check `docs/VERSION_MANAGEMENT.md`

## 🙏 Recognition

Contributors are recognized in:
- Release notes
- CHANGELOG.md
- GitHub contributors list
- Special mentions for significant contributions

Thank you for contributing to make this project better! 🎉
