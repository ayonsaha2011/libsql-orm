# Release Guide

This guide explains how to release new versions of libsql-orm to crates.io.

## Prerequisites

1. **GitHub Secrets**: Ensure you have the following secrets configured in your GitHub repository:
   - `CARGO_REGISTRY_TOKEN`: Your crates.io API token

2. **Crates.io Account**: You need an account on [crates.io](https://crates.io) with API access.

## Release Process

### Option 1: Automated Release (Recommended)

#### Step 1: Prepare the Release

Use the provided script to bump versions:

```bash
# Bump to a new version (e.g., 0.1.1)
./scripts/prepare-release.sh 0.1.1
```

This script will:
- Update version in both `Cargo.toml` files
- Update the dependency version in the main crate
- Verify version consistency
- Run basic build checks

#### Step 2: Commit and Tag

```bash
# Review changes
git diff

# Commit the version bump
git add .
git commit -m "Bump version to 0.1.1"

# Create and push a tag
git tag v0.1.1
git push origin v0.1.1
```

#### Step 3: Automated Release

The GitHub Actions workflow will automatically:
1. Validate the release (formatting, tests, documentation)
2. Publish the macro crate to crates.io
3. Wait for the macro crate to be available
4. Publish the main crate to crates.io
5. Create a GitHub release with changelog

### Option 2: Manual Release

#### Step 1: Trigger Manual Workflow

1. Go to the **Actions** tab in your GitHub repository
2. Select the **Release** workflow
3. Click **Run workflow**
4. Enter the version (e.g., `0.1.1`)
5. Set `dry_run` to `false`
6. Click **Run workflow**

#### Step 2: Monitor the Process

The workflow will:
1. **Validate**: Check formatting, run tests, validate versions
2. **Publish Macro Crate**: Publish `libsql-orm-macros` to crates.io
3. **Publish Main Crate**: Publish `libsql-orm` to crates.io
4. **Create Release**: Generate GitHub release with changelog

### Option 3: Dry Run

To test the release process without actually publishing:

```bash
# Use the script for dry run
./scripts/prepare-release.sh 0.1.1
git add .
git commit -m "Bump version to 0.1.1 (dry run)"
git tag v0.1.1
git push origin v0.1.1
```

Or trigger the manual workflow with `dry_run` set to `true`.

## Version Management

### Version Format

Use semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Version Consistency

Both crates (`libsql-orm` and `libsql-orm-macros`) must have the same version. The release workflow will verify this.

## Release Notes

The automated release will generate release notes from git commits. To write custom release notes:

1. Create a draft release in GitHub
2. Write your release notes
3. Publish the release manually

## Troubleshooting

### Common Issues

1. **Version Mismatch**: Ensure both crates have the same version
2. **Build Failures**: Fix any compilation errors before releasing
3. **Test Failures**: All tests must pass before release
4. **Documentation Errors**: Documentation must build successfully

### Manual Recovery

If the automated release fails:

1. Check the workflow logs for errors
2. Fix the issues
3. Re-run the workflow or create a new tag

### Rollback

To rollback a release:

1. **Crates.io**: Contact crates.io support (releases cannot be deleted)
2. **GitHub**: Delete the release and tag
3. **Code**: Revert the version bump commit

## Security

- Never commit the `CARGO_REGISTRY_TOKEN` to the repository
- Use GitHub secrets for sensitive information
- Review all changes before releasing

## Support

If you encounter issues with the release process:

1. Check the [GitHub Actions documentation](https://docs.github.com/en/actions)
2. Review the workflow logs for detailed error messages
3. Open an issue in the repository

---

**Note**: This release process is designed for the libsql-orm project structure with both a main crate and a macro crate. Adjust the process if your project structure changes. 