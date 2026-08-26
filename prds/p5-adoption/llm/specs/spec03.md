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

- [ ] `aggregate` calls `conserved::min_median_max` on the sorted `f64`
      view of `durations`, in place of the hand-rolled `len()/2` index.
- [ ] The `u64` -> `f64` -> `u64` round trip is proven exact for
      millisecond-scale durations by a test — the PRD's own words are
      "must be recorded, not assumed." `f64` has 52 bits of mantissa,
      exact up to 2^53; a test asserts the round trip is lossless across a
      representative range of ms values (including 0 and a large value near
      what a real grading run produces), not just that it compiles.
- [ ] Existing callers of `aggregate` see byte-identical `(min_ms, median_ms,
      max_ms)` on the same input before and after the swap — this changes
      the implementation, not the observable numbers. A test with a fixed
      even-length input (the length where upper-median and the
      interpolating definitions disagree, per p4's own acceptance) pins the
      before/after equality.
- [ ] `model/Cargo.toml` has the `conserved` dependency (idempotent with
      spec01/spec02 if either has already landed it).

## Verify and Proof

```sh
cd ../model && cargo build -p llm && cargo test -p llm --lib grade::measure::
```
