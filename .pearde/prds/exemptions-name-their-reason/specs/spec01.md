---
complexity: 12
footprint:
  - conserved/tests/done_boxes_are_ticked.rs
---

# spec01 — the counter reads the whole file, and the two spellings-holes are not inherited

Job 1 of the PRD, in one file. `conserved/tests/done_boxes_are_ticked.rs`
counted only the run between `## Acceptance` and the next `## `; it now counts
every unticked box in the file, its module doc and `EXEMPT` doc say so, and the
`EXEMPT` list stays `&[]`.

Landing this spec alone turns the gate **red on two rows**. That is the
correct output, stated by the PRD, and spec02 is what clears it.

## What already stands

**The file is rewritten and in the tree, uncommitted** — the analyst's probe.
Every measurement below was run against it; the implementer re-runs each and
quotes its own output.

- `root()`, `board_files()`, `walk()` and `frontmatter_state()` are unchanged.
  `walk` keeps its `name == "prd.md"` filter, so `specs/*.md` stay out.
- `unticked_boxes_in_acceptance` is gone. Two functions replace it:
  `opens_an_unticked_box(line)` and `unticked_boxes_in_file(text)`, the second
  a one-line filter-count over `text.lines()`. No heading state remains in the
  file.
- `#[test] fn every_done_prd_has_a_ticked_acceptance` is renamed
  `every_done_prd_has_no_unticked_box`, matching the family's spelling in
  `model/gates/tests/` and `realm/src/gates/tests/`. **This rename is not asked
  for by any PRD box** and is argued in *Findings* below.
- The module doc's *"the `## Requirements` section above it is intentionally
  read-write work-in-progress"* paragraph is gone. What replaces it names the
  whole-file population, cites
  `shared/learnings/exemptions-name-their-reason.md` and
  `../prds/memos/done-counts-which-boxes.md`, and says in its own words that a
  struck box records a bar the code did not clear rather than work still owed.
- `EXEMPT` is `&[(&str, &str, &str)] = &[]` — realm's 2-tuple widened to the
  family's three fields (PRD, commit, removal condition), so an entry that
  cannot fill all three will not compile. Its doc states the shrink-only rule
  **in this tree**, and records that the seven boxes this widening exposed were
  refused an entry because neither PRD could fill the removal-condition field.
- The panic text names the whole-file population and the three closure forms.

### Neither of the two ported holes is inherited

**Hole 1 — the literal matcher — is closed.** `opens_an_unticked_box` takes any
of Markdown's three bullets and any run of spaces on either side of the empty
bracket pair, rather than the single byte sequence the other three trees match.
Measured against the whole board, `reproduced`, fixture
`prds/p6-scope-unwind/prd.md` (a `state: done` file with zero boxes on either
matcher), eleven variants planted one at a time:

```
== variants the LITERAL matcher missed (want RED) ==
asterisk bullet  "* [ ]"           RED
plus bullet      "+ [ ]"           RED
no inner space   "- []"            RED
two inner spaces "- [  ]"          RED
two outer spaces "-  [ ]"          RED
tab-indented     "- [ ]"           RED

== the population widening (want RED) ==
under "## Fixture", not Acceptance RED

== closures and non-boxes (want green) ==
ticked           "- [x]"           green
struck           "- [~]"           green
markdown link    "- [t](u)"        green
emphasis         "*text*"          green
```

The widening costs nothing on today's board: a census of all **16** `prd.md`
under `prds/` counts **31** open boxes under the literal matcher and **31**
under the wide one — **zero escapes**, matching the board-wide sweep the other
three trees ran. The population enumerated: `done-means-done/shared-classify`,
`exemptions-name-their-reason`, `p0-foundation`, `p1-scope`, `p2-content-id`,
`p3-clock`, `p4-stats`, `p5-adoption`, `p5-adoption/close`, `p5-adoption/llm`,
`p5-adoption/load-proof`, `p5-adoption/mitosys`, `p5-adoption/ratchets`,
`p5-adoption/realm`, `p6-scope-unwind`, and the board root `prds/prd.md`.

**Hole 2 — the drifted vacuity floor — was never here to inherit.** shared's
gate has no `files.len() >= 40` magic number; it asserts `!files.is_empty()`,
which cannot drift against a real count. No number was added: a numeric floor
on a 16-file board would be the same magic number one repository smaller. The
assert is proven able to fail — see the box below. The weakness that remains,
recorded rather than fixed: a board reduced to one surviving `prd.md` would
still pass this floor.

## What is left

Re-run every proof below and quote it. No code changes.

## Findings — read before ticking

1. **The test rename ripples into two files this spec does not touch.**
   `learnings/toolchain.md:101` quotes a 2026-08-26 panic that printed
   `every_done_prd_has_a_ticked_acceptance`, and
   `prds/done-means-done/shared-classify/specs/spec01.md:34` names it in a
   closed acceptance box. **Both are correct as they stand and must not be
   edited** — a quoted historical run keeps the name it printed, and a closed
   spec records what was true when it closed. The rename was made anyway: a
   test named `..._has_a_ticked_acceptance` whose body reads the whole file is
   the same defect the PRD names for the doc comment, one identifier over.
   If the implementer judges the ripple worse than the lie, reverting the
   rename changes nothing else in this spec — every box below is by test file,
   not by test name.
2. **`specs/*.md` are proven out of the population by the standing board, with
   no fixture needed.** Six spec files hold **25** open boxes, and two of them
   sit under `state: done` PRDs —
   `prds/done-means-done/shared-classify/specs/spec01.md` (3 boxes) and
   `prds/p5-adoption/close/specs/spec04.md` (1 box). Neither PRD appears in the
   gate's violation list. Do not plant a fixture spec file to re-prove this:
   the first attempt at that proof deleted a real `specs/` directory and needed
   `git checkout` to recover.

## Acceptance

- [x] `conserved/tests/done_boxes_are_ticked.rs` holds no function named
      `unticked_boxes_in_acceptance` and no heading-state variable:
      `grep -n 'in_acceptance' conserved/tests/done_boxes_are_ticked.rs`
      returns nothing, and `grep -n 'fn unticked_boxes_in_file'` returns a line

      Implementer run 2026-08-28, `reproduced`:
      ```
      $ grep -n 'in_acceptance' conserved/tests/done_boxes_are_ticked.rs
      grep exit=1          # no output, no match
      $ grep -n 'fn unticked_boxes_in_file' conserved/tests/done_boxes_are_ticked.rs
      170:fn unticked_boxes_in_file(text: &str) -> usize {
      ```
      No heading-state variable survives: the file holds no `in_acceptance`,
      no `## Acceptance` scan and no section flag — `unticked_boxes_in_file`
      is a one-line filter-count over `text.lines()`.
- [x] The module doc's *"intentionally read-write work-in-progress"* sentence
      is gone and the replacement names the whole-file population and cites
      `shared/learnings/exemptions-name-their-reason.md`:
      `grep -c 'read-write work-in-progress'` returns 0 and
      `grep -c 'exemptions-name-their-reason'` returns at least 2 (module doc
      and `EXEMPT` doc)

      Implementer run 2026-08-28, `reproduced`:
      ```
      $ grep -c 'read-write work-in-progress' conserved/tests/done_boxes_are_ticked.rs
      0
      $ grep -c 'exemptions-name-their-reason' conserved/tests/done_boxes_are_ticked.rs
      5
      ```
      5, not 2: the module doc's `# What this file counts` heading, the
      `EXEMPT` doc, `unticked_boxes_in_file`'s doc and the panic text all cite
      it. The replacement paragraph reads *"**The whole `prd.md`, every
      heading.** An open box under `## Requirements`, `## Out of scope`, or
      under no heading at all counts exactly as one under `## Acceptance`
      does."*
- [x] `EXEMPT` is empty and is a 3-tuple —
      `grep -n 'const EXEMPT' conserved/tests/done_boxes_are_ticked.rs` returns
      a line reading `const EXEMPT: &[(&str, &str, &str)] = &[];` — and its doc
      states the shrink-only rule in this file rather than pointing at another
      repository

      Implementer run 2026-08-28, `reproduced`:
      ```
      $ grep -n 'const EXEMPT' conserved/tests/done_boxes_are_ticked.rs
      110:const EXEMPT: &[(&str, &str, &str)] = &[];
      ```
      The doc above it states the rule in this file, not by pointer:
      *"**Shrink-only.** Entries leave this list when a child PRD closes them;
      they never enter it to silence a regression. Widening an exemption to
      make a tree green is the move the family rule forbids by name."* The
      sibling-repo citation is present as a source, not as the statement.
- [x] `cargo test -p conserved --test done_boxes_are_ticked` runs and is **red
      on exactly two rows** until spec02 lands, with the counts the PRD's
      § Measured table predicts. Quote it. Analyst run 2026-08-28:
      ```
      running 2 tests
      test exemption_list_only_names_done_prds ... ok
      test every_done_prd_has_no_unticked_box ... FAILED

      2 `state: done` PRD(s) carry unticked boxes — the count is the whole
      file, every heading, per
      `shared/learnings/exemptions-name-their-reason.md`. …
        prds/p5-adoption/ratchets/prd.md: 4 unticked box(es)
        prds/p5-adoption/realm/prd.md: 3 unticked box(es)

      test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
      ```

      Implementer run 2026-08-28, `reproduced` — two rows, 4 and 3, matching
      the PRD's § Measured table exactly:
      ```
      running 2 tests
      test exemption_list_only_names_done_prds ... ok
      test every_done_prd_has_no_unticked_box ... FAILED

      thread 'every_done_prd_has_no_unticked_box' panicked at
      conserved/tests/done_boxes_are_ticked.rs:227:9:
      2 `state: done` PRD(s) carry unticked boxes — the count is the whole
      file, every heading, per
      `shared/learnings/exemptions-name-their-reason.md`. Either tick the box
      with quoted evidence, strike it with a measured reason, or correct the
      PRD's state; an exemption entry that cannot name a PRD, a commit and the
      condition that removes it is not written:
        prds/p5-adoption/ratchets/prd.md: 4 unticked box(es)
        prds/p5-adoption/realm/prd.md: 3 unticked box(es)

      test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
      ```
- [x] The whole-file widening is proven able to fail, **by planting a box under
      a non-acceptance heading** in a clean `state: done` PRD
      (`prds/p6-scope-unwind/prd.md`), running the gate, seeing that file named
      as a third row, and restoring with `git checkout`. Quote the three-row
      violation list and the clean `git status --short` after

      Implementer run 2026-08-28, `reproduced`. `- [ ] planted` appended under
      a fresh `## Fixture` heading — not `## Acceptance` — of a `state: done`
      file that carries zero boxes on either matcher:
      ```
      running 2 tests
      test exemption_list_only_names_done_prds ... ok
      test every_done_prd_has_no_unticked_box ... FAILED

      thread 'every_done_prd_has_no_unticked_box' panicked at
      conserved/tests/done_boxes_are_ticked.rs:227:9:
      3 `state: done` PRD(s) carry unticked boxes — ...:
        prds/p5-adoption/ratchets/prd.md: 4 unticked box(es)
        prds/p5-adoption/realm/prd.md: 3 unticked box(es)
        prds/p6-scope-unwind/prd.md: 1 unticked box(es)

      test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
      ```
      A third row appeared for a box under no acceptance heading — the old
      `## Acceptance`-scoped counter would have reported nothing. Restored:
      ```
      $ git checkout -- prds/p6-scope-unwind/prd.md
      $ git status --short prds/p6-scope-unwind/
      (no output — clean)
      ```
- [x] The bullet and spacing variants are proven able to fail: plant `* [ ]`,
      `+ [ ]`, `- []`, `- [  ]` and `-  [ ]` one at a time into the same
      fixture, quote each verdict, restore after each. All five must be RED.
      The script the analyst used is
      `<scratchpad>/fixture.sh`; re-derive it or re-run it

      Script re-derived (plant one line under `## Fixture`, run the gate, RED
      iff the fixture path appears in the violation list, `git checkout` after
      each). Implementer run 2026-08-28, `reproduced` — all five RED, plus the
      tab-indented sixth:
      ```
      == variants the LITERAL matcher missed (want RED) ==
      asterisk bullet  "* [ ]"           RED
      plus bullet      "+ [ ]"           RED
      no inner space   "- []"            RED
      two inner spaces "- [  ]"          RED
      two outer spaces "-  [ ]"          RED
      tab-indented     "- [ ]"           RED
      ```
      `git status --short prds/` after the whole run named no fixture file —
      every plant was restored.
- [x] `- [x]` and `- [~]` are proven to be closures: plant each into the same
      fixture, quote the gate green both times, restore. `- [~]` is the
      spelling settled by `../prds/memos/struck-box-spelling.md` (user,
      2026-08-27)

      Implementer run 2026-08-28, `reproduced` — green means the fixture is
      **not** named in the violation list (the two `p5-adoption` rows stay red
      until spec02, which is spec01's stated state):
      ```
      == closures and non-boxes (want green) ==
      ticked           "- [x]"           green
      struck           "- [~]"           green
      markdown link    "- [t](u)"        green
      emphasis         "*text*"          green
      ```
      Both closures pass, and the two non-box shapes that share the bracket or
      the bullet byte — a markdown link and an emphasis run — are not
      miscounted as boxes. Each plant restored with `git checkout`.
- [x] The vacuity assert fails rather than passes, **proven by breaking it**:
      point `walk` at `root.join("prds-moved-away")`, run, quote the panic,
      restore the line and quote it restored. Analyst run 2026-08-28,
      `reproduced`:
      ```
      thread 'every_done_prd_has_no_unticked_box' panicked at
      conserved/tests/done_boxes_are_ticked.rs:192:5:
      found no prd.md under /Users/feb/dev/infra/shared/prds — a gate that
      reads nothing must fail, because a moved board would otherwise turn this
      check silently off
      ```

      Implementer run 2026-08-28, `reproduced`. `board_files` pointed at
      `root.join("prds-moved-away")` at line 62:
      ```
      thread 'every_done_prd_has_no_unticked_box' panicked at
      conserved/tests/done_boxes_are_ticked.rs:192:5:
      found no prd.md under /Users/feb/dev/infra/shared/prds — a gate that
      reads nothing must fail, because a moved board would otherwise turn this
      check silently off
      ```
      The floor is `!files.is_empty()`, not a numeric ceiling, so it cannot
      drift against a real count. Restored — reverted by `sed`, never
      `git checkout`, because this file's whole rewrite is uncommitted:
      ```
      $ sed -n '62p' conserved/tests/done_boxes_are_ticked.rs
      	walk(&root.join("prds"), &mut out);
      $ diff <pre-break copy> conserved/tests/done_boxes_are_ticked.rs
      IDENTICAL
      ```
      The residual weakness spec01 records rather than fixes stands: a board
      reduced to one surviving `prd.md` still passes this floor.
- [x] `cargo fmt --check --all` and
      `cargo clippy --workspace --all-targets -- -D warnings` both exit 0 —
      analyst run 2026-08-28, `FMT_EXIT=0`, clippy silent

      Implementer run 2026-08-28, `reproduced`:
      ```
      $ cargo fmt --check --all
      FMT_EXIT=0                       # no diff printed
      $ cargo clippy --workspace --all-targets -- -D warnings
          Checking conserved v0.1.0 (/Users/feb/dev/infra/shared/conserved)
          Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
      ```
      Clippy emitted no warning line at all, so `-D warnings` had nothing to
      promote.
- [x] `cargo test --workspace --no-fail-fast` shows
      `done_boxes_are_ticked` as the **only** failing target in the workspace.
      Analyst run 2026-08-28: 13 test targets, 12 `ok`, one `FAILED`, and the
      passing targets total 82 tests, 0 failed

      Implementer run 2026-08-28, `reproduced` — 12 `ok`, one `FAILED`, and
      `error: 1 target failed:`:
      ```
      clock_instant       11 passed    content_id_serde     0 passed
      clock_serde          0 passed    load_scope           5 passed
      clock_source        10 passed    load_throughput      1 passed
      content_id          18 passed    load_unwind_panic    7 passed
      content_id_props     7 passed    scope                5 passed
                                       smoke                1 passed
                                       stats               14 passed
      Doc-tests conserved  3 passed

      done_boxes_are_ticked  test result: FAILED. 1 passed; 1 failed
      error: 1 target failed:
      ```
      79 + 3 doc-tests = **82 passing tests, 0 failed**, matching the analyst's
      figure. The single red target is the one spec01 lands red on purpose;
      spec02 clears it.

## Verify and Proof

```sh
cd /Users/feb/dev/infra/shared
grep -n 'in_acceptance\|fn unticked_boxes_in_file\|const EXEMPT' \
  conserved/tests/done_boxes_are_ticked.rs
grep -c 'read-write work-in-progress\|exemptions-name-their-reason' \
  conserved/tests/done_boxes_are_ticked.rs
cargo fmt --check --all; echo "FMT_EXIT=$?"
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p conserved --test done_boxes_are_ticked   # red on 2 rows, by design
cargo test --workspace --no-fail-fast
git status --short prds/                                # empty after each proof
```
