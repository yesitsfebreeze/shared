# goal

Close this ticket's stated acceptance criterion — *"`cargo build --workspace &&
cargo test --workspace` passes **from a fresh clone** of this repository"* —
with a check that actually clones. Building in place cannot fail the way a
fresh clone can: it never notices a file the build needs that was never
committed, or one `.gitignore` excludes.

The ticket's own frontmatter `verify:` line (`cargo fmt … && cargo clippy … &&
cargo test --workspace && git rev-parse --git-dir`) tests the working
directory, not a clone. That gap is what this spec closes; the check becomes a
committed script so the criterion stays runnable after p0 closes, and so p1
(whose whole job is proving the distribution *mechanism* on a clone) inherits
it rather than re-inventing it.

## Files and dirs

- `scripts/fresh-clone-check.sh` — new, executable (`chmod +x`), committed:

  ```bash
  #!/usr/bin/env bash
  # The acceptance criterion of .mi/prds/p0-foundation, made runnable: a fresh
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
  ```

  Tabs, not spaces, inside the script body — `.editorconfig` applies to `*`.

- A commit adding it (spec04's initial commit has already landed by now).
- `README.md` — one line under §"The board" naming the script as how the
  fresh-clone criterion is checked. No other prose change.

Not touched: `.mi/prds/p0-foundation/prd.md` frontmatter — the orchestrator owns
ticket state, including the `verify:` line. If that line is to be widened to
`… && ./scripts/fresh-clone-check.sh`, the orchestrator makes that edit, not
the implementer.

## Acceptance

- [x] `scripts/fresh-clone-check.sh` exists, is executable, and is tracked by
      git (`git ls-files` matches it).
- [x] Running it exits 0 and prints the short HEAD it verified.
- [x] It actually clones: the script contains `git clone` and builds in a
      directory under `mktemp -d`, not in the repo. It does not run
      `cargo build` in the working tree.
- [x] It cannot be fooled by a shared target directory — `CARGO_TARGET_DIR` is
      unset inside the script, and the clone's `target/` is created fresh.
- [x] It fails loudly on a dirty tree: with an untracked file present, the
      script exits non-zero and says the tree is dirty (a clone would have
      tested something other than what the implementer has).
- [x] It fails when the criterion is actually broken *and the working tree
      looks fine*: temporarily add `conserved/src/` to `.gitignore`,
      `git rm -r --cached conserved/src`, commit (the tree is now clean, the
      in-place build still passes) — the script must exit non-zero because the
      clone cannot build. Then `git reset --hard HEAD~1` and confirm it passes
      again. This is a check that the check works, not a change to keep.
- [x] The whole ticket passes end to end: from the repo root,
      `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
      warnings` and `scripts/fresh-clone-check.sh` all exit 0.

## est

0.5

verify: `sh -c 'set -e; test -x scripts/fresh-clone-check.sh; git ls-files --error-unmatch scripts/fresh-clone-check.sh >/dev/null; grep -q "git clone" scripts/fresh-clone-check.sh; grep -q "mktemp -d" scripts/fresh-clone-check.sh; grep -q "unset CARGO_TARGET_DIR" scripts/fresh-clone-check.sh; cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; ./scripts/fresh-clone-check.sh; touch .fresh-clone-dirty-probe; if ./scripts/fresh-clone-check.sh >/dev/null 2>&1; then rm -f .fresh-clone-dirty-probe; echo "dirty-tree guard does not fire"; exit 1; fi; rm -f .fresh-clone-dirty-probe; echo "fresh-clone gate holds"'`
