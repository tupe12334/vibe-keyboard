#!/usr/bin/env bash
# Regenerate png/<name>.png (85x85, white on transparent) from svg/<name>.svg.
# Needs: rsvg-convert (brew install librsvg).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
svg_dir="$here/svg"
png_dir="$here/png"
fg="#ffffff"

if ! command -v rsvg-convert >/dev/null 2>&1; then
    echo "render.sh: rsvg-convert not found (brew install librsvg)" >&2
    exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

shopt -s nullglob
svgs=("$svg_dir"/*.svg)
if [ "${#svgs[@]}" -eq 0 ]; then
    echo "render.sh: no .svg files in $svg_dir" >&2
    exit 1
fi

mkdir -p "$png_dir"
count=0
for svg in "${svgs[@]}"; do
    name="$(basename "$svg" .svg)"
    # Bind the single foreground colour: rsvg defaults currentColor to black.
    sed "s/currentColor/$fg/g" "$svg" > "$tmp/$name.svg"
    rsvg-convert --width 85 --height 85 --keep-aspect-ratio \
        --format png --output "$png_dir/$name.png" "$tmp/$name.svg"
    count=$((count + 1))
done
echo "render.sh: wrote $count PNG files to $png_dir"
