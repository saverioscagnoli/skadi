#!/usr/bin/env bash

# Development helper script for wwwidgets
# This script provides convenient commands for common development tasks

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_help() {
    cat << EOF
${BLUE}wwwidgets Development Helper${NC}

Usage: ./dev.sh [command]

Commands:
    ${GREEN}run${NC}         Run wwwidgets in development mode (with hot reload)
    ${GREEN}build${NC}       Build wwwidgets in release mode
    ${GREEN}clean${NC}       Clean build cache and Cargo artifacts
    ${GREEN}clean-cache${NC} Clean only the build cache (~/.local/share/wwwidgets)
    ${GREEN}install${NC}     Build and install to ~/.local/bin
    ${GREEN}config${NC}      Open config file in \$EDITOR
    ${GREEN}logs${NC}        Show the build cache directory contents
    ${GREEN}help${NC}        Show this help message

Examples:
    ${YELLOW}./dev.sh run${NC}          # Run with hot reload
    ${YELLOW}./dev.sh clean && ./dev.sh run${NC}  # Clean cache and run
    ${YELLOW}./dev.sh install${NC}      # Build and install

EOF
}

run_dev() {
    echo -e "${BLUE}==> Running wwwidgets in development mode...${NC}"
    cargo run --release -- --dev
}

build_release() {
    echo -e "${BLUE}==> Building wwwidgets in release mode...${NC}"
    cargo build --release
    echo -e "${GREEN}✓ Build complete!${NC}"
    echo -e "Binary location: ${YELLOW}$SCRIPT_DIR/target/release/wwwidgets${NC}"
}

clean_all() {
    echo -e "${BLUE}==> Cleaning Cargo artifacts...${NC}"
    cargo clean

    echo -e "${BLUE}==> Cleaning build cache...${NC}"
    if [ -d "$HOME/.local/share/wwwidgets" ]; then
        rm -rf "$HOME/.local/share/wwwidgets"
        echo -e "${GREEN}✓ Build cache cleaned${NC}"
    else
        echo -e "${YELLOW}Build cache directory doesn't exist${NC}"
    fi
}

clean_cache() {
    echo -e "${BLUE}==> Cleaning build cache at ~/.local/share/wwwidgets${NC}"

    if [ -d "$HOME/.local/share/wwwidgets" ]; then
        echo -e "${YELLOW}Contents before cleaning:${NC}"
        ls -lah "$HOME/.local/share/wwwidgets/"
        echo ""

        rm -rf "$HOME/.local/share/wwwidgets"
        mkdir -p "$HOME/.local/share/wwwidgets"

        echo -e "${GREEN}✓ Build cache cleaned${NC}"
    else
        echo -e "${YELLOW}Build cache directory doesn't exist, nothing to clean${NC}"
    fi
}

install_binary() {
    echo -e "${BLUE}==> Building wwwidgets...${NC}"
    cargo build --release

    local install_dir="$HOME/.local/bin"
    mkdir -p "$install_dir"

    echo -e "${BLUE}==> Installing to $install_dir${NC}"
    cp "$SCRIPT_DIR/target/release/wwwidgets" "$install_dir/wwwidgets"
    cp "$SCRIPT_DIR/target/release/wwwatch" "$install_dir/wwwatch"

    echo -e "${GREEN}✓ Installation complete!${NC}"
    echo -e "Make sure ${YELLOW}$install_dir${NC} is in your PATH"
}

open_config() {
    local config_file="$HOME/.config/wwwidgets/config.json"
    local editor="${EDITOR:-nano}"

    if [ ! -f "$config_file" ]; then
        echo -e "${YELLOW}Config file doesn't exist yet. It will be created on first run.${NC}"
        echo -e "Template location: ${BLUE}$SCRIPT_DIR/templates/config.default.json${NC}"
        return 1
    fi

    echo -e "${BLUE}==> Opening config file with $editor...${NC}"
    "$editor" "$config_file"
}

show_logs() {
    local build_dir="$HOME/.local/share/wwwidgets"

    if [ ! -d "$build_dir" ]; then
        echo -e "${YELLOW}Build cache directory doesn't exist yet${NC}"
        return 1
    fi

    echo -e "${BLUE}==> Build cache directory: $build_dir${NC}"
    echo ""

    tree "$build_dir" 2>/dev/null || ls -lah "$build_dir"

    echo ""
    echo -e "${BLUE}==> Config file:${NC}"
    if [ -f "$HOME/.config/wwwidgets/config.json" ]; then
        cat "$HOME/.config/wwwidgets/config.json"
    else
        echo -e "${YELLOW}Config file doesn't exist yet${NC}"
    fi
}

# Main command dispatcher
case "${1:-help}" in
    run)
        run_dev
        ;;
    build)
        build_release
        ;;
    clean)
        clean_all
        ;;
    clean-cache)
        clean_cache
        ;;
    install)
        install_binary
        ;;
    config)
        open_config
        ;;
    logs)
        show_logs
        ;;
    help|--help|-h)
        print_help
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        echo ""
        print_help
        exit 1
        ;;
esac
