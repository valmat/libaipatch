#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$PROJECT_ROOT/rust"
PKG_DIR="$SCRIPT_DIR/pkg"

PACKAGE_NAME="libaipatch-dev"
ARCHITECTURE="$(dpkg --print-architecture)"

version_from_cargo() {
    sed -n 's/^version = "\([^"]\+\)"$/\1/p' "$RUST_DIR/Cargo.toml" | head -n1
}

VERSION="$(version_from_cargo)"

echo "Building libaipatch release artifacts..."
cargo build --manifest-path "$RUST_DIR/Cargo.toml" --release

rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/lib"
mkdir -p "$PKG_DIR/usr/include"
mkdir -p "$PKG_DIR/usr/share/doc/$PACKAGE_NAME"

cp "$RUST_DIR/target/release/libaipatch.a" "$PKG_DIR/usr/lib/libaipatch.a"
cp "$PROJECT_ROOT/include/aipatch.h" "$PKG_DIR/usr/include/aipatch.h"
cp "$PROJECT_ROOT/README.md" "$PKG_DIR/usr/share/doc/$PACKAGE_NAME/README.md"
cp "$PROJECT_ROOT/LICENSE" "$PKG_DIR/usr/share/doc/$PACKAGE_NAME/copyright"

gzip -n -f "$PKG_DIR/usr/share/doc/$PACKAGE_NAME/README.md"

INSTALLED_SIZE="$(du -sk "$PKG_DIR" | cut -f1)"

sed \
    -e "s/%PACKAGE_NAME%/$PACKAGE_NAME/g" \
    -e "s/%VERSION%/$VERSION/g" \
    -e "s/%ARCHITECTURE%/$ARCHITECTURE/g" \
    -e "s/%INSTALLED_SIZE%/$INSTALLED_SIZE/g" \
    "$SCRIPT_DIR/control" > "$PKG_DIR/DEBIAN/control"

OUTPUT_DEB="$SCRIPT_DIR/${PACKAGE_NAME}_${VERSION}_${ARCHITECTURE}.deb"
rm -f "$OUTPUT_DEB"

fakeroot dpkg-deb --build "$PKG_DIR" "$OUTPUT_DEB"

echo
echo "Built package: $OUTPUT_DEB"
