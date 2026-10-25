#!/usr/bin/env bash
set -e

# ANSI Color codes
BOLD="\033[1m"
GREEN="\033[32m"
BLUE="\033[34m"
RED="\033[31m"
RESET="\033[0m"

echo -e "${BLUE}${BOLD}💾 diskbar installer${RESET}"
echo "-------------------------------------"

# Check if cargo is installed
if command -v cargo >/dev/null 2>&1; then
    echo -e "🚀 Rust toolchain detected. Compiling and installing ${BOLD}diskbar${RESET}..."
    cargo install --git https://github.com/parameters-tools/diskbar
    
    echo -e "\n${GREEN}✅ diskbar successfully installed!${RESET}"
    echo -e "Binary placed at: ${BOLD}$(command -v diskbar || echo "$HOME/.cargo/bin/diskbar")${RESET}"
    echo -e "Run ${BOLD}diskbar${RESET} to check your storage!"
    exit 0
fi

echo -e "${RED}❌ Rust/Cargo not found.${RESET}"
echo "Please install Rust using: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo "Then re-run this script."
exit 1
