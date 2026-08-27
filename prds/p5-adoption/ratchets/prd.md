---
state: done
mode: afk
priority: 41
est: 4h
repo: consumers
needs:
  - "@realm/15-gates"
verify: "see specs — each tree's own gate must fail on a new wall-clock read"
complexity: 30
blast-radius: low
commit: { mitosys: e0e4cdd, realm: 01254aa, model: b5ea9b2d, shared: 02d1889 }
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

## Held — 2026-08-21 — LIFTED 2026-08-26

**Not dispatchable from this board yet.** The user's instruction is to finish the
shared repo's own tools first and reconcile the consumer implementations later,
once everything here is tested and works. This node is fully specified and ready;
do not start it, and do not write into the consumer trees, until that hold lifts.

**The hold lifted on 2026-08-26** (user decision, recorded in full at the
parent's `## Answers — 2026-08-26`). Its condition is met: every other PRD in
this repository — `p0-foundation`, `p1-scope`, `p2-content-id`, `p3-clock`,
`p4-stats`, `p6-scope-unwind`, `load-proof`, `close` — is `state: done`. This
node is dispatchable, and writing into the consumer tree is now in scope.

Before pinning `conserved`, read the parent's Answer 5: the remote recorded in
Answer 1 (`inner-zirkle`) is stale, the live one is
`https://github.com/yesitsfebreeze/shared.git`, and this repository holds
commits that have never been pushed — a `rev` that is not on the remote cannot
be fetched from a container or another machine.


## Corrected 2026-08-26 — `failed` was a mis-sweep, this is `specced`

Set `failed` by a session sweep on 2026-08-25, which is what step 1 of the loop
does to a `claimed` PRD with no live worker. It is wrong here, and the specs on
disk say so: `specs/spec01.md`, `spec02.md` and `spec03.md` exist, complete,
with **0 of 16** acceptance boxes ticked and no `## Failure` section written.
That is the signature of an analyst that finished and an implementer that never
landed a box — not of an attempt that produced bad work.

The specs were never committed (`git status` reported the whole directory
untracked, reproduced 2026-08-26), so the analyst's output was one `git clean`
away from being lost. It is committed with this transition.

`needs: "@realm/15-gates"` resolves: that PRD is `done`.
