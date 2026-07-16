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

Screens: `MainPage` → `CentyProjectList` → `CentyProjectActions` → `CentyIssueList`

### Pages

**Main Page 0**: Centy projects, Terminal, Claude, Centy Web
**Main Page 1**: Log file, VS Code config

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

State is persisted to `~/.config/vibe-keyboard/state.toml`.
Logs are written to `~/.config/vibe-keyboard/app.log`.

## See also

- [moadim](https://moadim.io/) — loop engineering: build, schedule & run agent loops.

## License

MIT
