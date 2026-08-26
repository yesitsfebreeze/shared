---
complexity: 8
footprint:
  - ../model/Cargo.toml
  - ../model/src/grade/measure.rs
---

# spec03 — `aggregate` calls `conserved::min_median_max`

`grade::measure::aggregate` (`../model/src/grade/measure.rs:203`) sorts
`durations: &[u64]` in place with `sort_unstable`, then hand-indexes
`sorted[0]`, `sorted[len()/2]`, `sorted[len()-1]` — min / upper-median / max.
`conserved::min_median_max(sorted: &[f64]) -> Option<(f64, f64, f64)>` (p4)
is the one definition of this the family shares. Adoption is sort (as
today, on `u64`) -> cast to `f64` -> call -> cast the three results back to
`u64` for `Stats { min_ms, median_ms, max_ms, .. }`. The one call site
(inside `aggregate` itself, feeding the `Stats` built a few lines below) is
the only caller in the tree.

## Acceptance

- [x] `aggregate` calls `conserved::min_median_max` on the sorted `f64`
      view of `durations`, in place of the hand-rolled `len()/2` index.
- [x] The `u64` -> `f64` -> `u64` round trip is proven exact for
      millisecond-scale durations by a test — the PRD's own words are
      "must be recorded, not assumed." `f64` has 52 bits of mantissa,
      exact up to 2^53; a test asserts the round trip is lossless across a
      representative range of ms values (including 0 and a large value near
      what a real grading run produces), not just that it compiles.
- [x] Existing callers of `aggregate` see byte-identical `(min_ms, median_ms,
      max_ms)` on the same input before and after the swap — this changes
      the implementation, not the observable numbers. A test with a fixed
      even-length input (the length where upper-median and the
      interpolating definitions disagree, per p4's own acceptance) pins the
      before/after equality.
- [x] `model/Cargo.toml` has the `conserved` dependency (idempotent with
      spec01/spec02 if either has already landed it).

## Verify and Proof

```sh
cd ../model && cargo build -p llm && cargo test -p llm --lib grade::measure::
```

## Evidence — implemented 2026-08-26

**Box 1.** `aggregate` sorts on `u64` as before, casts to a `Vec<f64>`, calls
`conserved::stats::min_median_max`, and casts the three results back. The
`len()/2` index is gone from the function.

`unwrap_or((0.0, 0.0, 0.0))` rather than `expect`: the empty case returns
`(0, 0, 0)` above the call, so `None` is unreachable — an `expect` there would
be a panic message for a branch that cannot be taken, and the fallback matches
what the empty case already answers.

**Box 2 — proven, not assumed.**
`the_u64_f64_round_trip_is_exact_for_millisecond_durations` runs 12
representative values from `0` through `86_400_000` (a full day in ms) and
`u32::MAX`, up to `2^53 - 1`, asserting `(ms as f64) as u64 == ms` for each.

The test also asserts `(2^53 + 1) as f64 == (2^53) as f64` — the point where
`f64` stops distinguishing neighbours. Without that line the first half would
pass on any float width and would not be measuring the bound it cites; with it,
a change to the reasoning makes the test fail rather than quietly stay green.

**Box 3.** `the_shared_envelope_agrees_with_the_hand_rolled_one` keeps a copy
of `aggregate`'s pre-swap body and compares the two on six inputs. The lead
case is EVEN-length (`[40, 10, 30, 20]`) — the length p4's acceptance names as
where the three median definitions disagree — and it asserts the literal
`(10, 30, 40)`: the **upper** median 30, not the interpolating 25 and not the
lower 20. So the test pins the definition by value, not only by agreement with
a copy of the old code.

`conserved::stats::median`'s own doc states it is `percentile(sorted, 0.5)` ==
`sorted[n / 2]`, naming `llm`'s `grade::measure::aggregate` as the expression
it matches — which is why this adoption moves no number.

**Box 4.** The `conserved` dependency was landed by spec01. Nothing added
twice; `git diff Cargo.toml` shows one `conserved` entry in
`[workspace.dependencies]` and one in `[dependencies]`.

## Verify — run 2026-08-26

```
$ cargo build -p llm && cargo test -p llm --lib grade::measure::
running 4 tests
test grade::measure::tests::the_u64_f64_round_trip_is_exact_for_millisecond_durations ... ok
test grade::measure::tests::ops_failure_rate_graded_against_threshold ... ok
test grade::measure::tests::ops_line_parses ... ok
test grade::measure::tests::the_shared_envelope_agrees_with_the_hand_rolled_one ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1569 filtered out; finished in 0.00s
```
