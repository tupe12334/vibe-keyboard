# AKP153E tools

Scripts from the StreamDock.app protocol capture. Facts are in [`device.md`](../../device.md).

## interact.py

Initializes the device and prints key events.

Requires `brew install hidapi`. No root or setuid.

```
DYLD_LIBRARY_PATH=/opt/homebrew/lib uv run --with hid python interact.py 60
```

The argument is the run time in seconds.

## draw.py

Draws the key number on all 15 keys.

```
DYLD_LIBRARY_PATH=/opt/homebrew/lib uv run --with hid --with pillow python draw.py [size=85] [rotation=90]
```

## hidspy.c

IOKit traffic logger. Inject it into StreamDock.app to log HID reports.

Build:

```
clang -arch arm64 -dynamiclib -framework IOKit -framework CoreFoundation -o spy.dylib hidspy.c
```

Run:

```
cd /Applications/StreamDock.app/Contents/MacOS && DYLD_INSERT_LIBRARIES=/path/to/spy.dylib HIDSPY_LOG=/tmp/traffic.log ./StreamDock
```


StreamDock has the entitlements `allow-dyld-environment-variables` and `disable-library-validation`, so injection works. Its hidapi is linked statically inside `libSDLibrary`. The hooks therefore sit at IOKit (`IOHIDDeviceSetReport`, input report callbacks), not at `hid_write`.
