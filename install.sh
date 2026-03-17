#!/bin/sh
set -e

REPO="iii-hq/iii-connect"
BINARY="iii-connect"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)  PLATFORM="linux" ;;
  Darwin) PLATFORM="macos" ;;
  *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_TAG="x64" ;;
  aarch64|arm64) ARCH_TAG="arm64" ;;
  *)             echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

print_usage() {
  cat <<'USAGE'

Setup:

  Claude Desktop (~/.config/claude/claude_desktop_config.json):

    {
      "mcpServers": {
        "iii": {
          "command": "iii-connect",
          "args": ["--engine-url", "ws://localhost:49134"]
        }
      }
    }

  With A2A (headless):

    iii-connect --a2a --no-stdio

  Endpoints (on engine port 3111):

    MCP:  POST /mcp
    A2A:  POST /a2a
    Card: GET  /.well-known/agent-card.json

USAGE
}

build_from_source() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "Rust not installed. Get it from https://rustup.rs"
    exit 1
  fi

  echo "Building from source..."
  cargo install --git "https://github.com/${REPO}" ${1:+--tag "$1"}
  echo ""
  echo "Installed: $(iii-connect --version)"
  print_usage
  exit 0
}

ARCHIVE="${BINARY}-${PLATFORM}-${ARCH_TAG}.tar.gz"

echo "iii-connect installer"
echo ""

VERSION="${VERSION:-$(curl -sfL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)}"

if [ -z "$VERSION" ]; then
  echo "No release found."
  build_from_source
fi

URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

echo "Downloading ${BINARY} ${VERSION} (${PLATFORM}/${ARCH_TAG})..."

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

if ! curl -sfL "$URL" -o "${TMPDIR}/${ARCHIVE}"; then
  echo "Binary not available for ${PLATFORM}/${ARCH_TAG}."
  build_from_source "$VERSION"
fi

tar -xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"
mkdir -p "$INSTALL_DIR"
mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
chmod +x "${INSTALL_DIR}/${BINARY}"

echo "Installed to ${INSTALL_DIR}/${BINARY}"

if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo ""
  echo "Add to PATH:"
  echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

echo ""
echo "$("${INSTALL_DIR}/${BINARY}" --version)"
print_usage
