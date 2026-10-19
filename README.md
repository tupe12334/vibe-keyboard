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

### Build & Run

```bash
git clone https://github.com/tupe12334/vibe-keyboard
cd vibe-keyboard
cargo build --release
cargo run --release
```

The app runs as a normal user. No `sudo` or setuid is needed: it opens the
device's HID vendor interface in non-exclusive mode. To install the binary to
`~/.cargo/bin`, run `make install`.

The `hidapi` crate builds its bundled C backend. No system library is needed,
so no `brew install hidapi`.

### Troubleshooting

- `another vibe-keyboard controller is already running (lock <path>)`: a controller already drives the deck. Stop it first. Only one controller may run per deck; the lock file lives in the runtime dir (or the temp dir on macOS).

State is persisted to `~/.config/vibe-keyboard/state.toml`.
Logs are written to `~/.config/vibe-keyboard/app.log`.

## Customize

All main-page keys come from `~/.config/vibe-keyboard/profile.toml` (or `$XDG_CONFIG_HOME/vibe-keyboard/profile.toml`).
It is created on first run with the default layout.

```bash
cargo run --bin editor   # GUI editor; saving reloads the controller
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

Action types: `none`, `open_url`, `run_command`, `open_app`, `hotkey`, `go_to_page` (0-based index), `builtin`.
`open_app` takes an app name (`open -a`), or a bundle id (`open -b`) when the value contains a dot and no spaces. Names ending in `.app` are app names.
`run_command` runs `sh -c` with the text stored in `profile.toml`. Anyone who can write that file can run commands, so track it only in a repo you trust.
Built-in names: `centy_projects`, `terminal`, `claude`, `centy_web`, `numpad`, `log_file`, `vscode_config`, `claude_sessions`.
`claude_sessions` lists open Claude Code sessions (`claude` processes on a terminal). Each key shows the session's tab title, with a status bar: amber when Claude waits for you, green while it works. Press a session to focus its iTerm2 or Terminal.app tab. Other terminals (VS Code) activate VS Code. Key 15 pages when there are more than 10.
Keys 11, 12 and 13 are navigation and cannot be bound.
A built-in key shows its icon unless `image` is set.
A malformed file is logged with its line and column. The controller keeps the last good layout.

Run the controller without a terminal (for example at login) with `listen --headless`. It runs the same device loop and actions, with no TUI. Logs go to `~/.config/vibe-keyboard/app.log`. It exits cleanly on SIGINT or SIGTERM.

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

## See also

- [AKP153E protocol capture and hidapi tools](tools/akp153e/README.md)
- [moadim](https://moadim.io/) — loop engineering: build, schedule & run agent loops.

## License

MIT
