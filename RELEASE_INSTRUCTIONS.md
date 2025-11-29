# Release Instructions for v0.2.0

## Current Status

✅ Commit created: `9626a17` - "feat: OTLP SDK integration update (v0.2.0)"
✅ Tag created: `v0.2.0`
✅ Branch pushed: `002-otlp-sdk-update`
✅ Tag pushed: `v0.2.0`
✅ Pull Request created: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/pull/3
⚠️ GPG signing pending (GPG card not available - can be done later)

## Steps to Complete Release

### 1. GPG Sign the Commit and Tag

Once your GPG card is connected and unlocked:

```bash
# Option A: Use the helper script
./scripts/sign-commit.sh

# Option B: Manual signing
git commit --amend --no-edit -S
git tag -d v0.2.0
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
```

### 2. Push to Remote

After GPG signing:

```bash
# Push the branch
git push origin 002-otlp-sdk-update

# Push the tag
git push origin v0.2.0
```

**Note**: If SSH authentication fails due to GPG card, you may need to:
- Connect and unlock your GPG card
- Or use HTTPS authentication temporarily: `git remote set-url origin https://github.com/pixie79/arrow-zerobus-sdk-wrapper.git`

### 3. Create Pull Request

**Option A: Using GitHub CLI (if available)**
```bash
./scripts/create-pr.sh
```

**Option B: Manual PR Creation**
1. Go to: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/compare/main...002-otlp-sdk-update
2. Click "Create Pull Request"
3. Use the title: "feat: OTLP SDK Integration Update (v0.2.0)"
4. Copy the PR description from `scripts/create-pr.sh` or use:

```markdown
## OTLP SDK Integration Update

This PR updates the observability implementation to use the SDK from otlp-rust-service for metrics collection and logging via traces, replacing manual construction of observability data structures.

### BREAKING CHANGES

- `OtlpConfig` replaced with `OtlpSdkConfig`
  - Removed `extra` field
  - Added `output_dir` and `write_interval_secs` fields
  - Direct mapping to SDK ConfigBuilder requirements
- `ObservabilityManager::new_async()` now accepts `Option<OtlpSdkConfig>` instead of `Option<OtlpConfig>`
- Removed synchronous `ObservabilityManager::new()` method (dead code)

### Major Changes

- **Dead Code Removal**: Removed ~135 lines of dead code
- **SDK Integration**: Updated observability to use tracing infrastructure
- **Python Test Support**: Enabled Python tests with PyO3 pytest workaround
- **Comprehensive Test Updates**: All tests updated to use `OtlpSdkConfig`

### Migration Guide

See CHANGELOG.md for full migration guide.

### Testing

- ✅ All Rust tests pass
- ✅ All Python tests pass (with PyO3 workaround)
- ✅ Code compiles successfully
- ✅ No clippy warnings

### Related

- Closes: #002-otlp-sdk-update
- Feature branch: `002-otlp-sdk-update`
```

### 4. Create GitHub Release

After the PR is merged:

1. Go to: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/new
2. Tag: `v0.2.0`
3. Title: `v0.2.0: OTLP SDK Integration Update`
4. Description: Copy from CHANGELOG.md section for v0.2.0
5. Mark as "Latest release"
6. Publish release

## Verification

After completing all steps:

```bash
# Verify commit is signed
git log --show-signature -1

# Verify tag is signed
git tag -v v0.2.0

# Verify remote has the branch and tag
git ls-remote --heads origin 002-otlp-sdk-update
git ls-remote --tags origin v0.2.0
```

## Troubleshooting

### GPG Card Not Available

If you see "Card error" when trying to sign:

1. **Check GPG card connection**:
   ```bash
   gpg --card-status
   ```

2. **Unlock the card** (if it has a PIN):
   ```bash
   gpg --card-edit
   # Then type: admin
   # Then: passwd
   ```

3. **Verify key is available**:
   ```bash
   gpg --list-secret-keys
   ```

### SSH Authentication Fails

If SSH push fails because it requires GPG card:

1. **Temporarily use HTTPS**:
   ```bash
   git remote set-url origin https://github.com/pixie79/arrow-zerobus-sdk-wrapper.git
   git push origin 002-otlp-sdk-update
   git push origin v0.2.0
   ```

2. **Or connect/unlock GPG card** and retry SSH push

### Tag Already Exists

If you see "tag already exists":

```bash
# Delete local tag
git tag -d v0.2.0

# Delete remote tag (if pushed)
git push origin :refs/tags/v0.2.0

# Recreate tag
git tag -s v0.2.0 -m "Release message..."
```

