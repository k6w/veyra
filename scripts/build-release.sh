#!/bin/bash

# Veyra Release Script
# This script helps create a local release build for testing

set -e

VERSION=${1:-"0.1.0-local"}
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "$ARCH" = "x86_64" ]; then
    ARCH="x64"
elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
    ARCH="arm64"
fi

ARTIFACT_NAME="veyra-${PLATFORM}-${ARCH}"
RELEASE_DIR="release"

echo "Building Veyra v${VERSION} for ${PLATFORM}-${ARCH}"

# Clean previous builds
rm -rf "${RELEASE_DIR}"
mkdir -p "${RELEASE_DIR}/bin"
mkdir -p "${RELEASE_DIR}/stdlib"
mkdir -p "${RELEASE_DIR}/examples"

# Build compiler
echo "Building compiler..."
cd compiler
cargo build --release
cd ..

# Build tools
echo "Building tools..."
cd tools
cargo build --release
cd ..

# Copy binaries
echo "Copying binaries..."
cp compiler/target/release/veyc "${RELEASE_DIR}/bin/"
cp tools/target/release/veyra-repl "${RELEASE_DIR}/bin/"
cp tools/target/release/veyra-dbg "${RELEASE_DIR}/bin/"
cp tools/target/release/veyra-lint "${RELEASE_DIR}/bin/"
cp tools/target/release/veyra-fmt "${RELEASE_DIR}/bin/"
cp tools/target/release/veyra-lsp "${RELEASE_DIR}/bin/"
cp tools/target/release/veyra-pkg "${RELEASE_DIR}/bin/"

# Copy standard library and examples
echo "Copying standard library and examples..."
cp -r stdlib/* "${RELEASE_DIR}/stdlib/"
cp -r examples/* "${RELEASE_DIR}/examples/"

# Copy documentation
echo "Copying documentation..."
cp README.md "${RELEASE_DIR}/"
cp LICENSE "${RELEASE_DIR}/"
cp QUICK_START.md "${RELEASE_DIR}/"

# Create archive
echo "Creating archive..."
cd "${RELEASE_DIR}"
tar czf "../${ARTIFACT_NAME}.tar.gz" *
cd ..

echo "Release build completed: ${ARTIFACT_NAME}.tar.gz"
echo ""
echo "Contents:"
echo "- Compiler: veyc"
echo "- REPL: veyra-repl"
echo "- Debugger: veyra-dbg"
echo "- Linter: veyra-lint"
echo "- Formatter: veyra-fmt"
echo "- Language Server: veyra-lsp"
echo "- Package Manager: veyra-pkg"
echo "- Standard Library"
echo "- Examples"
echo "- Documentation"
echo ""
echo "To install:"
echo "1. Extract the archive: tar xzf ${ARTIFACT_NAME}.tar.gz"
echo "2. Add the bin directory to your PATH"