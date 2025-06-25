# 🚀 Creating Your GitHub Template Repository

Follow these steps to create a separate GitHub repository for the Rust project template.

## 📋 Step-by-Step Setup

### 1. Create New GitHub Repository

1. **Go to GitHub** and click "New repository"
2. **Repository name**: `rust-project-template` (or your preferred name)
3. **Description**: "Rust project template with major-version-only branching and automated workflows"
4. **Visibility**: Public (to use as template)
5. **Initialize**: Don't add README, .gitignore, or license (we have our own)
6. **Click "Create repository"**

### 2. Prepare Template Files

The template files are ready in `/tmp/rust-template/`. Let's set up the repository:

```bash
# Navigate to the template directory
cd /tmp/rust-template

# Initialize git repository
git init

# Add all template files
git add .

# Initial commit
git commit -m "feat: initial template with major-version-only branching

- Enhanced version management scripts
- Automated CI/CD workflows  
- Major-version-only branching strategy
- Comprehensive GitHub templates
- Security-first release automation"

# Add remote origin (replace with your repository URL)
git remote add origin https://github.com/YOUR_USERNAME/rust-project-template.git

# Push to GitHub
git branch -M main
git push -u origin main
```

### 3. Configure Repository as Template

1. **Go to your repository** on GitHub
2. **Click "Settings"** tab
3. **Scroll down** to "Template repository" section
4. **Check the box** "Template repository"
5. **Click "Save"**

### 4. Set Up Repository Settings

#### Branch Protection
1. **Settings → Branches**
2. **Add rule** for `main` branch:
   - ✅ Require pull request reviews before merging
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
   - ✅ Include administrators

#### Repository Topics
1. **Main repository page**
2. **Click the gear icon** next to "About"
3. **Add topics**: `rust`, `template`, `github-template`, `version-management`, `ci-cd`

#### Repository Description
Update the description to:
```
🚀 Rust project template with major-version-only branching strategy and automated release workflows
```

### 5. Test the Template

#### Create Test Repository
1. **Click "Use this template"** button
2. **Create a test repository**
3. **Clone and test**:

```bash
git clone https://github.com/YOUR_USERNAME/test-project.git
cd test-project

# Run setup
chmod +x scripts/setup-template.sh
./scripts/setup-template.sh

# Test version management
./scripts/bump-version.sh 1.0.0
./scripts/release.sh

# Verify everything works
cargo test --all-features
cargo build --release
```

## 🔧 Optional Enhancements

### Add Repository Secrets (For Template Testing)
1. **Settings → Secrets and variables → Actions**
2. **Add repository secret**: `CARGO_REGISTRY_TOKEN`
   - Get token from [crates.io/settings/tokens](https://crates.io/settings/tokens)
   - Only needed if you want to test publishing

### Create Environment Protection
1. **Settings → Environments**
2. **New environment**: `crates-io`
3. **Add protection rules**:
   - Required reviewers
   - Deployment branches: main only

### Add GitHub Pages (Optional)
If you want to host documentation:
1. **Settings → Pages**
2. **Source**: Deploy from a branch
3. **Branch**: main / docs folder

## 📚 Using Your Template

### For New Projects
Share this URL with others to use your template:
```
https://github.com/YOUR_USERNAME/rust-project-template/generate
```

### Template Features You Can Promote
- ✅ **Major-Version-Only Branching**: Clean repository management
- ✅ **Automated CI/CD**: Comprehensive testing and release automation
- ✅ **Security-First**: Multiple validation layers before release
- ✅ **Professional Setup**: Issues, PRs, documentation templates
- ✅ **Real-World Ready**: Production-quality workflows and scripts

## 🎯 Template Repository Structure

Your final template repository will have:

```
📂 rust-project-template/
├── 📄 README.md                    # Template overview and usage
├── 📄 Cargo.toml                   # Example Rust project manifest
├── 📄 LICENSE                      # MIT license
├── 📄 CHANGELOG.md                 # Changelog template
├── 📄 .gitignore                   # Rust-specific gitignore
├── 📂 .github/
│   ├── 📂 ISSUE_TEMPLATE/          # Bug report, feature request templates
│   ├── 📂 workflows/
│   │   ├── 📄 ci.yml              # Enhanced CI workflow
│   │   └── 📄 release.yml         # Automated release workflow
│   └── 📄 pull_request_template.md
├── 📂 scripts/
│   ├── 📄 setup-template.sh       # One-command initialization  
│   ├── 📄 bump-version.sh         # Smart version management
│   ├── 📄 release.sh              # Release preparation
│   └── 📄 create-tag.sh           # Secure tag creation
├── 📂 src/
│   ├── 📄 lib.rs                  # Example library code
│   └── 📄 main.rs                 # Example binary code
├── 📂 examples/                    # Usage examples
├── 📂 tests/                      # Integration tests
└── 📂 docs/                       # Documentation
    ├── 📄 SETUP.md                # Setup guide
    ├── 📄 VERSION_MANAGEMENT.md   # Workflow documentation
    └── 📄 CONTRIBUTING.md         # Contribution guidelines
```

## 🚀 Sharing Your Template

### README Badge
Add this badge to your template README:
```markdown
[![Use this template](https://img.shields.io/badge/use-this--template-blue?logo=github)](https://github.com/YOUR_USERNAME/rust-project-template/generate)
```

### Community Promotion
- Share on Reddit (r/rust)
- Tweet about it
- Add to awesome-rust lists
- Share in Rust community Discord/Slack

### Template Improvements
Keep improving your template based on:
- User feedback
- New Rust features
- GitHub Actions updates
- Community best practices

## 🎉 You're Done!

Your template repository is now ready to be used by the community! 

**Template URL**: `https://github.com/YOUR_USERNAME/rust-project-template`

**Generate URL**: `https://github.com/YOUR_USERNAME/rust-project-template/generate`

Anyone can now use your template to create new Rust projects with professional-grade version management and automation! 🎊
