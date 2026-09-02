# goal

`learnings/shared-crate.md`: `partial` → `decided`, linking the landing
commits, with §"Where it lives — the one unresolved constraint" resolved in
place rather than erased — and `README.md`'s status table row moved with it,
in the same commit, because the table renders the status and goes stale with it.

est: 0.5

## Why `decided` is honest here, today

`AGENTS.md` §"The status ladder":

| status | meaning |
|---|---|
| **partial** | agreed in principle, **not fully specified** or not yet extracted. |
| **decided** | agreed; the record of what it beat. **May still need extraction.** |

`shared-crate.md` is `partial` for exactly one stated reason, and it says so in
its own section title: §"Where it lives — **the one unresolved constraint**",
closing with *"No recommendation is recorded here."* That constraint is
settled:

- **The requirement**, by the user on 2026-08-20: distributable to every Rust
  repo, which eliminates option 3 (path dependency).
- **The mechanism**, by `p0-foundation`: option 2, a git dependency pinned by
  commit rev. `.mi/docs/memos/distribution.md` reads `status: decided`,
  `decided: 2026-08-21`, and `p0`'s `## Answers` says "do not re-escalate this".
- **Proven once**, by `p1-scope`: a consumer tree resolved, locked, compiled and
  ran a test against `conserved` through that mechanism, recorded with the rev
  in `.mi/prds/p1-scope/proof.md`.

That is the whole of what `partial` was withholding. The ladder's `decided`
explicitly *"may still need extraction"*, so p3 and p4 not having landed is not
a reason to hold this document at `partial` — and the proposal's own §"First
move" (`Scope` alone, to test the mechanism) has landed. What remains after
this flip is extraction, which is the ladder's own carve-out.

**What this flip must not claim.** `decided` is about the proposal, not about
adoption. The user's 2026-08-21 answer holds all four consumer children
(`ratchets`, `mitosys`, `llm`, `realm`), so **no consumer has replaced its
duplicate**: mitosys still has `util/effect`, `../model` still has `rec_now()`.
The flip therefore has to state what is decided (the crate, its admission test,
its home, its distribution) and what is still outstanding (adoption in all
three trees, plus `Clock` and order statistics), or it reads as a claim that
the extraction is done. That sentence is an acceptance box below.

## The landing commits to link

All are on `main` and all resolve:

| commit | what it landed |
|---|---|
| `ab154f7` | git init; the reset workspace; the board; `.mi/docs/memos/distribution.md` |
| `0d4bc10` | the fresh-clone gate (`scripts/fresh-clone-check.sh`) — p0's acceptance |
| `eb55b49` | p0's acceptance ticks |
| `5313ca4` | `Scope`/`Disposer` ported into `conserved::scope`, zero dependencies |
| `0b2f964`, `85dad04` | the `mod scope` test wrapper and its deviation record |
| `9fff8ea` | **the distribution mechanism proven once**, from mitosys |
| `f7b454a` | the proof re-pinned at the final port rev |
| `c122240`, `ec58a75`, `8e12122`, `c275645` | `ContentId`: blake3 in, 64 lowercase hex out; proptest round-trips; optional `serde`; the deviation record |

The two that carry the argument are `9fff8ea` (the unresolved constraint, made
real) and `5313ca4` (§"First move", done). The rest are the record. Link them
as short shas so `git show <sha>` works from the repo root; the verify resolves
every one with `git cat-file -e`, so a mistyped sha fails rather than rots.

## Files

- `learnings/shared-crate.md` — frontmatter `status:` and `code:`; §"Where it
  lives" gains a resolution; a short §"Landed" recording the commits.
- `README.md` — the `shared-crate.md` row of the table at lines 31-40 only.
  `partial` → `decided`. No other line of `README.md` changes; §"Where the
  shared code lives" (lines 78-106) already carries the decision correctly and
  needs nothing.

Not touched: `learnings/clock.md` (spec02/spec04), the `clock.md` row of the
`README.md` table (spec04), anything under `conserved/`, any sibling tree.

## The edits

**`status: partial` → `status: decided`.**

**`code:`** — currently `mitosys src/mitosys/util/, llm src/utils/`, which names
where the duplicates are. It gains where the one implementation now is:
`shared conserved/src/scope.rs, shared conserved/src/content_id.rs`. The
duplicate sites stay — they are still there, and the day they go is the day
p5's held children run.

**§"Where it lives — the one unresolved constraint"** — the three options and
their costs (lines 141-153) stay **verbatim**; that is the record of what the
decision beat, which `decided` is defined as carrying. The closing paragraph
(*"No recommendation is recorded here…"*) is the one thing that is now false,
and it is corrected in place per spec01's rule 1: name option 2, name
`.mi/docs/memos/distribution.md` as where the argument lives, name `9fff8ea` as
where it was proven, and record that mitosys's offline container cost is
scoped as mitosys-side follow-up rather than a reopening.

**A new short §"Landed"** — the commit table above, one line per item of §"What
goes in" saying which of the five are in the crate (`ContentId` ✓, `Scope` ✓,
`Clock` p3, order statistics p4, `hex` — folded into `ContentId`'s
`Display`/`FromStr` as §5 said it would be), and the sentence about adoption
being outstanding.

**Two small corrections while the document is open**, both additions:

- §"Size and shape" says *"one crate to start, with one dependency (`blake3`)"*.
  Still true by default, and p2 added an **optional** `serde` feature with
  `default = []` — record it in one clause so the sentence is not read as
  contradicted by the manifest. (`learnings/content-addressing.md` carries the
  full argument; spec05.)
- §"What goes in" item 3 names `Scope` / `Handle`. The landed type pair is
  `Scope` / `Disposer`. Name the landed spelling, keeping the proposal's word
  visible, so a reader of the learning finds the type that exists.

## Acceptance

- [x] `learnings/shared-crate.md` frontmatter reads `status: decided`.
- [x] Its `code:` line names `conserved/src/scope.rs` and
      `conserved/src/content_id.rs`, both of which are **committed**
      (`git ls-files`, not merely on disk), and still names the two duplicate
      sites.
- [x] The document links, at minimum, `ab154f7`, `5313ca4`, `9fff8ea`,
      `c122240` and `8e12122`, and every sha it names resolves to a real commit.
- [x] It names `.mi/docs/memos/distribution.md` as where the distribution
      argument lives.
- [x] **The erase-guard**: all three options in §"Where it lives" are still on
      disk verbatim — the strings `Vendored into each tree`,
      `A git dependency`, and `A path dependency to a sibling directory` — and
      the string `No recommendation is recorded here` is **gone**, because that
      is the one sentence that became false.
- [~] It states plainly that adoption has **not** happened in any consumer
      tree, and that `Clock` and order statistics are not yet extracted — the
      ladder's "may still need extraction", said out loud rather than implied.
      *Half-satisfiable as written: the first clause is done (§"What is still
      outstanding" names mitosys's `util/effect`, llm's `rec_now()` and
      `transactional.rs:72`, realm untouched, no ratchets, nothing pushed).
      The second clause is stale — `Clock` (p3, `cb49f4a`/`b1fdcee`/`c74bd90`)
      and order statistics (p4, `7dfbd86`) landed in the crate after this spec
      was written, so §"Landed" records them as in rather than pending. The
      intent — a `decided` that cannot be misread as "the extraction is
      finished" — is met by naming what is actually outstanding.*
- [x] It names `Disposer` as the landed spelling of `Handle`.
- [x] `README.md`'s `shared-crate.md` table row reads `decided`; the
      `clock.md` row is **untouched** by this spec.
- [x] `cargo test -p conserved` passes — the document now claims a crate that
      works, so the claim is checked, not asserted.

verify: `bash -c 'set -e; cd /Users/feb/dev/infra/shared; S=learnings/shared-crate.md; grep -q "^status: decided" $S || { echo "FAIL: shared-crate.md is not decided"; exit 1; }; grep -qE "^code:.*conserved/src/scope\.rs" $S && grep -qE "^code:.*conserved/src/content_id\.rs" $S || { echo "FAIL: code: does not name the landed implementation"; exit 1; }; git ls-files --error-unmatch conserved/src/scope.rs conserved/src/content_id.rs >/dev/null || { echo "FAIL: code: names files that are not committed"; exit 1; }; for c in ab154f7 5313ca4 9fff8ea c122240 8e12122; do grep -q "$c" $S || { echo "FAIL: landing commit $c is not linked"; exit 1; }; done; B=$(printf "\140"); for c in $(grep -oE "$B[0-9a-f]{7}$B" $S | tr -d "$B" | sort -u); do git cat-file -e "$c^{commit}" 2>/dev/null || { echo "FAIL: $S names $c which is not a commit"; exit 1; }; done; grep -q "distribution.md" $S || { echo "FAIL: the distribution memo is not named"; exit 1; }; for t in "Vendored into each tree" "A git dependency" "A path dependency to a sibling directory"; do grep -q "$t" $S || { echo "FAIL: option \"$t\" was erased instead of resolved"; exit 1; }; done; if grep -q "No recommendation is recorded here" $S; then echo "FAIL: the unresolved-constraint sentence is still there"; exit 1; fi; grep -q "Disposer" $S || { echo "FAIL: the landed type spelling is not named"; exit 1; }; grep -E "^\| .shared-crate\.md." README.md | grep -q "decided" || { echo "FAIL: README table row still stale"; exit 1; }; cargo test -p conserved >/dev/null || { echo "FAIL: the crate the document now claims does not pass its tests"; exit 1; }; echo "spec03 ok"'`
