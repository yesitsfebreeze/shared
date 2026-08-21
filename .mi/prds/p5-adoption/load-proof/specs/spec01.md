# goal

An **asserted** floor on `ContentId::of` throughput in MB/s at 1 B, 1 KiB and
1 MiB, measured by a std-only harness (no `criterion`), stable enough that a
slower CI box cannot flake it and blunt enough that it still catches a real
algorithmic regression.

est: 1.25h

## What this assumes (all landed on `main`)

- `conserved::ContentId` with `ContentId::of(&[u8]) -> ContentId` and
  `as_bytes() -> &[u8; 32]` (p2, `conserved/src/content_id.rs`).
- `conserved/tests/` is the test root; `rustfmt.toml` is `hard_tabs = true`,
  `tab_spaces = 2`; `#![forbid(unsafe_code)]` on the crate.
- **`conserved::Instant` is re-exported at the crate root** (p3,
  `conserved/src/lib.rs`). This test times with `std::time::Instant`. Do not
  `use conserved::*`; import `ContentId` by name and spell the std type
  `std::time::Instant`, or the two collide.

## Files

- `conserved/tests/load_throughput.rs` — **new**. The only file this spec
  writes.

Touches nothing else: not `conserved/src/`, not `conserved/Cargo.toml`, not
the root manifest, not `learnings/`, not another ticket's folder.

## Dependency decision: std only, no `criterion`

Settled, not deferred. The ticket asks the analyst to justify a disagreement
against two specific gates; there is no disagreement, and here is why neither
gate is the reason:

- mitosys's `src/mitosys/gates/tests/dependency_tree.rs` `CLOSURE` is
  unaffected either way — a consumer of a git dependency never builds that
  dependency's dev-dependencies, so `criterion` would never enter mitosys's
  closure.
- p2's `blake3_is_the_only_dependency` reads the `[dependencies]` table only
  (spec02 already proved a dev-dependency does not trip it), so `criterion`
  under `[dev-dependencies]` would not fail it either.

Both gates would pass. The reasons to stay std-only are the ones those gates
do not measure:

1. `criterion` is ~40 transitive crates and a `cargo test` that no longer
   finishes in a second; this whole harness is 60 lines of `std::time`.
2. `criterion` **prints and compares against a saved baseline in
   `target/`**. It reports regressions; it does not fail the build on an
   absolute floor, which is the one thing this ticket exists to produce. Using
   it would mean asserting on top of it anyway.
3. A fresh clone with no `target/criterion` baseline has nothing to compare
   against and reports green — the same shape of failure as the parent's bare
   `echo`.

## How the floors were chosen

Measured on this machine (Apple M5, 10 cores, rustc 1.94.0), best-of-7 timed
samples after a warm-up calibration, `ContentId::of` through
`std::hint::black_box`:

| input  | dev (opt-level 0) | release |
|--------|------------------:|--------:|
| 1 B    | 0.78 MB/s (1.3 µs/call) | 21.3 MB/s (47 ns/call) |
| 1 KiB  | 56.7 MB/s         | 1350 MB/s |
| 1 MiB  | 117 MB/s          | 2530 MB/s |

The 20×+ gap between profiles is real and is a property of `blake3` on
aarch64: with default features it has no NEON path, so the compression
function is portable Rust that LLVM vectorises in release and does not touch
at `opt-level = 0`. On x86_64 `blake3`'s build script compiles the SSE/AVX
assembly regardless of profile, so a Linux CI box is *faster* than these dev
numbers, not slower — the dev column is the binding constraint and the floors
below are safe on both.

**Floor = measured idle median / 5, rounded down to a round number** — release
— and **/ 7** for dev, where per-core variation between machines is wider
because none of the work is vectorised:

```
dev:      1 B ->   0.10 MB/s     1 KiB ->   8.0 MB/s     1 MiB ->  15.0 MB/s
release:  1 B ->   4.00 MB/s     1 KiB -> 250.0 MB/s     1 MiB -> 500.0 MB/s
```

What that buys, measured, not guessed — the harness above was run four times
against twelve competing CPU spinners on this box, the worst margin observed
in each cell:

| input | dev, contended | release, contended |
|-------|---------------:|-------------------:|
| 1 B   | 6.9× floor | 5.3× floor |
| 1 KiB | 5.6× floor | 2.4× floor |
| 1 MiB | 4.4× floor | 3.2× floor |

So a box **half this machine's per-core speed, fully contended**, still clears
every floor. And the floors are not so low as to be theatre: routing `of`
through a hex encode/decode, hashing byte-at-a-time, or copying the input per
call are 10×+ regressions and all six cells catch them. The 1 B floor is
stated in MB/s for uniformity but is really a **250 ns/call latency ceiling**;
say so in the comment, because 4 MB/s at one byte is not a number a reader can
otherwise interpret.

## Not flaking: the four things that do it

1. **Best-of-N, not median-of-N.** Contention can only ever make a sample
   slower, never faster, so the fastest of 7 samples is the least-perturbed
   estimate of what the machine can do — which is exactly the claim a floor
   makes. Measured: switching median→best raised the worst contended 1 MiB
   observation from 1.45× the floor to 3.2×.
2. **Warm-up by calibration.** Iterations double until one sample spans
   ≥ 25 ms; that loop is itself the warm-up, and it makes each timed sample
   long enough that scheduler granularity is noise rather than signal.
3. **Per-profile floors, both asserted.** `#[cfg(debug_assertions)]` selects
   the table. Neither profile gets a pass; the ticket's own frontmatter
   `verify` runs the dev table and this spec's `verify` runs both.
4. **One `#[test]` function for all three sizes.** `cargo test` runs test
   functions in a binary in parallel; three separate timing tests would
   contend with each other on a 2-core runner. One function measures the three
   sizes in sequence, and this file has no other test in it.

## Ignore / feature decision

**Neither.** No `#[ignore]`, no feature gate, no small-N default. Measured
wall time for the whole file is **0.88 s in release and 0.80 s in dev** — the
calibration targets a fixed 25 ms per sample, so the cost is bounded by
construction rather than by input size. Hiding a one-second proof behind
`--include-ignored` or a feature would add a second way for it to silently not
run, which is the exact failure this ticket exists to correct. The ticket's
frontmatter `verify` already passes `-- --include-ignored`; that stays a
harmless no-op.

## Shape

```rust
mod load_throughput {
	use conserved::ContentId;
	use std::hint::black_box;
	use std::time::{Duration, Instant};

	#[cfg(debug_assertions)]
	const FLOORS: [(usize, f64); 3] = [(1, 0.10), (1024, 8.0), (1024 * 1024, 15.0)];
	#[cfg(not(debug_assertions))]
	const FLOORS: [(usize, f64); 3] = [(1, 4.0), (1024, 250.0), (1024 * 1024, 500.0)];
	// ...
}
```

## Acceptance

- [ ] `conserved/tests/load_throughput.rs` exists and its **entire** contents
      are wrapped in `mod load_throughput { … }`, so `cargo test -p conserved
      load` selects it. Verified by running that filter and reading a non-zero
      `N passed` — an unwrapped file reports `0 tests ... filtered out` and
      exits 0, which is the failure p1 deviation 7 records.
- [ ] Exactly **one** `#[test]` function in the file, named
      `content_id_throughput_floor`, measuring all three sizes in sequence.
- [ ] The floor table is a `const` selected by `#[cfg(debug_assertions)]`,
      carrying the two rows exactly as printed above.
- [ ] The measurement calls `ContentId::of` with the input wrapped in
      `std::hint::black_box` and the returned `ContentId` passed to
      `black_box`, so release codegen cannot hoist the call out of the loop.
      Deleting either `black_box` and re-running must move the release
      numbers; if it does not, the loop is being elided and the gate is fake.
- [ ] Calibration: iterations double from 1 until one untimed sample takes
      ≥ 25 ms (bounded at `1 << 30` iterations so a pathologically slow
      machine terminates), and that calibration loop is the warm-up.
- [ ] 7 timed samples per size; the reported figure is the **maximum**
      MB/s (equivalently the minimum elapsed), not the mean and not the
      median. A comment states why: contention only slows samples.
- [ ] Every size is measured and reported before any assertion fires — the
      test collects failures and asserts once at the end, so a run that misses
      two floors names both rather than stopping at the first.
- [ ] The failure message names the input size, the measured MB/s and the
      floor, e.g. `1048576 B: 421.30 MB/s < floor 500.00 MB/s`.
- [ ] A passing run prints one line per size with the measured value and the
      margin over the floor (`5.06x`), so a human reading CI output can see
      the headroom shrinking before it fails. The print is **in addition to**
      the assertion, never instead of it.
- [ ] **The gate can fail.** Temporarily multiply the release 1 MiB floor by
      10 and confirm `cargo test -p conserved --release --test
      load_throughput` exits non-zero with the message above; then put it
      back. Record in the spec's implementation commit that this was done.
- [ ] No new entry in `[dependencies]` or `[dev-dependencies]`;
      `conserved/Cargo.toml` is byte-identical after this spec. `cargo tree -p
      conserved --edges normal --depth 1` still lists `blake3` and nothing
      else.
- [ ] The module doc comment records: which machine and rustc the floors were
      measured on, that they are idle-median/5 (release) and /7 (dev), that
      best-of-7 is deliberate, and the aarch64-vs-x86_64 `blake3` build
      asymmetry that makes the dev column the binding one. A future reader
      lowering a floor must first contradict that paragraph.
- [ ] No `unsafe`, no `unwrap` on a path that can be reached by a slow
      machine (the `partial_cmp` sort is the only candidate — use
      `f64::total_cmp`).
- [ ] `cargo clippy -p conserved --all-targets -- -D warnings` is clean.

verify: `cargo test -p conserved --test load_throughput -- --nocapture && cargo test -p conserved --release --test load_throughput -- --nocapture && cargo test -p conserved load_throughput 2>&1 | grep -qE '[1-9][0-9]* passed' && cargo clippy -p conserved --all-targets -- -D warnings && cargo fmt --all --check`
