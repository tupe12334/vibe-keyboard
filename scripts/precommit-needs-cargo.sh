#!/bin/sh
# Exit 0 when a staged path could change the Rust build, 1 when none can.
# Used by the pre-commit hook. Fail safe: an empty staged list counts as Rust-affecting.
set -eu

changed=$(git diff --cached --name-only --no-renames)
if [ -z "$changed" ]; then
  exit 0
fi

needs=1
while IFS= read -r path; do
  case "$path" in
    *.rs | *.toml | *.lock | *rust-toolchain*) needs=0 ;;
    .cargo/* | */.cargo/* | .cargo-husky/* | scripts/* | tests/* | src/* | assets/*) needs=0 ;;
    apps/desktop/src-tauri/* | apps/desktop/src/bindings.ts | apps/desktop/dist/*) needs=0 ;;
    Makefile) needs=0 ;;
  esac
done <<EOF
$changed
EOF

exit "$needs"
