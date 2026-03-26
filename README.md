# vibe-keyboard

A custom management system for Ajazz Stream Dock devices (AKP series). Control button images, brightness, boot logos, and handle button events — all from your own tooling.

## Device

The Ajazz Stream Dock shows up as `HOTSPOTEKUSB HID DEMO` on macOS/Linux (VendorID `0x0300`, ProductID `0x3010`). This is normal — the device uses generic OEM HID firmware.

Confirmed working with:

- AKP153 / AKP153E / AKP153R
- AKP815
- AKP03 / AKP03E / AKP03R / AKP03RV2

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
