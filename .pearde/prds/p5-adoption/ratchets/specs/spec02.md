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

- [x] `cargo test -p realm-gates` passes, including the four existing gates
      unchanged plus the three new tests above.

      **Closed 2026-08-27 by the orchestrator.** The two reds this box was
      left open for are gone. `dependency_tree` was closed by the
      `p5-adoption/realm` implementer's `OWNERS` entry; `done_boxes_are_ticked`
      went green when `realm-classify` and `02-linux-driver` were corrected
      from `done` to `blocked` — the gate source is byte-identical to `HEAD`.
      Six targets, 11 passed, 0 failed.

      **Not ticked, and not this spec's to fix.** The new file's own target is
      green — `cargo test -p realm-gates --test clock_read_ratchet`:

      ```
      running 3 tests
      test monotonic_reads_are_never_counted ... ok
      test a_walk_that_scans_nothing_fails_rather_than_passes ... ok
      test wall_clock_reads_may_only_decrease ... ok

      test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```

      `board_is_tracked` (1/1), `source_layout` (3/3) and `one_vocabulary`
      (1/1) pass. Two targets fail, both on causes that predate this run and
      both outside this spec's footprint.

      `dependency_tree`:

      ```
      ---- every_third_party_dependency_has_the_owners_recorded stdout ----
      the dependency ownership table moved (1 change(s)):
        conserved is new to the tree, declared by {"realm-cli", "realm-linux-driver", "realm-net"}
      ```

      Reproduced. `git diff -- Cargo.toml src/cli/Cargo.toml
      src/drivers/linux/Cargo.toml src/net/Cargo.toml` shows the uncommitted
      `conserved = { git = ..., rev = 9a342e1 }` adoption that caused it, in
      three manifests this spec may not touch. The fix is one entry in
      `src/gates/tests/dependency_tree.rs`'s `OWNERS`, owed by whoever lands
      the `conserved` adoption. Not written here.

      `done_boxes_are_ticked`:

      ```
      ---- every_done_prd_has_no_unticked_box stdout ----
      1 PRD(s) carry state: done with an unticked acceptance box, outside the exemption list:
        prds/done-means-done/realm-classify/prd.md
      ```

      Reproduced. Nothing in this run wrote to `realm/prds/`. Not fixed here.

- [x] `RATCHET_CEILING` equals a freshly re-measured wall-clock count on
      realm's tree at landing time (quote the measurement command and
      output), not the PRD's 3 or this spec's 3 — re-derive, do not copy.

      **Refuted: the count is 0, not 3.** Measured by the walker itself with
      `RATCHET_CEILING` at 0 — it stayed green, so there is nothing to name.
      Cross-checked against the tree:

      ```
      $ rg -n --pcre2 'SystemTime::now\b' src -g '!**/target/**' \
          -g '!**/tests/**' -g '!**/tests.rs' -g '!**/*_tests.rs'
      (no output)

      $ rg -n --pcre2 'SystemTime::now\b' src -g '!**/target/**'
      src/cli/tests/cli.rs:161:              created_at: std::time::SystemTime::now()
      src/cli/tests/unit/state.rs:103:       let now = std::time::SystemTime::now()
      src/cli/tests/unit/lib.rs:653:         let reference = std::time::SystemTime::now()
      src/drivers/linux/tests/unit/state.rs:91: let reference = std::time::SystemTime::now()
      ```

      All four live under `tests/`, so all four are out of scan. The board's
      3 is not stale — it is correct at `HEAD` — but the working tree this
      gate was measured against has moved those three reads from
      `src/cli/src/state.rs`, `src/cli/src/lib.rs` and
      `src/drivers/linux/src/state.rs` into `tests/unit/` files, uncommitted,
      by the concurrent `src/cli` + `src/drivers` work. `git log -S` and
      `git diff -U0 -- src | rg SystemTime::now` both show it.
      `RATCHET_CEILING = 0` records the tree, and the constant's doc comment
      names all three files so a revert of that work fails here loudly rather
      than quietly.

      No `now_nanos`/`now_ms` wrapper has appeared since analysis:
      `rg -n 'fn now_nanos|fn now_ms' src` returns nothing (2026-08-26), so
      `WALL_CLOCK_READS` holds `SystemTime::now` alone.

- [x] A temporarily added `SystemTime::now()` call anywhere under
      `src/*/src/` (added, tested, then reverted before commit) makes
      `wall_clock_reads_may_only_decrease` fail; quote the failing assertion,
      then confirm `git status --porcelain` is clean after the revert.

      Probe added at `src/gates/ratchet_probe.rs` — a new unreferenced file
      rather than an edit to an existing one, because `src/cli`, `src/net` and
      `src/drivers` are being edited by a concurrent implementer and a
      byte-exact revert of a file they may touch is not something this run can
      guarantee. `gates/Cargo.toml` sets `[lib] path = "lib.rs"`, so the probe
      compiles into nothing and is visible only to the textual walk:

      ```rust
      pub fn probe() -> std::time::SystemTime {
          std::time::SystemTime::now()
      }
      ```

      ```
      test wall_clock_reads_may_only_decrease ... FAILED
      realm now holds 1 wall-clock read(s) across 28 implementation file(s),
      over the ceiling of 0:
        src/gates/ratchet_probe.rs: 1
      test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
      ```

      After `rm src/gates/ratchet_probe.rs`: `3 passed; 0 failed`, and
      `git status --porcelain -- src/gates/` lists only
      `?? src/gates/tests/clock_read_ratchet.rs`.

- [x] A temporarily added `Instant::now()` call in the same spot (added,
      tested, then reverted) does not move the count — confirm the ratchet
      test still passes with it present.

      Same file, `std::time::Instant::now()` in place of the wall-clock read:

      ```
      test wall_clock_reads_may_only_decrease ... ok
      test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```

- [x] The four pre-existing gates (`source_layout`, `one_vocabulary`,
      `dependency_tree`, `board_is_tracked`) are unmodified.

      `git status --porcelain -- src/gates/` reports one line,
      `?? src/gates/tests/clock_read_ratchet.rs` — nothing else under
      `src/gates/` is touched, `lib.rs` included.

## Corrected 2026-08-26 — a ceiling of 0 forces the comparison to change

`RATCHET_CEILING = 0` is the true record of the tree, and `total <= 0` on an
unsigned `total` compares against the minimum of its type, which
`clippy::absurd_extreme_comparisons` refuses under `-D warnings`:

```
absurd_extreme_comparisons on `total <= RATCHET_CEILING`
```

Fixed in how the comparison is spelled, never by inflating the ceiling:
`total.saturating_sub(RATCHET_CEILING) == 0` says the same thing at every
ceiling, keeps working at 0, and names the more honest quantity — how many
reads over. Applied to all three trees' gates, not only realm's: mitosys is at
48 and model at 10 today, and both ratchets exist to be driven to 0.

```
$ cargo clippy --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.68s
```

Whole-workspace, exit 0, no warnings.

## Verify and Proof

```sh
cd ../realm
cargo test -p realm-gates --test clock_read_ratchet
cargo clippy --all-targets -- -D warnings
git status --porcelain -- src/gates/tests/clock_read_ratchet.rs
# should list only this one new file
```
