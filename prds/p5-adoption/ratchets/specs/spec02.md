---
complexity: 25
footprint:
  - ../realm/src/gates/tests/clock_read_ratchet.rs
---
<!-- footprint paths are relative to shared/ (this repo), pointing into the
     sibling repo the work actually lands in. -->

# spec02 — realm: a wall-clock-read ratchet in the existing `realm-gates` crate

realm's gates host already exists and is done (`../realm/src/gates`,
`15-gates`, four family gates: `source_layout`, `one_vocabulary`,
`dependency_tree`, `board_is_tracked` — the PRD's own "realm needs a gate
host before it can host a ratchet" is satisfied; `15-gates`'s `needs:` on
this PRD is already met). `15-gates` deliberately ports **no other gate** —
the clock ratchet is new work, a fifth test file in the same crate, not a
change to any of the four.

## What exists when done

A new file `src/gates/tests/clock_read_ratchet.rs` in the `realm-gates`
crate, same shape as `../mitosys`'s
`gates/tests/write_path_reads_no_clock.rs` (cite it as the shape, do not
copy realm-specific paths from it):

- `const WALL_CLOCK_READS: &[&str] = &["SystemTime::now"];` — realm has no
  `now_nanos`/`now_ms`-style wrapped helper (`rg 'fn now_nanos|fn now_ms'
  ../realm/src` returns nothing at analysis time; if the implementer finds
  one has been added since, include it).
- A walker over realm's five `src/<crate>/src/` trees (realm nests one level
  deeper than mitosys — `source_layout.rs` in this same crate already
  descends both levels; reuse that walk rather than re-deriving one),
  excluding comment lines, any path with a `tests/` component, files named
  `tests.rs`/`*_tests.rs`, and `#[cfg(test)]` blocks (realm's own
  `source_layout` gate already tracks 9 inline `mod tests {}` blocks as a
  named exemption list on 2026-08-23 — reuse that same list or its
  successor as the `#[cfg(test)]`-block boundary marker rather than
  re-deriving one).
- `const RATCHET_CEILING: usize = <N>;` — **re-measure at landing**, do not
  copy 3 from the PRD's table: `rg -n --pcre2 'SystemTime::now\b' src
  -g '!**/target/**'` on realm's tree, filtered the same way, found **3**
  wall-clock reads on 2026-08-25 (matches the PRD's 2026-08-21 count — stable
  so far, but re-measure anyway; do not assume it still holds by landing
  time).
- `#[test] fn wall_clock_reads_may_only_decrease()`.
- `#[test] fn monotonic_reads_are_never_counted()` — `Instant::now` is
  excluded from `WALL_CLOCK_READS` by construction; realm has 10 non-test
  `Instant::now()` call sites today (measured 2026-08-25; the PRD's table
  says 18 — a real drift, unexplained, not this spec's to chase down), none
  of which may ever move this ratchet.
- A vacuity test: the walk must touch at least as many files as some floor
  well under the crate count, in the style of `realm-gates`'s existing
  `source_layout.rs` vacuity check (reuse its threshold-setting rationale).

## Acceptance

- [ ] `cargo test -p realm-gates` passes, including the four existing gates
      unchanged plus the three new tests above.
- [ ] `RATCHET_CEILING` equals a freshly re-measured wall-clock count on
      realm's tree at landing time (quote the measurement command and
      output), not the PRD's 3 or this spec's 3 — re-derive, do not copy.
- [ ] A temporarily added `SystemTime::now()` call anywhere under
      `src/*/src/` (added, tested, then reverted before commit) makes
      `wall_clock_reads_may_only_decrease` fail; quote the failing assertion,
      then confirm `git status --porcelain` is clean after the revert.
- [ ] A temporarily added `Instant::now()` call in the same spot (added,
      tested, then reverted) does not move the count — confirm the ratchet
      test still passes with it present.
- [ ] The four pre-existing gates (`source_layout`, `one_vocabulary`,
      `dependency_tree`, `board_is_tracked`) are unmodified.

## Verify and Proof

```sh
cd ../realm
cargo test -p realm-gates
git status --porcelain -- src/gates/tests/clock_read_ratchet.rs
# should list only this one new file
```
