# Stream Dock 293S — Device Reference

Product page: https://sdk.key123.vip/en/guide/overview.html#stream-dock-293s

## USB Identity

| Field        | Value                      |
| ------------ | -------------------------- |
| Vendor ID    | `0x0300` (Mirabox V2)      |
| Product ID   | `0x3010` (unknown variant) |
| Manufacturer | HOTSPOTEKUSB               |
| Product name | HOTSPOTEKUSB HID DEMO      |
| Serial       | 4250D278552D               |

The SDK and community list other known VID/PID pairs for the same device family:

| VID    | PID    | Variant              |
| ------ | ------ | -------------------- |
| 0x5500 | 0x1001 | Standard 293S        |
| 0x0300 | 0x1010 | Ajazz AKP-153E (OEM) |
| 0x0300 | 0x3010 | This device          |

## Hardware

- 15 LCD buttons (100×100 px per key, JPEG)
- 3 rotary encoders / knobs
- Boot logo display (800×480 px, BGR)
- USB-C, USB 2.0

## USB Interfaces & Endpoints

| Interface | Usage page | Usage  | Direction | Endpoint | Purpose             |
| --------- | ---------- | ------ | --------- | -------- | ------------------- |
| 0         | `0xffa0`   | `0x01` | OUT       | `0x03`   | Send commands       |
| 0         | `0xffa0`   | `0x01` | IN        | `0x82`   | Receive key events  |
| 0         | `0xffa0`   | `0x02` | —         | —        | Secondary HID usage |
| 1         | `0x0001`   | `0x06` | IN        | `0x81`   | Keyboard (unused)   |
