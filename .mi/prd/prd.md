---
state: open
mode: afk
priority: 0
max-workers: 2
verify: "cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace"
---

# conserved — the shared crate every Rust tree depends on

This board turns `learnings/shared-crate.md` into claimable work. **The specs
live in `learnings/`, not here** — [[shared-crate]] is the proposal,
[[content-addressing]], [[clock]] and [[inventory]] carry the arguments. A node
here records claim state and worker decisions, never a second copy of the spec.

The product: one small crate, `conserved`, holding domain-free utilities that
more than one consumer needs today — `Scope`/`Handle` (reversible effects),
`ContentId` (blake3, `[u8; 32]`), `Clock`/`Instant`, order statistics, with hex
private behind `ContentId`'s `Display`/`FromStr`. Distributable to every Rust
tree in the family — `../model` (llm), `../mitosys`, `../realm` today, any
future repo tomorrow. Roughly 600–800 lines including tests.

## The admission test (from `learnings/shared-crate.md`)

A thing enters this crate only when **all four** hold: both trees need it
today; it is domain-free; it passes mitosys's `dependency_tree.rs` gate; one
implementation is genuinely better than two. Nothing enters on speculation.

## Consumers and their constraints

| consumer | path | constraint the crate inherits |
|---|---|---|
| llm | `../model` | edition 2024, no pin — permissive |
| mitosys | `../mitosys` | edition 2021, pinned **1.94.0**, offline dev container |
| realm | `../realm` | edition 2021, no pin |

The crate compiles for the oldest consumer: **edition 2021, rustc 1.94.0**.
The offline container is the constraint that decides distribution (p0).

## Children

- [ ] `p0-foundation/` — git init, condemn the pre-split scaffold
  (`.mi/docs/memos/scaffold-reset.md`), fix the workspace manifest, settle
  edition/pin/test-layout, and put the distribution decision to the user
  (`.mi/docs/memos/distribution.md`). **Everything else is blocked on this.**
- [ ] `p1-scope/` — port `Scope`/`Handle` from mitosys `util/effect`, as-is.
  262 lines, zero dependencies. The first move: it tests the *mechanism*
  (does the dependency resolve, on a clone, in the container), not the code.
- [ ] `p2-content-id/` — `ContentId`: blake3 into `[u8; 32]`, 64-lowercase-hex
  `Display`, rejecting `FromStr`, hex private behind it. The one dependency
  (`blake3`) enters here and is reachable only through this module.
- [ ] `p3-clock/` — `Instant(i64)`, `trait Clock`, `SystemClock` (the ONE
  implementation permitted to read the wall clock), `FixedClock`. No deps.
- [ ] `p4-stats/` — `percentile`/`median`/`min_median_max` over sorted slices.
  ONE definition of median, stated in the doc comment. No deps.
- [ ] `p5-adoption/` — each consumer replaces its hand-rolled duplicate with a
  `conserved` dependency; the load proof; the ratchets land in consumer gates.

## Order

p0 alone, then p1 alone (it proves the mechanism p0 chose), then p2/p3/p4 in
any order or parallel, then p5 per-consumer as extractions land. Run
`mi-gantt` against this board to derive and store the schedule.

## Out of scope — recorded in `learnings/shared-crate.md` §"What stays out"

Vector math/quantization, the event spine, the record and its fold, the file
watcher, the reload seam, the grade envelope. Each has a stated reason; do not
re-admit any of them here without a new learning superseding that section.
The crate stays domain-free: no agent, no model, no peer, no surface.
