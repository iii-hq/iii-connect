#!/bin/bash
set -e

REPO="MotiaDev/iii-mcp"
BINARY_NAME="iii-mcp"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

detect_platform() {
    local os arch
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux)
            case "$arch" in
                x86_64|amd64) echo "linux-x64" ;;
                aarch64|arm64) echo "linux-arm64" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        darwin)
            case "$arch" in
                x86_64|amd64) echo "macos-x64" ;;
                aarch64|arm64) echo "macos-arm64" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        *)
            echo "Unsupported OS: $os" >&2
            exit 1
            ;;
    esac
}

get_latest_version() {
    curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

main() {
    echo "Installing $BINARY_NAME..."

    local platform version archive_name download_url tmp_dir

    platform=$(detect_platform)
    version="${VERSION:-$(get_latest_version)}"

    if [ -z "$version" ]; then
        echo "Error: Could not determine latest version. No releases available yet." >&2
        echo "Try installing from source: cargo install --git https://github.com/$REPO" >&2
        exit 1
    fi

    archive_name="$BINARY_NAME-$platform.tar.gz"
    download_url="https://github.com/$REPO/releases/download/$version/$archive_name"

    echo "Platform: $platform"
    echo "Version: $version"
    echo "Download URL: $download_url"

    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    echo "Downloading..."
    if ! curl -fsSL "$download_url" -o "$tmp_dir/$archive_name"; then
        echo "Error: Failed to download binary. Release may not exist yet." >&2
        echo "Try installing from source: cargo install --git https://github.com/$REPO" >&2
        exit 1
    fi

    echo "Extracting..."
    tar -xzf "$tmp_dir/$archive_name" -C "$tmp_dir"

    echo "Installing to $INSTALL_DIR..."
    mkdir -p "$INSTALL_DIR"
    mv "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    echo ""
    echo "Successfully installed $BINARY_NAME to $INSTALL_DIR/$BINARY_NAME"
    echo ""

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo "Add the following to your shell profile to add $INSTALL_DIR to PATH:"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
}

main "$@"
