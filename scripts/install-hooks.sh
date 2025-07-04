#!/usr/bin/env bash

# Script to install Git hooks for kiteticker-async-manager

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"
SCRIPTS_DIR="$REPO_ROOT/scripts"

echo "🔧 Installing Git hooks for kiteticker-async-manager..."

# Install pre-push hook
if [ -f "$SCRIPTS_DIR/pre-push" ]; then
    cp "$SCRIPTS_DIR/pre-push" "$HOOKS_DIR/pre-push"
    chmod +x "$HOOKS_DIR/pre-push"
    echo "✅ Pre-push hook installed"
else
    echo "❌ Pre-push script not found at $SCRIPTS_DIR/pre-push"
    exit 1
fi

# Create a simple pre-commit hook for basic formatting
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/usr/bin/env bash

# Quick pre-commit check - just formatting
set -e

echo "🎨 Checking code formatting..."
if ! cargo fmt --check; then
    echo "❌ Code formatting issues found!"
    echo "Running cargo fmt to fix..."
    cargo fmt
    echo "✅ Code formatted. Please review changes and commit again."
    exit 1
fi

echo "✅ Code formatting is correct"
EOF

chmod +x "$HOOKS_DIR/pre-commit"
echo "✅ Pre-commit hook installed"

echo ""
echo "🎉 Git hooks installed successfully!"
echo ""
echo "The hooks will now:"
echo "  📝 Pre-commit: Check and fix code formatting"
echo "  🚀 Pre-push: Run comprehensive tests and checks"
echo ""
echo "To bypass hooks temporarily (not recommended):"
echo "  git commit --no-verify"
echo "  git push --no-verify"
