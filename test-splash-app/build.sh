#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PYAPP_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building PyApp with splash screen enabled..."
echo

# Project configuration — uses cowsay as a small real PyPI package
export PYAPP_PROJECT_NAME="cowsay"
export PYAPP_PROJECT_VERSION="6.1"
export PYAPP_EXEC_MODULE="cowsay"

# Splash screen configuration
export PYAPP_SPLASH_ENABLED="true"
export PYAPP_SPLASH_THEME="dark"
# export PYAPP_SPLASH_IMAGE="/path/to/logo.png"  # Set to a PNG/JPEG to test logo display

echo "Configuration:"
echo "  Project:  $PYAPP_PROJECT_NAME v$PYAPP_PROJECT_VERSION (from PyPI)"
echo "  Theme:    $PYAPP_SPLASH_THEME"
echo "  Image:    ${PYAPP_SPLASH_IMAGE:-none (text fallback)}"
echo

cd "$PYAPP_DIR"
cargo build --release --features splash

BINARY="$PYAPP_DIR/target/release/pyapp"
echo
echo "Build complete!"
echo "Binary: $BINARY"
echo
echo "To test the splash screen:"
echo "  1. First run (shows splash):  $BINARY Hello splash screen!"
echo "  2. Second run (no splash):    $BINARY Moo"
echo "  3. Re-test (shows splash):    $BINARY self restore"
echo
echo "To test light theme, re-run this script with:"
echo "  PYAPP_SPLASH_THEME=light ./build.sh"
