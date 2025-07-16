#!/bin/bash

# Script to prepare a new release by bumping versions in Cargo.toml files

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to update version in Cargo.toml
update_version() {
    local file=$1
    local new_version=$2
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$file"
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$file"
    fi
}

# Function to update dependency version
update_dependency_version() {
    local file=$1
    local dep_name=$2
    local new_version=$3
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/$dep_name = { version = \".*\"/$dep_name = { version = \"$new_version\"/" "$file"
    else
        # Linux
        sed -i "s/$dep_name = { version = \".*\"/$dep_name = { version = \"$new_version\"/" "$file"
    fi
}

# Check if version argument is provided
if [ $# -eq 0 ]; then
    print_error "Usage: $0 <new_version>"
    print_info "Example: $0 0.1.1"
    exit 1
fi

NEW_VERSION=$1

# Validate version format (simple check)
if [[ ! $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_error "Invalid version format. Expected format: X.Y.Z (e.g., 0.1.1)"
    exit 1
fi

print_info "Preparing release version $NEW_VERSION..."

# Get current versions
CURRENT_MAIN_VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
CURRENT_MACRO_VERSION=$(grep '^version = ' libsql-orm-macros/Cargo.toml | cut -d'"' -f2)

print_info "Current versions:"
print_info "  Main crate: $CURRENT_MAIN_VERSION"
print_info "  Macro crate: $CURRENT_MACRO_VERSION"

# Check if versions are already the same
if [ "$CURRENT_MAIN_VERSION" != "$CURRENT_MACRO_VERSION" ]; then
    print_warning "Main and macro crate versions are different!"
    print_warning "This might cause issues during release."
fi

# Update versions
print_info "Updating versions..."

update_version "Cargo.toml" "$NEW_VERSION"
update_version "libsql-orm-macros/Cargo.toml" "$NEW_VERSION"

# Update the dependency version in main crate
update_dependency_version "Cargo.toml" "libsql-orm-macros" "$NEW_VERSION"

print_success "Versions updated successfully!"

# Verify the changes
print_info "Verifying changes..."

NEW_MAIN_VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
NEW_MACRO_VERSION=$(grep '^version = ' libsql-orm-macros/Cargo.toml | cut -d'"' -f2)
NEW_DEP_VERSION=$(grep 'libsql-orm-macros = { version = ' Cargo.toml | cut -d'"' -f2)

print_info "New versions:"
print_info "  Main crate: $NEW_MAIN_VERSION"
print_info "  Macro crate: $NEW_MACRO_VERSION"
print_info "  Dependency version: $NEW_DEP_VERSION"

# Check if all versions match
if [ "$NEW_MAIN_VERSION" = "$NEW_VERSION" ] && [ "$NEW_MACRO_VERSION" = "$NEW_VERSION" ] && [ "$NEW_DEP_VERSION" = "$NEW_VERSION" ]; then
    print_success "All versions are consistent!"
else
    print_error "Version mismatch detected!"
    exit 1
fi

# Run basic checks
print_info "Running basic checks..."

if command -v cargo &> /dev/null; then
    print_info "Checking if project builds..."
    if cargo check --quiet; then
        print_success "Project builds successfully!"
    else
        print_error "Project build failed!"
        exit 1
    fi
else
    print_warning "cargo not found, skipping build check"
fi

print_info "Release preparation completed!"
print_info ""
print_info "Next steps:"
print_info "1. Review the changes:"
print_info "   git diff"
print_info ""
print_info "2. Commit the changes:"
print_info "   git add ."
print_info "   git commit -m \"Bump version to $NEW_VERSION\""
print_info ""
print_info "3. Create and push a tag:"
print_info "   git tag v$NEW_VERSION"
print_info "   git push origin v$NEW_VERSION"
print_info ""
print_info "4. Or trigger manual release workflow in GitHub Actions" 