# vibe-keyboard

A custom management system for the Stream Dock 293S. Control button images, brightness, and handle button events — all from your own tooling.

## Features

- [x] Connect to the device over USB HID
- [x] Read button press / release events in real time
- [x] Set custom images on individual buttons
- [x] Control display brightness
- [x] Persist device state (current page, brightness) to disk
- [x] Stack-based hierarchical navigation across screens
- [x] Centy project/issue integration (browse projects, open in VS Code/Terminal/Browser)
- [x] Terminal UI with 3×5 button grid, loading spinner, and log view
- [x] Keep-alive heartbeat to maintain device connection
- [x] User-defined pages, key labels/colors/images, and key actions in one TOML file
- [x] `editor` GUI to build those pages without editing TOML
- [x] Hot reload: saved changes reach the device within about a second

## Architecture

The project follows a Domain-Driven Design (DDD) layered structure:

```
┌─────────────────────────────────────────────────────────────┐
│                        main.rs                              │
│   USB init → spawn TUI thread → event loop                  │
├──────────────┬──────────────────────────────────────────────┤
│ presentation │  ratatui TUI: 3×5 button grid, spinner, logs │
├──────────────┼──────────────────────────────────────────────┤
│ application  │  handle_key_event(), render_screen(),        │
│              │  page action dispatchers, centy integration  │
├──────────────┼──────────────────────────────────────────────┤
│ domain       │  NavigationStack, key mappings, action types │
├──────────────┼──────────────────────────────────────────────┤
│infrastructure│  USB HID comms, RGB565 image transfer,       │
│              │  image generation, TOML state persistence    │
└──────────────┴──────────────────────────────────────────────┘
```

### Navigation

Navigation is managed by a `NavigationStack` (domain layer) with three dedicated hardware buttons:

| Button | Action |
|--------|--------|
| 11     | Back — cycle page left on MainPage, or pop action screens |
| 12     | Out — pop current screen from stack |
| 13     | Forward — cycle page right / next page |

Screens: `MainPage` → `CentyProjectList` → `CentyProjectActions` → `CentyIssueList` → `CentyIssueActions`. `InputNumber` is reachable directly from `MainPage`.

### Pages

**Main Page 0**: Centy projects, Terminal, Claude, Centy Web
**Main Page 1**: Input number (numpad), Log file, VS Code config

List screens (projects, issues) are paginated at 10 items per page.

## Getting Started

### Prerequisites

- Rust toolchain (`cargo`)
- macOS (uses `osascript` for opening apps)
- Stream Dock 293S connected via USB
- `pnpm` (for Centy integration)

### Download

Prebuilt macOS installers (DMG and app zip) are on the [Releases page](https://github.com/tupe12334/vibe-keyboard/releases). The builds are unsigned: the first time, right-click the app and choose **Open**. Push-to-talk also needs the Accessibility step below. After each app update, macOS can drop the Accessibility grant (unsigned builds get a new signature), so push-to-talk stops; re-enable Vibe Keyboard in System Settings > Privacy & Security > Accessibility, then confirm `app.log` no longer shows `push to talk needs Accessibility` ([steps](#hold-a-key)).

### Install

```bash
git clone https://github.com/tupe12334/vibe-keyboard
cd vibe-keyboard
packaging/macos/build-app.sh
```

This installs `Vibe Keyboard.app` to `~/Applications`. No `sudo` is needed.
To keep the Accessibility grant across rebuilds, see
[Keep the Accessibility grant across rebuilds](packaging/macos/README.md#keep-the-accessibility-grant-across-rebuilds).

The app runs as a normal user. It opens the device's HID vendor interface in
non-exclusive mode. The `hidapi` crate builds its bundled C backend, so no
`brew install hidapi` is needed.

The app has three parts:

- Controller: drives the Stream Dock 293S.
- Tray icon: menu bar entry. Start at login is in this menu. Quit is here too.
- Editor: key layout window.

Push-to-talk needs System Settings → Privacy & Security → Accessibility →
Vibe Keyboard.

Headless option: `cargo run --release --bin listen -- --headless` runs the
controller only, with no window and no tray. Use it for a CLI or a script.

### Troubleshooting

- `another vibe-keyboard controller is already running (lock <path>)`: a controller already drives the deck. Stop it first. Only one controller may run per deck; the lock file lives in the runtime dir (or the temp dir on macOS).

State is persisted to `~/.config/vibe-keyboard/state.toml`.
Logs are written to `~/.config/vibe-keyboard/app.log`.

### Desktop app (Tauri)

`apps/desktop` is the app that `packaging/macos/build-app.sh` installs. It holds
the device loop, a menu bar tray icon, and the key editor window. Closing the
window hides it. Quit from the tray.

Needs Node 22+ and `pnpm`.

```bash
pnpm -C apps/desktop install
pnpm -C apps/desktop tauri dev      # run with hot reload
```

Set `VIBE_NO_DEVICE=1` to run the UI without the controller. The app then never
opens the deck. Use it for every verification run, so a test cannot press keys
on a deck another controller uses:

```bash
VIBE_NO_DEVICE=1 pnpm -C apps/desktop tauri dev
```

Typed bindings for the frontend go to `apps/desktop/src/bindings.ts` on every
debug run. Commit the file when the Rust API changes.

The TUI and the egui editor are legacy, being replaced by the desktop app.

## Customize

All main-page keys come from `~/.config/vibe-keyboard/profile.toml` (or `$XDG_CONFIG_HOME/vibe-keyboard/profile.toml`).
It is created on first run with the default layout.

Legacy egui editor (being replaced by the desktop app; saving reloads the controller):

```bash
cargo run --bin editor
```

```toml
[[pages]]
name = "Dev"

[pages.keys.1]
label = "Terminal"
bg = "#202020"
fg = "#FFFFFF"
action = { type = "builtin", name = "terminal" }

[pages.keys.2]
label = "Docs"
image = "images/docs.png"          # relative to the config dir
action = { type = "open_url", url = "https://example.com" }

[pages.keys.3]
label = "Shot"
action = { type = "hotkey", keys = "cmd+shift+4" }

[pages.keys.4]
label = "Claude"
action = { type = "builtin", name = "claude_sessions" }
```

Action types: `none`, `open_url`, `run_command`, `open_app`, `hotkey`, `go_to_page` (0-based index), `builtin`, `push_to_talk`.
`open_app` takes an app name (`open -a`), or a bundle id (`open -b`) when the value contains a dot and no spaces. Names ending in `.app` are app names.
`run_command` runs `sh -c` with the text stored in `profile.toml`. Anyone who can write that file can run commands, so track it only in a repo you trust.
Built-in names: `centy_projects`, `terminal`, `claude`, `centy_web`, `numpad`, `log_file`, `vscode_config`, `claude_sessions`.
`claude_sessions` lists open Claude Code sessions (`claude` processes on a terminal). Each key shows the session's tab title, with a status bar: amber when Claude waits for you, green while it works. Press a session to focus its iTerm2 or Terminal.app tab. Other terminals (VS Code) activate VS Code. Key 15 pages when there are more than 10.
Keys 11, 12 and 13 are navigation and cannot be bound.
A built-in key shows its icon unless `image` is set.
A malformed file is logged with its line and column. The controller keeps the last good layout.

Run the controller headless (no window, no tray) with `listen --headless`. The desktop app runs the same controller. It runs the same device loop and actions, with no TUI. Logs go to `~/.config/vibe-keyboard/app.log`. It exits cleanly on SIGINT or SIGTERM.

### Hold a key

A key can have a second action, `hold`. A short press runs `action`, as before. A press held for `[input] hold_ms` (default `400`) starts `hold`. When the key comes up, the hold ends. A `push_to_talk` hold presses its key down at the start and releases it at the end.

```toml
[input]
hold_ms = 400

[pages.keys.5]
label = "Claude"
action = { type = "builtin", name = "claude" }   # short press: open Claude
hold = { type = "push_to_talk" }                  # hold: Claude Code dictation
```

`push_to_talk` takes an optional `key`, a shortcut such as `"space"` or `"cmd+k"`. Without it the key is `Space`. As a plain `action`, `push_to_talk` sends one press and release.
On the Claude sessions page, a held session key focuses that session, waits 150 ms, then starts dictation. Release stops it. A short press only focuses, as before. If the controller stalls for more than 2 seconds during a hold, the key is lifted and dictation stops.

Claude Code dictation (checked on 2026-10-20):

- [Voice dictation](https://code.claude.com/docs/en/voice-dictation). `/voice` turns it on. `/voice hold` is the default: hold `Space` to record, release to stop. `/voice tap` is the other mode: tap to start, tap to send.
- The key is the `voice:pushToTalk` binding in the `Chat` context of `~/.claude/keybindings.json`. It defaults to `Space`.
- Settings: `"voice": { "enabled": true, "mode": "hold", "autoSubmit": false }`.
- Hold mode detects a hold by counting key-repeat events from the terminal. A single key down does not start it. So `push_to_talk` sends key down, then repeated key down with the auto-repeat flag, then key up. Keep `/voice hold` on. In tap mode each repeat would toggle recording.
- Dictation needs a claude.ai sign-in and a local microphone.

Push-to-talk posts keys with CoreGraphics. macOS only lets a process post keys when it has Accessibility permission:

1. Open System Settings > Privacy & Security > Accessibility.
2. Click `+`. Add the app that runs `listen`: your terminal app when you start it by hand, or `Vibe Keyboard` for the desktop app.
3. Turn the switch on.
4. Hold the key again. Without permission, `app.log` shows `push to talk needs Accessibility`, and holds do nothing.

### App-aware pages

A page can list bundle ids in `apps`. When one of those apps is in front, the controller shows that page. When no page lists the app, it returns to the page you were on before. Manual paging (keys 11 and 13) stays in effect until the next app change. Sub-screens (Centy lists, numpad) are never interrupted.

```toml
[[pages]]
name = "Browser"
apps = ["com.google.Chrome", "com.apple.Safari"]

[pages.keys.1]
label = "Terminal"
action = { type = "open_app", name = "com.googlecode.iterm2" }
```

`open_app` takes an app name (`open -a`) or a bundle id (`open -b`, when the value contains a dot and no spaces).
The editor lists running apps for each page's `apps`. You can also type a bundle id.

### Side screen widgets

The 480x100 side screen next to keys 5, 10 and 15 can show live values. List them in order, left to right. When `widgets` is set, it replaces the side `label` and `image`.

```toml
[[pages]]
name = "Monitor"

[pages.side]
bg = "#101018"
fg = "#E6E6E6"
widgets = [
  { type = "cpu_bar" },
  { type = "cpu_percent" },
  { type = "temperature" },
  { type = "disk_free" },
]
```

Widget types: `cpu_percent`, `cpu_bar` (0-100 %, green below 60, amber below 85, red above), `mem_bar` (used / total physical RAM, same colors), `cpu_mem_bar` (one bar split in two: CPU on the left half, memory on the right half, each half filled to its own percent), `temperature` (hottest CPU die, `N/A` if none), `disk_free` (free space on the user volume), and `text` with `value = "..."`.
Values refresh every 2 seconds. Only a changed frame is sent to the device.

#### Bar style

The `cpu_bar` widget takes its look from `[pages.side.bar_style]`. Every field is optional. A field left out keeps the built-in look, and a profile without the table is not rewritten on save.

```toml
[pages.side.bar_style]
fill = "#2EA043"        # #RRGGBB, absent = green/amber/red by load
track = "#3C3C3C"       # #RRGGBB, absent = dark grey
thickness = 40         # 10 to 60 px, absent = 40
show_value = true      # the percent under the bar, absent = true
label_color = "#E6E6E6" # #RRGGBB, absent = the side text color
```

Track the config directory in git. Keep runtime files out of it:

```gitignore
# ~/.config/vibe-keyboard/.gitignore
state.toml
*.log
*.toml.tmp
```

Commit `profile.toml` and `images/`. The controller never writes `profile.toml`. Only the editor (and the first-run default) writes it.
Runtime state (current page, brightness, auto or manual page) stays out of it: `state.toml` and `app.log` are the ignored files.

The controller runs as your normal user and writes its files as that user, so they stay editable and committable.

## File size limit

Rust source files are limited by [`linecheck`](https://crates.io/crates/linecheck), configured in `linecheck.yml`.

- **General limit:** 100 lines (error) for every `**/*.rs` file.
- **Per-file caps:** each Rust file that is over 100 lines has its own rule in `linecheck.yml`. The cap equals its line count when the rule was set. Rules are first-match, so per-file rules come before the general rule.
- **Ratchet:** caps only go down. A capped file can shrink, never grow. When a file drops to 100 lines or fewer, delete its rule.
- **Plan:** split the largest file in each PR, lower its cap, and remove rules as files reach 100.
- **Run locally:** `cargo install linecheck --locked`, then `linecheck .` from the repo root. `linecheck --status .` shows how close each file is to its cap.

## See also

- [AKP153E protocol capture and hidapi tools](tools/akp153e/README.md)
- [moadim](https://moadim.io/) — loop engineering: build, schedule & run agent loops.

## License

MIT
