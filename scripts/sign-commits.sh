#!/bin/bash
# Script to GPG sign commits that haven't been pushed yet
# Usage: ./scripts/sign-commits.sh [branch-name]
# If branch-name is not provided, uses current branch

set -e

BRANCH="${1:-$(git branch --show-current)}"
REMOTE_BRANCH="origin/$BRANCH"

# Check if remote branch exists
if ! git rev-parse --verify "$REMOTE_BRANCH" >/dev/null 2>&1; then
    echo "⚠️  Remote branch $REMOTE_BRANCH does not exist."
    echo "   All local commits will be signed."
    COMMITS_TO_SIGN=$(git rev-list --count HEAD ^origin/main 2>/dev/null || git rev-list --count HEAD)
else
    COMMITS_TO_SIGN=$(git rev-list --count HEAD ^"$REMOTE_BRANCH" 2>/dev/null || echo "0")
fi

if [ "$COMMITS_TO_SIGN" -eq "0" ]; then
    echo "✅ No commits to sign - branch is up to date with remote"
    exit 0
fi

echo "📝 Found $COMMITS_TO_SIGN commit(s) to sign on branch: $BRANCH"

# Check if GPG is configured
if ! git config user.signingkey >/dev/null 2>&1; then
    echo "❌ Error: GPG signing key not configured"
    echo "   Run: git config user.signingkey <your-gpg-key-id>"
    exit 1
fi

echo "🔐 GPG signing key: $(git config user.signingkey)"

# Create rebase script
REBASE_SCRIPT=$(mktemp)
trap "rm -f $REBASE_SCRIPT" EXIT

# Generate exec commands for each commit
for i in $(seq 1 "$COMMITS_TO_SIGN"); do
    echo "exec git commit --amend --no-edit -S" >> "$REBASE_SCRIPT"
done

# Perform interactive rebase
if [ -n "$REMOTE_BRANCH" ] && git rev-parse --verify "$REMOTE_BRANCH" >/dev/null 2>&1; then
    BASE="$REMOTE_BRANCH"
else
    # Find the base branch (main or master)
    if git rev-parse --verify origin/main >/dev/null 2>&1; then
        BASE="origin/main"
    elif git rev-parse --verify origin/master >/dev/null 2>&1; then
        BASE="origin/master"
    else
        echo "❌ Error: Could not find base branch (origin/main or origin/master)"
        exit 1
    fi
fi

echo "🔄 Rebasing commits from $BASE..."
echo "   You will be prompted for your GPG PIN $COMMITS_TO_SIGN time(s)"

# Use GIT_SEQUENCE_EDITOR to automatically apply the exec commands
export GIT_SEQUENCE_EDITOR="cat $REBASE_SCRIPT >"
git rebase -i "$BASE"

echo "✅ All commits have been signed!"
echo ""
echo "📤 To push the signed commits, run:"
echo "   git push --force-with-lease origin $BRANCH"
