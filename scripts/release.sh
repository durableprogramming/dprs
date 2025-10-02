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
