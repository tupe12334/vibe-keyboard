# Stream Dock 293S — Device Reference

Product page: https://sdk.key123.vip/en/guide/overview.html#stream-dock-293s

> **Source legend used in this document:**
> - 🔌 **Device** — observed directly from the hardware (USB descriptors, captured packets, live testing)
> - 🌐 **Online** — sourced from SDK docs, community listings, or reference implementations

## USB Identity

🔌 **Device** — read from USB descriptors on this specific unit.

| Field        | Value                      |
| ------------ | -------------------------- |
| Vendor ID    | `0x0300` (Mirabox V2)      |
| Product ID   | `0x3010` (Ajazz AKP153E variant) |
| Manufacturer | HOTSPOTEKUSB               |
| Product name | HOTSPOTEKUSB HID DEMO      |
| Serial       | 4250D278552D               |

🌐 **Online** — the SDK and community list other known VID/PID pairs for the same device family:

| VID    | PID    | Variant                   |
| ------ | ------ | ------------------------- |
| 0x5500 | 0x1001 | StreamDock 293 (original) |
| 0x5548 | 0x6670 | StreamDock 293s           |
| 0x5548 | 0x6674 | Ajazz AKP153 (OEM)        |
| 0x0300 | 0x1010 | Ajazz AKP-153E (OEM)      |
| 0x0300 | 0x1020 | Ajazz AKP-153R (OEM)      |
| 0x6603 | 0x1014 | StreamDock 293sV3         |
| 0x0300 | 0x3010 | Ajazz AKP153E (this device, PID variant of 0x1010) |

## Hardware

🌐 **Online** — from SDK / product page.

- 15 LCD buttons (100×100 px per key, JPEG)
- 3 rotary encoders / knobs
- Boot logo display (800×480 px, BGR)
- USB-C, USB 2.0

## USB Interfaces & Endpoints

🔌 **Device** (interface 0, endpoints `0x03`/`0x82`) — confirmed by live USB capture and `rusb` testing.
🌐 **Online** (interface 1 / secondary usages) — from SDK HID descriptor documentation.

| Interface | Usage page | Usage  | Direction | Endpoint | Purpose             |
| --------- | ---------- | ------ | --------- | -------- | ------------------- |
| 0         | `0xffa0`   | `0x01` | OUT       | `0x03`   | Send commands       |
| 0         | `0xffa0`   | `0x01` | IN        | `0x82`   | Receive key events  |
| 0         | `0xffa0`   | `0x02` | —         | —        | Secondary HID usage |
| 1         | `0x0001`   | `0x06` | IN        | `0x81`   | Keyboard (unused)   |

## macOS Access Notes

🔌 **Device** — all observations made on macOS with this specific unit.

- Open with `hidapi` and `set_open_exclusive(false)`. No root and no sudo needed.
- The vendor interface is opened non-exclusively, so the system HID driver keeps its claim.
- Match usage page `0xFFA0` and usage `0x01` (the `0x02` collection is not used).
- Historical: exclusive `hidapi` open failed, and `rusb` was tried. That path is gone.

## Protocol

All communication over interrupt endpoints; packet size is always **512 bytes** for raw data, or **517 bytes** for commands (prefix + body + padding).

### Command Packet Format

🌐 **Online** — packet structure from SDK and the ajazz-sdk reference implementation.

```
[ CMD_PREFIX : 5 bytes ][ COMMAND_BODY : variable ][ zero-padding to 512 bytes ]
                                                      ──────────────────────────
                                        Total wire length: 5 + 512 = 517 bytes
```

**CMD_PREFIX** (always): `43 52 54 00 00` — ASCII `"CRT"` + two null bytes.

### Command Opcodes

🔌 **Device** — `CRT_CONNECT` confirmed by live testing (heartbeat keeps connection alive).
🌐 **Online** — all other opcodes from SDK docs and the ajazz-sdk / node reference implementations.

| Command       | Body after prefix (hex)                      | ASCII     | Source   | Description                                           |
| ------------- | -------------------------------------------- | --------- | -------- | ----------------------------------------------------- |
| `CRT_DIS`     | `44 49 53 00 00`                             | `DIS`     | 🌐       | Wake / enable screen                                  |
| `CRT_LIG`     | `4C 49 47 00 00 <value>`                     | `LIG`     | 🌐       | Set brightness; `value` = 0x00–0x64 (0–100%)          |
| `CRT_CLE`     | `43 4C 45 00 00 00 <target>`                 | `CLE`     | 🌐       | Clear key; `target` = key ID or `0xFF` all            |
| `CRT_STP`     | `53 54 50 00 00`                             | `STP`     | 🌐       | Commit / refresh display                              |
| `CRT_BAT`     | `42 41 54 00 00 <size_hi> <size_lo> <keyId>` | `BAT`     | 🌐       | Begin key image transfer (JPEG); size = 2B big-endian |
| `CRT_LOG`     | `4C 4F 47 00 11 94 00 01`                    | `LOG`     | 🌐       | Begin boot logo transfer (raw BGR)                    |
| `CRT_CONNECT` | `43 4F 4E 4E 45 43 54`                       | `CONNECT` | 🔌       | Keep-alive heartbeat; send every ~10 s                |
| `CRT_HAN`     | `48 41 4E`                                   | `HAN`     | 🌐       | Sleep screen                                          |
| `CRT_DC`      | `43 4C 45 00 00 44 43`                       | `CLE..DC` | 🌐       | Graceful disconnect / shutdown                        |

**Device ACK response:** `41 43 4B 00 00 4F 4B` — ASCII `ACK\0\0OK`. 🔌 **Device** — observed in live packet capture; byte `[9]` = `0xFF` in the input event stream also signals write completion.

### Key Image Transfer

🌐 **Online** — transfer sequence and image spec from SDK.

1. Send `CRT_BAT(size, keyId)` — announce JPEG byte-length (4-byte big-endian) and target key (1–15).
2. Send raw JPEG bytes in 512-byte chunks (no prefix).
3. Send `CRT_STP` to commit.

Image spec: 100×100 JPEG, quality 100, **rotated 180°** before sending.

### Boot Logo Transfer

🌐 **Online** — transfer sequence and image spec from SDK.

1. Send `CRT_LOG`.
2. Send raw 800×480 BGR pixels in 512-byte chunks (no prefix, no JPEG).
3. Send `CRT_STP` to commit.

Image spec: 800×480, **rotated 180°** before sending, BGR byte order (B, G, R per pixel).

### Firmware Version Query

🔌 **Device** — control transfer confirmed working; response format observed live.

USB control transfer (not interrupt):

```
bmRequestType = 0xA1
bRequest      = 0x01
wValue        = 0x0100
wIndex        = 0x0000
wLength       = 512
```

Response: UTF-8 string.

## Input Event Format

🔌 **Device** — byte offsets and state values determined by capturing packets while pressing keys.

Received as a 512-byte interrupt IN packet from endpoint `0x82`.

| Byte   | Field      | Values                                                        |
| ------ | ---------- | ------------------------------------------------------------- |
| `[9]`  | Raw key ID | Hardware index (see map below); `0x00` = no key; `0xFF` = ACK |
| `[10]` | State      | `1` = pressed, `2` = released                                 |

### Key ID Mapping (hardware → logical)

🔌 **Device** — raw IDs captured by pressing each physical key; logical numbering chosen by us.

The device addresses keys bottom-to-top; the map re-numbers them top-to-bottom, left-to-right (1-based):

```
Raw  Logical   Position
0x01 →  11    row 3, col 1
0x02 →  12    row 3, col 2
0x03 →  13    row 3, col 3
0x04 →  14    row 3, col 4
0x05 →  15    row 3, col 5
0x06 →   6    row 2, col 1
0x07 →   7    row 2, col 2
0x08 →   8    row 2, col 3
0x09 →   9    row 2, col 4
0x0A →  10    row 2, col 5
0x0B →   1    row 1, col 1
0x0C →   2    row 1, col 2
0x0D →   3    row 1, col 3
0x0E →   4    row 1, col 4
0x0F →   5    row 1, col 5
```

Layout (logical numbers, as seen from the front):

```
 1   2   3   4   5
 6   7   8   9  10
11  12  13  14  15
```

## Protocol captured from StreamDock.app (2026-10-20)

🔌 **Device** — captured by hooking IOKit HID calls inside StreamDock.app and verified on hardware. Scripts: [`tools/akp153e/`](tools/akp153e/README.md).

Conflicts with 🌐 entries above are marked **Conflict**. This section is the verified one.

### Identity and interfaces

- Firmware string read from device: `V3.AKP153E.02.009`.
- HID interfaces: boot keyboard (usage page 1, usage 6; sends nothing) and vendor page `0xFFA0` (no report IDs, 512-byte input, 1024-byte output).
- The vendor interface stays silent until init.

### Framing

Every command is `CRT\0\0` + body, zero-padded to **1024 bytes**. **Conflict:** the Command Packet Format above says 512 bytes.

### Init order

1. `DIS` — wake
2. `LIG\0\0\x0f` — brightness byte at offset 10
3. `QUCMD\x11\x11\x00\x11\x00\x11` — meaning unknown; sent because StreamDock sends it
4. `CLE\0\0\0\xff` — clear all
5. `STP` — refresh and commit

Then send `CONNECT` as a heartbeat about every 10 s.

### Key image

1. Send `BAT`, a 4-byte big-endian JPEG length at offsets 8–11, and the key index at offset 12.
2. Send the raw JPEG in 1024-byte chunks.
3. Send `STP`.

- Image size: 85×85 JPEG renders correctly. **Conflict:** the 100×100 note in Key Image Transfer.
- Rotation: rotate 90° counter-clockwise before sending (PIL `rotate(90)`). Unrotated images show turned 90° clockwise. **Conflict:** the 180° rotation note.

### Key index layout

Column-major, numbered from top-right:

```
13 10  7  4  1
14 11  8  5  2
15 12  9  6  3
```

**Conflict:** the Key ID Mapping table above. Check both against hardware before you rely on either.

### Key event

- ACK frame: `ACK\0\0OK\0\0`.
- Key index at byte 9 (1–15).
- State at byte 10: `01` down, `00` up. **Conflict:** the Input Event Format table (`1` pressed, `2` released).

### Side screen

🔌 **Device** — verified on hardware with a Python test script. The side screen sits next to keys 5, 10 and 15 and shares one panel with them.

- Output report `0x00` (no report ID), then one stream: a 32-byte header, then the JPEG. Cut the stream into 1024-byte chunks. Prefix each chunk with `0x00`.
- Header: `CRT\0\0` + `DRA` + size (u32 BE, JPEG length + 32) + key byte (`0`) + w, h, x, y (u16 BE each).
- Geometry: `w=480`, `h=100`, `x=0`, `y=0`. The panel is a vertical column, so the image is rotated 90° counter-clockwise before upload. Keys are rotated too, the side screen is rotated once here.
- Render portrait, 100 px wide × 480 px tall, top of the image = top of the column. Rotate it with `rotate270()` in the `image` crate (PIL `rotate(90)`). Portrait top-left lands at landscape bottom-left.
- Visible safe area, portrait coordinates: x 0..100, y 30..440. Content below y 440 is clipped. Keep about 4 px margins.
- Never send `h > 100`. A 480×272 image spills onto keys 5, 10 and 15.
- Commit with `STP`.

Rust: `side_header`, `side_upload` and `send_side_image` in `src/infrastructure/usb/image.rs`. Portrait render: `side_image` in `src/infrastructure/images/side.rs`.

### macOS access

- hidapi opens devices exclusively by default. The keyboard interface then fails with "open failed", even with Input Monitoring granted.
- `hid_darwin_set_open_exclusive(0)` fixes this.
- With non-exclusive hidapi, no root, sudo, or setuid is needed.

### Rust init

`device_init` in `src/infrastructure/usb/commands.rs` sends this order: `DIS`, `LIG`, `QUCMD`, `CLE`, `STP`, then `CONNECT`.

## Reference Implementations

🌐 **Online**

- Node.js (VID 0x5500): https://github.com/rigor789/mirabox-streamdock-node

## Display Insets: Bezel and Viewing Angle

🌐 **Online** — research for the hidden band at the top of keys and the side column. Not verified on this unit. Sources conflict on sizes and none gives a bezel overlap figure.

- Key image size: 96×96 for the AKP153 in the AKP153-DOCK notes ([xpxp23/AKP153-DOCK](https://github.com/xpxp23/AKP153-DOCK)); 85 for the v1 protocol in Deck Commander and in the python-elgato-streamdeck community PR ([ohotnik PR #148](https://ohotnik.duckdns.org/abcminiuser/python-elgato-streamdeck/pull/148), search snippet only, fetch failed).
- Side column: 80×80 display-only slots in column 5 for the AKP153 ([xpxp23/AKP153-DOCK](https://github.com/xpxp23/AKP153-DOCK)). Not the 100×480 strip used here. Verified 🔌 geometry stays in the section above.
- Rotation: the AKP153 panel is turned 90° relative to image coordinates, so the image is rotated before send ([xpxp23/AKP153-DOCK](https://github.com/xpxp23/AKP153-DOCK)). Matches the 🔌 side-screen note.
- Bezel: the AKP153-DOCK notes say long text wraps and shrinks so it is not hidden by the acrylic keycap edge. No pixel margin is given ([xpxp23/AKP153-DOCK](https://github.com/xpxp23/AKP153-DOCK)).
- Protocol packet sizes differ by version: 512 bytes for v1 (AKP153), 1024 bytes for v2/v3 ([4ndv/mirajazz](https://github.com/4ndv/mirajazz)). Matches the 🔌 1024-byte note.
- Vendor SDK (sdk.key123.vip) and the MiraBox product page gave no icon size or safe-area figure in search results ([MiraBox 293S](https://mirabox.net/products/mirabox-293s)).

**Decision:** no credible bezel margin found. Defaults use the fallback: key top = 8 px, side top = 30 px (existing safe area), side bottom = 40 px. Tune on the device.
