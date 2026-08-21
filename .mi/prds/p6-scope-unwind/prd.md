---
state: analyzing
claim: analyst-p6-scope-unwind
mode: afk
priority: 50
repo: shared
verify: "cargo test -p conserved scope && cargo test -p conserved load"
---

# P6 — Scope: make "leaving something behind is not expressible" true

Purpose: `conserved::scope` does not hold the invariant its own module doc
claims. **A panicking inverse silently abandons every inverse still to come.**

Found 2026-08-21 by the `p5-adoption/load-proof` analyst, measured, not inferred:

- `close()` sets `closed` and `mem::take`s both `order` and `live` **before**
  running any inverse. When one `undo()` panics the loop is abandoned, and
  dropping a `Box<dyn FnOnce()>` does not call it.
- Five effects, inverse 3 panics -> only `[4, 3]` run. 2, 1 and 0 never run and
  never will.
- **`held()` then returns `[]`** — the scope reports nothing outstanding while
  three inverses are still owed. That is the part that turns a leak into a lie.
- At 10_000 effects with the panic in the middle, half the scope is abandoned.

This contradicts the module doc's second sentence, *"Leaving something behind is
not expressible"* — the invariant `learnings/shared-crate.md` cites as the
reason `Scope` was extracted first.

## What this is not

- **Not unsound.** No UB, no double-free; `#![forbid(unsafe_code)]` holds. The
  effect is leaked, not corrupted.
- **Not a regression.** Inherited byte-for-byte from mitosys's `util/effect`
  (p1 ported it as-is, deliberately). mitosys has had this all along.
- **Not the load proof's job.** The user decided 2026-08-21: `load-proof`
  characterises today's behaviour and this node argues the fix on its own terms.

## Blocked on

`p5-adoption/load-proof`, which lands the characterisation tests. Those tests go
red if the behaviour changes in **either** direction — so this node's first act
is to update them deliberately, which is the point: a fix must be argued, not
slipped in.

## Requirements

- [ ] **Run every inverse, even when one panics.** Catch per inverse, continue
      the reverse loop, then resume unwinding. Decide and state what happens to
      the panic payloads when more than one inverse panics — one resumed, the
      rest reported, or all collected.
- [ ] **`held()` must never lie.** An inverse that did not run is still held, or
      is reported as failed — but it is never silently absent.
- [ ] **The abort case stays honest.** A panicking inverse *while a panic is
      already in flight* aborts the process (SIGABRT). That is Rust's rule and
      uncatchable; `catch_unwind` around each inverse does not change it. Say so
      in the doc rather than implying the fix covers it.
- [ ] **`#![forbid(unsafe_code)]` still holds**, and no dependency is added.
- [ ] **This is the first SEMANTIC divergence from the mitosys source.** p1's
      port is byte-for-byte with seven recorded deviations, all mechanical. This
      adds an eighth that changes behaviour. It must be recorded at its site AND
      carried into the held `p5-adoption/mitosys` child, which now has to
      reconcile two implementations that no longer agree — mitosys keeps the old
      behaviour until it adopts.
- [ ] **Update the characterisation tests deliberately**, quoting the old and
      new expectations side by side, so the change is legible in the diff.

## Acceptance

A scope holding N effects, with an inverse that panics at any position, runs all
N inverses; `held()` reflects reality at every point; the nested-scope depth
limit and the abort-during-abort case are documented rather than silently
"fixed"; and the divergence from mitosys is recorded in both trees' terms.
