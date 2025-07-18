#!/usr/bin/env bash
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Setting up Evo Design development environment...${NC}"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}Node.js is not installed. Please install Node.js LTS version.${NC}"
    echo -e "${YELLOW}Visit https://nodejs.org/ to download and install.${NC}"
    exit 1
fi

# Check if pnpm is installed
if ! command -v pnpm &> /dev/null; then
    echo -e "${YELLOW}pnpm is not installed. Installing pnpm...${NC}"
    npm install -g pnpm
fi

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo -e "${YELLOW}Rust is not installed. Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Check if cargo-watch is installed (for hot reloading)
if ! command -v cargo-watch &> /dev/null; then
    echo -e "${YELLOW}cargo-watch is not installed. Installing cargo-watch...${NC}"
    cargo install cargo-watch
fi

# Install dependencies
echo -e "${GREEN}Installing project dependencies...${NC}"
pnpm install

# Set up git hooks
echo -e "${GREEN}Setting up git hooks...${NC}"
pnpm prepare

# Check if SQLite is installed
if ! command -v sqlite3 &> /dev/null; then
    echo -e "${YELLOW}SQLite is not installed. Some features may not work correctly.${NC}"
    echo -e "${YELLOW}Please install SQLite manually for your operating system.${NC}"
fi

echo -e "${GREEN}Development environment setup complete!${NC}"
echo -e "${GREEN}You can now run 'pnpm dev' to start the development server.${NC}"