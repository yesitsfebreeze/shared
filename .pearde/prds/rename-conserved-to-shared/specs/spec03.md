---
complexity: 12
footprint:
  - shared/tests/landed_rev_is_published.rs
  - .pearde/prds/rename-conserved-to-shared/prd.md
---

# spec03 — the rendezvous cannot be skipped: a gate that fails until the renamed rev is on a remote

`mitosys`, `model` and `realm` each pin this repository by git sha and cannot
start until §Landed names the rev this PRD publishes. **No compiler enforces
that, and a vendored consumer build does not either** — `cargo` takes whatever
the source-replacement directory provides and never checks that the package at
that rev is called that (`learnings/crate-name.md` §"A vendored consumer build
does not force the rev bump", `reproduced` there).

This unit adds the producing side of that gate: a test in this repo that fails
whenever the PRD claims to be done but the rev it names is missing, is a
placeholder, is unreachable from any remote, or predates the rename. The rev
bump stops being a step someone has to remember.

**This is the highest-risk unit in the PRD and the only one three other trees
wait on.** Land spec01 and spec02 first.

## What already stands

**`shared/tests/landed_rev_is_published.rs` is in the tree, untracked.** It
compiles, `cargo fmt --check --all` and clippy at `-D warnings` are clean with
it in place, and it adds exactly **5 tests** — one gate plus four parser tests.
Measured by moving the file aside and re-counting: 85 without it, 90 with,
0 failures either way.

The gate reads `prds/rename-conserved-to-shared/prd.md` and returns early unless
its frontmatter says `state: done`. Marking the PRD done is what arms it, which
is exactly the transition the three consumers are waiting on. Once armed it
asserts the §Landed rev is a 40-hex sha, is a commit in this repo, is named by
`git branch -r --contains` (so a clone can fetch it), and that
`git show <rev>:shared/Cargo.toml` contains `name = "shared"` (so it is the
*renamed* rev, not an older one pasted in).

### The gate was proved to fail, not just to pass — three negative fixtures

A check written from the answer passes on the answer. Each failure path was
exercised by temporarily editing the PRD, running the test, and restoring the
file byte-for-byte (`shasum` equal before and after, both times).

| fixture | §Landed rev | result |
|---|---|---|
| `state: done`, placeholder untouched | `TBD` | **FAILS** — *"not a 40-character sha"* |
| `state: done`, a real pre-rename commit | `70d7e15c…` (tip of `origin/main`) | **FAILS** — *"has no `shared/Cargo.toml`… that rev predates `git mv`"* |
| `state: done`, a real unpushed commit | `0ce4fa4f…` (local `HEAD`) | **FAILS** — *"is on NO remote branch… nothing that clones the repo can ever fetch it"* |
| `state: analyzing` (today) | `TBD` | passes, by returning early |

`reproduced`, fixture `cargo test -p shared --test landed_rev_is_published`.

### `origin/main` is 10 commits behind — and it breaks one of the PRD's boxes

Measured today: `git merge-base origin/main HEAD` is
**`70d7e15cd21c6017ec928c63697d0c7f42f53a20`**, and `git rev-list --count
origin/main..HEAD` is **10**. That merge-base is not a placeholder — it is
literally the rev `mitosys` pins today.

The PRD's acceptance box 6 says to take the base from `git merge-base origin/main
HEAD` and require `git diff --name-only <base>..HEAD -- prds/ .mi/gantt/` to be
empty. **It is not empty and cannot be**: 13 files under `prds/` differ between
`origin/main` and `HEAD` because of ten unrelated commits that have not been
pushed. The box as written can never pass and does not measure this PRD's
changes at all. The check that does is `git status --porcelain -- prds/
.mi/gantt/` (worktree against `HEAD`), which is what spec02 uses.

Two consequences the implementer must plan for:

1. **Pushing this work publishes ten other commits with it.** That is fine and
   probably overdue, but it is not a silent side effect — say so in the report.
2. The rev the consumers pin is the sha of the commit carrying the rename, which
   must be pushed **before** §Landed can name it and pass the gate. That forces
   the ordering in the boxes below: commit the rename, push, then a second small
   commit writing the sha of the first into §Landed.

## Acceptance

- [x] `shared/tests/landed_rev_is_published.rs` is tracked (`git ls-files --error-unmatch` at `dfc98fba`), and
      `cargo test -p shared --test landed_rev_is_published` reports **5 passed**
- [x] The gate is proved to **fail**, not merely to pass. Re-run all three
      negative fixtures from the table above against your own tree, quote each
      panic message, and restore `prd.md` byte-identically each time —
      `shasum prds/rename-conserved-to-shared/prd.md` before and after must
      match. A gate only ever seen green is not evidence
- [x] With the gate in place the suite population is the spec01 population
      **plus exactly 5** — measured 85 → 90 on 2026-08-28, by moving the gate
      file aside and re-counting — and no test that passed without it fails with
      it. `cargo fmt --check --all` and
      `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] **Pushed.** `git branch -r --contains <rename-commit>` names `origin/main`,
      and the report states how many commits the push published
      (`git rev-list --count origin/main..HEAD` before pushing — it was 10 plus
      this PRD's own on 2026-08-28).
      `git branch -r --contains dfc98fba70039863797f7185d860ef392becb21f` →
      `origin/HEAD -> origin/main` / `origin/main`. The push landed
      **2026-09-02 09:27:44 +0200**, moving `origin/main` from `24997ea` to
      `978dbf6` and publishing **7** commits (`git rev-list --count
      24997ea..978dbf6` = 7), three of them this PRD's: `0567f75` (claim),
      `dfc98fb` (the rename), `673cc25` (the boxes it closed). Not the 10+1
      predicted on 2026-08-28 — `24997ea` was itself pushed at 13:57 that day,
      between the measurement and the rename commit, so it had already left the
      backlog. `git rev-list --count origin/main..HEAD` is now **0**
- [x] §Landed's `- rev:` is the 40-character sha of the commit that carries the
      rename, and `- pushed to origin/main:` names the date. `TBD` appears in
      neither. `- rev:` is
      `dfc98fba70039863797f7185d860ef392becb21f` (40 hex), and
      `- pushed to origin/main:` now reads `2026-09-02`.
      `grep -n 'TBD' .pearde/prds/rename-conserved-to-shared/prd.md` returns
      nothing — no `TBD` survives anywhere in the file, not just on those lines
- [x] The gate passes **for the right reason**, with the PRD at `state: done`:
      `cargo test -p shared --test landed_rev_is_published` green while
      `git show $(sed -n 's/^- rev: `\(.*\)`$/\1/p' .pearde/prds/rename-conserved-to-shared/prd.md):shared/Cargo.toml | grep 'name'`
      prints `name = "shared"`
      **(path corrected from `prds/…` to `.pearde/prds/…`: commit `27db1b7`
      moved the board and the gate's own `PRD` const with it. The command as
      first written reads a file that does not exist, and would fail the box for
      a path reason rather than a substance one. Nothing else in the check
      changed.)**

      **Proved on a fixture worktree, because on the live tree the box is a
      deadlock.** `pearde collect` will not write `state: done` while any box is
      open, and this box cannot be true until the state *is* `done`. Neither
      side can move first, so the proof is done on a copy and the collect
      re-runs the live gate after it writes `done`.

      The fixture: `git worktree add <scratch> HEAD` at `27db1b7`, this round's
      uncommitted `prd.md` and `spec03.md` copied in, `state: done` written
      **only in the copy**. The live `prd.md` frontmatter was never edited —
      `shasum` of it is `312accff2588df8b23719c6915156e1e7c02db05` before and
      after, and it still reads `state: claimed`. Worktree removed with
      `git worktree remove --force`; `git worktree list` is back to one entry.

      **The pass**, armed — no early return, all four assertions run:

      ```
      $ sed -n '2p' .pearde/prds/rename-conserved-to-shared/prd.md
      state: done
      $ cargo test -p shared --test landed_rev_is_published
      test parser::a_filled_rev_is_read_out_of_its_backticks ... ok
      test parser::state_is_read_out_of_the_frontmatter ... ok
      test parser::a_later_section_is_not_read_as_landed ... ok
      test parser::the_placeholder_is_not_mistaken_for_a_sha ... ok
      test landed_rev_is_a_published_rename_commit ... ok
      test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```

      The manifest half, which needs no arming:
      `git show dfc98fba70039863797f7185d860ef392becb21f:shared/Cargo.toml |
      grep '^name'` → `name = "shared"`.

      **The three failure paths, re-run in the same fixture at `state: done`.**
      spec03's original fixtures ran on 2026-08-28 against `prds/…`; `27db1b7`
      moved the board, so those proved nothing about the path in force today. A
      gate reading a file that no longer exists returns early on
      `read_to_string` and looks exactly as green as one that passed — which is
      why a green run alone is not evidence.

      | fixture | §Landed rev | line | panic |
      |---|---|---|---|
      | placeholder | `TBD` | `:99` | *"…names rev `TBD`, which is not a 40-character sha"* |
      | pre-rename commit | `70d7e15c…` — the rev `mitosys` pins today | `:127` | *"…has no `shared/Cargo.toml`. That rev predates the rename, so a consumer pinning it gets the crate under its old name"* |
      | unpushed commit | `e317b390…` — made in the fixture, `git branch -r --contains` empty | `:115` | *"…is on NO remote branch of this repository — it exists only on this machine, so nothing that clones the repo can ever fetch it"* |

      Every panic names `.pearde/prds/rename-conserved-to-shared/prd.md`, so the
      gate reads the migrated path and fires on it. Each fixture was restored
      before the next.

      **What the collect observes:** after it writes `state: done`,
      `cargo test -p shared --test landed_rev_is_published` reports
      `5 passed; 0 failed` on the live tree — the same run as the fixture's,
      against the same rev, which is already published and already renamed.
- [x] The three consumer PRDs are **not touched** by this work:
      `git -C .. status --porcelain mitosys/prds model/prds realm/prds` lists
      nothing from this session. Their rev bump is their own node's; this one
      only publishes the sha they read
- [x] The report names the rev on a line of its own — `dfc98fba70039863797f7185d860ef392becb21f`, named by the orchestrator at collect; the implementer returned before the commit existed — so the orchestrator can hand
      it to `mitosys`, `model` and `realm` without re-reading the file

## Verify and Proof

Every line is correct under `sh -e`. The PRD path is `.pearde/prds/…` — commit
`27db1b7` moved the board, and the old `prds/…` makes `sed` read a missing file,
which under `set -e` kills the block at the `REV=` assignment.

```sh
set -e
cd /Users/feb/dev/infra/shared

PRD=.pearde/prds/rename-conserved-to-shared/prd.md

# the gate, and its five tests
cargo test -p shared --test landed_rev_is_published

# what a push would publish — 0, everything is on origin/main
git rev-list --count origin/main..HEAD | awk '{print "unpushed:", $1}'
git log --oneline origin/main..HEAD

# §Landed names a published, renamed rev
REV=$(sed -n 's/^- rev: `\(.*\)`$/\1/p' "$PRD")
echo "rev = $REV"
git branch -r --contains "$REV"
git show "${REV}:shared/Cargo.toml" | grep '^name'
# the placeholder is gone. Zero matches is the PASS, so assert the negation.
! grep -q 'TBD' "$PRD"
echo "no TBD in the PRD"

# the whole suite, with the gate in place
cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test result:' \
  | awk '{p+=$4; f+=$6} END {print "passed="p, "failed="f}'
cargo fmt --check --all
cargo clippy --workspace --all-targets -- -D warnings

# nothing of the record moved, and the consumers are untouched
git status --porcelain -- .mi/gantt/ .pi/
git -C .. status --porcelain mitosys/.pearde model/.pearde realm/.pearde
echo "spec03 verify: OK"
```
