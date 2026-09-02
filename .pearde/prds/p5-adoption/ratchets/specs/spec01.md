---
complexity: 30
footprint:
  - ../mitosys/src/mitosys/gates/tests/write_path_reads_no_clock.rs
---
<!-- footprint paths are relative to shared/ (this repo), pointing into the
     sibling repo the work actually lands in — ratchets spans three repos, so
     unlike a single-repo PRD's footprint there is no one implicit prefix. -->

# spec01 — mitosys: widen the write-path clock gate into a whole-tree wall-clock ratchet

`src/mitosys/gates/tests/write_path_reads_no_clock.rs` already bans every
clock spelling on a hand-kept `WATCHED` list of write-path files. This spec
**extends that same file** (per the PRD's "extend, do not reinvent") with a
second, independent check: a whole-`src/` wall-clock-read count that may only
go down, counted the way `shared/learnings/clock.md` counts it — and it must
NOT touch the existing `WATCHED`/`no_watched_file_reads_the_machine_clock`
zero-tolerance check, which stays exactly as strict as it is today.

## What changes

Add to `write_path_reads_no_clock.rs` (do not remove or weaken anything already
there):

- `const WALL_CLOCK_READS: &[&str] = &["SystemTime::now", "now_nanos", "now_ms"];`
  — the existing `CLOCK_READS` constant stays for the zero-tolerance check;
  this is a **separate** list because `CLOCK_READS` currently includes
  `"Instant::now"`, which is monotonic and must never be counted by the new
  ratchet (PRD requirement 2).
- A recursive walker over `src/` (all of `src/mitosys/`, `src/surfaces/`,
  `src/plugins/` — the PRD's measured table (40 wall-clock reads,
  2026-08-21) was **not** scoped to `src/mitosys/` alone: re-measuring
  `rg -n --pcre2 'SystemTime::now\b' src -g '!**/target/**'` against
  the whole `src/` tree today (2026-08-25) found **41**, close to the PRD's
  40; scoping to `src/mitosys/` alone would undercount and silently admit
  reads in `surfaces/`/`plugins/`). Exclusions, matching
  `model/prds/clock-gate/prd.md`'s already-written matcher (same shared
  law, reuse its wording rather than re-deriving one): comment lines,
  any path containing a `tests/` directory component, files named
  `tests.rs` or `*_tests.rs`, and text inside `#[cfg(test)]` blocks.
- `fn whole_tree_wall_clock_reads() -> usize` — walks and counts, matching
  `WALL_CLOCK_READS` substrings under those exclusions.
- `const RATCHET_CEILING: usize = <N>;` — **re-measure at landing time**,
  do not copy 40 from the PRD table or 41 from this spec: both are already
  stale by the time this lands. Comment the measurement command and date
  beside the constant, same style as the file's existing comments.
- `#[test] fn wall_clock_reads_may_only_decrease()` — asserts
  `whole_tree_wall_clock_reads() <= RATCHET_CEILING`, with a failure message
  naming the offending file(s) (line-level detail is a nice-to-have, not
  required — the existing zero-tolerance test already gives line-level detail
  for the write path specifically).
- `#[test] fn monotonic_reads_are_never_counted()` — a matcher self-check:
  asserts `"Instant::now"` is not a member of `WALL_CLOCK_READS` (guards
  against someone re-merging the two lists later, which is exactly the
  regression requirement 2 exists to prevent).
- A vacuity test in the same shape as
  `a_watch_list_that_scans_nothing_fails_rather_than_passes`: the whole-tree
  walk must find at least as many files scanned as some floor well under the
  landing count (pick a floor and justify it in the same style as the
  existing "not a round number and not 1" comment).

## Acceptance

- [x] `cargo test -p mitosys-gates --test write_path_reads_no_clock` passes,
      including all four existing tests unchanged plus the new ones.

      ```
      running 7 tests
      test a_watch_list_that_scans_nothing_fails_rather_than_passes ... ok
      test monotonic_reads_are_never_counted ... ok
      test every_watched_file_still_exists ... ok
      test the_serde_skip_exemption_is_still_load_bearing ... ok
      test no_watched_file_reads_the_machine_clock ... ok
      test wall_clock_reads_may_only_decrease ... ok
      test a_walk_that_scans_nothing_fails_rather_than_passes ... ok

      test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```

- [x] `RATCHET_CEILING` equals a freshly re-measured whole-`src/` wall-clock
      count on the tree at landing time (quote the measurement command and
      its output in the report), not the PRD's 40 or this spec's 41.

      Measured by the walker itself, 2026-08-26, with `RATCHET_CEILING` set to
      0 so the failure prints the live number:

      ```
      $ cargo test -p mitosys-gates --test write_path_reads_no_clock
      the tree now holds 48 wall-clock read(s) across 203 production source
      file(s), over the ceiling of 0:
        src/mitosys/api/agentic/pool/pool.rs: 1
        src/mitosys/engine/base/base_types.rs: 2
        src/mitosys/engine/channel/channel.rs: 2
        src/mitosys/engine/commands/commands_admin.rs: 1
        src/mitosys/engine/commands/commands_export.rs: 1
        src/mitosys/engine/commands/commands_graph_ops.rs: 1
        src/mitosys/engine/commands/commands_intake_cmd.rs: 4
        src/mitosys/engine/commands/commands_query.rs: 3
        src/mitosys/engine/commands/lib.rs: 2
        src/mitosys/engine/graph/graph.rs: 3
        src/mitosys/engine/graph/persist.rs: 1
        src/mitosys/engine/hub/hub_registry.rs: 2
        src/mitosys/engine/identity/lib.rs: 3
        src/mitosys/engine/ingest/ingest_intake.rs: 2
        src/mitosys/engine/ingest/ingest_worker.rs: 1
        src/mitosys/engine/ingest_config/ingest_config.rs: 1
        src/mitosys/engine/record/store.rs: 1
        src/mitosys/engine/rpc/server.rs: 6
        src/mitosys/engine/tick_loop/tick.rs: 3
        src/mitosys/engine/transport/memory_rpc.rs: 1
        src/mitosys/util/util.rs: 6
        src/mitosys/util/watcher.rs: 1
      ```

      `RATCHET_CEILING = 48`. The bare `rg` cross-check is a different
      question and gives a different number — `rg -n --pcre2
      'SystemTime::now\b' src -g '!**/target/**' -g '!**/tests/**' -g
      '!**/tests.rs' -g '!**/*_tests.rs' | wc -l` reports `41`, because it
      neither counts the `now_nanos`/`now_ms` wrappers nor drops comment lines
      and `#[cfg(test)]` blocks.

- [x] A temporarily added `SystemTime::now()` call in any non-test file under
      `src/` (added, tested, then reverted before commit) makes
      `wall_clock_reads_may_only_decrease` fail; quote the failing assertion
      output, then confirm `git status --porcelain` is clean after the
      revert.

      Probe added at `src/mitosys/util/ratchet_probe.rs` (a new unreferenced
      file rather than an edit to an existing one — two other implementers are
      live in this tree and a byte-exact revert of a file they may touch is
      not something this run can guarantee):

      ```rust
      pub fn probe() -> std::time::SystemTime {
          std::time::SystemTime::now()
      }
      ```

      ```
      test wall_clock_reads_may_only_decrease ... FAILED
      the tree now holds 49 wall-clock read(s) across 204 production source
      file(s), over the ceiling of 48:
        src/mitosys/util/ratchet_probe.rs: 1
      test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
      ```

      After `rm src/mitosys/util/ratchet_probe.rs`: `7 passed; 0 failed`, and
      `git status --porcelain -- src/mitosys/gates/tests/write_path_reads_no_clock.rs
      src/mitosys/util/` lists only
      ` M src/mitosys/gates/tests/write_path_reads_no_clock.rs`.

- [x] A temporarily added `Instant::now()` call in the same spot (added,
      tested, then reverted) does **not** move `whole_tree_wall_clock_reads()`
      — confirm the ratchet test still passes with it present, proving
      `Instant::now` is excluded from the count.

      Same file, `std::time::Instant::now()` in place of the wall-clock read:

      ```
      test wall_clock_reads_may_only_decrease ... ok
      test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```

- [x] The pre-existing `no_watched_file_reads_the_machine_clock`,
      `every_watched_file_still_exists`,
      `a_watch_list_that_scans_nothing_fails_rather_than_passes` and
      `the_serde_skip_exemption_is_still_load_bearing` tests are byte-for-byte
      unmodified in behavior (still pass, still against the same `WATCHED`
      list) — this spec is additive only.

      `git diff` on the file is an append plus one `rustfmt` reflow inside the
      appended block. `WATCHED`, `CLOCK_READS`, `EXEMPT_ON_LINE`, `root`,
      `is_comment` and all four tests are untouched, and all four pass in every
      run quoted above.

## Corrected 2026-08-26 — the ceiling comparison, shared with spec02 and spec03

`total <= RATCHET_CEILING` breaks the day a ceiling reaches 0: `total` is
unsigned, so the comparison is against the minimum of its type and
`clippy::absurd_extreme_comparisons` refuses it under `-D warnings`. realm's
sibling gate is at 0 today and hit exactly that. Both assertions here are
spelled `saturating_sub(..) == 0` instead — behaviour identical at 48, still
correct at 0. `cargo clippy -p mitosys-gates --all-targets -- -D warnings` is
green; the seven tests above are unchanged and still pass.

## Verify and Proof

```sh
cd ../mitosys
cargo test -p mitosys-gates --test write_path_reads_no_clock
git status --porcelain -- src/mitosys/gates/tests/write_path_reads_no_clock.rs
# should list only this one file
```
