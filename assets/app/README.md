# App bundle icon

`AppIcon.svg` is the source. `AppIcon.icns` is generated from it. The grid
shows the device's 5 × 3 key layout.

## Rebuild the icns

Needs `librsvg` (`rsvg-convert`) and macOS `iconutil`.

```bash
A=assets/app
SET="$(mktemp -d)/AppIcon.iconset"
mkdir -p "$SET"
for spec in 16:icon_16x16 32:icon_16x16@2x 32:icon_32x32 64:icon_32x32@2x \
            128:icon_128x128 256:icon_128x128@2x 256:icon_256x256 \
            512:icon_256x256@2x 512:icon_512x512 1024:icon_512x512@2x; do
  px="${spec%%:*}"
  name="${spec#*:}"
  rsvg-convert -w "$px" -h "$px" -f png -o "$SET/$name.png" "$A/AppIcon.svg"
done
iconutil -c icns "$SET" -o "$A/AppIcon.icns"
```

`packaging/macos/build-app.sh` (PR #78) is meant to consume `AppIcon.icns`:
copy it into `Contents/Resources/` and set `CFBundleIconFile` to `AppIcon`.
That wiring is not on this branch yet; it lands with PR #78.

## License

Same as this repository. Original work.
