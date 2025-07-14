# Publishing Guide for libsql-orm

This guide provides step-by-step instructions for publishing libsql-orm to crates.io.

## Pre-Publishing Checklist

### ✅ Required Files
- [x] `LICENSE` - MIT license file
- [x] `README.md` - Comprehensive documentation
- [x] `CHANGELOG.md` - Release notes and version history
- [x] `Cargo.toml` - Main crate metadata
- [x] `libsql-orm-macros/Cargo.toml` - Macro crate metadata
- [x] `.gitignore` - Ignore target/ and other build artifacts

### ✅ Cargo.toml Metadata
- [x] `name` - "libsql-orm"
- [x] `version` - "0.1.0"
- [x] `edition` - "2021"
- [x] `authors` - ["Ayon Saha <ayonsaha2011@gmail.com>"]
- [x] `description` - Comprehensive description
- [x] `documentation` - https://docs.rs/libsql-orm
- [x] `homepage` - GitHub repository URL
- [x] `repository` - GitHub repository URL
- [x] `license` - "MIT"
- [x] `readme` - "README.md"
- [x] `keywords` - ["orm", "libsql", "database", "sqlite", "wasm"]
- [x] `categories` - ["database", "web-programming", "wasm", "asynchronous"]

### ✅ Documentation
- [x] API documentation in source code
- [x] Examples in README.md
- [x] Usage examples in examples/ directory
- [x] Comprehensive feature documentation

## Publishing Steps

### 1. Verify Build and Tests

```bash
# Clean build
cargo clean

# Build both crates
cargo build --release
cargo build --release -p libsql-orm-macros

# Run tests (if any)
cargo test

# Check for warnings
cargo clippy -- -D warnings

# Verify documentation builds
cargo doc --no-deps
```

### 2. Check Package Contents

```bash
# Dry run to see what will be packaged
cargo package --dry-run

# Check macro crate packaging
cargo package --dry-run -p libsql-orm-macros
```

### 3. Version Management

Update versions in both `Cargo.toml` files if needed:
- `libsql-orm/Cargo.toml` - Main crate version
- `libsql-orm-macros/Cargo.toml` - Macro crate version
- Update the dependency version in main crate if macro version changes

### 4. Publish Macro Crate First

The macro crate must be published before the main crate since it's a dependency.

```bash
# Navigate to macro crate directory
cd libsql-orm-macros

# Login to crates.io (if not already logged in)
cargo login

# Publish macro crate
cargo publish

# Wait for it to be available on crates.io (usually takes a few minutes)
```

### 5. Update Main Crate Dependency

After the macro crate is published, update the main `Cargo.toml`:

```toml
[dependencies]
libsql-orm-macros = "0.1.0"  # Remove the path = "..." part
```

### 6. Publish Main Crate

```bash
# Navigate back to root directory
cd ..

# Build and test again
cargo build --release
cargo test

# Publish main crate
cargo publish
```

### 7. Post-Publishing

- Create a Git tag for the release:
  ```bash
  git tag v0.1.0
  git push origin v0.1.0
  ```

- Create a GitHub release with changelog
- Update documentation if needed
- Announce the release

## Version Update Process

For future releases:

1. Update `CHANGELOG.md` with new features/fixes
2. Bump version in both `Cargo.toml` files
3. Update any version-specific documentation
4. Follow publishing steps above
5. Create Git tag and GitHub release

## Troubleshooting

### Common Issues

**Error: "crate name is already taken"**
- Choose a different crate name in `Cargo.toml`

**Error: "macro crate not found"**
- Ensure macro crate is published first
- Wait for crates.io to update (can take 5-10 minutes)
- Remove `path = "..."` from macro dependency

**Error: "documentation generation failed"**
- Fix any doc comments that cause warnings
- Ensure all examples in docs compile

**Error: "missing required fields"**
- Verify all required metadata is present in `Cargo.toml`
- Check that LICENSE file exists

### Validation Commands

```bash
# Validate package contents
cargo package --list

# Check dependencies
cargo tree

# Validate metadata
cargo metadata --format-version 1 | jq '.packages[] | select(.name == "libsql-orm")'

# Test installation locally
cargo install --path . --force
```

## Notes

- **Publishing is irreversible** - You cannot unpublish a version from crates.io
- **Semantic versioning** - Follow SemVer for version numbers
- **Breaking changes** - Increment major version for breaking API changes
- **Documentation** - Ensure docs.rs can build documentation successfully
- **Testing** - Test on multiple platforms if possible (especially WASM)

## Support

If you encounter issues during publishing:
- Check [crates.io documentation](https://doc.rust-lang.org/cargo/reference/publishing.html)
- Review [Cargo Book](https://doc.rust-lang.org/cargo/)
- Open an issue in the repository