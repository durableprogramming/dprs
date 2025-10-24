#!/bin/bash

set -e


VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Build Debian package
echo "Building Debian package..."
cargo install cargo-deb 2>/dev/null || true
cargo deb

# Build RPM package
echo "Building RPM package..."
cargo install cargo-generate-rpm 2>/dev/null || true
cargo build --release
cargo generate-rpm

# Create tar.gz
echo "Creating tar.gz package..."
mkdir -p release
cp target/release/dprs release/
cp target/release/dplw release/
tar -czf dprs-$VERSION.tar.gz -C release .

echo "Release packages created:"
echo "- dprs-$VERSION.tar.gz"

# Collect all release artifacts
RELEASE_FILES=("dprs-$VERSION.tar.gz")

if ls target/debian/*.deb 1> /dev/null 2>&1; then
  RELEASE_FILES+=(target/debian/*.deb)
  echo "- target/debian/*.deb"
fi

if ls target/generate-rpm/*.rpm 1> /dev/null 2>&1; then
  RELEASE_FILES+=(target/generate-rpm/*.rpm)
  echo "- target/generate-rpm/*.rpm"
fi

echo "- Published to crates.io"

# Create GitHub release
echo ""
echo "Creating GitHub release v$VERSION..."
gh release create "v$VERSION" \
  --title "v$VERSION" \
  --generate-notes \
  "${RELEASE_FILES[@]}"

echo ""
echo "GitHub release v$VERSION created successfully!"
