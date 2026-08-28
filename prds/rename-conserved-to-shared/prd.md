---
state: specced
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

### 1. `mv conserved shared`

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

And one look-alike that is **not** an exclusion, ruled here so it is not read as
one: **`conserved-id` (`learnings/shared-crate.md:135`) becomes `shared-id`.**
It names a hypothetical future split of the crate being renamed — the `blake3`
split the dependency gate might force — not a thing that has ever existed. It
gets the same ruling as mitosys's `conserved-scope`; see
`learnings/crate-name.md` §"Three traps", item 2.

### 5. `cargo fmt --all` afterwards

rustfmt re-sorts the import block: `shared` sorts **after** `proptest`/`serde`
where `conserved` sorted before. Three files reformat for that reason alone —
`tests/clock_serde.rs`, `tests/content_id_props.rs`, `tests/content_id_serde.rs`.
Run `cargo fmt --all`, then `cargo fmt --check --all` and
`cargo clippy --workspace --all-targets -- -D warnings`; both were clean on the
probe.

### 6. Live prose only

In: `README.md` (6 hits), `AGENTS.md` (6, minus the one survivor below),
`.github/workflows/ci.yml` (line 6 — a path citation
`conserved/tests/done_boxes_are_ticked.rs` that changes because the **directory**
moves; and line 15, see the visibility section), `learnings/*.md` (path
citations and `code:` fields only — decisions are amended, not substituted).

`.pi/ontology/digest.md` (3 hits) is **held**, not renamed — see its own section
below.

Three live-looking files that are **not** renamed:

- `.mi/gantt/plan.{md,json}` — a historical planning artifact describing a
  `.mi/prd/` layout that no longer exists.
- `.mi/docs/memos/scaffold-reset.md` — it is *about* `conserved-core`,
  `conserved-alloc`, `conserved-net` and the rest of the condemned scaffold.
  Every token in it is a proper noun.
- **`.mi/docs/memos/distribution.md` (5 hits) — amended, not renamed.** An
  earlier draft of this PRD had it in the "renamed" column, contradicting
  `learnings/crate-name.md`. The decision ruled, and this is the ruling: **no
  memo body is substituted in any tree**, because a memo is a record and the
  family does not rewrite what was decided under the name it was decided under
  (`mitosys/.mi/docs/memos/membrane-is-the-core.md:270-272`, and
  `learnings/README.md` rule 1). It gets **one dated amendment line** saying the
  crate is now `shared` and that every `conserved` below is the name as it stood.
  The file stays in the footprint because it is edited — just not substituted.

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

## The trap in this tree specifically: the decision document is written in the old name

`learnings/crate-name.md` is the `binds:` document this PRD implements. Its
subject **is** the old name, so **45 of its tokens are `conserved` and every one
must survive** (`grep -oI conserved learnings/crate-name.md | wc -l` = 45 across
39 lines, measured 2026-08-28). A rename run over `learnings/` without excluding
it destroys the record of why the rename happened.

That is the sharp edge of a fourth exclusion class, on top of item 4's three.
**These survive, and the verify grep is written around them:**

| survivor | why |
|---|---|
| `learnings/crate-name.md`, all 45 tokens | the decision document; its subject is the old name |
| `learnings/README.md:164` | its index entry, which quotes the old name |
| `learnings/shared-crate.md:13`, `:15` | the renamed-from sentence and the genomics etymology |
| `learnings/shared-crate.md:198-199`, `:215` | the superseded private-remote text, kept under rule 1 |
| `learnings/shared-crate.md:335-336` | `mitosys/prds/adopt-conserved`, `model/prds/adopt-conserved` — PRD addresses |
| `learnings/shared-crate.md:359` | the `conserved-crate` PRD — a real directory, `/Users/feb/dev/infra/prds/conserved-crate/` |
| `AGENTS.md:159` | `conserved-*`, the condemned pre-split scaffold family |
| all of `.mi/docs/memos/` | amended, never substituted — see item 6 |
| all of `.mi/gantt/` | historical planning artifact |
| all of `prds/` | records; a done PRD's directory name is an address |

## Verify

**Scoped to the crate, deliberately.**

```sh
cargo test -p shared
cargo fmt --check --all
cargo clippy --workspace --all-targets -- -D warnings
git grep -nI conserved -- . \
  ':(exclude)prds' \
  ':(exclude).mi/gantt' \
  ':(exclude).mi/docs/memos' \
  ':(exclude)learnings/crate-name.md' \
  ':(exclude).pi/ontology/digest.md' \
  ':(exclude)vendor'
```

The last command must return **exactly** these and nothing else — count them,
do not eyeball them:

- the three `b"conserved"` sites (`shared/tests/content_id.rs:31`, `:108`,
  `shared/tests/content_id_serde.rs:30`)
- the two `conserved-core` sites (`shared/src/clock.rs:86`,
  `shared/tests/clock_source.rs:48`)
- `learnings/README.md:164`
- `learnings/shared-crate.md`'s six survivor lines from the table above
- `AGENTS.md:159`
- any `conserved-rev-drift` mention

`.pi/ontology/digest.md` is excluded from the grep because it is held; if the
user's ruling releases it, drop that exclusion and the file renames with the
rest.

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

## `.pi/ontology/digest.md` is held pending a ruling — do not rename it

`.pi/ontology/digest.md:23` binds the crate as an **entity name to an external
store id**:

```
conserved kern:629fc82759c9 — proposed shared crate: ContentId, Clock, Scope/Handle,
  order stats, hex; partial | see: learnings/shared-crate.md
```

The file's own header says *"Pointers, not data: kern IDs … The substance lives
in the memory store and in files — this file only says where."* Renaming the
pointer without renaming the entity in the store desyncs the index; renaming it
in the store is work no PRD owns.

**Measured 2026-08-28, and it changes the question — see this PRD's parent
report.** The binding is already dangling: `kern get 629fc82759c9` returns *"no
thought with id"*, as do four other ids sampled from the same file, and
`kern health` on this directory reports `thoughts: 0  reasons: 0`. `kern` v2.0.0
reads `shared/.kern/data`, not the `.pi/kern/data` the file was written against;
both are untracked and `.pi/kern/data/` is gitignored (`.gitignore:8`).

So renaming the token here cannot desync anything — but the right move is
plausibly to fix or delete the whole orphaned index, which is not this PRD's.
**Leave all three hits alone until the ruling lands**, and keep the grep
exclusion above.

## Acceptance

- [ ] `mv conserved shared` done and nothing was lost:
      `git ls-files shared/ | wc -l` is **19** — the pre-move count, measured
      2026-08-28 and pinned here because it is recorded nowhere else. If it is
      not 19 when you start, the crate has changed since; record the new number
      here before moving and match *that*.
- [ ] `Cargo.toml`'s `members = ["shared"]`; `shared/Cargo.toml`'s
      `name = "shared"`
- [ ] `cargo test -p shared` green, and the workspace baseline is unchanged
      from the count measured immediately before the rename (84/1 as of
      2026-08-28), with `done_boxes_are_ticked` the only failure
- [ ] `cargo fmt --check --all` clean and
      `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] The scoped `git grep` above returns **exactly** the survivor set
      enumerated under §Verify, and its line count equals the number of entries
      there — not "roughly", counted
- [ ] `prds/` and `.mi/gantt/` are untouched, and `.mi/docs/memos/` changed by
      amendment only:
      `git diff --name-only <merge-base with origin/main>..HEAD -- prds/ .mi/gantt/`
      is empty, and `git diff -- .mi/docs/memos/` shows **added lines only**
      (`git diff --numstat -- .mi/docs/memos/` has `0` in the deletions column
      for every row). Take the base from
      `git merge-base origin/main HEAD` — it is not a placeholder
- [ ] `learnings/shared-crate.md`'s admission test is byte-identical to its
      pre-change form. The region is **`## The admission test` (line 25) up to
      but not including `## What goes in`** — extract it both sides and compare:
      `git show <base>:learnings/shared-crate.md | awk '/^## The admission test/,/^## What goes in/' | shasum`
      equals the same command on the worktree file. Named because "the admission
      test" is otherwise an undelimited region no check can bound
- [ ] Every document asserting the remote is private carries a dated
      correction. The population is **enumerated, not searched by judgement** —
      `git grep -nI 'PRIVATE\|is still \*\*private\*\*\|(private)' -- . ':(exclude)prds' ':(exclude)vendor'`
      returns exactly two files today:
      - `.github/workflows/ci.yml:15` — *"this repo is PRIVATE, so these minutes
        are metered against the …"*. **A live cost claim that is now false**:
        Actions minutes on a public repository are not metered. Correct it.
      - `learnings/shared-crate.md:198-199`, `:215` — already carries a dated
        correction block; the old text stays under rule 1.
      The box passes when the grep returns only lines inside a dated correction
      block or inside `learnings/crate-name.md`
- [ ] **Pushed.** `git branch -r --contains HEAD` names `origin/main`, and the
      sha is written into §Landed below where the three consumer PRDs read it

## Landed

<!-- The rev the consumers pin. Fill this in the moment it is on origin/main;
     mitosys, model and realm are all blocked until it is here. -->

- rev: `TBD`
- pushed to `origin/main`: `TBD`

> **Use plain `mv`, never `git mv` — measured 2026-08-28, and it already bit
> once.** `git mv` stages the rename into the shared index, and a concurrent
> orchestrator commit then sweeps it in. That is exactly what happened in
> `shared`: a commit whose only intended file was a PRD body carried all 19
> renames with it as `R100`, without the root `Cargo.toml` edit that goes with
> them, and `cargo metadata --no-deps` exited 101 on a clean checkout of `main`
> — *"failed to load manifest for workspace member"*. The working tree looked
> green the whole time, because the one-line fix sat in it uncommitted.
>
> **`git mv` buys nothing here.** Git computes rename detection from content
> similarity at diff time, not from the index, so a plain `mv` produces exactly
> the same `R100` rows in the eventual commit. The only thing `git mv` adds is
> the window in which someone else's commit can take your half-finished rename.
