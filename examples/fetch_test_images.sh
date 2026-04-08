#!/bin/bash
set -euo pipefail

# Downloads the test images needed by the Firecracker SDK examples.
#
# Images fetched:
#   examples/images/vmlinux-alpine  — stripped Linux kernel (Alpine 3.19, x86_64)
#   examples/images/alpine.ext4    — minimal Alpine Linux root filesystem

IMAGES_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/images"
mkdir -p "$IMAGES_DIR"

command_exists() { command -v "$1" > /dev/null 2>&1; }

fetch() {
    local url="$1"
    local dest="$2"
    local name
    name=$(basename "$dest")

    if [ -f "$dest" ]; then
        echo "  ✓ $name already present, skipping."
        return
    fi

    echo "  Downloading $name..."
    if command_exists curl; then
        curl -fsSL "$url" -o "$dest"
    elif command_exists wget; then
        wget -q "$url" -O "$dest"
    else
        echo "ERROR: Neither curl nor wget found." >&2
        exit 1
    fi
    echo "  ✓ $name downloaded."
}

echo "==> Fetching Firecracker example images into $IMAGES_DIR"
echo ""

# Firecracker requires an *uncompressed* ELF kernel (vmlinux).
# We use the pre-built kernel published by the Firecracker team for CI tests.
KERNEL_VERSION="6.1.102"
KERNEL_URL="https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v1.10/x86_64/vmlinux-${KERNEL_VERSION}"
fetch "$KERNEL_URL" "$IMAGES_DIR/vmlinux-alpine"

# Firecracker CI Alpine rootfs (ext4, ~40 MB).
ROOTFS_VERSION="1.10.1"
ROOTFS_URL="https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v${ROOTFS_VERSION}/x86_64/ubuntu-22.04.ext4"
fetch "$ROOTFS_URL" "$IMAGES_DIR/alpine.ext4"

echo ""
echo "==> Done! Images are in $IMAGES_DIR"
echo ""
echo "Run the Alpine example:"
echo "  cargo run --example simple_vm_alpine"
