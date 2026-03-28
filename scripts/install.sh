#!/usr/bin/env sh
# Clotho installer
# Usage (remote):  curl -fsSL https://raw.githubusercontent.com/colliery-io/clotho/main/scripts/install.sh | sh
# Usage (local):   scripts/install.sh --local
set -e

REPO="colliery-io/clotho"
LOCAL_MODE=false
for arg in "$@"; do
    case "$arg" in
        --local) LOCAL_MODE=true ;;
    esac
done

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

# Install from local build
install_local() {
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
    REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

    for binary in clotho clotho-mcp; do
        src="$REPO_ROOT/target/release/$binary"
        [ -f "$src" ] || error "$src not found. Run 'cargo build --release' first."
    done

    INSTALL_DIR="${CLOTHO_INSTALL_DIR:-$HOME/.local/bin}"
    mkdir -p "$INSTALL_DIR"

    for binary in clotho clotho-mcp; do
        cp "$REPO_ROOT/target/release/$binary" "$INSTALL_DIR/$binary"
        chmod +x "$INSTALL_DIR/$binary"
    done

    info "Installed to $INSTALL_DIR/"
    check_path
}

# Download and install from GitHub releases
install_remote() {
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    CLI_NAME="clotho-${TARGET}"
    MCP_NAME="clotho-mcp-${TARGET}"

    BASE_URL="https://github.com/$REPO/releases/download/v${VERSION}"

    info "Downloading clotho..."
    curl -fsSL "$BASE_URL/$CLI_NAME" -o "$TMPDIR/clotho" || error "Failed to download clotho"

    info "Downloading clotho-mcp..."
    curl -fsSL "$BASE_URL/$MCP_NAME" -o "$TMPDIR/clotho-mcp" || error "Failed to download clotho-mcp"

    [ -s "$TMPDIR/clotho" ] || error "Downloaded clotho is empty"
    [ -s "$TMPDIR/clotho-mcp" ] || error "Downloaded clotho-mcp is empty"

    INSTALL_DIR="${CLOTHO_INSTALL_DIR:-$HOME/.local/bin}"
    mkdir -p "$INSTALL_DIR"

    chmod +x "$TMPDIR/clotho" "$TMPDIR/clotho-mcp"
    cp "$TMPDIR/clotho" "$INSTALL_DIR/clotho"
    cp "$TMPDIR/clotho-mcp" "$INSTALL_DIR/clotho-mcp"

    info "Installed to $INSTALL_DIR/"
    check_path
}

check_path() {
    INSTALL_DIR="${CLOTHO_INSTALL_DIR:-$HOME/.local/bin}"
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            warn "$INSTALL_DIR is not in your PATH."
            warn "Add this to your shell profile:"
            warn "  export PATH=\"$INSTALL_DIR:\$PATH\""
            ;;
    esac
}

# Initialize workspace
setup_workspace() {
    INSTALL_DIR="${CLOTHO_INSTALL_DIR:-$HOME/.local/bin}"
    CLOTHO="$INSTALL_DIR/clotho"
    WORKSPACE="$HOME/.clotho"

    if [ ! -d "$WORKSPACE" ]; then
        info "Initializing workspace at $WORKSPACE..."
        "$CLOTHO" init --path "$HOME" 2>/dev/null || true
    fi
}

# Install Claude Code plugin (nuke and reinstall)
install_claude_plugin() {
    if ! command -v claude >/dev/null 2>&1; then
        warn "Claude Code not found — skipping plugin install"
        warn "  Install Claude Code, then run: claude plugin install clotho@colliery-io-clotho"
        return
    fi

    info "Installing Clotho Claude Code plugin..."

    # Nuke existing (both scopes)
    (cd "$HOME/.clotho" 2>/dev/null && claude plugin uninstall clotho@colliery-io-clotho 2>/dev/null) || true
    claude plugin uninstall clotho@colliery-io-clotho 2>/dev/null || true

    # Register/update marketplace
    claude plugin marketplace add colliery-io/clotho 2>/dev/null || true
    claude plugin marketplace update colliery-io-clotho 2>/dev/null || true

    # Install fresh as project-level plugin in ~/.clotho
    WORKSPACE="$HOME/.clotho"
    mkdir -p "$WORKSPACE"
    if (cd "$WORKSPACE" && claude plugin install -s project clotho@colliery-io-clotho); then
        info "Clotho Claude Code plugin installed"
    else
        warn "Failed to install Claude Code plugin"
        warn "  You can install manually: cd ~/.clotho && claude plugin install -s project clotho@colliery-io-clotho"
    fi
}

if [ "$LOCAL_MODE" = true ]; then
    install_local
else
    detect_platform
    get_version
    install_remote
fi
setup_workspace
install_claude_plugin

info "Done! Run 'clotho' to launch."
