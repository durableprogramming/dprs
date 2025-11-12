#!/bin/bash

set -e

echo "Building release binaries..."
cargo br

echo "Build completed. Binaries available in target/release/"
