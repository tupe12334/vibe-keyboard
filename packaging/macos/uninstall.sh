#!/usr/bin/env bash
# Remove the VibeKeyboard controller LaunchAgent and the installed app.
# Logs are kept. Delete ~/Library/Logs/vibe-keyboard by hand if unwanted.
set -euo pipefail

AGENT_LABEL="com.tupe12334.vibe-keyboard.controller"
AGENT_PLIST="${HOME}/Library/LaunchAgents/${AGENT_LABEL}.plist"
INSTALL_APP_PATH="${HOME}/Applications/VibeKeyboard.app"

# Stop the agent. Ignore the error when it is not loaded.
launchctl bootout "gui/$(id -u)/${AGENT_LABEL}" 2>/dev/null || true

rm -f "${AGENT_PLIST}"
rm -rf "${INSTALL_APP_PATH}"

echo "Uninstalled ${AGENT_LABEL} and ${INSTALL_APP_PATH}."
