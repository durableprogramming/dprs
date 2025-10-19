#!/bin/bash

set -e


VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Create tar.gz
echo "Creating tar.gz package..."
mkdir -p release
cp target/release/dprs release/
cp target/release/dplw release/
tar -czf dprs-$VERSION.tar.gz -C release .

echo "Release packages created:"
echo "- dprs-$VERSION.tar.gz"
echo "- target/debian/*.deb"
echo "- target/generate-rpm/*.rpm"
echo "- Published to crates.io"

# Create GitHub release
echo ""
echo "Creating GitHub release v$VERSION..."
gh release create "v$VERSION" \
  --title "v$VERSION" \
  --generate-notes \
  dprs-$VERSION.tar.gz \
  target/debian/*.deb \
  target/generate-rpm/*.rpm

echo ""
echo "GitHub release v$VERSION created successfully!"
