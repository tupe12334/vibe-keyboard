# vibe-keyboard

A custom management system for the Stream Dock 293S. Control button images, brightness, boot logos, and handle button events — all from your own tooling.

## Goals

- [x] Connect to the device over USB HID
- [x] Read button press / release events in real time
- [x] Set custom images on individual buttons
- [x] Control display brightness
- [x] Set a custom boot logo
- [x] Define button actions (launch app, run script, send keypress, etc.)
- [x] Persist profiles/layouts to disk
- [x] Hot-reload config without restarting the daemon

## Architecture (planned)

```
┌─────────────────────────────────────────┐
│              vibe-keyboard              │
├─────────────┬───────────────────────────┤
│   daemon    │  Connects to device,      │
│             │  reads events, applies    │
│             │  images/brightness        │
├─────────────┼───────────────────────────┤
│   config    │  TOML/JSON profiles       │
│             │  mapping buttons to       │
│             │  actions and images       │
├─────────────┼───────────────────────────┤
│   cli / ui  │  Control the daemon,      │
│             │  switch profiles,         │
│             │  preview button layout    │
└─────────────┴───────────────────────────┘
```

## Getting Started

### Build

```bash
git clone https://github.com/tupe12334/vibe-keyboard
cd vibe-keyboard
cargo build --release
```

### Run

```bash
cargo run -- --config config.toml
```

## License

MIT
