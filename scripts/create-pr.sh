#!/bin/bash
# Script to create a PR for the current branch
# Requires GitHub CLI (gh) to be installed and authenticated

set -e

BRANCH=$(git branch --show-current)
BASE_BRANCH="main"

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed"
    echo "Install it from: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "Error: Not authenticated with GitHub CLI"
    echo "Run: gh auth login"
    exit 1
fi

# Get repository info
REPO=$(git remote get-url origin | sed -E 's/.*github.com[:/]([^/]+\/[^/]+)(\.git)?$/\1/')

echo "Creating PR for branch: $BRANCH"
echo "Base branch: $BASE_BRANCH"
echo "Repository: $REPO"
echo ""

# Create PR with detailed description
gh pr create \
    --base "$BASE_BRANCH" \
    --head "$BRANCH" \
    --title "feat: OTLP SDK Integration Update (v0.2.0)" \
    --body "## OTLP SDK Integration Update

This PR updates the observability implementation to use the SDK from otlp-rust-service for metrics collection and logging via traces, replacing manual construction of observability data structures.

### BREAKING CHANGES

- \`OtlpConfig\` replaced with \`OtlpSdkConfig\`
  - Removed \`extra\` field
  - Added \`output_dir\` and \`write_interval_secs\` fields
  - Direct mapping to SDK ConfigBuilder requirements
- \`ObservabilityManager::new_async()\` now accepts \`Option<OtlpSdkConfig>\` instead of \`Option<OtlpConfig>\`
- Removed synchronous \`ObservabilityManager::new()\` method (dead code)

### Major Changes

- **Dead Code Removal**: Removed ~135 lines of dead code
  - \`create_batch_metrics()\` method
  - \`create_span_data()\` method
  - \`convert_config()\` method
  - Synchronous \`new()\` method

- **SDK Integration**: Updated observability to use tracing infrastructure
  - SDK picks up tracing events automatically
  - No manual ResourceMetrics/SpanData construction
  - Direct SDK ConfigBuilder usage

- **Python Test Support**: Enabled Python tests with PyO3 pytest workaround
  - Added \`pytest-forked\` for process isolation
  - Added \`conftest.py\` with proper fixtures
  - Updated CI workflow for Python test support

- **Comprehensive Test Updates**
  - All tests updated to use \`OtlpSdkConfig\`
  - Added validation tests for new config structure
  - Updated observability tests for SDK-based approach

### Migration Guide

\`\`\`rust
// Before (0.1.x)
use arrow_zerobus_sdk_wrapper::{OtlpConfig, ObservabilityManager};

let config = OtlpConfig {
    endpoint: Some(\"https://otlp-endpoint\".to_string()),
    log_level: \"info\".to_string(),
    extra: HashMap::new(),
};

// After (0.2.0)
use arrow_zerobus_sdk_wrapper::{OtlpSdkConfig, ObservabilityManager};
use std::path::PathBuf;

let config = OtlpSdkConfig {
    endpoint: Some(\"https://otlp-endpoint\".to_string()),
    output_dir: Some(PathBuf::from(\"/tmp/otlp\")),
    write_interval_secs: 5,
    log_level: \"info\".to_string(),
};
\`\`\`

### Testing

- ✅ All Rust tests pass
- ✅ All Python tests pass (with PyO3 workaround)
- ✅ Code compiles successfully
- ✅ No clippy warnings
- ✅ Code formatted with rustfmt

### Documentation

- Updated CHANGELOG.md with v0.2.0 release notes
- Added PyO3 pytest workaround documentation
- Updated README.md with Python test instructions
- Updated quickstart.md examples

### Related

- Closes: #002-otlp-sdk-update
- Feature branch: \`002-otlp-sdk-update\`

---

**Note**: This PR includes a GPG signed commit. The commit can be signed using \`./scripts/sign-commit.sh\` if the GPG card is available."

echo ""
echo "PR created successfully!"
echo "View it at: https://github.com/$REPO/pull/..."

