#!/usr/bin/env bash
# The acceptance criterion of .mi/prd/p0-foundation, made runnable: a fresh
# clone of this repository builds and tests green. Catches what an in-place
# build cannot — a file the build needs that was never committed, or one
# .gitignore excludes.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
git -C "$repo" rev-parse --git-dir >/dev/null

if [ -n "$(git -C "$repo" status --porcelain)" ]; then
	echo "working tree is dirty — a clone tests HEAD, not what you have:" >&2
	git -C "$repo" status --short >&2
	exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
git clone --quiet "$repo" "$work/clone"

unset CARGO_TARGET_DIR   # or the clone reuses this tree's build output
cd "$work/clone"
cargo build --workspace
cargo test --workspace
echo "fresh clone of $(git -C "$repo" rev-parse --short HEAD) builds and tests green"
