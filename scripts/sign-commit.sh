#!/bin/bash
# Script to GPG sign the latest commit and tag
# Run this after your GPG card is connected and unlocked

set -e

echo "GPG signing latest commit and tag..."

# Get the latest commit hash
COMMIT_HASH=$(git rev-parse HEAD)

echo "Signing commit: $COMMIT_HASH"

# Amend the commit with GPG signature
git commit --amend --no-edit -S

# Create signed tag
echo "Creating signed tag v0.2.0..."
git tag -s v0.2.0 -m "Release v0.2.0: OTLP SDK Integration Update

BREAKING CHANGE: OtlpConfig → OtlpSdkConfig

This release updates observability to use otlp-rust-service SDK directly,
removing manual construction methods and simplifying configuration.

Key Features:
- New OtlpSdkConfig structure aligned with SDK requirements
- Removed ~135 lines of dead code
- SDK-based metrics and traces via tracing infrastructure
- Python test support with PyO3 pytest workaround
- Comprehensive test coverage for SDK integration

See CHANGELOG.md for full details and migration guide."

echo "✓ Commit and tag signed successfully!"
echo ""
echo "To push:"
echo "  git push origin 002-otlp-sdk-update"
echo "  git push origin v0.2.0"

