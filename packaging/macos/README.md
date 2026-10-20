# macOS packaging

Builds `VibeKeyboard.app`. The app starts the headless controller. A per-user
LaunchAgent can start it at login. No terminal, no sudo, no root.

## Bundle

| Key | Value |
| --- | --- |
| Bundle id | `com.tupe12334.vibe-keyboard` |
| Executable | `Contents/MacOS/launcher` (shell script, `exec`s `listen --headless`) |
| Controller | `Contents/MacOS/listen` |
| Editor | `Contents/MacOS/editor` |
| Dock icon | none (`LSUIElement`) |
| Icon | `Contents/Resources/AppIcon.icns` (from `assets/app/`) |

## Build

```sh
packaging/macos/build-app.sh
```

Output: `target/VibeKeyboard.app` (or `$CARGO_TARGET_DIR/VibeKeyboard.app`),
ad-hoc signed. Building does not start anything.

## Install and run at login

```sh
packaging/macos/build-app.sh --install
```

This copies the app to `~/Applications/VibeKeyboard.app`, writes
`~/Library/LaunchAgents/com.tupe12334.vibe-keyboard.controller.plist`, and
loads it. The agent runs `listen --headless` at login and restarts after a
crash. It exits cleanly if another controller already holds the lock.

## Open

- Double-click `VibeKeyboard.app`: starts the controller. No window.
- Open the editor: `open -a ~/Applications/VibeKeyboard.app --args --editor`.

Only one controller runs at a time. A second start exits without error.

## Permissions

The HID device opens non-exclusively. No Input Monitoring grant is needed.
Some keys run `osascript`: they control Terminal, Claude, and System Events.
macOS asks for Automation for those apps. A key that sends a keystroke through
System Events needs Accessibility. Grant these to `VibeKeyboard` when asked.
Ad-hoc signing pins each build. After a rebuild, macOS can drop a manual grant.
A stable signing identity would keep grants across rebuilds.

## Uninstall

```sh
packaging/macos/uninstall.sh
```

Stops the agent, then removes the plist and `~/Applications/VibeKeyboard.app`.

## Logs

`~/Library/Logs/vibe-keyboard/controller.log` (stdout and stderr). The uninstall
script keeps this file.
