---
state: open
mode: afk
priority: 41
est: 4h
repo: consumers
verify: "see specs — each tree's own gate must fail on a new wall-clock read"
---

# P5a — ratchets: a clock-read count that may only go down

Purpose: a count of direct wall-clock reads per tree, enforced in **that tree's
own gate**, never here. Per `learnings/clock.md` the fix order is "make it
visible **first**, with the allowlist at today's count" — so this lands BEFORE
`Clock` is adopted anywhere, and it is **blocked on nothing**.

Counts measured 2026-08-21, non-test code:

| tree | `SystemTime::now` (wall) | `Instant::now` (monotonic — DO NOT count) |
|---|---|---|
| `../mitosys` | 40 | 27 |
| `../model` | 15 | 65 |
| `../realm` | 3 | 18 |

## Requirements

- [ ] **Extend, do not reinvent, mitosys's gate** — the shape already exists at
      `../mitosys/src/mitosys/gates/tests/write_path_reads_no_clock.rs`: a
      `WATCHED` file list and `CLOCK_READS = ["SystemTime::now", "Instant::now",
      "now_nanos", "now_ms"]`. Note `SystemTime::now` is spelled without parens
      deliberately — a bare function reference escaped an earlier grep. Widen it
      from the write path to a whole-tree count.
- [ ] **Count only the wall-clock kind.** `Instant::now()` is monotonic —
      profilers, deadlines, timeouts. Counting it would force exactly the wrong
      migration later. The two must be distinguished by the gate itself.
- [ ] **`../model` and `../realm` have no such gate** — each needs one built.
      realm's `just check` is `cargo check --workspace` alone (no fmt, no
      clippy, no test), so realm needs a gate host before it can host a ratchet.
- [ ] **Landed when a new read fails a named check** in each tree.

## Held — 2026-08-21

**Not dispatchable from this board yet.** The user's instruction is to finish the
shared repo's own tools first and reconcile the consumer implementations later,
once everything here is tested and works. This node is fully specified and ready;
do not start it, and do not write into the consumer trees, until that hold lifts.
