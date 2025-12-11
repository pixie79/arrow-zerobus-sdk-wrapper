#!/bin/bash
# Version consistency check script
# Verifies that version numbers match across CHANGELOG.md, Cargo.toml, and pyproject.toml

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Checking version consistency..."

# Extract version from Cargo.toml
CARGO_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/' | tr -d ' ')
if [ -z "$CARGO_VERSION" ]; then
    echo -e "${RED}❌ Error: Could not extract version from Cargo.toml${NC}"
    exit 1
fi

# Extract version from pyproject.toml
PYPROJECT_VERSION=$(grep '^version = ' pyproject.toml | sed 's/version = "\(.*\)"/\1/' | tr -d ' ')
if [ -z "$PYPROJECT_VERSION" ]; then
    echo -e "${RED}❌ Error: Could not extract version from pyproject.toml${NC}"
    exit 1
fi

# Extract latest release version from CHANGELOG.md
# Look for the first "## [X.Y.Z]" pattern (not Unreleased)
CHANGELOG_VERSION=$(grep -E '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' CHANGELOG.md | head -1 | sed 's/^## \[\(.*\)\].*/\1/')
if [ -z "$CHANGELOG_VERSION" ]; then
    echo -e "${YELLOW}⚠️  Warning: Could not extract version from CHANGELOG.md${NC}"
    echo -e "${YELLOW}   Make sure CHANGELOG.md has a release entry like: ## [0.5.0] - YYYY-MM-DD${NC}"
    exit 1
fi

# Display versions
echo "   Cargo.toml:      $CARGO_VERSION"
echo "   pyproject.toml:  $PYPROJECT_VERSION"
echo "   CHANGELOG.md:    $CHANGELOG_VERSION"

# Check if all versions match
ERRORS=0

if [ "$CARGO_VERSION" != "$PYPROJECT_VERSION" ]; then
    echo -e "${RED}❌ Version mismatch: Cargo.toml ($CARGO_VERSION) != pyproject.toml ($PYPROJECT_VERSION)${NC}"
    ERRORS=$((ERRORS + 1))
fi

if [ "$CARGO_VERSION" != "$CHANGELOG_VERSION" ]; then
    echo -e "${RED}❌ Version mismatch: Cargo.toml ($CARGO_VERSION) != CHANGELOG.md ($CHANGELOG_VERSION)${NC}"
    ERRORS=$((ERRORS + 1))
fi

if [ "$PYPROJECT_VERSION" != "$CHANGELOG_VERSION" ]; then
    echo -e "${RED}❌ Version mismatch: pyproject.toml ($PYPROJECT_VERSION) != CHANGELOG.md ($CHANGELOG_VERSION)${NC}"
    ERRORS=$((ERRORS + 1))
fi

if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}✅ All versions match!${NC}"
    exit 0
else
    echo -e "${RED}❌ Found $ERRORS version mismatch(es). Please update versions to match.${NC}"
    exit 1
fi

