#!/bin/bash
# Script to GPG sign commits and push them
# Usage: ./scripts/sign-and-push.sh [branch-name]
# If branch-name is not provided, uses current branch

set -e

BRANCH="${1:-$(git branch --show-current)}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🔐 Signing commits on branch: $BRANCH"
"$SCRIPT_DIR/sign-commits.sh" "$BRANCH"

if [ $? -eq 0 ]; then
    echo ""
    read -p "Push signed commits to origin/$BRANCH? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "📤 Pushing signed commits..."
        git push --force-with-lease origin "$BRANCH"
        echo "✅ Pushed successfully!"
    else
        echo "⏭️  Skipped push. Run 'git push --force-with-lease origin $BRANCH' when ready."
    fi
else
    echo "❌ Signing failed. Aborting push."
    exit 1
fi

