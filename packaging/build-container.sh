#!/bin/bash
set -e

CONTAINER_ENGINE="${CONTAINER_ENGINE:-podman}"

echo "Building container image..."
$CONTAINER_ENGINE build -t miniclient-builder -f packaging/Containerfile .

echo "Building AppImage..."
$CONTAINER_ENGINE run --rm -v "$PWD:/src:Z" miniclient-builder bash -c "
    cargo build --release &&
    cp icon.png miniclient.png &&
    QMAKE=qmake linuxdeploy \
        --appdir AppDir \
        --executable target/release/miniclient \
        --desktop-file packaging/miniclient.desktop \
        --icon-file miniclient.png \
        --plugin qt \
        --output appimage
"

echo "Done!"
ls -lh *.AppImage 2>/dev/null
