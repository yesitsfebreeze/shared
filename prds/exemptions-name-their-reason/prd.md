---
state: open
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

- [ ] `unticked_boxes_in_acceptance` counts every `- [ ]` in the whole
      `prd.md` and is renamed for it; `- [x]` and `- [~]` stay closures
- [ ] The doc comment's *"intentionally read-write work-in-progress"*
      paragraph is replaced by one naming the whole-file population and citing
      `shared/learnings/exemptions-name-their-reason.md`
- [ ] `EXEMPT` is still `&[]`, with no entry added for either PRD
- [ ] `prds/p5-adoption/ratchets`: four boxes struck with a reason, each
      naming its gate file and the commit from the PRD's own frontmatter
- [ ] `prds/p5-adoption/realm`: three boxes struck with a reason; the
      `ContentId`/`stats` box carries the re-run grep output quoted
- [ ] `cargo test -p conserved --test done_boxes_are_ticked` is run after both
      jobs and its output quoted green
