# goal

Make the repository's navigation documents agree with the distribution
decision before the initial commit enshrines the contradiction: the mechanism
is **settled**, not open.

The decision itself is not reopened here. `.mi/docs/memos/distribution.md`
already carries `status: decided`, `decided: 2026-08-21`, and a
`## Decision (2026-08-21)` section: **Option A — git dependency pinned by
commit rev**, with mitosys's offline container scoped as mitosys-side
follow-up work (a `cargo vendor` output or pre-populated registry cache),
designed before p5's adoption step and not a blocker here. This spec verifies
that record and repairs the two documents that still call it open.

## Files and dirs

- `AGENTS.md` — the "Key documents to read" table, the `Distribution memo`
  row: `how the crate reaches consumers — open, decided by the user` becomes a
  line stating the decision (git dependency pinned by commit rev; mitosys
  carries the offline cost).
- `README.md` §"The board" (the sentence ending `and the open distribution
  decision`) and §"Where the shared code lives" (which opens `The constraint
  is unresolved` and then lists three options as live). Rewrite the section to
  record the outcome: option 3 (path dependency) eliminated by the user's
  2026-08-20 requirement that the crate be distributable to every Rust repo;
  option 1 (vendoring) considered and not chosen; **option 2, a git dependency
  pinned by commit rev, is the mechanism**, with the offline-container story
  owned by mitosys. Point at the memo for the full argument.

Explicitly NOT touched:

- `learnings/shared-crate.md` — a learning is never edited to erase what it
  recorded. Its §"Where it lives" stays as the argument that was made; the
  memo is the decision.
- `.mi/docs/memos/distribution.md` — already `decided`; the record only grows.
- Any `prd.md`, on this ticket or any other — the orchestrator owns board state.

## Acceptance

- [x] `.mi/docs/memos/distribution.md` still reads `status: decided` and still
      contains its `## Decision` section naming the git dependency pinned by
      commit — unchanged by this spec (`git diff` would show no edit once the
      repo exists; before that, byte-identical content).
- [x] `AGENTS.md` no longer contains the string `open, decided by the user`,
      and its distribution row names the git-dependency-pinned-by-commit
      decision.
- [x] `README.md` no longer contains `The constraint is unresolved` or
      `the open distribution decision`.
- [x] `README.md` §"Where the shared code lives" states the chosen mechanism
      and says who carries the offline cost (mitosys), and does not present
      three live options.
- [x] `learnings/shared-crate.md` is byte-identical to its pre-spec content.

## est

0.5

verify: `sh -c 'set -e; grep -q "^status: decided" .mi/docs/memos/distribution.md; grep -q "## Decision" .mi/docs/memos/distribution.md; for p in "open, decided by the user:AGENTS.md" "The constraint is unresolved:README.md" "the open distribution decision:README.md"; do pat=${p%%:*}; f=${p##*:}; if grep -qF "$pat" "$f"; then echo "stale in $f: $pat"; exit 1; fi; done; grep -qiE "pinned by commit" README.md; grep -qiE "pinned by commit" AGENTS.md; grep -q "No recommendation is recorded here" learnings/shared-crate.md; echo "distribution decision recorded consistently"'`
