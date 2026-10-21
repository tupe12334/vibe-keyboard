#!/bin/sh
# Tests the pre-commit cargo skip. Runs the real hook and matcher in temp git repos with a fake cargo.
# Run: sh scripts/test/precommit-skip.sh
set -eu

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
for v in $(env | sed -n 's/^\(GIT_[A-Za-z_]*\)=.*/\1/p'); do unset "$v"; done

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/bin"
# shellcheck disable=SC2016 # the fake cargo reads $CARGO_LOG when it runs, not now
printf '#!/bin/sh\necho "cargo $*" >> "$CARGO_LOG"\n' > "$TMP/bin/cargo"
chmod +x "$TMP/bin/cargo"
export PATH="$TMP/bin:$PATH"

fails=0
n=0

# new_repo DIR: fresh git repo with the hook and matcher copied in, no cargo log yet.
new_repo() {
  mkdir -p "$1/scripts" "$1/.cargo-husky/hooks"
  cp "$ROOT/scripts/precommit-needs-cargo.sh" "$1/scripts/"
  cp "$ROOT/.cargo-husky/hooks/pre-commit" "$1/.cargo-husky/hooks/"
  git -C "$1" init -q
}

# run_hook DIR: runs the hook; sets out and rc.
run_hook() {
  if out=$(cd "$1" && CARGO_LOG="$1/cargo.log" sh .cargo-husky/hooks/pre-commit 2>&1); then
    rc=0
  else
    rc=$?
  fi
}

# verdict NAME EXPECT: EXPECT is run (cargo called) or skip (cargo not called, skip line printed).
verdict() {
  name=$1
  expect=$2
  if [ "$expect" = run ]; then
    if [ -s "$R/cargo.log" ] && [ "$rc" -eq 0 ]; then
      echo "ok   $name"
    else
      echo "FAIL $name (expected cargo to run, rc=$rc)"
      fails=$((fails + 1))
    fi
  else
    case "$out" in
      *"Skipping cargo steps"*)
        if [ ! -e "$R/cargo.log" ] && [ "$rc" -eq 0 ]; then
          echo "ok   $name"
        else
          echo "FAIL $name (cargo ran or rc=$rc)"
          fails=$((fails + 1))
        fi
        ;;
      *)
        echo "FAIL $name (no skip line printed)"
        fails=$((fails + 1))
        ;;
    esac
  fi
}

# check NAME EXPECT PATH...: stages the given paths as empty files, then runs the hook.
check() {
  name=$1
  expect=$2
  shift 2
  n=$((n + 1))
  R="$TMP/case$n"
  new_repo "$R"
  for f in "$@"; do
    mkdir -p "$R/$(dirname "$f")"
    # Keep existing files (the copied hook) as they are; create the rest empty.
    if [ ! -e "$R/$f" ]; then : > "$R/$f"; fi
    git -C "$R" add -- "$f"
  done
  run_hook "$R"
  verdict "$name" "$expect"
}

check "docs-only staged skips cargo" skip README.md docs/guide.md
check "mixed docs and frontend skips cargo" skip docs/guide.md apps/desktop/src/App.tsx
check ".rs change runs cargo" run src/main.rs
check "nested .rs runs cargo" run apps/desktop/src-tauri/src/lib.rs
check "Cargo.toml runs cargo" run Cargo.toml
check "Cargo.lock runs cargo" run Cargo.lock
check "build.rs runs cargo" run build.rs
check "rust-toolchain.toml runs cargo" run rust-toolchain.toml
check ".cargo config runs cargo" run .cargo/config.toml
check "deny.toml runs cargo" run deny.toml
check "tauri src-tauri runs cargo" run apps/desktop/src-tauri/tauri.conf.json
check "bindings.ts alone runs cargo" run apps/desktop/src/bindings.ts
check "embedded asset runs cargo" run assets/icons/png/back.png
check "hook file runs cargo" run .cargo-husky/hooks/pre-commit
check "matcher script runs cargo" run scripts/precommit-needs-cargo.sh
check "tests fixture runs cargo" run tests/fixtures/real_profile.toml
check "nothing staged runs cargo" run

# Rename out of src/ must still run cargo: the old path is staged as a deletion.
n=$((n + 1))
R="$TMP/case$n"
new_repo "$R"
mkdir -p "$R/src"
: > "$R/src/a.rs"
git -C "$R" add -- src/a.rs
git -C "$R" -c user.name=test -c user.email=test@example.invalid commit -q -m init
git -C "$R" mv src/a.rs docs-a.md
run_hook "$R"
verdict "rename out of src/ runs cargo" run

# A git failure in the matcher must run cargo, not skip it. Docs-only stage, fake git exits 128.
n=$((n + 1))
R="$TMP/case$n"
new_repo "$R"
: > "$R/README.md"
git -C "$R" add -- README.md
mkdir -p "$TMP/badgit"
printf '#!/bin/sh\nexit 128\n' > "$TMP/badgit/git"
chmod +x "$TMP/badgit/git"
OLDPATH=$PATH
PATH="$TMP/badgit:$PATH"
run_hook "$R"
PATH=$OLDPATH
verdict "git failure in matcher runs cargo" run

if [ "$fails" -gt 0 ]; then
  echo "$fails check(s) failed"
  exit 1
fi
echo "all checks passed"
