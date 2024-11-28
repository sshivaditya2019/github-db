#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}Setting up build environment...${NC}"

# Check for required dependencies
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}Rust not found. Please install Rust using rustup:${NC}"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Install build dependencies if on Ubuntu/Debian
if command -v apt-get &> /dev/null; then
    echo -e "\n${GREEN}Installing build dependencies...${NC}"
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev
fi

echo -e "\n${GREEN}Building binary...${NC}"

# Build the binary
cargo build --release

# Copy and make executable
cp target/release/github-db github-db-linux-x86_64
chmod +x github-db-linux-x86_64

echo -e "\n${GREEN}Testing binary...${NC}"
./github-db-linux-x86_64 --help

echo -e "\n${GREEN}Build complete!${NC}"
echo "Binary location: ./github-db-linux-x86_64"
echo "SHA256 checksum: $(sha256sum github-db-linux-x86_64 | awk '{print $1}')"
echo -e "\nTo use the binary:"
echo "chmod +x github-db-linux-x86_64"
echo "./github-db-linux-x86_64 --help"
