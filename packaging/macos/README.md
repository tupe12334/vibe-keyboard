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
| `VK_SIGN_IDENTITY` | `Vibe Keyboard Local` | Code-signing identity. See [Keep the Accessibility grant](#keep-the-accessibility-grant-across-rebuilds) |

## Permissions

Push-to-talk needs System Settings → Privacy & Security → Accessibility →
Vibe Keyboard. Start at login is in the tray menu.

## Keep the Accessibility grant across rebuilds

Without a stable signing identity, macOS drops the Accessibility grant on
every reinstall. Push-to-talk then stops silently. Create one local identity
once:

1. Open Keychain Access. Choose Keychain Access → Certificate Assistant →
   Create a Certificate….
2. Name: `Vibe Keyboard Local`. Identity type: Self Signed Root. Certificate
   type: Code Signing. Click Create.
3. Check it. Run `security find-identity -v -p codesigning`. The output must list
   `Vibe Keyboard Local`. If it is missing, open the certificate and set Trust →
   Code Signing to Always Trust.
4. Run `packaging/macos/build-app.sh`. The script signs with that identity.
5. Grant Accessibility once to the installed app. Later rebuilds keep the grant.

To use another identity (for example an Apple Development certificate), set
`VK_SIGN_IDENTITY` to its name:

```sh
VK_SIGN_IDENTITY="Apple Development: you@example.com (TEAMID)" packaging/macos/build-app.sh
```

If the identity is missing, the script warns and signs ad-hoc.

## Legacy

`packaging/macos/uninstall.sh` removes the old `VibeKeyboard.app` and its
LaunchAgent from earlier builds. It does nothing for `Vibe Keyboard.app`.
