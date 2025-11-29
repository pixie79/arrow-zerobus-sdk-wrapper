#!/bin/bash
# Quick script to sign just the latest commit
# Run this after your GPG card is connected and unlocked

set -e

echo "Signing latest commit..."

# Check if GPG card is available
if ! gpg --card-status &>/dev/null; then
    echo "Error: GPG card is not available"
    echo "Please connect and unlock your GPG card, then run this script again"
    exit 1
fi

# Get current commit hash
CURRENT_COMMIT=$(git rev-parse HEAD)
echo "Current commit: $CURRENT_COMMIT"
echo ""

# Check if already signed
SIGNATURE_STATUS=$(git log -1 --format="%G?")
if [ "$SIGNATURE_STATUS" = "G" ] || [ "$SIGNATURE_STATUS" = "B" ]; then
    echo "✓ Commit is already signed!"
    exit 0
fi

# Sign the commit
echo "Signing commit..."
git commit --amend --no-edit -S

echo ""
echo "✓ Commit signed successfully!"
echo ""
echo "New commit hash: $(git rev-parse HEAD)"
echo ""
echo "To push:"
echo "  git push origin $(git branch --show-current) --force-with-lease"

