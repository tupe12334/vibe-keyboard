#!/bin/sh
# CFBundleExecutable of VibeKeyboard.app. Finder and `open` run this.
# Default: start the headless controller (no window, no terminal needed).
# `open -a VibeKeyboard.app --args --editor` opens the editor window instead.
set -eu
DIR="$(cd "$(dirname "$0")" && pwd)"
if [ "${1:-}" = "--editor" ]; then
    exec "${DIR}/editor"
fi
exec "${DIR}/listen" --headless
