---
state: done
est: 3.5h
mode: afk
priority: 30
verify: "cargo test -p conserved clock"
---

# P3 — Clock: time as a parameter

Purpose: the type that makes law 2's verification possible — *serialize the
record, fold from empty, compare* is impossible wherever a fold reads the
wall clock. Spec is `learnings/clock.md`: both trees read the clock ~65 times
each in non-test code against a shared law that forbids it, and llm's
`rec_now()` feeds a live read into a content-hash preimage. Blocked on
`p0-foundation`.

## Requirements

- [x] **The types** — `Instant(i64)` (a real timestamp, not a tick counter —
      the condemned scaffold got this wrong), `trait Clock { fn now(&self) ->
      Instant }`, `SystemClock` as the ONE implementation permitted to read
      the OS, `FixedClock(Instant)` for folds and tests. No dependencies —
      `SystemClock` reads `std::time`, not `chrono`.
- [x] **The unit, stated** — `Instant`'s epoch and resolution written in the
      type's doc comment and pinned by a test, so the two trees cannot adopt
      it meaning different things.
- [x] **The ratchet stays out** — the count-of-clock-reads ratchet described
      in `learnings/clock.md` is a consumer gate, not crate code. p5 carries
      it; this node only makes the compliant path exist.

## Acceptance

`conserved::clock` compiles with no dependencies; `FixedClock` makes a
deterministic fold expressible; the unit test pins epoch and resolution.
