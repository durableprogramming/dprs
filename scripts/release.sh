#!/bin/bash

set -e


VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Build static binaries with musl
echo "Building static binaries with musl..."
cargo build --release --target x86_64-unknown-linux-musl

# Build Debian package
echo "Building Debian package..."
cargo install cargo-deb 2>/dev/null || true
cargo deb --target x86_64-unknown-linux-musl --no-build

# Build RPM package
echo "Building RPM package..."
cargo install cargo-generate-rpm 2>/dev/null || true
cargo generate-rpm --target x86_64-unknown-linux-musl

# Create tar.gz
echo "Creating tar.gz package..."
mkdir -p release
cp target/x86_64-unknown-linux-musl/release/dprs release/
cp target/x86_64-unknown-linux-musl/release/dplw release/
tar -czf dprs-$VERSION.tar.gz -C release .

echo "Release packages created:"
echo "- dprs-$VERSION.tar.gz"

# Collect all release artifacts
RELEASE_FILES=("dprs-$VERSION.tar.gz")

if ls target/x86_64-unknown-linux-musl/debian/*.deb 1> /dev/null 2>&1; then
  RELEASE_FILES+=(target/x86_64-unknown-linux-musl/debian/*.deb)
  echo "- target/x86_64-unknown-linux-musl/debian/*.deb"
fi

if ls target/x86_64-unknown-linux-musl/generate-rpm/*.rpm 1> /dev/null 2>&1; then
  RELEASE_FILES+=(target/x86_64-unknown-linux-musl/generate-rpm/*.rpm)
  echo "- target/x86_64-unknown-linux-musl/generate-rpm/*.rpm"
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
