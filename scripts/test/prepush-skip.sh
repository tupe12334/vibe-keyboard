#!/bin/sh
# Tests the pre-push cargo skip. Runs the real hook and matcher in temp git repos with fake cargo, linecheck and cargo-machete.
# Run: sh scripts/test/prepush-skip.sh
set -eu

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
for v in $(env | sed -n 's/^\(GIT_[A-Za-z_]*\)=.*/\1/p'); do unset "$v"; done

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/bin" "$TMP/target"
# shellcheck disable=SC2016 # the fake cargo reads $CARGO_LOG when it runs, not now
printf '#!/bin/sh\necho "cargo $*" >> "$CARGO_LOG"\n' > "$TMP/bin/cargo"
printf '#!/bin/sh\nexit 0\n' > "$TMP/bin/linecheck"
printf '#!/bin/sh\nexit 0\n' > "$TMP/bin/cargo-machete"
chmod +x "$TMP/bin/"*
export PATH="$TMP/bin:$PATH"
# Fixed target dir and no free-space floor: the disk guard then makes no git call and never blocks.
export CARGO_TARGET_DIR="$TMP/target" VK_MIN_FREE_KIB=0

ZERO=0000000000000000000000000000000000000000
UNKNOWN=1111111111111111111111111111111111111111

fails=0
n=0

# new_repo DIR: fresh repo with the hook, matcher and shellcheck inputs copied in.
# Base commit holds src/a.rs; BASE is its sha and origin/main points at it.
new_repo() {
  mkdir -p "$1/scripts/test" "$1/.cargo-husky/hooks" "$1/src"
  cp "$ROOT/scripts/precommit-needs-cargo.sh" "$1/scripts/"
  cp "$ROOT/scripts/test/precommit-skip.sh" "$ROOT/scripts/test/prepush-skip.sh" "$1/scripts/test/"
  cp "$ROOT/.cargo-husky/hooks/pre-commit" "$ROOT/.cargo-husky/hooks/pre-push" "$1/.cargo-husky/hooks/"
  printf '#!/bin/sh\nexit 0\n' > "$1/scripts/coverage.sh"
  chmod +x "$1/scripts/coverage.sh"
  echo 'fn main() {}' > "$1/src/a.rs"
  git -C "$1" init -q
  git -C "$1" add -A
  git -C "$1" -c user.name=test -c user.email=test@example.invalid commit -q -m base
  git -C "$1" update-ref refs/remotes/origin/main HEAD
  BASE=$(git -C "$1" rev-parse HEAD)
}

# feature_commit DIR PATH...: writes each path as a new file and commits; LSHA is the new sha.
feature_commit() {
  d=$1
  shift
  for f in "$@"; do
    mkdir -p "$d/$(dirname "$f")"
    echo change > "$d/$f"
    git -C "$d" add -- "$f"
  done
  git -C "$d" -c user.name=test -c user.email=test@example.invalid commit -q -m feature
  LSHA=$(git -C "$d" rev-parse HEAD)
}

# run_hook DIR STDIN_FILE: runs the hook; sets out and rc.
run_hook() {
  if out=$(cd "$1" && CARGO_LOG="$1/cargo.log" sh .cargo-husky/hooks/pre-push < "$2" 2>&1); then
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

# case_push NAME EXPECT KIND PATH...: one feature commit, one pushed ref.
# KIND: existing (remote sha = BASE), new (remote sha = zeros), unknown (remote sha not in repo), delete (local sha = zeros).
case_push() {
  name=$1
  expect=$2
  kind=$3
  shift 3
  n=$((n + 1))
  R="$TMP/case$n"
  new_repo "$R"
  feature_commit "$R" "$@"
  lsha=$LSHA
  rsha=$BASE
  case "$kind" in
    new) rsha=$ZERO ;;
    unknown) rsha=$UNKNOWN ;;
    delete) lsha=$ZERO ;;
  esac
  printf 'refs/heads/feat %s refs/heads/feat %s\n' "$lsha" "$rsha" > "$TMP/stdin$n"
  run_hook "$R" "$TMP/stdin$n"
  verdict "$name" "$expect"
}

case_push "docs-only push skips cargo" skip existing README.md docs/guide.md
case_push "mixed docs and frontend push skips cargo" skip existing docs/guide.md apps/desktop/src/App.tsx
case_push ".rs push runs cargo" run existing src/main.rs
case_push "Cargo.toml push runs cargo" run existing Cargo.toml
case_push "new branch with .rs runs cargo" run new src/lib.rs
case_push "new branch docs-only skips cargo" skip new docs/guide.md
case_push "uncomputable range runs cargo" run unknown docs/guide.md
case_push "deleted branch runs cargo" run delete docs/guide.md

# Rename out of src/: git diff with --no-renames shows the src/ deletion, so cargo must run.
n=$((n + 1))
R="$TMP/case$n"
new_repo "$R"
git -C "$R" mv src/a.rs docs-a.md
git -C "$R" -c user.name=test -c user.email=test@example.invalid commit -q -m rename
LSHA=$(git -C "$R" rev-parse HEAD)
printf 'refs/heads/feat %s refs/heads/feat %s\n' "$LSHA" "$BASE" > "$TMP/stdin$n"
run_hook "$R" "$TMP/stdin$n"
verdict "rename out of src/ runs cargo" run

# Empty stdin (no refs): nothing to judge, cargo must run.
n=$((n + 1))
R="$TMP/case$n"
new_repo "$R"
: > "$TMP/stdin$n"
run_hook "$R" "$TMP/stdin$n"
verdict "empty stdin runs cargo" run

# Forced git failure: fake git exits 128 for every call, so the range cannot be computed. Full gate must run.
n=$((n + 1))
R="$TMP/case$n"
new_repo "$R"
feature_commit "$R" docs/guide.md
printf 'refs/heads/feat %s refs/heads/feat %s\n' "$LSHA" "$BASE" > "$TMP/stdin$n"
mkdir -p "$TMP/badgit"
printf '#!/bin/sh\nexit 128\n' > "$TMP/badgit/git"
chmod +x "$TMP/badgit/git"
OLDPATH=$PATH
PATH="$TMP/badgit:$PATH"
run_hook "$R" "$TMP/stdin$n"
PATH=$OLDPATH
verdict "forced git failure runs full gate" run

if [ "$fails" -gt 0 ]; then
  echo "$fails check(s) failed"
  exit 1
fi
echo "all checks passed"
