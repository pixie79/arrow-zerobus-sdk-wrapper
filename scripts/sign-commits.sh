#!/bin/bash
# Script to GPG sign all unsigned commits on the current branch
# Run this after your GPG card is connected and unlocked

set -e

echo "Checking for unsigned commits..."

# Check if GPG card is available
if ! gpg --card-status &>/dev/null; then
    echo "Error: GPG card is not available"
    echo "Please connect and unlock your GPG card, then run this script again"
    exit 1
fi

# Get the base branch (usually main or master)
BASE_BRANCH=$(git remote show origin | grep "HEAD branch" | awk '{print $NF}')
if [ -z "$BASE_BRANCH" ]; then
    BASE_BRANCH="main"
fi

echo "Base branch: $BASE_BRANCH"
echo ""

# Get list of unsigned commits
UNSIGNED_COMMITS=$(git log --format="%H %G? %s" origin/$BASE_BRANCH..HEAD | grep " N " | awk '{print $1}')

if [ -z "$UNSIGNED_COMMITS" ]; then
    echo "✓ All commits are already signed!"
    exit 0
fi

echo "Found unsigned commits:"
git log --format="  %h %s" origin/$BASE_BRANCH..HEAD | grep -E "$(echo $UNSIGNED_COMMITS | tr ' ' '|')"
echo ""

# Count commits
COMMIT_COUNT=$(echo "$UNSIGNED_COMMITS" | wc -l | tr -d ' ')
echo "Signing $COMMIT_COUNT commit(s)..."
echo ""

# Sign commits using interactive rebase
# We'll use rebase to sign each commit
FIRST_UNSIGNED=$(echo "$UNSIGNED_COMMITS" | tail -1)
BASE_COMMIT=$(git merge-base HEAD origin/$BASE_BRANCH)

echo "Starting interactive rebase to sign commits..."
echo "You will be prompted to sign each commit."
echo ""

# Use rebase to sign commits
git rebase --exec 'git commit --amend --no-edit -S' -i "$BASE_COMMIT"

echo ""
echo "✓ All commits signed successfully!"
echo ""
echo "Verifying signatures..."
git log --show-signature origin/$BASE_BRANCH..HEAD | grep -E "(Good signature|gpg: Signature made)"

echo ""
echo "To push the signed commits:"
echo "  git push origin $(git branch --show-current) --force-with-lease"

