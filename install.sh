#!/usr/bin/env bash
# Install cosmic-ext-calculator — from the latest GitHub release by default,
# or from a local build with --local.
#
# Usage:
#   Install from latest release (per-user, ~/.local):
#     curl -fsSL https://raw.githubusercontent.com/dcristob/cosmic-ext-calculator/main/install.sh | bash
#   System-wide (all users, /usr):
#     curl -fsSL https://raw.githubusercontent.com/dcristob/cosmic-ext-calculator/main/install.sh | sudo bash
#   From a local cargo build (run inside the repo):
#     ./install.sh --local
#     sudo ./install.sh --local     # system-wide

set -euo pipefail

REPO="dcristob/cosmic-ext-calculator"
APP_ID="dev.dcristob.Calculator"
BIN="cosmic-ext-calculator"
BASE_URL="https://github.com/${REPO}/releases/latest/download"

LOCAL=0
for arg in "$@"; do
    case "$arg" in
        --local) LOCAL=1 ;;
        -h|--help)
            sed -n '2,13p' "$0"
            exit 0
            ;;
        *) echo "Unknown argument: $arg" >&2; exit 2 ;;
    esac
done

if [ "$(id -u)" -eq 0 ]; then
    PREFIX="/usr"
else
    PREFIX="$HOME/.local"
fi

BIN_DIR="${PREFIX}/bin"
APP_DIR="${PREFIX}/share/applications"
METAINFO_DIR="${PREFIX}/share/metainfo"
ICON_DIR="${PREFIX}/share/icons/hicolor/scalable/apps"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

if [ "$LOCAL" -eq 1 ]; then
    echo "Installing from local build to ${PREFIX}"
    if [ ! -x "target/release/${BIN}" ]; then
        echo "target/release/${BIN} not found. Run 'cargo build --release' first." >&2
        exit 1
    fi
    cp "target/release/${BIN}"                               "${tmp}/${BIN}"
    cp "res/${APP_ID}.desktop"                               "${tmp}/${APP_ID}.desktop"
    cp "res/${APP_ID}.metainfo.xml"                          "${tmp}/${APP_ID}.metainfo.xml"
    cp "res/icons/hicolor/scalable/apps/${APP_ID}.svg"       "${tmp}/${APP_ID}.svg"
else
    echo "Installing ${BIN} to ${PREFIX} from latest GitHub release"
    fetch() {
        local src="$1" dst="$2"
        echo "  fetching ${src##*/}"
        curl -fsSL -o "${dst}" "${src}"
    }
    fetch "${BASE_URL}/${BIN}"                    "${tmp}/${BIN}"
    fetch "${BASE_URL}/${APP_ID}.desktop"         "${tmp}/${APP_ID}.desktop"
    fetch "${BASE_URL}/${APP_ID}.metainfo.xml"    "${tmp}/${APP_ID}.metainfo.xml"
    fetch "${BASE_URL}/${APP_ID}.svg"             "${tmp}/${APP_ID}.svg"
fi

install -Dm755 "${tmp}/${BIN}"                 "${BIN_DIR}/${BIN}"
install -Dm644 "${tmp}/${APP_ID}.desktop"      "${APP_DIR}/${APP_ID}.desktop"
install -Dm644 "${tmp}/${APP_ID}.metainfo.xml" "${METAINFO_DIR}/${APP_ID}.metainfo.xml"
install -Dm644 "${tmp}/${APP_ID}.svg"          "${ICON_DIR}/${APP_ID}.svg"

# Refresh caches so launchers pick up the new app without a re-login.
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f "${PREFIX}/share/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${APP_DIR}" >/dev/null 2>&1 || true
fi

echo "Installed. Launch from your app menu or run: ${BIN}"
if [ "$(id -u)" -ne 0 ] && ! echo ":$PATH:" | grep -q ":${BIN_DIR}:"; then
    echo "Note: ${BIN_DIR} is not on your PATH. Add it to your shell config to run ${BIN} directly."
fi
