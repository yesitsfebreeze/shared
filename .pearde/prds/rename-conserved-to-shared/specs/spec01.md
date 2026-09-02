---
complexity: 8
footprint:
  - shared/
  - shared/
  - Cargo.toml
  - Cargo.lock
---

# spec01 — the directory and the package move together, and the suite does not move

`git mv conserved shared`, `name = "shared"`, `members = ["shared"]`, every
`conserved` token inside the crate corrected except the five the PRD names, then
`cargo fmt --all`. The crate's own suite must come out at **exact parity** with
the count measured immediately before the move.

This unit is self-contained: it touches only the crate, the two manifests and
the lock. No prose outside the crate is in it — that is spec02 — and nothing is
committed or pushed — that is spec03.

## `main` is broken right now, and landing this spec is what fixes it

**Read this before anything else.** `git mv` stages its rename into the shared
index. While this probe was running, a concurrent node on another PRD committed
— and swept the staged rename into **`1e21445` "p5-adoption/mitosys — the seven
boxes measured…"**, a commit about something else entirely. That commit carries
all 19 files as `R100 conserved/… → shared/…` and **not** the root
`Cargo.toml`, which was still an unstaged edit.

So `main` now has the crate at `shared/` and a workspace that still says
`members = ["conserved"]`. `reproduced`, fixture: a detached `git worktree` at
`1e21445`, `cargo metadata --no-deps` →

```
exit=101
error: failed to load manifest for workspace member `…/conserved`
  referenced by workspace at `…/Cargo.toml`
Caused by:
  failed to read `…/conserved/Cargo.toml`
```

Every cargo command fails on a clean checkout of `main`. CI is red. The working
tree is green only because the one-line fix is sitting in it uncommitted.

Two consequences:

1. **Land this spec before anything pushes.** The fix is `members = ["shared"]`
   and it is already written; it just needs to be committed.
2. **Prefer plain `mv` over `git mv` while other nodes may commit**, or stage
   nothing until the unit is whole. Rename detection survives a plain `mv`
   anyway — git computes it at diff time from content similarity, not from the
   index — so `git mv` bought nothing here and cost a broken `main`.

## What already stands

**The content half of this spec is in the tree, uncommitted; the directory move
is already committed, by accident, in `1e21445`.** Re-run every measurement
below and quote your own output; do not inherit these numbers.

- `git ls-files conserved` was **19** before the move and `git ls-files shared/`
  is **19** after. The PRD's pinned figure holds today.
- `conserved/tests/done_boxes_are_ticked.rs` moved with **0 lines changed** —
  `git diff HEAD --stat` renders it as a pure rename. `reproduced`, fixture
  `git diff HEAD --stat -- shared/tests/done_boxes_are_ticked.rs`.
- The token pass was a per-file perl that sentinels the survivors first
  (`b"conserved"` → sentinel, `conserved-core` → sentinel), renames, then
  restores. Afterwards `git grep -nI conserved -- shared/` returns **exactly 5**
  lines and they are the five the PRD names:

```
shared/src/clock.rs:86        conserved-core     (proper noun)
shared/tests/clock_source.rs:48  conserved-core  (proper noun)
shared/tests/content_id.rs:31    b"conserved"    (blake3 input)
shared/tests/content_id.rs:108   b"conserved"    (blake3 input)
shared/tests/content_id_serde.rs:30 b"conserved" (blake3 input)
```

- `shared/tests/smoke.rs`'s test function `conserved_links` became
  `shared_links`. It is an identifier, so rule 1 of `learnings/crate-name.md`
  corrects it in place.
- `Cargo.lock` regenerated on the first `cargo build`; its `name = "shared"`
  entry moved from line 92 to line 457 on the alphabetical re-sort. It was not
  hand-edited.
- `cargo fmt --all` reformatted **exactly the three files the PRD predicts** and
  no others — `shared/tests/{clock_serde,content_id_props,content_id_serde}.rs`,
  the import-block re-sort where `shared` now sorts after `proptest`/`serde`.
  `reproduced`, fixture `cargo fmt --check --all` before the format.

### Parity is `reproduced` — and the baseline moved underneath the probe

Fixture: `cargo test --workspace --no-fail-fast` in `/Users/feb/dev/infra/shared`.

| point | passed | failed | population |
|---|---|---|---|
| `main`, measured at the start of the probe | 84 | 1 | 85 |
| after the move + rename + fmt, gate excluded | 85 | 0 | 85 |

**The test population is 85 on both sides. The rename moved neither the
population nor any result** — that is the parity claim, and it is `reproduced`.

What did move is `every_done_prd_has_no_unticked_box`, and **not because of this
work**. It failed at the start of the probe and passes now, on a
`prds/p5-adoption/mitosys/prd.md` that is byte-identical to `HEAD`
(`shasum` of the worktree file equals `git show HEAD:…| shasum`,
`c03a1e4a68caac4de42f124486657573bb513bcb`) and carries zero unticked boxes.
Something outside this session closed it while the probe was running. The PRD
pins 84/1 as the baseline; **do not.** Re-measure immediately before you start
and require *your* pair afterwards — this is exactly the volatility the PRD
warns about, caught in the act.

`cargo fmt --check --all` clean and
`cargo clippy --workspace --all-targets -- -D warnings` clean after the rename.

### A defect in the PRD's own verify, found by running it

**This workspace has exactly one member**, so after the rename `-p shared` *is*
`--workspace`. The PRD's *"Not `just check`, and not `cargo test --workspace`"*
scoping therefore buys nothing — the two commands run the identical set, and
`reproduced`: both returned 84/1 at the start of the probe. The PRD's
`verify: "cargo test -p shared"` cannot be scoped around
`done_boxes_are_ticked`; only `--skip` can. The boxes below assert **parity**
rather than green, and name the skip form for the case where the board gate is
red again for someone else's reason.

## Acceptance

- [x] Before moving anything, record the two baselines and paste them into your
      report: `git ls-files conserved | wc -l` and the passed/failed pair from
      `cargo test --workspace --no-fail-fast`. If the file count is not 19 the
      crate has changed since 2026-08-28 — record the new number and match
      *that*, per the PRD's own instruction
- [x] The move is done and nothing was lost: `git ls-files shared/ | wc -l`
      equals the pre-move count, `ls conserved` reports no such directory, and
      `shared/tests/done_boxes_are_ticked.rs` is content-identical to the file
      that was `conserved/tests/done_boxes_are_ticked.rs`
- [x] **`main` loads a workspace again.** (`dfc98fba`, detached worktree: `cargo metadata` exit 0, no `conserved/`.) In a detached worktree at the commit
      that lands this spec, `cargo metadata --no-deps --format-version 1` exits
      **0**. It exits 101 at `1e21445` today — see the section above; this box is
      the one that closes that hole, and a `cargo` command run in the dirty
      working tree does not test it
- [x] `Cargo.toml` line 3 reads `members = ["shared"]` and its `serde` comment
      says *"Optional in `shared`"*; `shared/Cargo.toml` reads `name = "shared"`
      and its `[features]` comment says `cargo tree -p shared --edges normal`.
      `description = "Domain-free primitives shared by the Rust trees."` is
      **byte-identical** to its pre-change form
- [x] `git grep -nI conserved -- shared/` returns **exactly 7** lines
      (`in-crate survivors: 7`, run under `sh -e` on 2026-09-02): the five
      listed above plus `shared/tests/landed_rev_is_published.rs:13` and `:29`,
      both the address `.pearde/prds/rename-conserved-to-shared/prd.md`,
      immutable under rule 3 — counted with `awk`, not eyeballed. Amended from 5
      at collect, when spec03's gate file was still untracked and plain
      `git grep` could not see it. It is tracked as of `dfc98fba`, so
      `--untracked` is no longer required
- [x] `Cargo.lock` was regenerated by cargo, not edited: `git diff -- Cargo.lock`
      shows only the package-name/ordering change, and
      `git grep -nI conserved -- Cargo.lock` returns nothing
- [x] **Parity, not an absolute.** `cargo test --workspace --no-fail-fast`
      returns the same `passed + failed` **total** as box 1 — the population must
      not move — and no test that passed before now fails. If
      `every_done_prd_has_no_unticked_box` is red, it is the only permitted
      failure and it is not yours: `prds/p5-adoption/mitosys/prd.md` owns it.
      It was red at 84/1 when this probe started and green at 85/0 when it
      finished, from an external cause; measure, do not assume
- [x] `cargo test -p shared -- --skip every_done_prd_has_no_unticked_box` exits
      **0** — the form that is green regardless of the board gate's state. Plain
      `cargo test -p shared` is not scoped around it, because this workspace has
      one member and `-p shared` is `--workspace`
- [x] `cargo fmt --check --all` clean, and the files `cargo fmt --all` changed
      are exactly `shared/tests/{clock_serde,content_id_props,content_id_serde}.rs`
      — if a fourth file reformats, say which and why before proceeding
- [x] `cargo clippy --workspace --all-targets -- -D warnings` exits 0

## Verify and Proof

Every line is correct under `sh -e`: the runner executes this block as one
script and stops at the first non-zero exit. A grep whose **wanted** result is
zero matches therefore never appears bare — a bare `git grep` exits 1 on
success here, and the runner reads that as failure.

```sh
set -e
cd /Users/feb/dev/infra/shared

# the in-crate census — exactly 7 (was 5 before spec03 added the gate file)
git grep -nI conserved -- shared/
git grep -cI conserved -- shared/ | awk -F: '{s+=$2} END {print "in-crate survivors:", s+0}'

# manifests: the names are right...
git grep -nI 'members\|^name = ' -- Cargo.toml shared/Cargo.toml
# ...and the old name is gone. Zero matches is the PASS, so assert the negation.
! git grep -qI conserved -- Cargo.toml Cargo.lock shared/Cargo.toml
echo "manifests + lock: 0 conserved"

# main must load a workspace — run against the COMMIT, not the dirty worktree
W=$(mktemp -d)/wt
git worktree add --detach "$W" HEAD >/dev/null
md=0
( cd "$W" && cargo metadata --no-deps --format-version 1 >/dev/null ) || md=$?
echo "metadata exit=$md"
git worktree remove --force "$W"
[ "$md" -eq 0 ]

# parity — compare against the pair recorded in box 1
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test result:' \
  | awk '{p+=$4; f+=$6} END {print "passed="p, "failed="f}'

# the green form
cargo test -p shared -- --skip every_done_prd_has_no_unticked_box

cargo fmt --check --all
cargo clippy --workspace --all-targets -- -D warnings
echo "spec01 verify: OK"
```
