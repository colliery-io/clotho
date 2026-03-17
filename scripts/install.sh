#!/usr/bin/env sh
# Clotho installer
# Usage: curl -fsSL https://raw.githubusercontent.com/colliery-io/clotho/main/scripts/install.sh | sh
set -e

REPO="colliery-io/clotho"

# Colors (only if terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    NC=''
fi

info() { printf "${GREEN}[clotho]${NC} %s\n" "$1"; }
warn() { printf "${YELLOW}[clotho]${NC} %s\n" "$1"; }
error() { printf "${RED}[clotho]${NC} %s\n" "$1" >&2; exit 1; }

# Detect platform
detect_platform() {
    OS=$(uname -s)
    ARCH=$(uname -m)

    case "$OS" in
        Darwin)
            case "$ARCH" in
                arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
                x86_64)        TARGET="x86_64-apple-darwin" ;;
                *)             error "Unsupported macOS architecture: $ARCH" ;;
            esac
            ;;
        Linux)
            case "$ARCH" in
                x86_64|amd64) TARGET="x86_64-unknown-linux-gnu" ;;
                *)            error "Unsupported Linux architecture: $ARCH" ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            TARGET="x86_64-pc-windows-msvc"
            ;;
        *)
            error "Unsupported operating system: $OS"
            ;;
    esac
}

# Get latest version from GitHub API
get_version() {
    if [ -n "${CLOTHO_VERSION:-}" ]; then
        VERSION="$CLOTHO_VERSION"
    else
        VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | \
            grep '"tag_name"' | sed 's/.*"v\(.*\)".*/\1/')
        if [ -z "$VERSION" ]; then
            error "Failed to determine latest version. Set CLOTHO_VERSION manually."
        fi
    fi
    info "Installing Clotho v$VERSION"
}

# Download and install
install() {
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    # Determine binary names
    case "$TARGET" in
        *windows*)
            CLI_NAME="clotho-${TARGET}.exe"
            MCP_NAME="clotho-mcp-${TARGET}.exe"
            ;;
        *)
            CLI_NAME="clotho-${TARGET}"
            MCP_NAME="clotho-mcp-${TARGET}"
            ;;
    esac

    BASE_URL="https://github.com/$REPO/releases/download/v${VERSION}"

    # Download CLI
    info "Downloading clotho..."
    curl -fsSL "$BASE_URL/$CLI_NAME" -o "$TMPDIR/clotho" || error "Failed to download clotho"

    # Download MCP server
    info "Downloading clotho-mcp..."
    curl -fsSL "$BASE_URL/$MCP_NAME" -o "$TMPDIR/clotho-mcp" || error "Failed to download clotho-mcp"

    # Verify downloads are non-empty
    [ -s "$TMPDIR/clotho" ] || error "Downloaded clotho is empty"
    [ -s "$TMPDIR/clotho-mcp" ] || error "Downloaded clotho-mcp is empty"

    # Install based on platform
    case "$OS" in
        Darwin|Linux)
            INSTALL_DIR="${CLOTHO_INSTALL_DIR:-$HOME/.local/bin}"
            mkdir -p "$INSTALL_DIR"

            chmod +x "$TMPDIR/clotho" "$TMPDIR/clotho-mcp"
            cp "$TMPDIR/clotho" "$INSTALL_DIR/clotho"
            cp "$TMPDIR/clotho-mcp" "$INSTALL_DIR/clotho-mcp"

            info "Installed to $INSTALL_DIR/"

            # Check if in PATH
            case ":$PATH:" in
                *":$INSTALL_DIR:"*) ;;
                *)
                    warn "$INSTALL_DIR is not in your PATH."
                    warn "Add this to your shell profile:"
                    warn "  export PATH=\"$INSTALL_DIR:\$PATH\""
                    ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            INSTALL_DIR="${CLOTHO_INSTALL_DIR:-$HOME/bin}"
            mkdir -p "$INSTALL_DIR"
            cp "$TMPDIR/clotho" "$INSTALL_DIR/clotho.exe"
            cp "$TMPDIR/clotho-mcp" "$INSTALL_DIR/clotho-mcp.exe"
            info "Installed to $INSTALL_DIR/"
            ;;
    esac

    info "Done! Run 'clotho --version' to verify."
}

detect_platform
get_version
install
