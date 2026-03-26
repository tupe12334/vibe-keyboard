# vibe-keyboard

A custom management system for Ajazz Stream Dock devices (AKP series). Control button images, brightness, boot logos, and handle button events — all from your own tooling.

## Device

The Ajazz Stream Dock shows up as `HOTSPOTEKUSB HID DEMO` on macOS/Linux (VendorID `0x0300`, ProductID `0x3010`). This is normal — the device uses generic OEM HID firmware.

Confirmed working with:

- AKP153 / AKP153E / AKP153R
- AKP815
- AKP03 / AKP03E / AKP03R / AKP03RV2

## Goals

- [ ] Connect to the device over USB HID
- [ ] Read button press / release events in real time
- [ ] Set custom images on individual buttons
- [ ] Control display brightness
- [ ] Set a custom boot logo
- [ ] Define button actions (launch app, run script, send keypress, etc.)
- [ ] Persist profiles/layouts to disk
- [ ] Hot-reload config without restarting the daemon

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

## Tech Stack

| Layer              | Choice                                            | Notes                           |
| ------------------ | ------------------------------------------------- | ------------------------------- |
| Language           | Rust                                              | Performance, strong HID support |
| HID communication  | [`hidapi`](https://crates.io/crates/hidapi)       | Cross-platform USB HID          |
| Device abstraction | [`ajazz-sdk`](https://crates.io/crates/ajazz-sdk) | Rust adapter for AKP devices    |
| Image processing   | [`image`](https://crates.io/crates/image)         | Resize/convert button images    |
| Async runtime      | [`tokio`](https://crates.io/crates/tokio)         | Async event loop for daemon     |
| Config format      | TOML                                              | Human-editable profiles         |

## Getting Started

### Prerequisites

- Rust toolchain (`rustup`)
- On Linux: `libudev-dev` and a udev rule to access the HID device without root

```bash
# Linux udev rule (create /etc/udev/rules.d/99-ajazz.rules)
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="0300", ATTRS{idProduct}=="3010", MODE="0666"
```

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

## HID Protocol

The Ajazz AKP series communicates over USB HID. Protocol notes are documented in the community:

- [Notes on the Ajazz Stream Deck HID protocol](https://gist.github.com/ZCube/430fab6039899eaa0e18367f60d36b3c)
- Reference implementation: [mishamyrt/ajazz-sdk](https://github.com/mishamyrt/ajazz-sdk)

Key facts:

- Button images are sent as raw JPEG/BMP frames per button
- Events are read as HID reports (button index + press/release state)
- Brightness is set via a control report
- Boot logo is written similarly to button images

## References

- [ajazz-sdk (Rust)](https://github.com/mishamyrt/ajazz-sdk)
- [HID protocol notes](https://gist.github.com/ZCube/430fab6039899eaa0e18367f60d36b3c)
- [OpenDeck fork for Ajazz](https://github.com/nekename/OpenDeck)
- [Bitfocus Companion issue — Ajazz support](https://github.com/bitfocus/companion/issues/3072)
- [Ajazz official software](https://ajazzstore.com/blogs/software)

## License

MIT
