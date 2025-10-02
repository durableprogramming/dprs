#!/bin/bash

set -e

VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

echo "Building release for version $VERSION..."

# Build release binaries
cargo build --release

# Build deb package
echo "Building deb package..."
cargo install cargo-deb
cargo deb

# Build rpm package
echo "Building rpm package..."
cargo install cargo-generate-rpm
cargo generate-rpm

# Publish to crates.io
echo "Publishing to crates.io..."
cargo publish