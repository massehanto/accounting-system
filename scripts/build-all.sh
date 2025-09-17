#!/bin/bash

# Build all services for Indonesian Accounting System

echo "Building Indonesian Accounting System..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build workspace
echo -e "${YELLOW}Building workspace...${NC}"
if cargo build --workspace --release; then
    echo -e "${GREEN}✓ Workspace build successful${NC}"
else
    echo -e "${RED}✗ Workspace build failed${NC}"
    exit 1
fi

# Test all services
echo -e "${YELLOW}Running tests...${NC}"
if cargo test --workspace; then
    echo -e "${GREEN}✓ All tests passed${NC}"
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi

# Check formatting
echo -e "${YELLOW}Checking code formatting...${NC}"
if cargo fmt --all -- --check; then
    echo -e "${GREEN}✓ Code is properly formatted${NC}"
else
    echo -e "${RED}✗ Code formatting issues found${NC}"
    echo "Run 'cargo fmt --all' to fix formatting"
    exit 1
fi

# Run clippy
echo -e "${YELLOW}Running clippy...${NC}"
if cargo clippy --workspace --all-targets -- -D warnings; then
    echo -e "${GREEN}✓ No clippy warnings${NC}"
else
    echo -e "${RED}✗ Clippy warnings found${NC}"
    exit 1
fi

echo -e "${GREEN}All builds and checks completed successfully!${NC}"