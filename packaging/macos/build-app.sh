#!/usr/bin/env bash
# Build VibeKeyboard.app (ad-hoc signed) from the release binaries.
#
# Usage: packaging/macos/build-app.sh [--install]
#   --install  also copy the app to ~/Applications and install the
#              controller LaunchAgent (runs listen --headless at login).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target}"
APP_NAME="VibeKeyboard.app"
APP_PATH="${TARGET_DIR}/${APP_NAME}"
BUNDLE_ID="com.tupe12334.vibe-keyboard"
AGENT_LABEL="com.tupe12334.vibe-keyboard.controller"
INSTALL_APP_PATH="${HOME}/Applications/${APP_NAME}"
AGENT_PLIST="${HOME}/Library/LaunchAgents/${AGENT_LABEL}.plist"
LOG_DIR="${HOME}/Library/Logs/vibe-keyboard"

INSTALL=0
for arg in "$@"; do
    case "${arg}" in
        --install) INSTALL=1 ;;
        -h | --help)
            echo "Usage: $0 [--install]"
            exit 0
            ;;
        *)
            echo "error: unknown argument: ${arg}" >&2
            exit 2
            ;;
    esac
done

cd "${REPO_ROOT}"

echo "==> Building release binaries"
cargo build --release --bins

BIN_DIR="${TARGET_DIR}/release"
for bin in listen editor; do
    if [[ ! -x "${BIN_DIR}/${bin}" ]]; then
        echo "error: ${BIN_DIR}/${bin} not found after cargo build." >&2
        exit 1
    fi
done

VERSION="$(cargo metadata --no-deps --format-version 1 |
    python3 -c 'import json, sys; print(json.load(sys.stdin)["packages"][0]["version"])')"

echo "==> Assembling ${APP_NAME} (version ${VERSION})"
rm -rf "${APP_PATH}"
mkdir -p "${APP_PATH}/Contents/MacOS" "${APP_PATH}/Contents/Resources"
cp "${BIN_DIR}/listen" "${APP_PATH}/Contents/MacOS/listen"
cp "${BIN_DIR}/editor" "${APP_PATH}/Contents/MacOS/editor"
cp "${SCRIPT_DIR}/launcher.sh" "${APP_PATH}/Contents/MacOS/launcher"
chmod 755 "${APP_PATH}/Contents/MacOS/"*
cp "${REPO_ROOT}/assets/app/AppIcon.icns" "${APP_PATH}/Contents/Resources/AppIcon.icns"
# Runtime icons for the editor. Harmless if the editor never reads them.
cp -R "${REPO_ROOT}/assets/icons/png" "${APP_PATH}/Contents/Resources/icons"
sed -e "s|@VERSION@|${VERSION}|g" \
    "${SCRIPT_DIR}/Info.plist.template" > "${APP_PATH}/Contents/Info.plist"
plutil -lint "${APP_PATH}/Contents/Info.plist" >/dev/null

echo "==> Ad-hoc codesigning (identifier ${BUNDLE_ID})"
# Fixed identifier per binary, so the code identity does not depend on the
# file name. Ad-hoc signatures still pin the exact build (cdhash): a rebuild
# can reset Accessibility or Input Monitoring grants.
for bin in listen editor; do
    codesign --force -s - -i "${BUNDLE_ID}" "${APP_PATH}/Contents/MacOS/${bin}"
done
codesign --force -s - -i "${BUNDLE_ID}" "${APP_PATH}"
codesign --verify --strict "${APP_PATH}"

echo "Built: ${APP_PATH}"

if [[ "${INSTALL}" -eq 1 ]]; then
    echo "==> Installing app to ${INSTALL_APP_PATH}"
    mkdir -p "${HOME}/Applications" "${HOME}/Library/LaunchAgents" "${LOG_DIR}"
    rm -rf "${INSTALL_APP_PATH}"
    cp -R "${APP_PATH}" "${INSTALL_APP_PATH}"

    echo "==> Installing LaunchAgent ${AGENT_PLIST}"
    sed -e "s|@HOME@|${HOME}|g" \
        "${SCRIPT_DIR}/${AGENT_LABEL}.plist.template" > "${AGENT_PLIST}"
    plutil -lint "${AGENT_PLIST}" >/dev/null

    # Unload an earlier copy first. Ignore the error when none is loaded.
    launchctl bootout "gui/$(id -u)/${AGENT_LABEL}" 2>/dev/null || true
    launchctl bootstrap "gui/$(id -u)" "${AGENT_PLIST}"

    echo "Installed. Logs: ${LOG_DIR}/controller.log"
fi
