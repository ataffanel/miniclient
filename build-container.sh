#!/bin/bash
set -e

CONTAINER_ENGINE="${CONTAINER_ENGINE:-podman}"

echo "Building container image..."
$CONTAINER_ENGINE build -t miniclient-builder -f Containerfile .

echo "Building AppImage..."
$CONTAINER_ENGINE run --rm -v "$PWD:/src:Z" miniclient-builder \
    cargo appimage

echo "Done!"
ls -lh target/appimage/*.AppImage 2>/dev/null
