import ctypes, sys, time
import hid

# Mirabox/Ajazz "CRT" protocol, recovered from StreamDock's libSDLibrary (SDDevice::add*Pack)
ctypes.CDLL("/opt/homebrew/lib/libhidapi.dylib").hid_darwin_set_open_exclusive(0)
SECS = int(sys.argv[1]) if len(sys.argv) > 1 else 60
PKT = 1024  # MaxOutputReportSize of the 0xFFA0 interface


def cmd(body):
    return b"\x00" + (b"CRT\x00\x00" + body).ljust(PKT, b"\x00")  # leading 0 = no report ID


path = next(d["path"] for d in hid.enumerate(0x0300, 0x3010) if d["usage_page"] == 0xFFA0)
h = hid.Device(path=path)
h.nonblocking = True
# init sequence as StreamDock sends it to the AKP153E (captured with hidspy)
INIT = [("wake", b"DIS"), ("brightness", b"LIG\x00\x00\x0f"), ("qucmd", b"QUCMD\x11\x11\x00\x11\x00\x11"),
        ("clear all", b"CLE\x00\x00\x00\xff"), ("refresh", b"STP"), ("heartbeat", b"CONNECT")]
for name, body in INIT:
    print("sent", name, h.write(cmd(body)), "bytes", flush=True)

start = beat = time.time()
while time.time() - start < SECS:
    if time.time() - beat > 10:
        h.write(cmd(b"CONNECT"))
        beat = time.time()
    r = h.read(512)
    if r:
        t = round(time.time() - start, 1)
        if r[:3] == b"ACK" and r[5:7] == b"OK":
            print(t, "key", r[9], "down" if r[10] else "up", "raw", r[:16].hex(), flush=True)
        else:
            print(t, "raw", r[:32].hex(), r[:32], flush=True)
    time.sleep(0.005)
print("done", flush=True)
