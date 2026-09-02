---
type: learning
learning: exemptions-name-their-reason
subject: a `done_boxes_are_ticked` gate reads the whole `prd.md`, every tree carries one, and every exemption entry names the PRD, the commit and the condition that removes it
binds: [mitosys, model, realm, shared]
status: decided
date: 2026-08-27
code: mitosys src/mitosys/gates/tests/done_boxes_are_ticked.rs; realm src/gates/tests/done_boxes_are_ticked.rs; shared shared/tests/done_boxes_are_ticked.rs
---

# exemptions-name-their-reason — one population, one contract, four trees

`done-means-done` shipped `done_boxes_are_ticked` into three trees and left
three questions open: which boxes the gate counts, what an exemption entry
owes, and whether `model` carries the gate at all. The three answers are here.

A gate reporting green on a condition that does not hold is worse than no
gate — it buys confidence nobody paid for. Each clause below exists because
one tree was in that state on 2026-08-27.

## The scope rule

**A `done_boxes_are_ticked` gate counts every `- [ ]` in the whole `prd.md`,
not the ones under `## Acceptance` only.**

| clause | value |
|---|---|
| population | the whole file, every heading, `prd.md` only — `specs/*.md` stay out |
| `- [~]` | a closure, not an open box |
| `- [x]` | a closure |
| the decision | `.pearde/memos/done-counts-which-boxes.md`, taken by the user 2026-08-27 |
| the counter already on it | `resources/board/plan.py`, `body_has_open_box` at `:305`, required clear by `standing` at `:325` before a PRD is offered for collect |

Whole-file wins because the widest reading is the only one that converges.
`realm`'s gate is whole-file and `mitosys`'s stops at the next `##`; a board
that dispatches on the narrower one goes on recommending closures a gate will
refuse.

## The exemption contract

Every `EXEMPT` entry names three things. All three, or the entry is not
written:

| field | what it names |
|---|---|
| the PRD | the board node whose completion removes this entry |
| the commit | the commit that justifies the claim the exempt PRD is making |
| the condition | the observable event that removes the entry — distinct per entry |

**An entry that cannot fill all three means the PRD's `state` is wrong, and
the state is what gets corrected.** Not the entry, not the box, not the gate.
The three forms a state correction takes are `done-means-done`'s:
strike-with-reason, `blocked` with `needs:`, or a child PRD.

Two further rules, unchanged from `shared/learnings/gates.md` § The port
rules: the list is shrink-only, and a reason string shared between entries is
one reason, not many. `mitosys` holds 8 entries carrying the single string
`"closed by this node's master-board lane"` — 8 occurrences, `grep -c`,
2026-08-27. That is one exemption wearing eight paths.

## The family subset

**Every tree carries `done_boxes_are_ticked`.** It joins `source_layout`,
`one_vocabulary`, `dependency_tree` and `board_is_tracked` in the subset
`shared/learnings/gates.md` names.

| tree | gate | state 2026-08-27 |
|---|---|---|
| mitosys | `src/mitosys/gates/tests/done_boxes_are_ticked.rs` | red on `p6k10d-production-fold`, 5 boxes; `## Acceptance`-scoped |
| realm | `src/gates/tests/done_boxes_are_ticked.rs` | green; already whole-file |
| shared | `shared/tests/done_boxes_are_ticked.rs` | green; `## Acceptance`-scoped, `EXEMPT` empty |
| model | **none** | `gates/tests/` holds 5 files and this is not one |

`shared/learnings/gates.md`'s scoping sentence — *"the 2026-08-23 scan named
PRDs only in mitosys/realm/shared; model was never flagged"* — is **expired**.
It was true on 2026-08-23 and false by 2026-08-27: `model/next-wave/sampler`
is `state: done` with three open acceptance boxes, one of them *"Incremental
decode is ≥ 1.5× faster than the full-sequence forward"*, which
`model/phase-8/live-grade` recorded as its own blocker. The tree the defect
was found in is the tree with no gate.

## The cost, measured — 2026-08-27

Widening turns **25 `## Requirements` boxes across 7 `done` PRDs** into
violations that `## Acceptance`-scoping hid. Counted off disk, per PRD:

| tree | PRD | `## Requirements` open |
|---|---|---|
| mitosys | `.pearde/prds/p6-rust-core/p6k-kern-merge/p6k9-fold-and-seq` | 1 |
| mitosys | `.pearde/prds/p6-rust-core/p6k-kern-merge/p6k8-read-path-remaining` | 3 |
| mitosys | `.pearde/prds/p7-plugin-abi/p7e-wasm-host` | 6 |
| mitosys | `.pearde/prds/p7-plugin-abi/p7f-arena` | 4 |
| mitosys | `.pearde/prds/p7-plugin-abi/p7b-service-seam` | 4 |
| **mitosys** | **5 PRDs** | **18** |
| shared | `.pearde/prds/p5-adoption/ratchets` | 4 |
| shared | `.pearde/prds/p5-adoption/realm` | 3 |
| **shared** | **2 PRDs** | **7** |
| **all** | **7 PRDs** | **25** |

Both `## Acceptance`-scoped gates carry a doc comment reading *"the
`## Requirements` section above it is intentionally read-write
work-in-progress; the rule is about the acceptance gate, not the work log"*
(`mitosys/src/mitosys/gates/tests/done_boxes_are_ticked.rs`,
`shared/shared/tests/done_boxes_are_ticked.rs`).

**This decision overrides that sentence.** A `## Requirements` box in a
`state: done` PRD is not a work log — it is an open claim in a file whose
frontmatter asserts the work is finished, which is the same defect
`done-means-done` exists to remove. The sentence is replaced, not left
standing beside its own contradiction.

The wider population is also why the widening is not free: it costs 25 real
closures across two trees, and each of the seven PRDs takes one of
`done-means-done`'s three forms rather than an exemption entry.

## The board-wide population

Whole-file, `prd.md` only, 2026-08-27:

| board | `done` PRDs with an open box | open boxes |
|---|---|---|
| mitosys | 9 | 42 |
| model | 1 | 3 |
| realm | 0 | 0 |
| shared | 2 | 7 |
| **the four member boards** | **12** | **52** |
| `.pearde/` (the master board) | 1 | 5 |

A further ~235 boxes sit in `specs/` of `done` PRDs across the family. They
are outside this decision's population and outside every gate's: no
`done_boxes_are_ticked` reads `specs/*.md`, by the `name == "prd.md"` filter
each of the three walks carries.

**Corrected 2026-08-27, on the round that closed this PRD.** The mitosys row
read 9 / 42 at measurement (10:53). At 12:13, `@mitosys/…/p6k10d-production-fold`
closed its five open boxes (commit `4b3edf5`), so the row now reads **8 / 37**
and the four-member total **11 / 47**. The conclusion the census supports is
unchanged: the master board's own `.pearde/prds/shared-crate-home` remains the one
`done` PRD no gate can reach.

## Nothing covers the master board

`.pearde/` — the master board at `/Users/feb/dev/infra/.pearde` — has **no gate, and
nowhere to put one**. Said plainly rather than left to be discovered:

1. `/Users/feb/dev/infra` is **not a git repository**. `git rev-parse
   --show-toplevel` returns `fatal: not a git repository`, so
   `board_is_tracked` has nothing to ask.
2. `/Users/feb/dev/infra` carries **no Cargo workspace**. There is no root
   `Cargo.toml`, so the `root()` walk every gate in the family uses — climb
   until a `Cargo.toml` contains `[workspace]` — never resolves, and there is
   no crate for a test to live in.

**Corrected 2026-08-27, on the round that closed this PRD.** Point 1 is
stale: `/Users/feb/dev/infra` entered version control at 11:30, commit
`20d84db`, after this document was measured at 10:53. `board_is_tracked`
could ask it something today. The conclusion stands on point 2 alone — no
root `Cargo.toml`, no `[workspace]`, no crate for a test to live in — so
**no gate covers the master board** remains true, for a reason that is now
the only one.

The live violation this leaves standing, named rather than implied:
**`.pearde/prds/shared-crate-home` is `state: done` with 5 unticked `## Acceptance`
boxes.** No gate in any tree covers it. It is the thirteenth PRD of the census
above and the only one no gate can ever reach as things stand.

## Who amends `references/parts/loop.md`

`references/parts/loop.md:100` still reads *"A PRD is **finished** when every
acceptance box in its specs is `[x]`"* — the narrow rule, superseded by the
decision above and not yet edited.

That file belongs to **`pearde/.pearde`**, by that board's own admission rule:
*"A PRD lives here if it changes this repo."* `pearde` is not a member of the
master board and not one of the four trees this document binds. The amendment
is `pearde/.pearde/prds/finished-counts-both-files`, and no tree's board edits it.

## What each tree owes

| tree | child PRD | owed |
|---|---|---|
| mitosys | `.pearde/prds/exemptions-name-their-reason` | widen to whole-file; replace one reason string with 8 entries that each name a PRD, a commit and a distinct removal condition |
| shared | `.pearde/prds/exemptions-name-their-reason` | widen to whole-file; resolve `p5-adoption/ratchets` and `p5-adoption/realm` by one of the three forms; `EXEMPT` stays `&[]` |
| model | `.pearde/prds/exemptions-name-their-reason` | port `realm/src/gates/tests/done_boxes_are_ticked.rs`; land `EXEMPT` empty; close `next-wave/sampler` |
| realm | `.pearde/prds/exemptions-name-their-reason` | delete the one dead `EXEMPT` entry; no scope change owed |
