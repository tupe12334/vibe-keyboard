#!/usr/bin/env bash
# Build "Vibe Keyboard.app" (Tauri desktop app) and install it. No sudo.
#
# Usage: packaging/macos/build-app.sh
# Env:   CARGO_TARGET_DIR  Cargo target root (default: <repo>/target)
#        DEST              install folder (default: ~/Applications)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target}"
APP_NAME="Vibe Keyboard.app"
BUNDLE_DIR="${TARGET_DIR}/release/bundle/macos"
DEST="${DEST:-${HOME}/Applications}"

case "${1:-}" in
    "") ;;
    -h | --help)
        echo "Usage: $0"
        exit 0
        ;;
    *)
        echo "error: unknown argument: $1" >&2
        exit 2
        ;;
esac

echo "==> Building ${APP_NAME} with Tauri"
cd "${REPO_ROOT}/apps/desktop"
pnpm install --frozen-lockfile
pnpm tauri build --bundles app

SRC_APP="${BUNDLE_DIR}/${APP_NAME}"
if [[ ! -d "${SRC_APP}" ]]; then
    echo "error: ${SRC_APP} not found after build." >&2
    exit 1
fi

# Replace the old copy only now, after the build has succeeded.
echo "==> Installing to ${DEST}/${APP_NAME}"
mkdir -p "${DEST}"
rm -rf "${DEST:?}/${APP_NAME}"
cp -R "${SRC_APP}" "${DEST}/${APP_NAME}"
# Seal the bundle ad-hoc. Tauri leaves only the binary signed. A sealed bundle
# keeps a stable identity for macOS privacy grants.
codesign --force -s - "${DEST}/${APP_NAME}"
codesign --verify --strict "${DEST}/${APP_NAME}"

echo "Installed: ${DEST}/${APP_NAME}"
echo "Note: push-to-talk needs System Settings → Privacy & Security → Accessibility → Vibe Keyboard. Start at login is in the tray menu."
