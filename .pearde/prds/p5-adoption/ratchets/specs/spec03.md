---
complexity: 20
footprint:
  - ../model/gates/tests/clock_read_ratchet.rs
  - ../model/gates/tests/source_layout.rs
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

## Corrected 2026-08-26 — the premise above is stale; the file lands in `gates/`

The section above was written against a tree where `gates/` did not exist. It
does now. Measured 2026-08-26:

| claim in the section above | measured | verdict |
|---|---|---|
| `adopt-gates` is `state: open` | landed — `gates/` is a workspace member (`[workspace] members = ["gates", ...]`) holding `board_is_tracked.rs`, `source_layout.rs`, `one_vocabulary.rs`, `dependency_tree.rs` | refuted |
| the root `tests/` is a free landing spot | frozen — `gates/tests/source_layout.rs::crate_root_tests_dir_is_frozen` allows the repo root exactly `peer_churn.rs` | refuted |

A root-level `tests/clock_read_ratchet.rs` therefore reddens model's own
`source_layout` gate:

```
the crate-root tests/ growth cap was violated (1 time(s)):
  the repo root's tests/clock_read_ratchet.rs is not on the frozen allow-list
  — AGENTS.md §Testing rules: no new file lands in a crate-root tests/ folder
    until the crate split
```

**The footprint moves to `../model/gates/tests/clock_read_ratchet.rs`**, beside
the family gates and beside where `model/prds/clock-gate` plans
`clock_reads_are_declared.rs`. The reconciliation of the two remains model's
own board's call, as the section above says.

`gates/tests/source_layout.rs` joins the footprint for one entry and its doc
comment. `crate_root_tests_dir_is_frozen` freezes *every* member's crate-root
`tests/`, `gates` included, and the two arms are different rules: `"."` is a
moratorium (`AGENTS.md` §Testing rules — nothing is added to it, and this spec
honoured that by moving out of the repo root), `"gates"` is a register whose own
doc comment asks to be told — *"a fifth gate test file is a new, deliberate
decision, not a silent pass."* The entry is that decision, carrying the PRD that
made it. The doc comment above the function was rewritten to state the rule
rather than a count, so the next gate does not falsify it.

That the register still refuses an unnamed file was proved rather than assumed —
see the second probe under the first acceptance box.

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

- [x] `cargo test --test clock_read_ratchet` (from `../model`) passes — read
      as `cargo test -p gates --test clock_read_ratchet` after the footprint
      moved into `gates/`. `cargo test -p gates` as a whole passes too.

      ```
      Running tests/clock_read_ratchet.rs
      test monotonic_reads_are_never_counted ... ok
      test a_walk_that_scans_nothing_fails_rather_than_passes ... ok
      test wall_clock_reads_may_only_decrease ... ok
      test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

      Running tests/source_layout.rs
      test crate_root_tests_dir_is_frozen ... ok
      test no_beside_the_module_tests_dir_outside_the_frozen_list ... ok
      test every_unit_test_file_is_declared_by_its_crate ... ok
      test no_file_carries_the_test_prefix ... ok
      test no_implementation_file_holds_a_test ... ok
      test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```

      `board_is_tracked` 1/1, `dependency_tree` 2/2, `one_vocabulary` 1/1 — the
      whole crate green. `cargo clippy -p gates --all-targets -- -D warnings`
      green. No `Cargo.toml` change was needed: `gates/tests/` auto-discovers
      the target beside the family gates.

      **The register still refuses an unnamed gate file** — the entry named one
      file, it did not widen the rule. An empty sixth file dropped into
      `gates/tests/` and then removed:

      ```
      $ printf '#[test]\nfn probe() {}\n' > gates/tests/register_probe.rs
      the crate-root tests/ growth cap was violated (1 time(s)):
        gates's tests/register_probe.rs is not on the frozen allow-list
      test result: FAILED. 4 passed; 1 failed

      $ rm gates/tests/register_probe.rs
      test result: ok. 5 passed; 0 failed
      ```

- [x] `RATCHET_CEILING` equals a freshly re-measured wall-clock count at
      landing time (quote the measurement command and output), not the 15
      measured by this spec or the PRD's table — re-derive, do not copy.

      **Refuted: the count is 10, not 15.** Measured by the walker with
      `RATCHET_CEILING` at 0 so the failure prints the live number:

      ```
      $ cargo test --test clock_read_ratchet
      model now holds 10 direct wall-clock read(s) across 130 implementation
      file(s), over the ceiling of 0:
        src/chat/mod.rs: 1
        src/daemon/leases.rs: 1
        src/daemon/mod.rs: 1
        src/gossip/routing.rs: 1
        src/grade/inproc.rs: 1
        src/loop/checkpoint.rs: 1
        src/mcp/fold.rs: 1
        src/node/live.rs: 1
        src/record/event.rs: 1
        src/version/ledger.rs: 1
      ```

      `RATCHET_CEILING = 10`. The `rg` cross-check answers a different question
      and gives 14 — `rg -n --pcre2 'SystemTime::now\b' src -g '!**/target/**'
      -g '!**/tests/**' -g '!**/tests.rs' -g '!**/*_tests.rs'`. All four extra
      lines were opened and read by hand: one is a comment
      (`src/record/mod.rs:247`), three are inside `#[cfg(test)]` items —
      `src/node/hot_swap.rs:497` (under `#[cfg(test)]` at 488),
      `src/improve/llm.rs:708` (under 691), `src/node/transactional.rs:151`
      (`tmp_registry_path`, under 149).

      No `now_nanos`/`now_ms` wrapper has appeared since analysis:
      `rg -n 'fn now_nanos|fn now_ms' src` returns nothing (2026-08-26), so
      `WALL_CLOCK_READS` holds `SystemTime::now` alone.

- [x] A temporarily added `SystemTime::now()` call anywhere under `src/`
      (added, tested, then reverted before commit) makes
      `wall_clock_reads_may_only_decrease` fail; quote the failing assertion,
      then confirm `git status --porcelain` is clean after the revert.

      Probe added at `src/ratchet_probe.rs` — a new file declared by no `mod`,
      so it compiles into nothing and is visible only to the textual walk. A new
      file rather than an edit to an existing one because this tree is dirty
      from other work and a byte-exact revert of a file someone else is editing
      is not something this run can guarantee:

      ```rust
      pub fn probe() -> std::time::SystemTime {
          std::time::SystemTime::now()
      }
      ```

      ```
      model now holds 11 direct wall-clock read(s) across 131 implementation
      file(s), over the ceiling of 10:
        src/ratchet_probe.rs: 1
      test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
      ```

      Re-run from the `gates/` home after the register entry landed, with the
      same result — the ratchet is unaffected by where its file sits.

      After `rm src/ratchet_probe.rs`: `3 passed; 0 failed`, and
      `git status --porcelain -- tests/ gates/ src/ratchet_probe.rs` lists only
      `?? gates/tests/clock_read_ratchet.rs`.

- [x] A temporarily added `Instant::now()` call in the same spot (added,
      tested, then reverted) does not move the count — confirm the ratchet
      test still passes with it present.

      Same file, `std::time::Instant::now()` in place of the wall-clock read:

      ```
      test wall_clock_reads_may_only_decrease ... ok
      test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```

- [x] No file outside this spec's footprint is modified, scoped to the paths
      this spec owns: `git -C ../model status --porcelain -- tests/ gates/
      src/ratchet_probe.rs` names only `gates/tests/clock_read_ratchet.rs` and
      the one authorised register entry in `gates/tests/source_layout.rs`.

      **Rewritten by the orchestrator, 2026-08-27, with the reasoning in the
      open so a later reader can overrule it.** As written this box asks for a
      clean `git status` across the whole tree. It can never pass here, and it
      was wrong on the day it was written rather than only today:
      `@references/parts/commits.md` says the inherited tree is not the
      board's — step 1 records what is dirty before the round and those paths
      are never the round's to clean. A box asking the board to satisfy a
      condition the board's own commit rule disclaims is a defective box, and
      `@references/parts/workers.md` says the orchestrator catches that class
      at the `specced` transition. This one was missed there. Scoping it to
      the paths this spec actually owns is the check the box was reaching for.

      Re-run by the orchestrator 2026-08-27:

      ```
      $ git -C model status --porcelain -- tests/ gates/ src/ratchet_probe.rs
       M gates/tests/source_layout.rs
      ?? gates/tests/clock_read_ratchet.rs
      ```

      Both probe files are gone and the repo root's `tests/` is untouched.

      **The original reading, kept:** this tree was already dirty before the
      run.** `git status --porcelain` names 29 other paths —
      `src/version/ledger.rs`, `src/config/mod.rs`, `prds/**`, `grade.json`,
      `.pi/kern/data/**` and the rest — none of them touched here and none of
      them reverted. Scoped to what this run wrote,
      `git status --porcelain -- tests/ gates/ src/ratchet_probe.rs` names
      exactly `?? gates/tests/clock_read_ratchet.rs`, plus
      ` M gates/tests/source_layout.rs` for the one register entry. Both probe
      files are gone and the repo root's `tests/` is untouched.

- [x] `cargo fmt --check` is silent on the new file.

      `cargo fmt --check` from `../model` is silent across the whole workspace
      and exits 0.

## Verify and Proof

```sh
cd ../model
cargo test -p gates
cargo clippy -p gates --all-targets -- -D warnings
cargo fmt --check
git status --porcelain -- gates/tests/
# the new gate, plus source_layout.rs for its register entry
```
