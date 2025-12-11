#!/bin/bash
# Install pre-commit hook for version checking

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

HOOK_FILE="$REPO_ROOT/.git/hooks/pre-commit"
HOOK_CONTENT="$REPO_ROOT/.git/hooks/pre-commit"

# Check if .git/hooks directory exists
if [ ! -d "$REPO_ROOT/.git/hooks" ]; then
    echo "Error: .git/hooks directory not found. Are you in a git repository?"
    exit 1
fi

# Copy pre-commit hook
if [ -f "$REPO_ROOT/.git/hooks/pre-commit" ]; then
    echo "⚠️  Pre-commit hook already exists. Backing up to pre-commit.backup"
    cp "$REPO_ROOT/.git/hooks/pre-commit" "$REPO_ROOT/.git/hooks/pre-commit.backup"
fi

# Create pre-commit hook
cat > "$HOOK_FILE" << 'EOF'
#!/bin/bash
# Pre-commit hook to check version consistency
# This hook runs before each commit to ensure version numbers match

# Get the repository root
REPO_ROOT="$(git rev-parse --show-toplevel)"

# Run version check script
if [ -f "$REPO_ROOT/scripts/check_version.sh" ]; then
    "$REPO_ROOT/scripts/check_version.sh"
    if [ $? -ne 0 ]; then
        echo ""
        echo "❌ Pre-commit hook failed: Version mismatch detected"
        echo "   Please ensure versions match in:"
        echo "   - Cargo.toml"
        echo "   - pyproject.toml"
        echo "   - CHANGELOG.md (latest release)"
        exit 1
    fi
fi

exit 0
EOF

chmod +x "$HOOK_FILE"
echo "✅ Pre-commit hook installed successfully!"
echo "   The hook will check version consistency before each commit."

