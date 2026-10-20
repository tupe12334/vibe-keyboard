# macOS packaging

`build-app.sh` builds the Tauri desktop app and installs it to
`~/Applications/Vibe Keyboard.app`. No terminal window is needed after install,
and no sudo or root.

## Install

```sh
packaging/macos/build-app.sh
```

The script runs `pnpm install --frozen-lockfile` and
`pnpm tauri build --bundles app` in `apps/desktop`. It copies the result to
`~/Applications`. The old copy is replaced only after the build succeeds.

Environment variables:

| Variable | Default | Effect |
| --- | --- | --- |
| `CARGO_TARGET_DIR` | `<repo>/target` | Build output root |
| `DEST` | `~/Applications` | Install folder |

## Permissions

Push-to-talk needs System Settings → Privacy & Security → Accessibility →
Vibe Keyboard. Start at login is in the tray menu.

## Legacy

`packaging/macos/uninstall.sh` removes the old `VibeKeyboard.app` and its
LaunchAgent from earlier builds. It does nothing for `Vibe Keyboard.app`.
