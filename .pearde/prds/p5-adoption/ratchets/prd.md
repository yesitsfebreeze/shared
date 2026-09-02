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

- [~] **Extend, do not reinvent, mitosys's gate** — the shape already exists at
      `../mitosys/src/mitosys/gates/tests/write_path_reads_no_clock.rs`: a
      `WATCHED` file list and `CLOCK_READS = ["SystemTime::now", "Instant::now",
      "now_nanos", "now_ms"]`. Note `SystemTime::now` is spelled without parens
      deliberately — a bare function reference escaped an earlier grep. Widen it
      from the write path to a whole-tree count.

      **Struck — the closure is held in mitosys's own gate, and this board
      cannot run it as its own.** Extended, not reinvented: `e0e4cdd`
      ("p5-adoption/ratchets — a wall-clock ratchet gate in each of the three
      trees", 2026-08-27) is **1 file changed, 228 insertions(+)**, all into
      the existing `write_path_reads_no_clock.rs` — no new gate file in
      mitosys. The file is 419 lines and the whole-tree ratchet sits below the
      write-path check it was added to. Re-run in mitosys 2026-08-28:
      `cargo test -p mitosys-gates --test write_path_reads_no_clock` —
      **7 passed; 0 failed**. Verifying it is mitosys's gate's job on every
      run there; this board can only cite it.
- [~] **Count only the wall-clock kind.** `Instant::now()` is monotonic —
      profilers, deadlines, timeouts. Counting it would force exactly the wrong
      migration later. The two must be distinguished by the gate itself.

      **Struck — the distinction is enforced by construction in each consumer
      tree's own gate.** The const that holds it is `WALL_CLOCK_READS` at
      `../mitosys/src/mitosys/gates/tests/write_path_reads_no_clock.rs:216`,
      `&["SystemTime::now", "now_nanos", "now_ms"]` — `Instant::now` is absent
      — guarded by the self-check `monotonic_reads_are_never_counted` at
      `:375`, which fails if the two lists are ever merged. realm
      (`src/gates/tests/clock_read_ratchet.rs:41`) and model
      (`gates/tests/clock_read_ratchet.rs:68`) each carry the same pair with
      `WALL_CLOCK_READS = ["SystemTime::now"]`. Commits `e0e4cdd`, `01254aa`,
      `b5ea9b2d`. Re-run 2026-08-28: `monotonic_reads_are_never_counted ... ok`
      in all three.

      **Not `CLOCK_READS` at `:66`.** This PRD's own § Job 2 table in
      `@shared/exemptions-name-their-reason` cites "the same file's
      `CLOCK_READS` list, which excludes `Instant::now`". Measured 2026-08-28,
      that is `refuted` — line 66 reads
      `const CLOCK_READS: &[&str] = &["SystemTime::now", "Instant::now",
      "now_nanos", "now_ms"];` and **includes** it. `CLOCK_READS` is the older
      write-path check's zero-tolerance list, which forbids *any* clock read on
      the watched files and is right to include the monotonic spelling. The
      whole-tree ratchet this box asks for is the other const. A strike citing
      the wrong evidence is the defect the widening exists to catch, so the
      right one is named here.
- [~] **`../model` and `../realm` have no such gate** — each needs one built.
      realm's `just check` is `cargo check --workspace` alone (no fmt, no
      clippy, no test), so realm needs a gate host before it can host a ratchet.

      **Struck — both were built, in the trees that own them, and neither is
      this board's to re-verify.** `../model/gates/tests/clock_read_ratchet.rs`
      (296 lines, commit `b5ea9b2d`) and
      `../realm/src/gates/tests/clock_read_ratchet.rs` (264 lines, commit
      `01254aa`), both 2026-08-27, both with the same message as `e0e4cdd`.
      Re-run in their own trees 2026-08-28:
      `cargo test -p gates --test clock_read_ratchet` (model) — **3 passed;
      0 failed**; `cargo test -p realm-gates --test clock_read_ratchet`
      (realm) — **3 passed; 0 failed**. realm's gate host followed with
      `c239677` ("16-fmt-gate — cargo fmt --check is green, and just check is
      what proves it", 2026-08-27), which is the precondition this box names.
- [~] **Landed when a new read fails a named check** in each tree.

      **Struck — the named check exists in all three trees, and each tree's own
      run is what closes it.** The check is
      `wall_clock_reads_may_only_decrease`: mitosys `:337`, realm `:190`, model
      `:227`. `e0e4cdd`'s message records the proof, which was done by hand in
      each tree and cannot be redone from here: *"Every ceiling proved
      red/green by hand in its own tree: a SystemTime::now probe drives the
      count over, an Instant::now probe does not, removal restores green."*
      All three checks are green on 2026-08-28 (7 / 3 / 3 passed above).

      **The counts in this PRD's table above are `refuted` by the landing
      commit, and the table is left standing as the *before* reading.**
      `e0e4cdd`: *"Two of the three PRD-table counts are refuted by
      measurement: realm holds 0, not 3; model holds 10, not 15. mitosys is 48,
      not 40."* The ceilings on disk agree — `RATCHET_CEILING` is 48 (mitosys
      `:228`), 10 (model `:82`), 0 (realm `:55`). A strike records that the bar
      was not cleared here; it does not move the bar, so no number inside a box
      is edited.

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
