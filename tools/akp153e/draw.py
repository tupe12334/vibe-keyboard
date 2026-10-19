import ctypes, io, sys, time
import hid
from PIL import Image, ImageDraw, ImageFont

ctypes.CDLL("/opt/homebrew/lib/libhidapi.dylib").hid_darwin_set_open_exclusive(0)
PKT = 1024
SIZE = int(sys.argv[1]) if len(sys.argv) > 1 else 85  # ponytail: guessed key size, tune if image is cropped
ROT = int(sys.argv[2]) if len(sys.argv) > 2 else 90  # device shows images turned 90° clockwise


def cmd(body):
    return b"\x00" + (b"CRT\x00\x00" + body).ljust(PKT, b"\x00")


def jpeg(text):
    im = Image.new("RGB", (SIZE, SIZE), (20, 60, 160))
    d = ImageDraw.Draw(im)
    f = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", SIZE // 2)
    d.text((SIZE / 2, SIZE / 2), text, font=f, fill="white", anchor="mm")
    b = io.BytesIO()
    im.rotate(ROT).save(b, "JPEG", quality=90)
    return b.getvalue()


path = next(d["path"] for d in hid.enumerate(0x0300, 0x3010) if d["usage_page"] == 0xFFA0)
h = hid.Device(path=path)
for body in [b"DIS", b"LIG\x00\x00\x0f", b"CONNECT"]:
    h.write(cmd(body))
for key in range(1, 16):
    data = jpeg(str(key))
    h.write(cmd(b"BAT" + len(data).to_bytes(4, "big") + bytes([key])))
    for i in range(0, len(data), PKT):
        h.write(b"\x00" + data[i:i + PKT].ljust(PKT, b"\x00"))
h.write(cmd(b"STP"))
print("drew 15 keys, size", SIZE, "rot", ROT)
