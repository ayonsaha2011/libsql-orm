#!/bin/bash

# Pre-publish verification script for libsql-orm
# Run this before publishing to crates.io

set -e

echo "🔍 Pre-publish verification for libsql-orm"
echo "========================================="

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "libsql-orm-macros" ]]; then
    echo "❌ Error: Must be run from the libsql-orm root directory"
    exit 1
fi

echo "✅ In correct directory"

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Check Rust formatting
echo "📏 Checking code formatting..."
cargo fmt --all -- --check || {
    echo "❌ Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
}
echo "✅ Code formatting is correct"

# Run clippy
echo "📎 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings || {
    echo "❌ Clippy found issues. Please fix them."
    exit 1
}
echo "✅ Clippy checks passed"

# Build workspace
echo "🔨 Building workspace..."
cargo build --workspace --release || {
    echo "❌ Build failed"
    exit 1
}
echo "✅ Workspace builds successfully"

# Check WASM target
echo "🌐 Checking WASM build..."
cargo check --target wasm32-unknown-unknown || {
    echo "❌ WASM build failed"
    exit 1
}
echo "✅ WASM build successful"

# Generate documentation
echo "📚 Generating documentation..."
cargo doc --no-deps --workspace || {
    echo "❌ Documentation generation failed"
    exit 1
}
echo "✅ Documentation generated successfully"

# Test packaging (dry run)
echo "📦 Testing package creation..."
echo "  - Testing macro crate packaging..."
(cd libsql-orm-macros && cargo package --allow-dirty > /dev/null) || {
    echo "❌ Macro crate packaging failed"
    exit 1
}
echo "✅ Macro crate packaging test passed"

echo "  - Testing main crate structure (build only)..."
# Note: Main crate packaging will fail until macro crate is published to crates.io
# So we test the build instead to verify everything is correct
cargo build --release > /dev/null || {
    echo "❌ Main crate build failed"
    exit 1
}
echo "✅ Main crate structure test passed"

echo "ℹ️  Note: Main crate packaging will succeed only after macro crate is published to crates.io"

# Check required files
echo "📄 Checking required files..."
required_files=("LICENSE" "README.md" "CHANGELOG.md" "Cargo.toml")
for file in "${required_files[@]}"; do
    if [[ ! -f "$file" ]]; then
        echo "❌ Missing required file: $file"
        exit 1
    fi
done
echo "✅ All required files present"

# Check Cargo.toml metadata
echo "🔍 Checking Cargo.toml metadata..."
check_field() {
    local field=$1
    local file=$2
    if ! grep -q "^${field} = " "$file"; then
        echo "❌ Missing field '$field' in $file"
        exit 1
    fi
}

check_field "name" "Cargo.toml"
check_field "version" "Cargo.toml"
check_field "description" "Cargo.toml"
check_field "license" "Cargo.toml"
check_field "repository" "Cargo.toml"
check_field "keywords" "Cargo.toml"
check_field "categories" "Cargo.toml"

check_field "name" "libsql-orm-macros/Cargo.toml"
check_field "version" "libsql-orm-macros/Cargo.toml"
check_field "description" "libsql-orm-macros/Cargo.toml"
check_field "license" "libsql-orm-macros/Cargo.toml"

echo "✅ Cargo.toml metadata is complete"

echo ""
echo "🎉 Pre-publish verification completed successfully!"
echo ""
echo "📋 Publishing checklist:"
echo "  1. Commit all changes to git"
echo "  2. Create and push a version tag"
echo "  3. Publish macro crate first: (cd libsql-orm-macros && cargo publish --allow-dirty)"
echo "  4. Wait for macro crate to be available on crates.io (usually 2-5 minutes)"
echo "  5. Verify macro crate is available: cargo search libsql-orm-macros"
echo "  6. Publish main crate: cargo publish --allow-dirty"
echo "  7. Create GitHub release"
echo ""
echo "⚠️  Important: The macro crate MUST be published and available on crates.io"
echo "    before the main crate can be packaged/published successfully."
echo ""
echo "🔗 Useful commands:"
echo "  cargo login                    # Login to crates.io"
echo "  cargo package --list         # See what files will be included"
echo "  cargo publish --dry-run      # Test publishing without actually publishing"
echo ""