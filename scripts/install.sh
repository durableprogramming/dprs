#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Installing DPRS - Docker container management TUI${NC}"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Cargo is not installed. Please install Rust and Cargo first:${NC}"
    echo "https://www.rust-lang.org/tools/install"
    exit 1
fi

# Check for additional dependencies
echo "Checking dependencies..."


# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Docker is not installed. DPRS requires Docker to function.${NC}"
    echo "Please install Docker: https://docs.docker.com/get-docker/"
    exit 1
fi

# Build and install the application
echo "Building and installing DPRS..."
cargo build --release

cp ./target/release/dprs ~/.local/bin/dprs
cp ./target/release/dplw ~/.local/bin/dplw

if [ $? -eq 0 ]; then
    echo -e "${GREEN}DPRS installed successfully!${NC}"
    echo "Run 'dprs' to start the application."
else
    echo -e "${RED}Installation failed.${NC}"
    exit 1
fi

