#!/usr/bin/env bash
set -euo pipefail

REPO="foksiny/discordscript"
BINARY="discordscript"
DEFAULT_INSTALL_DIR="/usr/local/bin"

print_banner() {
    cat <<'EOF'
  ____  _           ____                      _       _
 |  _ \(_)___  ___ / ___|  ___ _ __ ___  ___(_)_ __ | |_
 | | | | / __|/ __|\___ \ / __| '__/ _ \/ __| | '_ \| __|
 | |_| | \__ \ (__  ___) | (__| | |  __/ (__| | |_) | |_
 |____/|_|___/\___||____/ \___|_|  \___|\___|_| .__/ \__|
                                              |_|
EOF
    echo "DiscordScript Installer"
    echo "======================="
    echo ""
}

detect_platform() {
    local os
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    local arch
    arch="$(uname -m)"

    case "$os" in
        linux) os="linux" ;;
        darwin) os="darwin" ;;
        mingw*|msys*|cygwin*) os="windows" ;;
        *)
            echo "Unsupported OS: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            echo "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    echo "${arch}-${os}"
}

install_via_cargo() {
    echo "Installing via cargo..."
    if ! command -v cargo &>/dev/null; then
        echo "Error: cargo is not installed. Install Rust at https://rustup.rs/"
        exit 1
    fi
    cargo install --git "https://github.com/${REPO}" --force
    echo ""
    echo "Installed successfully! Run '${BINARY} --help' to get started."
}

install_via_download() {
    local platform="$1"
    local install_dir="$2"
    local release_url="https://github.com/${REPO}/releases/latest/download/${BINARY}-${platform}.tar.gz"
    local tmp_dir
    tmp_dir="$(mktemp -d)"

    echo "Downloading ${BINARY} for ${platform}..."
    if command -v curl &>/dev/null; then
        curl -fsSL "$release_url" -o "${tmp_dir}/${BINARY}.tar.gz"
    elif command -v wget &>/dev/null; then
        wget -q "$release_url" -O "${tmp_dir}/${BINARY}.tar.gz"
    else
        echo "Error: neither curl nor wget found. Install via cargo instead."
        exit 1
    fi

    echo "Extracting..."
    tar -xzf "${tmp_dir}/${BINARY}.tar.gz" -C "$tmp_dir"

    echo "Installing to ${install_dir}..."
    if [ ! -w "$install_dir" ]; then
        echo "No write permission for ${install_dir}, using sudo..."
        sudo mv "${tmp_dir}/${BINARY}" "${install_dir}/${BINARY}"
        sudo chmod +x "${install_dir}/${BINARY}"
    else
        mv "${tmp_dir}/${BINARY}" "${install_dir}/${BINARY}"
        chmod +x "${install_dir}/${BINARY}"
    fi

    rm -rf "$tmp_dir"
    echo ""
    echo "Installed to ${install_dir}/${BINARY}"
    echo "Run '${BINARY} --help' to get started."
}

main() {
    print_banner

    local install_dir="${DEFAULT_INSTALL_DIR}"

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --dir|--prefix)
                install_dir="$2"
                shift 2
                ;;
            --local)
                install_dir="$HOME/.local/bin"
                mkdir -p "$install_dir"
                shift
                ;;
            --help|-h)
                echo "Usage: curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | bash"
                echo "   or: curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | bash -s -- --local"
                echo "   or: cargo install --git https://github.com/${REPO}"
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    echo "This script will install ${BINARY} globally."
    echo ""

    # Prefer cargo install as it's the most reliable for Rust projects
    if command -v cargo &>/dev/null; then
        install_via_cargo
    else
        echo "Cargo not found. Trying pre-built binary download..."
        local platform
        platform="$(detect_platform)"
        install_via_download "$platform" "$install_dir"
    fi

    # Add to PATH reminder
    if [[ ":$PATH:" != *":${install_dir}:"* ]]; then
        echo ""
        echo "NOTE: ${install_dir} is not in your PATH."
        echo "Add it by running:"
        echo "  export PATH=\"${install_dir}:\$PATH\""
        case "$SHELL" in
            */zsh) echo "  Or add to ~/.zshrc" ;;
            */bash) echo "  Or add to ~/.bashrc" ;;
        esac
    fi
}

main "$@"
