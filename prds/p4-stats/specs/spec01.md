# goal

`conserved::stats` — `percentile` / `median` / `min_median_max` over an
already-sorted `&[f64]`, with **one** definition of median written into the
doc comment and both rejected alternatives named there. Zero dependencies,
zero allocation, no hidden sort.

## The decision this spec encodes

The two live definitions in the family disagree on even-length input:

| where | expression | `[1.0, 2.0, 3.0, 4.0]` |
|---|---|---|
| llm `grade::measure::aggregate` (`../model/src/grade/measure.rs:203`) | `sorted[n / 2]` — **upper median** | `3.0` |
| mitosys `percentile_sorted` (`../mitosys/src/mitosys/util/util.rs:104`) | nearest rank, `ceil(p·n)` — **lower median** at `p = 0.5` | `2.0` |
| (a third answer nobody implements) | interpolating — mean of the two central elements | `2.5` |

**The upper median wins.** The evidence, read at both call sites:

1. **llm's is load-bearing; mitosys's is dead.** `percentile_sorted` has zero
   callers — its only user is its own unit test
   (`../mitosys/src/mitosys/util/tests/unit/util.rs:26`). llm's upper median is
   called from five sites in `measure.rs` (lines 122, 176, 191, 670, 685), is
   documented twice in public prose (`Stats::median_ms`, `run_window`), and is
   deliberately pinned by a test named for it —
   `aggregate_even_count_upper_median`, `aggregate(&[100, 200, 300, 400]) ==
   (100, 300, 400)` (`../model/src/grade/tests/measure.rs:300`).
2. **A second llm site already agrees.** `LatencyStats::from`
   (`../model/src/chat/state.rs:639`) computes `latencies[latencies.len() / 2]`
   — the same upper median, independently, for the chat session envelope. The
   family's count is two live upper medians against one dead lower median.
3. **It is persisted.** `median_ms` becomes `Row::best_ms`, the stored best
   `grade_row` compares each run against (`../model/src/grade/report.rs:272`).
   Changing the definition would silently move every recorded best by one
   element of the sample — a regression in the grade envelope, which is the
   exact failure `learnings/shared-crate.md` §4 says this extraction exists to
   prevent.
4. **The interpolating median is not in the family at all.** Choosing it would
   invent a third definition, return a value that is not a measurement that
   ever happened, and force a rounding decision at llm's call site — which
   aggregates `u64` milliseconds and has no such decision today. Note the p4
   PRD's phrase "the interpolating one" describes mitosys's function
   inaccurately: `percentile_sorted` selects, it never averages. There is no
   interpolating implementation to preserve.

The percentile convention is chosen to *make* that median, so the crate cannot
carry two definitions internally:

```text
index = min(floor(p · n), n - 1)      p clamped to [0, 1], n = sorted.len()
```

The sorted slice is `n` equal buckets covering `[0, 1)`; `percentile(p)`
returns the element whose bucket contains `p`, with `p = 1` pinned to the last
element. It is monotone in `p`, total, and always returns an element that is
actually in the input. At `p = 0.5` it is `floor(n/2)` = `n / 2` in integer
arithmetic — **literally llm's expression, for every `n`** — so
`percentile(s, 0.5)`, `median(s)` and llm's `sorted[n / 2]` are one thing.
`median` is therefore *defined as* `percentile(sorted, 0.5)`, not as a parallel
computation that could drift.

Deliberately overturned: mitosys's assertion `percentile_sorted(&xs, 0.5) ==
Some(5.0)` on `1.0..=10.0` becomes `Some(6.0)`. Its `p = 0.0`, `p = 1.0` and
`p = 0.95` vectors on that same slice are unchanged. That is one dead-code
assertion against a live grade envelope; p5-adoption inherits the rewrite, and
spec02 pins the surviving vectors so p5 is not surprised by which ones moved.

## NaN — stated, not accidental

These functions **never compare elements**. `percentile` indexes; `min` is
`sorted[0]`; `max` is `sorted[n - 1]`. So:

- A NaN in the input is returned verbatim if and only if it sits at the
  selected index. It is never sanitised, never swallowed into `None`, and can
  never make a *non*-NaN element come out wrong.
- A slice of length ≥ 2 containing a NaN is not ascending-sorted under any
  total order, so it violates the caller's contract — and the sortedness
  `debug_assert` catches it for free, because `a <= b` is false whenever either
  side is NaN. Debug builds panic; release builds stay total and deterministic.
- `Option` means **empty input and nothing else**. Returning `None` for a NaN
  would collapse "this data is broken" into llm's existing all-zeros
  "no run succeeded" signal (`Stats::default()`), turning a data bug into a
  fabricated measurement.
- A NaN `p` is handled by an explicit branch (`0.0`, the first element), not by
  the accident that `(f64::NAN * n) as usize` saturates to `0`, and
  `debug_assert!(!p.is_nan(), …)` fires in debug.

Justified against the call sites: llm aggregates `u64` milliseconds cast to
`f64`, which cannot be NaN, so the policy costs it nothing; mitosys sorts
floats with `cmp_partial`, which explicitly treats incomparable pairs as
`Equal` (`util.rs:88`), so a NaN-bearing "sorted" slice is reachable there —
and the debug assertion is what turns that into a loud failure at the site that
produced it instead of a quiet wrong percentile.

## Files and dirs

Assumes p0 has landed: `conserved/` with edition 2021, `rust-version =
"1.94.0"`, an empty `[dependencies]`, tests at `conserved/tests/`.

- `conserved/src/stats.rs` — **new**. The whole module.
- `conserved/src/lib.rs` — gains the single line `pub mod stats;`. Nothing else
  on that file. (`pub mod`, matching p1's `pub mod scope;` — the names
  `percentile` and `median` are too generic to re-export at the crate root;
  `conserved::stats::median` says which median it is.)

Not touched: `conserved/Cargo.toml` (no dependency, not even a dev-dependency —
p2's `blake3_is_reachable_only_through_content_id` test asserts the
`[dependencies]` table names `blake3` and nothing else), the root `Cargo.toml`,
`learnings/shared-crate.md` (a learning is never edited to erase what it
recorded; §4's signature block is already what this spec implements), anything
under `../mitosys` or `../model`.

## Shape

```rust
#[must_use]
pub fn percentile(sorted: &[f64], p: f64) -> Option<f64>
#[must_use]
pub fn median(sorted: &[f64]) -> Option<f64>              // = percentile(sorted, 0.5)
#[must_use]
pub fn min_median_max(sorted: &[f64]) -> Option<(f64, f64, f64)>
```

- One private helper carries the sortedness assertion:
  `debug_assert!(sorted.windows(2).all(|w| w[0] <= w[1]), …)`. `windows` does
  not allocate; the O(n) walk exists only in debug builds.
- `min_median_max` delegates: `Some((*sorted.first()?, median(sorted)?,
  *sorted.last()?))`. It does not repeat the assertion (`median` asserts) and
  does not re-derive the median.
- Repo style: hard tabs, width 2 (`rustfmt.toml`). `#![forbid(unsafe_code)]` is
  already at the crate root.

## Acceptance

- [x] `conserved/src/stats.rs` exists and `conserved/src/lib.rs` contains the
      line `pub mod stats;`; `cargo build --workspace` passes.
- [x] The three public signatures are exactly as written above — verifiable by
      literal match, not by "something like it".
- [x] `median` is *defined as* `percentile(sorted, 0.5)`: the token
      `percentile(sorted, 0.5)` appears in `median`'s body, and `median` does
      no indexing of its own. The two cannot drift apart.
- [x] The module or `median` doc comment states the decision and names **both**
      rejected alternatives: it contains the phrases `upper median`,
      `interpolating`, `percentile_sorted`, and `rejected`.
- [x] The doc comment carries a runnable example on the discriminating input:
      `median(&[1.0, 2.0, 3.0, 4.0]) == Some(3.0)`, with the rejected answers
      `2.5` (interpolating) and `2.0` (mitosys's lower median) named in prose
      beside it. At least one doctest under the `stats` filter passes.
- [x] The docs state that sortedness is the **caller's** contract and that the
      functions never sort; `stats.rs` contains no `sort_unstable`, no
      `.sort(`, no `to_vec(`, no `vec!`, no `collect(` outside comments.
- [x] The only assertions in the module are `debug_assert!` — no `assert!`
      survives into release builds.
- [x] NaN is documented in the module doc: propagated positionally, never
      turned into `None`, caught in debug by the sortedness assertion; and
      `percentile` has an explicit `p.is_nan()` branch rather than relying on
      the saturating cast.
- [x] This spec adds no dependency and no dev-dependency: every name in
      `conserved/Cargo.toml`'s `[dependencies]` table belongs to another
      ticket (`blake3` is p2's, `serde` is p2's optional feature) and nothing
      new appears; `stats.rs` has no `use` of anything outside `core`/`std`.
- [x] `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --
      -D warnings` are clean.

## Notes for the implementer

- `(p * sorted.len() as f64) as usize` truncates toward zero, which is `floor`
  for a non-negative finite `p` — that is the intended `floor`. Keep the
  `.min(len - 1)` so `p = 1.0` lands on the last element rather than one past
  it.
- `f64::clamp` returns NaN when `self` is NaN, so the `is_nan` branch must come
  first, not after the clamp.
- Do not make these generic over `T: Copy` to match mitosys's shape. The PRD
  pins `&[f64]`, and `min_median_max` returning a tuple of the element type is
  where a generic version starts needing bounds. p5-adoption converts mitosys's
  one `u128` call — which lives in that function's own unit test — at the call
  site.

## est

1.0

verify: `bash -c 'set -e; cd /Users/feb/dev/infra/shared; test -f conserved/src/stats.rs; grep -qxF "pub mod stats;" conserved/src/lib.rs; for s in "upper median" "interpolating" "percentile_sorted" "rejected"; do grep -qiF "$s" conserved/src/stats.rs || { echo "doc comment does not name: $s"; exit 1; }; done; for s in "pub fn percentile(sorted: &[f64], p: f64) -> Option<f64>" "pub fn median(sorted: &[f64]) -> Option<f64>" "pub fn min_median_max(sorted: &[f64]) -> Option<(f64, f64, f64)>" "percentile(sorted, 0.5)" "is_nan()" "debug_assert!"; do grep -qF "$s" conserved/src/stats.rs || { echo "missing from stats.rs: $s"; exit 1; }; done; body=$(grep -v "^[[:space:]]*//" conserved/src/stats.rs); if printf "%s" "$body" | grep -qE "sort_unstable|[.]sort[(]|to_vec[(]|vec!|collect[(]"; then echo "stats.rs sorts or allocates"; exit 1; fi; if printf "%s" "$body" | grep "assert" | grep -qv "debug_assert"; then echo "a non-debug assertion survives into release"; exit 1; fi; deps=$(awk "/^\[dependencies\]/{f=1;next} /^\[/{f=0} f" conserved/Cargo.toml | grep -oE "^[a-zA-Z0-9_-]+" || true); for d in $deps; do case "$d" in blake3|serde) ;; *) echo "p4 must add no dependency; found $d"; exit 1;; esac; done; if grep -E "^[[:space:]]*use " conserved/src/stats.rs | grep -qvE "use (core|std)::"; then echo "stats.rs imports outside core/std"; exit 1; fi; cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test -p conserved --doc stats 2>&1 | grep -qE "test result: ok[.] [1-9][0-9]* passed"; echo "spec01 ok"'`
