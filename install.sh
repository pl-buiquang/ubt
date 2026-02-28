#!/usr/bin/env bash
set -euo pipefail

REPO="OWNER/ubt"
BIN_NAME="ubt"
INSTALL_DIR="${UBT_INSTALL_DIR:-$HOME/.local/bin}"
FROM_SOURCE=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { printf "${GREEN}[ubt-install]${NC} %s\n" "$*"; }
warn()  { printf "${YELLOW}[ubt-install]${NC} %s\n" "$*"; }
error() { printf "${RED}[ubt-install]${NC} %s\n" "$*" >&2; exit 1; }

usage() {
  cat <<EOF
Usage: install.sh [OPTIONS]

Options:
  --from-source    Clone the repo and build with cargo instead of downloading a binary
  --dir <path>     Install directory (default: ~/.local/bin)
  -h, --help       Show this help message
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --from-source) FROM_SOURCE=true; shift ;;
    --dir) INSTALL_DIR="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) error "Unknown option: $1. Run with --help for usage." ;;
  esac
done

# ── Build from source ─────────────────────────────────────────────────────────

install_from_source() {
  if ! command -v cargo &>/dev/null; then
    error "cargo not found. Install Rust via https://rustup.rs and try again."
  fi

  RUST_VERSION="$(cargo --version)"
  info "Building from source ($RUST_VERSION)..."

  TMPDIR="$(mktemp -d)"
  trap 'rm -rf "$TMPDIR"' EXIT

  if command -v git &>/dev/null; then
    info "Cloning https://github.com/$REPO..."
    git clone --depth 1 "https://github.com/$REPO" "$TMPDIR/ubt"
  else
    error "git not found. Install git and try again."
  fi

  info "Running cargo build --release..."
  (cd "$TMPDIR/ubt" && cargo build --release)

  mkdir -p "$INSTALL_DIR"
  cp "$TMPDIR/ubt/target/release/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
  chmod +x "$INSTALL_DIR/$BIN_NAME"
}

# ── Download pre-built binary ─────────────────────────────────────────────────

install_from_release() {
  # Detect OS
  OS="$(uname -s)"
  case "$OS" in
    Linux*)  OS="linux" ;;
    Darwin*) OS="darwin" ;;
    MINGW*|MSYS*|CYGWIN*)
      error "Windows is not supported by this script. Download the .zip from https://github.com/$REPO/releases"
      ;;
    *) error "Unsupported OS: $OS" ;;
  esac

  # Detect architecture
  ARCH="$(uname -m)"
  case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) error "Unsupported architecture: $ARCH" ;;
  esac

  # Map to release target triple
  if [ "$OS" = "linux" ]; then
    TARGET="${ARCH}-unknown-linux-musl"
  else
    TARGET="${ARCH}-apple-darwin"
  fi

  info "Detected platform: $OS/$ARCH (target: $TARGET)"

  # Resolve latest version via GitHub API
  info "Fetching latest release version..."
  if command -v curl &>/dev/null; then
    VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
      | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
  elif command -v wget &>/dev/null; then
    VERSION="$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" \
      | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
  else
    error "Neither curl nor wget found. Please install one of them."
  fi

  [ -n "$VERSION" ] || error "Could not determine latest release version."
  info "Latest version: $VERSION"

  ARCHIVE="${BIN_NAME}-${VERSION}-${TARGET}.tar.gz"
  DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$ARCHIVE"

  TMPDIR="$(mktemp -d)"
  trap 'rm -rf "$TMPDIR"' EXIT

  info "Downloading $ARCHIVE..."
  if command -v curl &>/dev/null; then
    curl -fsSL "$DOWNLOAD_URL" -o "$TMPDIR/$ARCHIVE"
  else
    wget -q "$DOWNLOAD_URL" -O "$TMPDIR/$ARCHIVE"
  fi

  info "Extracting..."
  tar -xzf "$TMPDIR/$ARCHIVE" -C "$TMPDIR"

  mkdir -p "$INSTALL_DIR"
  mv "$TMPDIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
  chmod +x "$INSTALL_DIR/$BIN_NAME"
}

# ── Main ──────────────────────────────────────────────────────────────────────

if $FROM_SOURCE; then
  install_from_source
else
  install_from_release
fi

info "Installed to $INSTALL_DIR/$BIN_NAME"

# PATH reminder
case ":${PATH}:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    warn "$INSTALL_DIR is not in your PATH."
    warn "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    warn "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac

# Verify
INSTALLED_VERSION="$("$INSTALL_DIR/$BIN_NAME" --version 2>&1 || true)"
info "Installation successful: $INSTALLED_VERSION"
