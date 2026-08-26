---
complexity: 20
footprint:
  - ../model/tests/clock_read_ratchet.rs
---
<!-- footprint paths are relative to shared/ (this repo), pointing into the
     sibling repo the work actually lands in. -->

# spec03 — model: a standalone wall-clock-read ratchet, no new crate needed

## A finding this spec deliberately routes around — read before implementing

`../model`'s own board already carries `model/prds/clock-gate/prd.md`
(`state: open`, `needs: adopt-gates`) planning close to this same gate, as a
fifth file inside a not-yet-built `gates/` crate that `model/prds/adopt-gates`
(`state: open`, itself needing `adopt-test-law` [done] and `workspace-deps`
[**`state: blocked`** at analysis time]) would create. Chaining onto that
plan would make this PRD's model leg blocked on `workspace-deps` — which
contradicts this PRD's own "blocked on nothing" purpose and `learnings/clock.md`'s
"make it visible first" ordering. So **this spec does not use that chain**:
it lands as one standalone root-level integration test file, the same way
`../model/tests/peer_churn.rs` already exists as a root-level integration
test with no crate of its own — `llm` is a single package, and its root
`tests/` directory already auto-compiles as its own test binary under
`cargo test`, so no `gates/` crate, no `Cargo.toml` change, and no dependency
on `adopt-gates`/`workspace-deps` is needed to satisfy this PRD's own
requirement. Whoever later lands `adopt-gates`/`clock-gate` on model's own
board should reconcile the two (move this file in, or retire one) — that
reconciliation is model's own board's call, not this spec's, and is reported
as a finding rather than acted on here.

## What exists when done

`../model/tests/clock_read_ratchet.rs`, a new root-level integration test:

- `const WALL_CLOCK_READS: &[&str] = &["SystemTime::now"];` — model has no
  wrapped `now_nanos`/`now_ms`-style helper the way mitosys does (`rg 'fn
  now_nanos|fn now_ms' ../model/src` returns nothing at analysis time; if the
  implementer finds one has been added since, include it). `Instant::now` is
  excluded by construction (monotonic; PRD requirement 2) — model's 68
  non-test `Instant::now()` call sites (measured 2026-08-25; `learnings/clock.md`'s
  2026-08-23 count was 65, this PRD's table says 65 too — some drift since,
  unexplained, not this spec's to chase) must never move this ratchet.
- A walker over `src/` from the workspace root, matching
  `model/prds/clock-gate/prd.md`'s already-written matcher exactly (reuse its
  wording, it is the same law): comment lines, any path with a `tests/`
  component, files named `tests.rs`/`*_tests.rs`, and `#[cfg(test)]` blocks
  excluded.
- `const RATCHET_CEILING: usize = <N>;` — **re-measure at landing**, do not
  copy 15: `rg -n --pcre2 'SystemTime::now\b' src -g '!**/target/**'`
  filtered the same way found **15** wall-clock reads on 2026-08-25,
  matching the PRD's 2026-08-21 count of 15 exactly — re-derive anyway, do
  not assume it still holds by landing time.
- `#[test] fn wall_clock_reads_may_only_decrease()`.
- `#[test] fn monotonic_reads_are_never_counted()`.
- A vacuity test: the walk must touch at least as many files as some floor
  well under the landing count, same rationale style as
  `mitosys/src/mitosys/gates/tests/write_path_reads_no_clock.rs`'s own
  vacuity test.
- Module doc naming the two known content-hash bugs `learnings/clock.md`
  already names (`rec_now()` at `src/record/mod.rs:239`, `Commit::new` at
  `src/node/transactional.rs:72`) as visible-but-not-fixed by this gate — the
  gate's job is to stop the count from growing, not to fix either; fixing is
  separate work already tracked on `../model`'s own board (`clock-gate`'s
  own text says the same).

## Acceptance

- [ ] `cargo test --test clock_read_ratchet` (from `../model`) passes.
- [ ] `RATCHET_CEILING` equals a freshly re-measured wall-clock count at
      landing time (quote the measurement command and output), not the 15
      measured by this spec or the PRD's table — re-derive, do not copy.
- [ ] A temporarily added `SystemTime::now()` call anywhere under `src/`
      (added, tested, then reverted before commit) makes
      `wall_clock_reads_may_only_decrease` fail; quote the failing assertion,
      then confirm `git status --porcelain` is clean after the revert.
- [ ] A temporarily added `Instant::now()` call in the same spot (added,
      tested, then reverted) does not move the count — confirm the ratchet
      test still passes with it present.
- [ ] No file outside this spec's footprint is modified: `git -C ../model
      status --porcelain` names only `tests/clock_read_ratchet.rs`.
- [ ] `cargo fmt --check` is silent on the new file.

## Verify and Proof

```sh
cd ../model
cargo test --test clock_read_ratchet
cargo fmt --check
git status --porcelain -- tests/clock_read_ratchet.rs
# should list only this one new file
```
