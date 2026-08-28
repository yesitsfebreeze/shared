---
type: learning
learning: crate-name
subject: the crate is named `shared`, not `conserved` — directory and package move together, live prose is renamed and `prds/` is not, and the rev bump is a correctness requirement no compiler enforces
binds: [mitosys, model, realm, shared]
status: decided
date: 2026-08-28
code: shared conserved/Cargo.toml, mitosys Cargo.toml:376, model Cargo.toml:38, realm Cargo.toml:66
---

# The crate is named `shared`

The repository is `shared`. The crate inside it is `conserved`. Every consumer
writes `conserved` in its `Cargo.toml`, `use conserved::…` in its source, and
`vendor/conserved-0.1.0` on disk, while the address it resolves from says
`shared`. One thing with two names.

**Decided by the user, 2026-08-28**, answering `shared-remote-is-private`:

> make it public aswell, but actually name the crate shared, not conserved

## The decision

`shared/conserved/` becomes `shared/shared/` — **the directory moves with the
package**, not one without the other. `shared/Cargo.toml`'s
`members = ["conserved"]` becomes `["shared"]`, and `shared/shared/Cargo.toml`
reads `name = "shared"`.

The fork the PRD left open was whether to rename the package in place and leave
the directory alone. It is closed by three lines. Moving the directory costs
exactly three lines more than renaming in place:

1. `shared/Cargo.toml`'s `members = ["shared"]`;
2. `git archive "$rev_manifest" conserved` → `shared` in
   `model/scripts/conserved-vendor-check.sh`;
3. the same line in `realm/scripts/conserved-vendor-check.sh`.

Both scripts have to be rewritten for the package name regardless — they are
named `conserved-vendor-check.sh` and every token inside them says `conserved`
— so the marginal cost of the directory move is one line in each of two files
that are being rewritten anyway.

Renaming in place buys those three lines back and **keeps the exact defect this
decision exists to remove**, one directory down:
`shared/conserved/Cargo.toml` reading `name = "shared"` is still one thing with
two names. Three lines is not a price worth paying for that.

## What the rename costs, measured

Every number below was measured on 2026-08-28. The build that produced them was
a probe across all four working trees; it was reverted deliberately once it had
answered the question, because the code for this decision lands through the four
child PRDs named below and under each tree's own gates, never from here.

### The crate holds still under the move — `reproduced`

`shared`'s suite is at parity with its pre-rename baseline. The analyst measured
**82 passed / 1 failed** before and after the probe's rename; re-measured on
`main` today (fixture: `cargo test --workspace --no-fail-fast` in
`/Users/feb/dev/infra/shared`) the baseline is **84 passed / 1 failed** — two
tests were added to the crate between the probe and now. What matters is not the
absolute number but that the rename does not move it. `cargo fmt --check --all`
and `cargo clippy --workspace --all-targets -- -D warnings` were clean after the
rename.

The one failure is **not** the rename and is not this decision's to fix:

```
thread 'every_done_prd_has_no_unticked_box' panicked at
conserved/tests/done_boxes_are_ticked.rs:276:9:
  prds/p5-adoption/mitosys/prd.md: 7 unticked box(es)
```

It is red on `main` today for an unrelated reason. Every child PRD's verify is
scoped around it rather than inheriting it.

### Nothing in the crate resolves its own path by name — `reproduced`

Every test that reads the tree at runtime goes through
`env!("CARGO_MANIFEST_DIR")` — `conserved/tests/clock_instant.rs:17`,
`clock_serde.rs:165`, `clock_source.rs:19` — so the directory move is invisible
to them. This is the single fact that makes moving the directory cheap.

### No name collides — census over all four trees

`vendor/` and `target/` excluded, matcher
`git grep -nIE '^\s*(pub )?mod shared\b|crate::shared\b'` plus a scan of every
workspace `members` list:

| population | matches |
|---|---|
| `mod shared` declarations | 0 |
| `crate::shared` paths | 0 |
| workspace members named `shared` | 0 |

### rustfmt re-sorts the import block

`conserved` sorts before `proptest`/`serde`; `shared` sorts after. Files needing
reformatting for that reason alone:

| tree | files |
|---|---|
| shared | 3 (`tests/{clock_serde,content_id_props,content_id_serde}.rs`) |
| realm | 6 |
| model | 2 (`src/record/{event,log}.rs`) |
| mitosys | 0 |

mitosys is the one tree where the re-sort costs nothing — `rustfmt --check` on
all its renamed `.rs` files showed no drift.

### The population being renamed — census, 2026-08-28

`git grep -lI conserved`, `vendor/` and `target/` excluded, per tree. Reproduced
today, exact:

| tree | files naming `conserved` | of which under `prds/` |
|---|---|---|
| shared | 97 | 56 |
| mitosys | 38 | 8 |
| model | 26 | 3 |
| realm | 21 | 2 |
| master board (`infra/prds/`) | 44 | all of it |

**226 files family-wide.** Wide, mechanical, and it crosses every tree — which
is why the decision is filed here and the work is filed there.

## History is not renamed

**798 of `shared`'s 1041 `conserved` tokens live under `prds/`, in 56 files**
recording completed work. (The analyst counted 1036 on the same day; two commits
have landed since. The 798 is unchanged and exact.)

Those are not renamed, in any tree. The reason is not sentiment: **a done PRD's
directory name is an address.** `infra/prds/vision.md` names two of them as
graph edges —

```
- "@shared/p5-adoption -> @mitosys/adopt-conserved"
- "@shared/p5-adoption -> @model/adopt-conserved"
```

— and renaming `mitosys/prds/adopt-conserved/` breaks an edge in the master
board's own destination graph to gain nothing. The same holds for
`mitosys/.mi/docs/memos/`: `conserved-rev-drift.md` keeps its filename, because
it is an open memo about the two revs *as they were named*, and renaming a memo
file reddens mitosys's memo-index gate until its README link is rewritten too.

**Live prose is renamed; the record is not.** In: `learnings/`, `README.md`,
`AGENTS.md`, `.cargo/config.toml`, justfiles, CI workflows, source, manifests,
`vendor/`, `scripts/`. Out: `prds/`, `.mi/docs/memos/`, and two historical
artifacts in `shared` — `.mi/gantt/plan.{md,json}` (a planning record naming a
`.mi/prd/` layout that no longer exists) and `.mi/docs/memos/scaffold-reset.md`
(about `conserved-core`).

## Three traps a blanket `sed` walks into

All three were found by running the rename, not by reading it.

1. **`b"conserved"` is a hash input, not a name.**
   `conserved/tests/content_id.rs:31`, `:108` and
   `conserved/tests/content_id_serde.rs:30` feed the literal bytes `conserved`
   to blake3 and compare against the fixed vector
   `8d369871266d2453da564f5748e5a3070f25068aa5be7db442dd2c2b1b31f08e`.
   Renaming the bytes changes the digest and fails
   `fixed_vectors_render_as_expected`.
2. **`conserved-core` and `conserved-rev-drift` are proper nouns.**
   `conserved-core` (`conserved/src/clock.rs:86`,
   `conserved/tests/clock_source.rs:48`) is the name of a condemned scaffold, a
   historical fact. `conserved-rev-drift` is a memo's filename.
3. **The etymology cannot be substituted.**
   [[shared-crate]] named the crate from biology — *a conserved region is the
   part of a genome that is identical across species because it cannot afford to
   drift.* A find-and-replace turns that into "a shared region", which is not a
   term and explains nothing. It is rewritten by hand, in that document, and the
   admission test it introduces is unchanged: **what goes in the crate does not
   move because its name did.**

## The ordering, and the trap inside it

Four child PRDs, one per tree, `state: open`:

| child | tree | pinned rev today |
|---|---|---|
| `shared/prds/rename-conserved-to-shared` | shared | — it *produces* the rev |
| `mitosys/prds/rename-conserved-to-shared` | mitosys | `70d7e15cd21c6017ec928c63697d0c7f42f53a20` |
| `model/prds/rename-conserved-to-shared` | model | `9a342e1e849dd5775cbadfe6b32e275a076e5f09` |
| `realm/prds/rename-conserved-to-shared` | realm | `9a342e1e849dd5775cbadfe6b32e275a076e5f09` |

**`shared` goes first and pushes.** The other three pin a rev by git sha, so a
package rename in `shared` is a new commit and every consumer's rev bumps in the
same change — or the family forks onto two package names at once. The consumers
land on the rev `shared` publishes to `origin/main`.

### A vendored consumer build does not force the rev bump — `reproduced`

This is the whole risk in this decision and it is silent.

All three consumers carry a committed `vendor/` and a source replacement in
`.cargo/config.toml`. With `vendor/shared-0.1.0` in place and the source key
left at the *old* rev, `cargo check --workspace --offline` resolves happily and
prints `Checking shared v0.1.0 (…?rev=9a342e1e…)` — **cargo takes whatever the
replacement directory provides and never checks that the package at that rev is
called that.** A green consumer build before the new rev is published is a false
green.

What catches it is `scripts/conserved-vendor-check.sh`, in `model` and `realm`
only, which compares the vendored bytes against `git archive` at the manifest's
rev:

```
fatal: pathspec 'shared' did not match any files
shared-vendor-check: FAIL git archive of 9a342e1e… produced no shared/src
```

So the rev bump is a correctness requirement enforced by an existing script, not
by the compiler. **mitosys has no such script** — no `scripts/` directory and no
equivalent gate; `vendor_is_complete.rs` only asserts that nothing under
`vendor/` is ignored or untracked. On mitosys the rev agreement is a manual step,
and the check its own `.cargo/config.toml` names is
`git -C ../shared branch -r --contains <rev>`.

Every child PRD carries this fact. None of them may treat a green `cargo check`
as proof.

## The remote is public, and the vendoring stays

`yesitsfebreeze/shared` was flipped to public on 2026-08-28 at the user's
decision. Verified anonymously: `https://api.github.com/repos/yesitsfebreeze/shared`
returns HTTP 200 with `"private": false, "visibility": "public"`.

**The vendored copies stay regardless.** A public remote is still a network
round trip, and it still breaks an offline build — mitosys's dev container has
no network at build time, and `--offline` is how every consumer's vendored path
is exercised. Public fixes *authentication*; vendoring fixes *reachability*, and
those were always two problems. Every `.cargo/config.toml` in the family
currently states in prose that the remote is PRIVATE. That prose is stale as of
today and each child PRD corrects it in the same edit that renames it.

## Out of scope, stated so nobody widens it

`conserved-rev-drift` (`mitosys/.mi/docs/memos/conserved-rev-drift.md`,
`status: open`) records that mitosys pins `70d7e15c` where model and realm pin
`9a342e1e`. This decision **forces** mitosys off `70d7e15c`, because every
consumer must land on the rev `shared` publishes. It does **not** own
reconciling that drift on its own terms. The memo's `status:` is this work's to
update; its filename and its argument are not.

## Pointers

- [[shared-crate]] — the admission rule and what the crate holds; its etymology
  paragraph is rewritten by this decision, its admission test is not
- `infra/prds/crate-is-named-shared/prd.md` — the master-board node this
  document is the deliverable of
- `infra/prds/shared-remote-is-private/prd.md` — the answer it was derived from
- `mitosys/.mi/docs/memos/conserved-rev-drift.md` — the two revs, `status: open`
