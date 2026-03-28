#!/usr/bin/env sh
# Clotho installer
# Usage: curl -fsSL https://raw.githubusercontent.com/colliery-io/clotho/main/scripts/install.sh | sh
# Local: scripts/install.sh --local
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
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    RED='\033[0;31m'
    NC='\033[0m'
else
    GREEN=''
    YELLOW=''
    RED=''
    NC=''
fi

info() { printf "${GREEN}[clotho]${NC} %s\n" "$1"; }
warn() { printf "${YELLOW}[clotho]${NC} %s\n" "$1"; }
error() { printf "${RED}[clotho]${NC} %s\n" "$1" >&2; exit 1; }

# Check dependencies
check_deps() {
    command -v cargo >/dev/null 2>&1 || error "Rust toolchain required. Install from https://rustup.rs"
    ensure_tmux
}

# Ensure tmux is installed
ensure_tmux() {
    if command -v tmux >/dev/null 2>&1; then
        return
    fi

    info "tmux not found — attempting to install..."

    if command -v brew >/dev/null 2>&1; then
        brew install tmux && return
    fi

    if command -v port >/dev/null 2>&1; then
        sudo port install tmux && return
    fi

    if command -v apt-get >/dev/null 2>&1; then
        sudo apt-get update && sudo apt-get install -y tmux && return
    fi

    if command -v dnf >/dev/null 2>&1; then
        sudo dnf install -y tmux && return
    fi

    if command -v pacman >/dev/null 2>&1; then
        sudo pacman -S --noconfirm tmux && return
    fi

    error "Could not install tmux. Install it manually and re-run."
}

# Install from local source
install_local() {
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
    REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

    info "Installing clotho from local source..."
    cargo install --path "$REPO_ROOT/clotho-cli" --root "$HOME/.local" --force || error "Failed to install clotho"

    info "Installing clotho-mcp from local source..."
    cargo install --path "$REPO_ROOT/clotho-mcp" --root "$HOME/.local" --force || error "Failed to install clotho-mcp"
}

# Install from GitHub
install_remote() {
    info "Installing clotho from source..."
    cargo install --git "https://github.com/$REPO" clotho-cli --root "$HOME/.local" --force || error "Failed to install clotho"

    info "Installing clotho-mcp from source..."
    cargo install --git "https://github.com/$REPO" clotho-mcp --root "$HOME/.local" --force || error "Failed to install clotho-mcp"
}

# Initialize workspace
setup_workspace() {
    WORKSPACE="$HOME/.clotho"
    if [ ! -d "$WORKSPACE" ]; then
        info "Initializing workspace at $WORKSPACE..."
        clotho init --path "$HOME" 2>/dev/null || true
    fi
}

# Install Claude Code plugin (nuke and reinstall)
install_claude_plugin() {
    if ! command -v claude >/dev/null 2>&1; then
        warn "Claude Code not found — skipping plugin install"
        warn "  Install Claude Code, then run:"
        warn "  cd ~/.clotho && claude plugin install -s project clotho@colliery-io-clotho"
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
    mkdir -p "$HOME/.clotho"
    if (cd "$HOME/.clotho" && claude plugin install -s project clotho@colliery-io-clotho); then
        info "Clotho Claude Code plugin installed"
    else
        warn "Failed to install Claude Code plugin"
        warn "  You can install manually:"
        warn "  cd ~/.clotho && claude plugin install -s project clotho@colliery-io-clotho"
    fi
}

check_deps

if [ "$LOCAL_MODE" = true ]; then
    install_local
else
    install_remote
fi

setup_workspace
install_claude_plugin

info "Done! Run 'clotho' to launch."
