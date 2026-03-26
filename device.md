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

| VID    | PID    | Variant                   |
| ------ | ------ | ------------------------- |
| 0x5500 | 0x1001 | StreamDock 293 (original) |
| 0x5548 | 0x6670 | StreamDock 293s           |
| 0x5548 | 0x6674 | Ajazz AKP153 (OEM)        |
| 0x0300 | 0x1010 | Ajazz AKP-153E (OEM)      |
| 0x0300 | 0x1020 | Ajazz AKP-153R (OEM)      |
| 0x6603 | 0x1014 | StreamDock 293sV3         |
| 0x0300 | 0x3010 | This device               |

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

## macOS Access Notes

- The `AppleUserHIDDrivers` DriverKit dext claims interface 0 exclusively via `kIOHIDOptionsTypeSeizeDevice`.
- `hidapi` with `set_open_exclusive(true)` fails with `kIOReturnExclusiveAccess (0xE00002C5)` — cannot evict the dext via HID layer.
- Use `rusb` / `libusb` instead: `detach_kernel_driver(interface)` calls `USBInterfaceOpenSeize` which **can** evict the dext.
- Requires `sudo` (or appropriate udev rules / IOKit entitlements).

## Protocol

All communication over interrupt endpoints; packet size is always **512 bytes** for raw data, or **517 bytes** for commands (prefix + body + padding).

### Command Packet Format

```
[ CMD_PREFIX : 5 bytes ][ COMMAND_BODY : variable ][ zero-padding to 512 bytes ]
                                                      ──────────────────────────
                                        Total wire length: 5 + 512 = 517 bytes
```

**CMD_PREFIX** (always): `43 52 54 00 00` — ASCII `"CRT"` + two null bytes.

### Command Opcodes

| Command       | Body after prefix (hex)                      | ASCII     | Description                                           |
| ------------- | -------------------------------------------- | --------- | ----------------------------------------------------- |
| `CRT_DIS`     | `44 49 53 00 00`                             | `DIS`     | Wake / enable screen                                  |
| `CRT_LIG`     | `4C 49 47 00 00 <value>`                     | `LIG`     | Set brightness; `value` = 0x00–0x64 (0–100%)          |
| `CRT_CLE`     | `43 4C 45 00 00 00 <target>`                 | `CLE`     | Clear key; `target` = key ID or `0xFF` all            |
| `CRT_STP`     | `53 54 50 00 00`                             | `STP`     | Commit / refresh display                              |
| `CRT_BAT`     | `42 41 54 00 00 <size_hi> <size_lo> <keyId>` | `BAT`     | Begin key image transfer (JPEG); size = 2B big-endian |
| `CRT_LOG`     | `4C 4F 47 00 11 94 00 01`                    | `LOG`     | Begin boot logo transfer (raw BGR)                    |
| `CRT_CONNECT` | `43 4F 4E 4E 45 43 54`                       | `CONNECT` | Keep-alive heartbeat; send every ~10 s                |
| `CRT_HAN`     | `48 41 4E`                                   | `HAN`     | Sleep screen                                          |
| `CRT_DC`      | `43 4C 45 00 00 44 43`                       | `CLE..DC` | Graceful disconnect / shutdown                        |

**Device ACK response:** `41 43 4B 00 00 4F 4B` — ASCII `ACK\0\0OK`. Received after image write or other acknowledged commands; byte `[9]` = `0xFF` in the input event stream also signals write completion.

### Key Image Transfer

1. Send `CRT_BAT(size, keyId)` — announce JPEG byte-length (4-byte big-endian) and target key (1–15).
2. Send raw JPEG bytes in 512-byte chunks (no prefix).
3. Send `CRT_STP` to commit.

Image spec: 100×100 JPEG, quality 100, **rotated 180°** before sending.

### Boot Logo Transfer

1. Send `CRT_LOG`.
2. Send raw 800×480 BGR pixels in 512-byte chunks (no prefix, no JPEG).
3. Send `CRT_STP` to commit.

Image spec: 800×480, **rotated 180°** before sending, BGR byte order (B, G, R per pixel).

### Firmware Version Query

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

Received as a 512-byte interrupt IN packet from endpoint `0x82`.

| Byte   | Field      | Values                                                        |
| ------ | ---------- | ------------------------------------------------------------- |
| `[9]`  | Raw key ID | Hardware index (see map below); `0x00` = no key; `0xFF` = ACK |
| `[10]` | State      | `1` = pressed, `2` = released                                 |

### Key ID Mapping (hardware → logical)

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

## Reference Implementations

- Node.js (VID 0x5500): https://github.com/rigor789/mirabox-streamdock-node
- Rust ajazz-sdk fork (VID 0x0300): https://github.com/tupe12334/ajazz-sdk
