---
complexity: 12
footprint:
  - shared/tests/landed_rev_is_published.rs
  - prds/rename-conserved-to-shared/prd.md
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

- [ ] `shared/tests/landed_rev_is_published.rs` is tracked, and
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
- [ ] **Pushed.** `git branch -r --contains <rename-commit>` names `origin/main`,
      and the report states how many commits the push published
      (`git rev-list --count origin/main..HEAD` before pushing — it was 10 plus
      this PRD's own on 2026-08-28)
- [ ] §Landed's `- rev:` is the 40-character sha of the commit that carries the
      rename, and `- pushed to origin/main:` names the date. `TBD` appears in
      neither
- [ ] The gate passes **for the right reason**, with the PRD at `state: done`:
      `cargo test -p shared --test landed_rev_is_published` green while
      `git show $(sed -n 's/^- rev: `\(.*\)`$/\1/p' prds/rename-conserved-to-shared/prd.md):shared/Cargo.toml | grep 'name'`
      prints `name = "shared"`
- [x] The three consumer PRDs are **not touched** by this work:
      `git -C .. status --porcelain mitosys/prds model/prds realm/prds` lists
      nothing from this session. Their rev bump is their own node's; this one
      only publishes the sha they read
- [ ] The report names the rev on a line of its own so the orchestrator can hand
      it to `mitosys`, `model` and `realm` without re-reading the file

## Verify and Proof

```sh
cd /Users/feb/dev/infra/shared

# the gate, and its five tests
cargo test -p shared --test landed_rev_is_published

# what the push will publish — run BEFORE pushing
git rev-list --count origin/main..HEAD
git log --oneline origin/main..HEAD

# after pushing and filling §Landed
REV=$(sed -n 's/^- rev: `\(.*\)`$/\1/p' prds/rename-conserved-to-shared/prd.md)
echo "rev = $REV"
git branch -r --contains "$REV"
git show "$REV:shared/Cargo.toml" | grep '^name'

# the whole suite, with the gate armed
cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test result:' \
  | awk '{p+=$4; f+=$6} END {print "passed="p, "failed="f}'
cargo fmt --check --all
cargo clippy --workspace --all-targets -- -D warnings

# nothing of the record moved
git status --porcelain -- prds/ .mi/gantt/ .pi/
```
