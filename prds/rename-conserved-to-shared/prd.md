---
state: open
repo: shared
origin: derived
from: "@master/crate-is-named-shared"
priority: 70
complexity: 30
blast-radius: high
verify: "cargo test -p shared"
footprint:
  - conserved/
  - Cargo.toml
  - Cargo.lock
  - README.md
  - AGENTS.md
  - learnings/
  - .github/workflows/ci.yml
  - .pi/ontology/digest.md
  - .mi/docs/memos/distribution.md
---

# `conserved` becomes `shared` — directory, package, and the rev the family pins

The decision is `shared/learnings/crate-name.md`, `binds: [mitosys, model,
realm, shared]`. It is not restated here; this PRD is `shared`'s half of it.

`shared/conserved/` becomes `shared/shared/` — **the directory moves with the
package.** When this is done, `cargo test -p shared` passes, nothing in the tree
outside `prds/` says `conserved` except the three proper nouns and one hash
input named below, and **the commit is on `origin/main`**, because three other
trees pin it by sha and cannot start until it is.

## This PRD blocks three others

`mitosys/prds/rename-conserved-to-shared`,
`model/prds/rename-conserved-to-shared` and
`realm/prds/rename-conserved-to-shared` each pin `shared` by git rev. A package
rename here is a new commit, so **every consumer's rev bumps to the one this
PRD publishes** or the family forks onto two package names at once.

The push is therefore part of the deliverable, not a follow-up, and the rev must
be recorded where the three consumer PRDs can read it — write the sha into this
file's §Landed before any of them is dispatched.

Two of the three consumers already have a tripwire that refuses a rev which is
not on a remote. `model/scripts/conserved-vendor-check.sh` and
`realm/scripts/conserved-vendor-check.sh` are byte-identical (`diff` is empty,
verified 2026-08-28) and their check 2 is:

```sh
remotes=$(git -C "$shared" branch -r --contains "$rev_manifest" 2>/dev/null || true)
[ -n "$remotes" ] ||
	fail "rev $rev_manifest is on NO remote branch of ../shared — it exists only on this machine, so nothing that clones the repo can ever fetch it. Push it before vendoring it."
```

`cargo vendor` resolves a rev out of the **local** git db, so a commit that never
left this machine vendors in looking healthy while naming something no clone can
fetch. That script is what catches it. **mitosys has no equivalent** — see its
own child PRD.

## The work

Measured on 2026-08-28 by a probe that was run and then deliberately reverted;
every number below is reproduced against `main` as it stands today.

### 1. `git mv conserved shared`

**19 tracked files** (`git ls-files conserved | wc -l` = 19).
`conserved/tests/done_boxes_are_ticked.rs` moves **unmodified** — its content
names no crate. The other **18** need the token change.

(The analyst's probe counted 17/16; two `load_*` tests have been added to the
crate since. Count before you start rather than trusting either figure.)

### 2. `Cargo.toml` — the workspace root

- line 3: `members = ["conserved"]` → `["shared"]`
- line 26: the `serde` comment, *"Optional in `conserved` (feature `serde`, off
  by default)"*

### 3. `shared/Cargo.toml` — the package

- `name = "conserved"` → `name = "shared"`
- the `[features]` comment's `cargo tree -p conserved --edges normal`

`description = "Domain-free primitives shared by the Rust trees."` needs no
change and should not get one.

### 4. Token rename through the crate — with three exclusions the build found

`conserved` → `shared`, `conserved::` → `shared::`, **except**:

- **`b"conserved"` — a blake3 input, not a name.**
  `conserved/tests/content_id.rs:31`, `conserved/tests/content_id.rs:108`
  (`ContentId::of(b"conserved")`), `conserved/tests/content_id_serde.rs:30`.
  Each is paired with the fixed vector
  `8d369871266d2453da564f5748e5a3070f25068aa5be7db442dd2c2b1b31f08e`. Renaming
  the bytes changes the digest and fails `fixed_vectors_render_as_expected`.
- **`conserved-core` — a historical proper noun.** `conserved/src/clock.rs:86`
  and `conserved/tests/clock_source.rs:48` both name the condemned scaffold.
  It was called that; it stays called that.
- **`conserved-rev-drift`** — the filename of mitosys's open memo. It appears
  in prose here and must survive as written.

### 5. `cargo fmt --all` afterwards

rustfmt re-sorts the import block: `shared` sorts **after** `proptest`/`serde`
where `conserved` sorted before. Three files reformat for that reason alone —
`tests/clock_serde.rs`, `tests/content_id_props.rs`, `tests/content_id_serde.rs`.
Run `cargo fmt --all`, then `cargo fmt --check --all` and
`cargo clippy --workspace --all-targets -- -D warnings`; both were clean on the
probe.

### 6. Live prose only

In: `README.md` (6 hits), `AGENTS.md` (6), `.github/workflows/ci.yml` (1, line 6
— a path citation `conserved/tests/done_boxes_are_ticked.rs` that changes because
the **directory** moves), `learnings/*.md`, `.mi/docs/memos/distribution.md` (5),
`.pi/ontology/digest.md` (3).

Two live-looking files that are **not** renamed:

- `.mi/gantt/plan.{md,json}` — a historical planning artifact describing a
  `.mi/prd/` layout that no longer exists.
- `.mi/docs/memos/scaffold-reset.md` — it is *about* `conserved-core`,
  `conserved-alloc`, `conserved-net` and the rest of the condemned scaffold.
  Every token in it is a proper noun.

`Cargo.lock` changes, but not by hand — it regenerates on the first `cargo
build`. Check that it did and that the package entry reads `name = "shared"`.

### 7. `prds/` is not touched

**798 of this tree's 1041 `conserved` tokens live under `prds/`, in 56 files**
(`git grep -oI conserved -- prds | wc -l` = 798; `git grep -lI conserved --
prds | wc -l` = 56; tree total 1041, and the analyst measured 1036 on the same
day before two commits landed). They are records of completed work.

The reason is not sentiment: **a done PRD's directory name is an address.** The
master board's `infra/prds/vision.md:13-14` names two of them as graph edges —

```
- "@shared/p5-adoption -> @mitosys/adopt-conserved"
- "@shared/p5-adoption -> @model/adopt-conserved"
```

— so renaming a done PRD's directory breaks an edge in the destination graph to
gain nothing.

### 8. `learnings/shared-crate.md`'s etymology is rewritten, not replaced

Already done by `crate-name.md`'s implementer: the genomics sentence (*"a
conserved region is the part of a genome that is identical across species
because it cannot afford to drift"*) is now marked historical, because "a shared
region" is not a term and explains nothing. **The admission test below it is
unchanged and must stay unchanged** — what goes in the crate does not move
because its name did.

What this PRD still owes in `learnings/`: the `code:` frontmatter fields that
name `conserved/src/*.rs` paths, and every remaining `conserved/…` path citation
in the folder's prose.

## The remote is public — correct the prose while renaming it

`yesitsfebreeze/shared` was made **public** on 2026-08-28.
`https://api.github.com/repos/yesitsfebreeze/shared` returns HTTP 200 anonymously
with `"private": false, "visibility": "public"`.

Several documents in this tree still assert it is private —
`.mi/docs/memos/distribution.md` and `learnings/shared-crate.md` among them.
Correct that in the same edit, and correct it the way `learnings/README.md`
§"How a document is corrected" requires: **rule 1, an edit adds** — extend or
qualify the stale sentence, never delete it.

State the consequence plainly, because it is easy to get backwards: **the
vendored copies in mitosys, model and realm stay.** Public fixes
*authentication*. It does not fix *reachability* — a public remote is still a
network round trip, and mitosys's dev container has no network at build time.

## Verify

**Scoped to the crate, deliberately.**

```sh
cargo test -p shared
cargo fmt --check --all
cargo clippy --workspace --all-targets -- -D warnings
git grep -nI conserved -- . ':(exclude)prds' ':(exclude).mi/gantt' \
  ':(exclude).mi/docs/memos/scaffold-reset.md' ':(exclude)vendor'
```

The last command must return **only** the three exclusions from item 4: the
three `b"conserved"` sites, the two `conserved-core` sites, and any
`conserved-rev-drift` mention.

**Not `just check`, and not `cargo test --workspace`.** This tree's workspace
suite is red on `main` today for a reason this PRD does not own:

```
thread 'every_done_prd_has_no_unticked_box' panicked at
conserved/tests/done_boxes_are_ticked.rs:276:9:
1 `state: done` PRD(s) carry unticked boxes …
  prds/p5-adoption/mitosys/prd.md: 7 unticked box(es)
```

The baseline on `main` today is **84 passed, 1 failed**
(`cargo test --workspace --no-fail-fast`, measured 2026-08-28). The probe held
the suite at exact parity across the rename — the analyst measured 82/1 before
and after, on the tree as it then stood. **Parity is the acceptance criterion,
not an absolute count**: re-measure the baseline immediately before the rename
and require the same numbers after.

## Acceptance

- [ ] `git mv conserved shared` done; `git ls-files shared/ | wc -l` matches the
      pre-move `git ls-files conserved/ | wc -l`
- [ ] `Cargo.toml`'s `members = ["shared"]`; `shared/Cargo.toml`'s
      `name = "shared"`
- [ ] `cargo test -p shared` green, and the workspace baseline is unchanged
      from the count measured immediately before the rename (84/1 as of
      2026-08-28), with `done_boxes_are_ticked` the only failure
- [ ] `cargo fmt --check --all` clean and
      `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] The scoped `git grep` above returns only the three `b"conserved"` sites,
      the two `conserved-core` sites, and `conserved-rev-drift` mentions
- [ ] `prds/` is untouched: `git diff --name-only <base>..HEAD -- prds/` is empty
- [ ] `learnings/shared-crate.md`'s admission test is byte-identical to its
      pre-change form
- [ ] Every document asserting the remote is private carries a dated correction
      saying it is public and that the vendored copies stay anyway
- [ ] **Pushed.** `git branch -r --contains HEAD` names `origin/main`, and the
      sha is written into §Landed below where the three consumer PRDs read it

## Landed

<!-- The rev the consumers pin. Fill this in the moment it is on origin/main;
     mitosys, model and realm are all blocked until it is here. -->

- rev: `TBD`
- pushed to `origin/main`: `TBD`
