---
state: done
complexity: 22
blast-radius: mid
origin: derived
from: exemptions-name-their-reason
priority: 60
complexity: 0
blast-radius: mid
repo: shared
footprint:
  - conserved/tests/done_boxes_are_ticked.rs
  - prds/p5-adoption/ratchets/prd.md
  - prds/p5-adoption/realm/prd.md
---

# exemptions-name-their-reason — the cheapest widening, and the two PRDs it turns red

`conserved/tests/done_boxes_are_ticked.rs` is green and its `EXEMPT` list is
empty. Both facts survive only because the counter stops at the next `## `
heading.

When this is done: the gate counts every `- [ ]` in `prd.md`, `EXEMPT` is still
`&[]`, and the seven boxes the old counter never saw are each closed by one of
`done-means-done`'s three forms.

The rule is not restated here. It is
`shared/learnings/exemptions-name-their-reason.md`, which binds all four trees;
this PRD is shared's half of it.

## Job 1 — widen the counter to the whole file

| what | change |
|---|---|
| `unticked_boxes_in_acceptance` | counts every `- [ ]` in the whole `prd.md`, not the run between `## Acceptance` and the next `## `. Rename it for what it now does |
| `- [~]` | stays a closure. `- [x]` stays a closure |
| `specs/*.md` | stay out — the `name == "prd.md"` filter in `walk` is unchanged |
| `EXEMPT` | stays `&[]` |

**The doc comment is replaced, not left standing.** The file currently reads:

> - Boxes outside `## Acceptance`: the `## Requirements` section above it
>   is intentionally read-write work-in-progress; the rule is about the
>   acceptance gate, not the work log.

That sentence is the scoping decision this PRD reverses, and it is the reason
the two PRDs below are green today. A whole-file counter sitting under it is a
comment that lies about its own code. It is rewritten to name the whole-file
population and to cite `shared/learnings/exemptions-name-their-reason.md`.

## Measured, 2026-08-27

Green under `## Acceptance` scoping:

```
$ cargo test -p conserved --test done_boxes_are_ticked
running 2 tests
test exemption_list_only_names_done_prds ... ok
test every_done_prd_has_a_ticked_acceptance ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Whole-file, the same walk fails:

| PRD | state | `## Acceptance` open | `## Requirements` open | whole-file |
|---|---|---|---|---|
| `prds/p5-adoption/ratchets` | `done` | 0 | 4 | 4 |
| `prds/p5-adoption/realm` | `done` | 0 | 3 | 3 |
| **total** | | **0** | **7** | **7** |

**Every one of the 7 boxes sits under `## Requirements`.** Both PRDs are green
today for exactly one reason: the counter stops at the heading.

## Job 2 — seven boxes, two PRDs, one form each

Seven boxes and an empty `EXEMPT` is the shape that invites two entries. The
parent forbids it: *"Do not widen an exemption to make a tree green."* Each PRD
takes one of `done-means-done`'s three forms instead.

### `p5-adoption/ratchets` — 4 boxes → **strike-with-reason**

The four boxes are directives to the *consumer* trees, not deliverables of this
repo. Their closure lives in each consumer's own gate, which is
strike-with-reason's stated condition: *the work belongs elsewhere and the
local PRD cannot move it.*

| box | who holds the closure |
|---|---|
| *"Extend, do not reinvent, mitosys's gate"* | `mitosys/src/mitosys/gates/tests/write_path_reads_no_clock.rs` |
| *"Count only the wall-clock kind"* | the same file's `CLOCK_READS` list, which excludes `Instant::now` |
| *"`../model` and `../realm` have no such gate — each needs one built"* | `model/gates/tests/clock_read_ratchet.rs` and `realm/src/gates/tests/clock_read_ratchet.rs`, both on disk 2026-08-27 |
| *"Landed when a new read fails a named check in each tree"* | each of those three gates, run in its own tree |

The PRD's own frontmatter records the landing commits —
`commit: { mitosys: e0e4cdd, realm: 01254aa, model: b5ea9b2d, shared: 02d1889 }`
— and all four resolve to *"p5-adoption/ratchets — a wall-clock ratchet gate in
each of the three trees"*. Each strike quotes the gate file and the commit
beside the box. The box stays visible as `- [ ]` with its reason, per
`done-means-done`.

### `p5-adoption/realm` — 3 boxes → **strike-with-reason**

Same form, a different reason per box. All three assert facts about `../realm`,
a tree this board does not own and cannot re-verify from here.

| box | the reason the strike carries |
|---|---|
| *"`Scope` has three real sites"* | the three sites are named in the box itself (`drivers/linux/src/overlay.rs:385`, `zfs_volumes.rs:139`, `net/src/lib.rs:437`); the closure is realm's, recorded in `commit: { realm: c862aec + c239677 }` |
| *"`Clock`: 3 non-test wall-clock reads"* | the three reads are named by file:line; the ratchet that keeps the count is `realm/src/gates/tests/clock_read_ratchet.rs`, in realm's tree |
| *"Record the `ContentId`/`stats` refusal"* | **discharged in this PRD's own body**, in the paragraph directly above the section: `grep` for `blake3\|sha2\|Sha256` and `median\|percentile` across realm's `src/` and manifests returns zero hits. The strike quotes that paragraph rather than pointing at it, and the implementer re-runs the grep and quotes the output |

### The fourth form is refused

An `EXEMPT` entry for either PRD is **refused**. Neither can name a removal
condition today: the boxes are not waiting on an observable event, they are
waiting on someone writing down evidence that already exists. An entry that
cannot fill the removal-condition field means the PRD's `state` is wrong, and
the state is what gets corrected — never the list.

`EXEMPT` stays `&[]`. It has never held an entry and this PRD does not give it
its first one.

## The gate is red until both PRDs are resolved

Landing job 1 before job 2 turns `cargo test -p conserved --test
done_boxes_are_ticked` red on two rows. **That is the correct output of this
change, not a failure of it.** A gate reporting green on a condition that does
not hold is worse than no gate; seven open boxes in two `state: done` PRDs is
that condition, and it has been true since the PRDs were closed.

Land both jobs in one commit if the red window is unwanted. Do not land job 1
with two `EXEMPT` entries to keep it green — that is the move the parent's
constraint forbids by name.

## Constraints

- No box is ticked to make the gate green.
- The `conserved` crate's test carries no dependency on what it guards.
- `specs/*.md` stay outside the population. This PRD does not widen the walk to
  the spec files.

## Pointers

- `shared/learnings/exemptions-name-their-reason.md` — the rule, for all four trees
- `shared/learnings/done-means-done.md` — the three forms
- `prds/memos/done-counts-which-boxes.md` on the master board — the decision
- `../prds/exemptions-name-their-reason/prd.md` — the master-board parent

## Acceptance

- [x] `unticked_boxes_in_acceptance` counts every `- [ ]` in the whole
      `prd.md` and is renamed for it; `- [x]` and `- [~]` stay closures

      Proved by `spec01` boxes 1, 5, 6 and 7. **One deviation, recorded rather
      than argued away:** the function was not renamed in place — it was
      **replaced by two**, `opens_an_unticked_box(line)` and
      `unticked_boxes_in_file(text)`, and the `#[test]` was renamed too, from
      `every_done_prd_has_a_ticked_acceptance` to
      `every_done_prd_has_no_unticked_box`. This PRD asked only for the
      function. The test rename is argued in `spec01` § Findings 1: a test
      named `..._has_a_ticked_acceptance` whose body reads the whole file is
      the same defect this PRD names for the doc comment, one identifier over.
      ```
      $ grep -n 'in_acceptance' conserved/tests/done_boxes_are_ticked.rs
      (no output — exit 1)
      $ grep -n 'fn unticked_boxes_in_file' conserved/tests/done_boxes_are_ticked.rs
      170:fn unticked_boxes_in_file(text: &str) -> usize {
      ```
      Whole-file population proved able to fail by planting a box under a
      `## Fixture` heading in `prds/p6-scope-unwind/prd.md` — a third violation
      row appeared, where the old scoping reported nothing. Closures hold:
      `- [x]` and `- [~]` both planted into the same fixture, gate green both
      times. The matcher is wider than the family's: `* [ ]`, `+ [ ]`, `- []`,
      `- [  ]`, `-  [ ]` and a tab-indented box are all RED, while
      `- [text](url)` and `*emphasis*` stay green.
- [x] The doc comment's *"intentionally read-write work-in-progress"*
      paragraph is replaced by one naming the whole-file population and citing
      `shared/learnings/exemptions-name-their-reason.md`

      Proved by `spec01` box 2:
      ```
      $ grep -c 'read-write work-in-progress' conserved/tests/done_boxes_are_ticked.rs
      0
      $ grep -c 'exemptions-name-their-reason' conserved/tests/done_boxes_are_ticked.rs
      5
      ```
      The replacement is the module doc's `# What this file counts`: *"**The
      whole `prd.md`, every heading.** An open box under `## Requirements`,
      `## Out of scope`, or under no heading at all counts exactly as one under
      `## Acceptance` does."* Five citations, not the two required — module
      doc, `EXEMPT` doc, `unticked_boxes_in_file` doc and the panic text.
- [x] `EXEMPT` is still `&[]`, with no entry added for either PRD

      Proved by `spec01` box 3 and `spec02` box 8:
      ```
      $ grep -n 'const EXEMPT' conserved/tests/done_boxes_are_ticked.rs
      110:const EXEMPT: &[(&str, &str, &str)] = &[];
      ```
      Widened from a 2-tuple to the family's 3-tuple (PRD, commit, removal
      condition), so an entry that cannot fill all three will not compile —
      which is why neither `p5-adoption` PRD could have been given one. The
      fourth form stayed refused. `EXEMPT` has still never held an entry.
- [x] `prds/p5-adoption/ratchets`: four boxes struck with a reason, each
      naming its gate file and the commit from the PRD's own frontmatter

      Proved by `spec02` boxes 1, 2, 3 and 7. Four `- [~]`, zero `- [ ]`, box
      wording unedited, frontmatter untouched. Gate files and commits named per
      box: `write_path_reads_no_clock.rs` / `e0e4cdd`,
      `realm/src/gates/tests/clock_read_ratchet.rs` / `01254aa`,
      `model/gates/tests/clock_read_ratchet.rs` / `b5ea9b2d`. All three re-run
      in their own trees 2026-08-28: **7 / 3 / 3 passed, 0 failed each**.

      **§ Job 2's strike reason for box 2 was `refuted` and is not what
      landed.** It cites *"the same file's `CLOCK_READS` list, which excludes
      `Instant::now`"*; measured, `:66` **includes** it. The strike names
      `WALL_CLOCK_READS` at `:216` and the self-check
      `monotonic_reads_are_never_counted` at `:375` instead. A strike citing
      the wrong evidence is the defect this node exists to prevent.
- [x] `prds/p5-adoption/realm`: three boxes struck with a reason; the
      `ContentId`/`stats` box carries the re-run grep output quoted

      Proved by `spec02` boxes 4, 5, 6 and 7. Three `- [~]`, zero `- [ ]`. The
      grep was re-run in realm and pasted into the strike, not pointed at:
      ```
      $ cd ../realm
      $ grep -rnE 'blake3|sha2|Sha256' --include='*.rs' --include='*.toml' src/ Cargo.toml
      EXIT=1
      $ grep -rnE 'median|percentile' --include='*.rs' --include='*.toml' src/ Cargo.toml
      EXIT=1
      ```
      Both exit 1 across 75 files — `reproduced`. Two of the three boxes carry
      figures that are `refuted` and the strikes say so without editing the
      boxes: box 1's `385`/`139`/`437` are pre-adoption (now `overlay.rs:422`
      and `:450`, `zfs_volumes.rs:237`, `net/src/lib.rs:1122` — four sites, not
      three), and box 2's "3 wall-clock reads" is **0**, held there by
      `RATCHET_CEILING = 0`.
- [x] `cargo test -p conserved --test done_boxes_are_ticked` is run after both
      jobs and its output quoted green

      Proved by `spec02` boxes 9 and 10. Run 2026-08-28 with both jobs in the
      tree:
      ```
      running 2 tests
      test exemption_list_only_names_done_prds ... ok
      test every_done_prd_has_no_unticked_box ... ok

      test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```
      And the repo's own gate whole: **`just check` exits 0**, 15 targets `test
      result: ok`, zero `FAILED` lines. The constraint held — **no box was
      ticked to make the gate green**: all seven closures are `- [~]` strikes
      and `EXEMPT` is `&[]`.


## Built 2026-08-28 — and shared's gate inherits neither hole

Two specs. The gate is **built and in the tree, uncommitted**:
`unticked_boxes_in_acceptance` replaced by `opens_an_unticked_box` +
`unticked_boxes_in_file`, the "intentionally read-write work-in-progress" doc
paragraph gone, `EXEMPT` widened to the family's 3-tuple and still `&[]`, and
the test renamed `every_done_prd_has_no_unticked_box`.

Job 1 **reproduces this PRD's own § Measured table exactly**: `ratchets` 4,
`realm` 3, total 7, both under `## Requirements`. Rewriting all seven markers
`- [ ]` → `- [~]` with no other edit takes the gate to 2 passed / 0 failed;
files restored by `git checkout`. `cargo fmt --check --all` exit 0, clippy
silent, and `done_boxes_are_ticked` is the only red target in the workspace.

### This is the best version of the gate across the four trees

The two latent holes recorded on the other three ports
(`@model/memos/the-gate-reads-a-character-not-a-reason`) are **not inherited
here**:

- **Hole 1 is closed.** The matcher takes any of `-`/`*`/`+` and any spacing.
  Eleven variants against fixture `prds/p6-scope-unwind/prd.md`: `* [ ]`,
  `+ [ ]`, `- []`, `- [  ]`, `-  [ ]`, tab-indented, and a box under a
  non-acceptance heading **all red**; `- [x]`, `- [~]`, `- [text](url)` and
  `*emphasis*` all green. Census of the 16 `prd.md` on this board: 31 open
  boxes on the literal matcher, 31 on the wide one — **zero escapes**, the same
  answer the other trees got, now proven by a matcher that would have caught
  them.
- **Hole 2 was never here.** shared's vacuity floor is `!files.is_empty()`, not
  a magic number, and it is proven able to fail. No number was added. The
  residual weakness — a board reduced to one file still passes — is recorded in
  `spec01` rather than papered over.

`specs/*.md` are proven out of the population by the standing board with no
fixture needed: 25 open boxes live in six spec files, two of them under
`state: done` PRDs, and neither PRD is reported.

**The other three trees should adopt this matcher and this floor.** They are
strictly better and they cost nothing.

## Findings — three refutations, all in this PRD's own text

1. **§ Job 2's instruction is refuted by a memo the board already decided.** It
   says "The box stays visible as `- [ ]` with its reason, per
   `done-means-done`." Under `memos/struck-box-spelling.md` (user, 2026-08-27)
   `- [ ]` **means open** — following that sentence would leave the gate red on
   the same two rows it is meant to close. `spec02` opens with the correction.
   This PRD's § Acceptance boxes 4 and 5 and § Job 1's table already say
   `- [~]`, so it is the stale half of its own file.
2. **The proposed strike reason for `ratchets` box 2 names the wrong
   constant.** It cites "the same file's `CLOCK_READS` list, which excludes
   `Instant::now`"; measured,
   `mitosys/.../write_path_reads_no_clock.rs:66` **includes** it. The
   whole-tree ratchet uses `WALL_CLOCK_READS` at `:216` plus the self-check
   `monotonic_reads_are_never_counted` at `:375`. `spec02` requires the strike
   to name the right one — a strike citing the wrong evidence is the defect
   this whole node exists to prevent.
3. **Two counts are refuted by their own landing commit.** `e0e4cdd` says
   "realm holds 0, not 3; model holds 10, not 15. mitosys is 48, not 40";
   realm's `RATCHET_CEILING = 0`. And `p5-adoption/realm`'s box-1 line numbers
   (`385`/`139`/`437`) are pre-adoption — `c862aec` moved them to
   `overlay.rs:422,450`, `zfs_volumes.rs:237`, `net/src/lib.rs:1122`. The
   `ContentId`/`stats` refusal grep is `reproduced`: both patterns exit 1
   across 75 files.

**The canonical shrink-only statement does live here.**
`shared/learnings/exemptions-name-their-reason.md` § The exemption contract and
`shared/learnings/gates.md:76` both state it, so realm's dangling pointer
**resolves at its destination** — the problem is reachability from inside
realm, not absence. And the new `EXEMPT` doc comment states the rule in
`conserved/tests/done_boxes_are_ticked.rs` **itself** rather than pointing at a
sibling repo, which is the pattern realm should copy.

**No deferral wearing a strike's clothes on this board** — all 4 `- [~]` in
`prd.md` files and 7 in specs were read; every one records a bar the code did
not clear.

**Footprint addition:** the declared footprint names three paths; this PRD's own
`## Acceptance` boxes must be ticked in `prds/exemptions-name-their-reason/prd.md`,
so that file is a fourth.


## Done 2026-08-28 — 21 of 21, and the green came from strikes alone

`spec01` 10/10, `spec02` 11/11, and this PRD's own six `## Acceptance` boxes —
the fourth footprint path the analyst found. `just check` **exit 0**, 15 targets
`test result: ok`, zero `FAILED`. Independently re-run by the board on the same
tree: same result.

```
running 2 tests
test exemption_list_only_names_done_prds ... ok
test every_done_prd_has_no_unticked_box ... ok
test result: ok. 2 passed; 0 failed
```

Consumer gates re-run in their own trees: mitosys **7 passed**, model **3
passed**, realm **3 passed**, 0 failed each. The refusal grep re-run in realm:
both patterns exit 1 across 75 files.

**`EXEMPT` stayed `&[]` throughout and not one `[x]` was added to either
`p5-adoption` file.** The gate went green on **seven strikes**, each naming a
bar the code did not clear. That is the difference between a corrected board and
a silenced one, and here it is measurable: `grep -c '^- \[ \]'` returns 0 on
both files, `- [~]` returns 4 and 3.

`spec01` needed **no code change** — the probe's file was already correct, and
what was left was re-running the eleven break-proofs and quoting them. All five
matcher variants plus a tab-indented box came back **red**; `- [x]`, `- [~]`,
`- [text](url)` and `*emphasis*` correctly green. The vacuity floor was proven
able to fail by breaking `walk` (panic at `:192:5`), then reverted **by `sed`
rather than `git checkout`** — the file is uncommitted, so a checkout would have
destroyed it — and `diff` confirmed identical.

### The three corrections were followed, not the PRD

1. **`- [~]`, not `- [ ]`.** § Job 2's instruction was the stale half of this
   file; `struck-box-spelling` (user, 2026-08-27) decides that `- [ ]` means
   open. Following the PRD would have left the gate red on the same two rows it
   exists to close.
2. **`WALL_CLOCK_READS:216`, not `CLOCK_READS:66`** — `refuted` and reproduced
   as its opposite: `:66` *includes* `Instant::now`. The strike names `:216`
   plus the self-check `monotonic_reads_are_never_counted:375`, **and states in
   the file why `CLOCK_READS` is the wrong evidence**. A strike citing wrong
   evidence is the exact defect this node exists to prevent.
3. **Counts refuted by their own landing commit** — realm 0 (not 3;
   `RATCHET_CEILING = 0`), model 10, mitosys 48, sourced to `e0e4cdd`'s own
   message. realm's box-1 line numbers named as pre-adoption with the current
   sites given. **No number inside a box was edited** — the correction lives in
   the strike's reason, where a reader can argue with it.

### One measurement beyond the specs

`p5-adoption/realm` box 1 says "three real sites". The current count is **four
call sites across those three files** — `overlay.rs` holds two, `:422` and
`:450`. Recorded in the strike; the box wording left unedited, because narrowing
a box to match a measurement is the move this node forbids.

### This is the best of the four ports

Neither hole the other three carry is inherited. The matcher takes any of
`-`/`*`/`+` and any spacing; the vacuity floor is `!files.is_empty()`, not a
magic number that drifts. The other three trees should adopt both — recorded in
`@model/memos/the-gate-reads-a-character-not-a-reason`.
