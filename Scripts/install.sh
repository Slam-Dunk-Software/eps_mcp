#!/usr/bin/env bash
set -e

REPO="Slam-Dunk-Software/eps_mcp"
BINARY="eps_mcp"
VERSION="${EPM_PACKAGE_VERSION}"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}-${ARCH}" in
  Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
  Darwin-x86_64) TARGET="x86_64-apple-darwin" ;;
  Linux-x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
  *)
    echo "Unsupported platform: ${OS}-${ARCH}"
    exit 1
    ;;
esac

URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY}-${TARGET}.tar.gz"

echo "Downloading ${BINARY} v${VERSION} for ${TARGET}..."

mkdir -p target/release
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$URL" -o "$TMP/${BINARY}.tar.gz"
tar -xzf "$TMP/${BINARY}.tar.gz" -C "$TMP"
mv "$TMP/${BINARY}" "target/release/${BINARY}"
chmod +x "target/release/${BINARY}"

echo "Done."
