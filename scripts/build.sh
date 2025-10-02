#!/bin/bash

set -e

echo "Building release binaries..."
cargo build --release

echo "Build completed. Binaries available in target/release/"