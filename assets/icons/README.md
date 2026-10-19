# Icon pack

Original, simple glyphs for key images. Drawn for this repo; no third-party
icon set or brand logo is used.

## Icons

| Name | Glyph |
|------|-------|
| back | left arrow |
| forward | right arrow |
| home | house |
| page-next | page with right arrow |
| page-prev | page with left arrow |
| terminal | window with `>_` prompt |
| browser | globe |
| search | magnifier |
| numpad | 3×3 grid |
| brightness-up | filled sun with 8 rays |
| brightness-down | filled sun with 4 rays |
| settings | gear |
| centy | board with task rows |
| claude | chat bubble with sparkle |
| folder | folder |
| play-pause | play triangle and pause bars |
| mute | speaker with cross |

## Files

- `svg/<name>.svg`: source, viewBox `0 0 85 85`, single colour via `currentColor`.
- `png/<name>.png`: 85×85 white on transparent, generated from the SVG.

## Regenerate PNGs

Install `librsvg` (`brew install librsvg`), then run:

```bash
assets/icons/render.sh
```

Edit an SVG, run the script, and commit both files.

## License

Same as this repository. All glyphs are original work.
