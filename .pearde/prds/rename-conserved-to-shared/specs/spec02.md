---
complexity: 10
footprint:
  - README.md
  - AGENTS.md
  - .github/workflows/ci.yml
  - learnings/
  - .mi/docs/memos/distribution.md
---

# spec02 — live prose is corrected, the record is amended, and the survivor set is counted

Everything outside the crate: `README.md`, `AGENTS.md`,
`.github/workflows/ci.yml`, all of `learnings/` except the decision document,
and one dated amendment line on `.mi/docs/memos/distribution.md`. The three-part
rule in `learnings/crate-name.md` decides each site — **a path or identifier is
corrected in place, a decision or name-as-recorded is amended and never
substituted, a filename never changes.**

The unit is finished when the PRD's scoped `git grep` returns the survivor set
and **nothing else**, counted.

## What already stands

**The whole of this spec is in the tree, uncommitted.** Re-run every count.

- `README.md` — 6 hits, all live crate references, all corrected. No survivors.
- `AGENTS.md` — 6 hits, 5 corrected, **`:159` survives**: `conserved-*` names the
  condemned pre-split scaffold family. Protected by sentinel before the pass.
- `.github/workflows/ci.yml:6` — the path citation
  `conserved/tests/done_boxes_are_ticked.rs` → `shared/tests/…`, because the
  directory moved. **Line 15's cost note was already corrected before this PRD
  started** — it reads *"this repo went PUBLIC on 2026-08-28, so Actions minutes
  are free rather than metered"*. Nothing is owed there; the PRD's box implies
  otherwise and is stale.
- `learnings/` — 18 tracked files. `crate-name.md` is skipped **entirely**:
  every token in it is its subject. The PRD says 45 tokens across 39 lines;
  measured on `main` today it is **57 across 50**. `README.md:164` and
  `shared-crate.md` `:13 :15 :335 :336 :359` are held by line number.
  Everything else renamed: 63 lines across 15 files.

### Three judgement calls the mechanical pass got wrong, fixed by hand

1. **`code:` frontmatter fields are corrected.** `learnings/{clock,
   content-addressing,exemptions-name-their-reason,shared-crate}.md` each carry a
   `code:` line naming `conserved/src/*.rs`. These are path citations that
   resolve to nothing after the move, so rule 1 applies. They now read
   `shared shared/src/clock.rs` — tree name then path, the same shape
   `learnings/crate-name.md:8` already uses. *If the board's "never edit
   frontmatter" rule is meant to reach learnings and not only PRDs, this is the
   one place to overrule me — but then the PRD's own §Verify grep can never come
   out clean, because it excludes no frontmatter.*
2. **The crate name used as an adjective degrades under a blind rename.**
   `learnings/toolchain.md:97` was *"`shared`'s own conserved test fails"*, which
   a blanket pass turns into *"`shared`'s own shared test"*; `:137` was *"The
   conserved tests assert on document shape"* → *"The shared tests"*. Both were
   rewritten by hand to *"`shared`'s own crate test"* and *"The `shared` crate's
   tests"*. The PRD's §6 describes `learnings/` as *"path citations and `code:`
   fields only"* — these two sites are neither, and a sed-only implementer ships
   them broken.
3. **`learnings/shared-crate.md:249` is corrected, not held.** It cites
   `scripts/conserved-vendor-check.sh`. Rule 3 says a filename never changes, so
   this looks like a survivor — but `model/prds/rename-conserved-to-shared`
   line 143 does `git mv scripts/conserved-vendor-check.sh scripts/shared-vendor-check.sh`,
   so the citation will stop resolving and rule 1 applies. Verified by reading
   model's PRD, not inferred. Note the ordering: this tree cites the new script
   name **before** model and realm have renamed it — that is inherent to
   `shared` going first and is not a defect.

### The survivor count is 12, not what the PRD implies — `reproduced`

The PRD's §Verify says `learnings/shared-crate.md`'s **"six survivor lines from
the table above"**, and the table lists **eight** line references for that file:
`:13 :15`, `:198-199 :215`, `:335-336`, `:359`. Neither number is right.
`:198`, `:199` and `:215` contain the word *private*, not the token
*conserved* — they are survivors of the **visibility** grep, not this one, and
they cannot appear in a `git grep conserved` at all. The table fuses two
different survivor populations into one.

Enumerated, not searched by judgement — the scoped grep returns **12 lines**:

| survivor | count |
|---|---|
| `b"conserved"` blake3 inputs | 3 |
| `conserved-core` proper nouns | 2 |
| `learnings/README.md:164` | 1 |
| `learnings/shared-crate.md` `:13 :15 :335 :336 :359` | 5 |
| `AGENTS.md:159` (`conserved-*`) | 1 |
| `conserved-rev-drift` mentions in this tree | 0 |
| **total** | **12** |

### The visibility population is already clean — one PRD claim `refuted`

The PRD's §"The remote is public" names `.mi/docs/memos/distribution.md` among
the documents still asserting the remote is private. **It contains no such
text.** Fixture: `grep -in private .mi/docs/memos/distribution.md` → zero
matches, measured on the pre-change file. The enumerating grep the acceptance
box specifies returns exactly two files today —

- `learnings/crate-name.md:292` — inside the decision document, allowed by the
  box's own wording;
- `learnings/shared-crate.md:198,199,215` — already sitting under the
  **Corrected 2026-08-28** block at `:206`, kept under `learnings/README.md`
  rule 1.

So the visibility box passes as the tree stands and this spec owes it nothing
but the check.

### The memo is amended, never substituted

`.mi/docs/memos/distribution.md` gained a `## Amendment` section: 14 lines
added, **0 deleted**. Its 5 original `conserved` tokens — including the
`subject:` frontmatter line — are untouched, and the amendment says so
explicitly and names them. `git diff --numstat -- .mi/docs/memos/` shows
`14	0	.mi/docs/memos/distribution.md`.

`.mi/docs/memos/scaffold-reset.md`, `.mi/gantt/plan.{md,json}`,
`.pi/ontology/digest.md` (3 hits, held pending
`the-ontology-digest-indexes-a-dead-store`) and all of `prds/` are untouched.

## Acceptance

- [x] The scoped grep from the PRD's §Verify returns **exactly 18 lines**
      (`survivors: 18`, run under `sh -e` on 2026-09-02). Amended twice:
      12 → 14 at the first collect, 14 → 18 now.
      - The exclude must read `':(exclude).pearde'`. `27db1b7` moved the board
        out of `prds/`, so the old spelling leaves the whole board in the
        census — **861 lines**, and the box measures nothing.
      - `+2` `shared/tests/landed_rev_is_published.rs:13`, `:29` — the PRD's own
        address, the same class as the `adopt-conserved` addresses the table
        already counts. The file is tracked now, so `--untracked` is no longer
        needed to see it.
      - `+4` `learnings/a-shared-name-is-not-a-shared-function.md:17`, `:18`,
        `:176`, `:177` — `mitosys/.pearde/prds/adopt-conserved` and
        `model/.pearde/prds/adopt-conserved`, PRD addresses. The file arrived in
        `978dbf6` from another PRD, after this spec was written. Not a
        regression of this work
- [x] `AGENTS.md:159`'s `conserved-*` and `learnings/README.md:164` survive
      verbatim: `git diff -- AGENTS.md learnings/README.md` shows no change on
      those lines
- [x] `learnings/crate-name.md` is **byte-identical** to its pre-change form —
      `git diff --numstat -- learnings/crate-name.md` is empty — and still holds
      every `conserved` token it had. **Measure the population; do not trust the
      PRD's figure.** The PRD says 45 tokens across 39 lines; measured on `main`
      today it is **57 tokens across 50 lines**, because commit `5e36ff8`
      ("the skeptic's eleven findings closed") extended the file after the PRD
      was written. Record your own count before you start and match it after
      (`grep -oI conserved learnings/crate-name.md | wc -l`,
      `grep -cI conserved learnings/crate-name.md`)
- [x] `learnings/shared-crate.md`'s admission test is byte-identical:
      `git show $(git merge-base origin/main HEAD):learnings/shared-crate.md | awk '/^## The admission test/,/^## What goes in/' | shasum`
      equals the same `awk | shasum` on the worktree file.
      Measured value today: `826a62076bdfb1d8e19f0a39bba2cee4a0e173cb`
- [x] `learnings/shared-crate.md:135`'s `conserved-id` became `shared-id`, per
      `learnings/crate-name.md` §"Three traps" item 2 — it names a hypothetical
      split of the crate being renamed, not a thing that ever existed
- [x] No prose reads *"shared's own shared"*, *"the shared tests"* or any other
      artefact of renaming the crate name used as an adjective:
      `git grep -nI 'shared shared\|own shared \|The shared tests' -- . ':(exclude).pearde' ':(exclude)vendor'`
      returns only the `code:` frontmatter lines of the form
      `shared shared/src/…`, which are `<tree> <path>` and correct
- [x] `.mi/docs/memos/` changed by **addition only**:
      `git diff --numstat -- .mi/docs/memos/` has `0` in the deletions column for
      every row, and `distribution.md` still contains its original 5 `conserved`
      tokens
- [x] The board, `.mi/gantt/` and `.pi/ontology/digest.md` are untouched by
      this work: `git status --porcelain -- .mi/gantt/ .pi/` lists **nothing**
      (verified 2026-09-02 under `sh -e`). The board is `.pearde/` since
      `27db1b7`, not `prds/`; its modified entries are
      `.pearde/.state/history.jsonl` and `.pearde/.state/plan.json`, board
      machinery already modified before this PRD started, plus this PRD's own
      folder, which is the board's record and not the rename's.
      **Do not use the PRD's `git diff <merge-base with origin/main>..HEAD` form
      — see spec03; it does not measure this**
- [x] Every document asserting the remote is private carries a dated correction.
      `git grep -nI 'PRIVATE\|is still \*\*private\*\*\|(private)' -- . ':(exclude).pearde' ':(exclude)vendor'`
      returns only lines inside `learnings/crate-name.md` or inside
      `learnings/shared-crate.md`'s **Corrected 2026-08-28** block at `:206`.
      Measured 2026-09-02: four lines — `crate-name.md:292` and
      `shared-crate.md:198`, `:199`, `:215`. All four are inside the two
      permitted regions.
      `.github/workflows/ci.yml:15` was already corrected and must stay corrected
- [x] The suite population is still spec01's, and `cargo fmt --check --all` and
      `cargo clippy --workspace --all-targets -- -D warnings` are clean — this
      tree's tests read the repository at runtime, so prose can break a test
      here. Measured after the whole prose pass: **85 passed / 0 failed**.
      With spec03's gate in the tree the same command reads **90 / 0** — the
      same 85 plus the gate's 5, no test lost

## Verify and Proof

Every line is correct under `sh -e`. Two corrections the board move forced:
the exclude list drops `prds` for **`.pearde`** — commit `27db1b7` moved the
board, and excluding the old name leaves the whole board in the census (**861**
matches instead of 18) — and `git status` measures `.pearde/` for the same
reason.

```sh
set -e
cd /Users/feb/dev/infra/shared

# the survivor census — 18
git grep -nI conserved -- . \
  ':(exclude).pearde' \
  ':(exclude).mi/gantt' \
  ':(exclude).mi/docs/memos' \
  ':(exclude)learnings/crate-name.md' \
  ':(exclude).pi/ontology/digest.md' \
  ':(exclude)vendor'
git grep -nI conserved -- . ':(exclude).pearde' ':(exclude).mi/gantt' \
  ':(exclude).mi/docs/memos' ':(exclude)learnings/crate-name.md' \
  ':(exclude).pi/ontology/digest.md' ':(exclude)vendor' \
  | wc -l | awk '{print "survivors:", $1}'

# the decision document is untouched — count its tokens, do not trust 45
git diff --numstat -- learnings/crate-name.md
grep -oI conserved learnings/crate-name.md | wc -l | awk '{print "crate-name.md tokens:", $1}'

# the admission test did not move
BASE=$(git merge-base origin/main HEAD)
git show "${BASE}:learnings/shared-crate.md" | awk '/^## The admission test/,/^## What goes in/' | shasum
awk '/^## The admission test/,/^## What goes in/' learnings/shared-crate.md | shasum

# memos: additions only
git diff --numstat -- .mi/docs/memos/

# the record is not touched — nothing of this PRD's outside its own folder
git status --porcelain -- .mi/gantt/ .pi/

# visibility: the population is enumerated, not judged
git grep -nI 'PRIVATE\|is still \*\*private\*\*\|(private)' -- . ':(exclude).pearde' ':(exclude)vendor'

# the tree still builds and the suite has not moved
cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test result:' \
  | awk '{p+=$4; f+=$6} END {print "passed="p, "failed="f}'
cargo fmt --check --all
cargo clippy --workspace --all-targets -- -D warnings
echo "spec02 verify: OK"
```
