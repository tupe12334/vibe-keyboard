#!/bin/sh
# Enforce 100% line coverage on pure-logic modules (hardware/UI code excluded).
# Shared by the pre-push hook and CI. Needs cargo-llvm-cov and llvm-tools-preview.
# bin/ is the egui editor (UI code, no unit coverage target).
set -eu

IGNORE="main\.rs|logging\.rs|controller/(run|tick|input|setup|side|reload|ctx|mod)\.rs|usb/|images/|application/|tui/mod|tui/render|presentation/mod|persistence|bin/|infrastructure/frontmost/os\.rs|infrastructure/app_icon/os\.rs$|infrastructure/app_icon/icns_render\.rs$|infrastructure/claude_sessions/os\.rs$|infrastructure/claude_sessions/os/terminal\.rs$|infrastructure/claude_sessions/os/front_tty\.rs$|infrastructure/claude_sessions/os/fullscreen\.rs$|infrastructure/claude_sessions/os/summary\.rs$|infrastructure/claude_sessions/os/tty_age\.rs$|infrastructure/process/os\.rs$|infrastructure/login_path/os\.rs$|infrastructure/focused_session/os\.rs$|infrastructure/system_metrics/os\.rs$|infrastructure/moadim/os\.rs|infrastructure/running_apps/os\.rs|infrastructure/keyboard/os|infrastructure/mouse_release/os\.rs$|(^|/)apps/"

cargo llvm-cov \
  --ignore-filename-regex "$IGNORE" \
  --fail-under-lines 100

# Drop the instrumented build dir (about 2 GiB) after a pass. CI runners are discarded, so skip there.
if [ -z "${CI:-}" ]; then
  rm -rf "${CARGO_TARGET_DIR:-target}/llvm-cov-target"
fi
