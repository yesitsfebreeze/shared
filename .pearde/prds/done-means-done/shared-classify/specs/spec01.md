---
complexity: 100   # this PRD's only spec — carries the whole PRD's weight
footprint:
  - prds/p0-foundation/prd.md
  - conserved/tests/done_boxes_are_ticked.rs
---
<!-- Add your own keys freely; nothing outside complexity and footprint is read. -->

# spec01 — tick p0-foundation's four boxes, shrink the gate's exemption list, quote green

Close out the per-row classification the PRD's `## Classification` table already
computed: run the workspace test suite and quote it, tick `p0-foundation`'s four
`## Requirements` boxes with the table's evidence, then remove `p0-foundation`
from `done_boxes_are_ticked.rs`'s `EXEMPT` list (shrink-only, per that file's own
doc comment) and re-run the gate to prove the shrink didn't break anything.

Order matters: run the workspace suite first (box 1) before touching anything,
so the "quoted green output" is captured against the pre-edit tree exactly as
box 1 asks. Then edit `p0-foundation/prd.md` (box 2). Then edit the `EXEMPT`
array and re-verify (box 3).

## Acceptance

- [ ] `cargo test -p conserved --workspace`, run from `shared/`, quoted in the
      implementer's report, shows every test passing (0 failed)
- [ ] `prds/p0-foundation/prd.md`'s four `## Requirements` boxes (`git init`,
      `Condemn the scaffold`, `One crate, one manifest`, `Distribution
      decision`) each read `- [x]`, with the corresponding evidence line from
      this PRD's `## Classification` table copied in next to (or under) each
      box — not just flipped bare
- [ ] `conserved/tests/done_boxes_are_ticked.rs`'s `EXEMPT` const no longer
      contains the `prds/p0-foundation/prd.md` entry, and
      `cargo test -p conserved --test done_boxes_are_ticked` still passes both
      `every_done_prd_has_a_ticked_acceptance` and
      `exemption_list_only_names_done_prds` (quoted)

<!-- The implementer ticks a box [x] only for a check it actually ran, quoting
     the output in its report — and ticks it WHEN it runs it, not in a batch
     at the end: these boxes are the only thing on the board that moves while
     a run is in flight, and the plan is drawn from them.
     Never write a box that asks for a commit or a commit message — the
     orchestrator commits the PRD on the transition that lands it. -->

## Verify and Proof

```sh
cd /Users/feb/dev/infra/shared
cargo test -p conserved --workspace
cargo test -p conserved --test done_boxes_are_ticked
```
