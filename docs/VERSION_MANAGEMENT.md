# Version Management Guide

This document describes how version numbers are managed across the project and how to ensure consistency.

## Version Files

Version numbers must be kept consistent across three files:

1. **Cargo.toml** - Rust package version
   ```toml
   version = "0.5.0"
   ```

2. **pyproject.toml** - Python package version
   ```toml
   version = "0.5.0"
   ```

3. **CHANGELOG.md** - Latest release version
   ```markdown
   ## [0.5.0] - 2025-12-11
   ```

## Automated Checks

### Pre-commit Hook

A pre-commit hook automatically checks version consistency before each commit. If versions don't match, the commit will be rejected.

**Installation:**
```bash
./scripts/install_pre_commit_hook.sh
```

**Manual Check:**
```bash
./scripts/check_version.sh
```

### CI/CD Pipeline

The GitHub Actions CI workflow includes a version consistency check that runs before formatting and linting checks. If versions don't match, the CI pipeline will fail.

**Location:** `.github/workflows/ci.yml` → `format-lint` job → `Check version consistency` step

## Release Process

When releasing a new version:

1. **Update Cargo.toml**
   ```toml
   version = "X.Y.Z"
   ```

2. **Update pyproject.toml**
   ```toml
   version = "X.Y.Z"
   ```

3. **Update CHANGELOG.md**
   ```markdown
   ## [X.Y.Z] - YYYY-MM-DD
   
   ### Added
   - ...
   ```

4. **Run version check**
   ```bash
   ./scripts/check_version.sh
   ```

5. **Commit changes**
   ```bash
   git add Cargo.toml pyproject.toml CHANGELOG.md
   git commit -m "chore: bump version to X.Y.Z"
   ```

The pre-commit hook will automatically verify versions match before the commit is accepted.

## Version Check Script

The version check script (`scripts/check_version.sh`) performs the following:

1. Extracts version from `Cargo.toml`
2. Extracts version from `pyproject.toml`
3. Extracts latest release version from `CHANGELOG.md`
4. Compares all three versions
5. Reports any mismatches

**Exit Codes:**
- `0` - All versions match
- `1` - Version mismatch detected

## Troubleshooting

### Pre-commit Hook Not Running

If the pre-commit hook doesn't run:

1. Check if hook is installed:
   ```bash
   ls -la .git/hooks/pre-commit
   ```

2. Reinstall the hook:
   ```bash
   ./scripts/install_pre_commit_hook.sh
   ```

3. Verify hook is executable:
   ```bash
   chmod +x .git/hooks/pre-commit
   ```

### Version Mismatch Error

If you get a version mismatch error:

1. Check current versions:
   ```bash
   ./scripts/check_version.sh
   ```

2. Update the mismatched file(s)

3. Re-run the check:
   ```bash
   ./scripts/check_version.sh
   ```

### Bypassing Pre-commit Hook (Not Recommended)

If you need to bypass the pre-commit hook (e.g., for WIP commits):

```bash
git commit --no-verify -m "wip: work in progress"
```

**Note:** The CI pipeline will still check version consistency, so mismatches will be caught before merge.

## Best Practices

1. **Always update all three files** when releasing
2. **Run version check** before committing
3. **Don't bypass pre-commit hook** unless absolutely necessary
4. **Check CI logs** if version check fails in pipeline
5. **Use semantic versioning** (MAJOR.MINOR.PATCH)

## Related Files

- `scripts/check_version.sh` - Version consistency check script
- `scripts/install_pre_commit_hook.sh` - Pre-commit hook installer
- `.git/hooks/pre-commit` - Pre-commit hook (installed locally)
- `.github/workflows/ci.yml` - CI workflow with version check

