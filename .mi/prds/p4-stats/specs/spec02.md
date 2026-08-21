# goal

`conserved/tests/stats.rs` — the pin. The even-length test that makes the
median decision fail loudly if anyone reverses it, plus the coverage this
ticket's Acceptance names: empty, single, odd, even, percentile edges, and NaN
stated rather than accidental.

Depends on spec01 (the module). Touches nothing spec01 touches.

## Why the tests live in `mod stats { … }`

The ticket's gate is `cargo test -p conserved stats`. That argument is a **test
name filter**, not a target filter — cargo passes it to every test binary,
which matches it against each test's full path. A test called
`even_length_median_is_the_upper_median` in `tests/stats.rs` does not contain
the string `stats`, so the ticket's own gate would run it zero times and report
green. Wrapping the file's contents in `mod stats { … }` makes every test's
reported name `stats::<name>`, which the filter selects. (spec01's doctests are
already caught — rustdoc names them `src/stats.rs - stats::median (line N)`.)

## Files and dirs

- `conserved/tests/stats.rs` — **new**. Nothing else. One file, opening
  `mod stats {` at column 0, with
  `use conserved::stats::{median, min_median_max, percentile};` inside it.

Not touched: `conserved/src/`, `conserved/Cargo.toml` (no dev-dependency —
these are `assert_eq!` on `f64` values that are exact small integers and one
`is_nan()`, no float-tolerance crate needed), any other test file,
`../mitosys`, `../model`.

## The tests

Names are load-bearing — the verify line greps for them.

1. `empty_is_none` — all three functions return `None` on `&[]`, for
   `p ∈ {0.0, 0.5, 1.0}`. `None` means empty and only empty.
2. `single_element_is_its_own_min_median_max` — `[42.0]` gives
   `Some((42.0, 42.0, 42.0))` and `percentile` returns `42.0` at every `p`.
3. `odd_length_median_is_the_middle` — `[1.0, 2.0, 3.0] -> Some(2.0)`. Comment
   that this input deliberately does **not** discriminate: all three candidate
   definitions agree on odd lengths, which is why it cannot be the pin.
4. **`even_length_median_is_the_upper_median`** — the pin. On
   `[1.0, 2.0, 3.0, 4.0]`: `median` is `Some(3.0)`; and, as separate asserts
   with messages naming what lost, it is **not** `Some(2.5)` (the interpolating
   median — invents a value that is not in the sample) and **not** `Some(2.0)`
   (mitosys `percentile_sorted`'s `ceil`-rank lower median). Three distinct
   answers, one input, one assert each.
5. `median_equals_llm_aggregate_index_for_every_length` — for `n` in `1..=16`
   over `xs = (1..=n) as f64`, `median(&xs) == Some(xs[n / 2])`. `xs[n / 2]` is
   llm's expression copied verbatim from
   `../model/src/grade/measure.rs:211`; this test is the adoption proof that
   `grade::measure::aggregate` can call `min_median_max` with no change in
   recorded numbers, at every length, not just at four.
6. `percentile_at_half_is_median` — over the same `n` range,
   `percentile(&xs, 0.5) == median(&xs)`. The crate cannot carry two
   definitions internally.
7. `percentile_clamps_p_at_both_ends` — on `1.0..=10.0`: `p = 0.0` and
   `p = -1.0` and `f64::NEG_INFINITY` all give `Some(1.0)`; `p = 1.0`, `2.0`
   and `f64::INFINITY` all give `Some(10.0)`.
8. `percentile_is_monotone_in_p` — on `1.0..=10.0`, walking `p` from `0.0` to
   `1.0` in 101 steps, the result never decreases.
9. `percentile_returns_an_element_of_the_input` — over those same 101 values of
   `p`, every result is a member of the slice. This is what "never
   interpolates" means as a property rather than a single vector.
10. `mitosys_percentile_vectors_that_survive` — mitosys's own unit test
    (`../mitosys/src/mitosys/util/tests/unit/util.rs:26`) on `xs = 1.0..=10.0`,
    split into what this ticket keeps and what it deliberately overturns:
    `p = 0.0 -> 1.0`, `p = 1.0 -> 10.0`, `p = 0.95 -> 10.0` are unchanged;
    `p = 0.5` moves from mitosys's `5.0` to `6.0`, asserted as `6.0` with a
    message saying it is the deliberate consequence of the upper-median
    decision. p5-adoption reads this test to know exactly which mitosys
    assertion it must rewrite.
11. `nan_is_propagated_positionally` — `median(&[f64::NAN])` is `Some(x)` with
    `x.is_nan()`, and `min_median_max(&[f64::NAN])` is `Some((a, b, c))` with
    all three NaN. A single-element slice is vacuously sorted, so this pins the
    release-build behaviour — NaN comes back verbatim, is never sanitised, and
    is never collapsed into `None` — without tripping the debug assertion.
12. `unsorted_input_trips_the_debug_assertion` — `#[cfg(debug_assertions)]`,
    `#[should_panic]`: `median(&[2.0, 1.0])`.
13. `nan_in_a_multi_element_slice_trips_the_debug_assertion` —
    `#[cfg(debug_assertions)]`, `#[should_panic]`:
    `median(&[1.0, f64::NAN, 2.0])`. A slice of length ≥ 2 holding a NaN is not
    ascending-sorted under any total order, and `a <= b` is false whenever
    either side is NaN, so the sortedness assertion is the NaN check.
14. `nan_p_trips_the_debug_assertion` — `#[cfg(debug_assertions)]`,
    `#[should_panic]`: `percentile(&[1.0, 2.0], f64::NAN)`. NaN `p` is a
    contract violation, not a silent `0.0`.

## Acceptance

- [ ] `conserved/tests/stats.rs` exists, opens `mod stats {` at column 0, and
      every `#[test]` in it is inside that module — so
      `cargo test -p conserved stats` reports each as `stats::<name>` and runs
      it, rather than filtering all of them out.
- [ ] All fourteen tests above exist under those exact names and pass under the
      ticket's own gate command, `cargo test -p conserved stats`.
- [ ] The pin actually pins. Mutate `median` in `conserved/src/stats.rs` to the
      lower median (`percentile(sorted, 0.5)` becomes
      `sorted.get((sorted.len().saturating_sub(1)) / 2).copied()`) and confirm
      `cargo test -p conserved stats` **fails** in
      `even_length_median_is_the_upper_median`,
      `median_equals_llm_aggregate_index_for_every_length` and
      `mitosys_percentile_vectors_that_survive`. Revert. This is a check that
      the check works, not a change to keep — the working tree must be back to
      spec01's module afterwards.
- [ ] The three `#[should_panic]` tests are gated `#[cfg(debug_assertions)]`,
      so a `--release` test run compiles and passes rather than failing on
      assertions that were compiled out.
- [ ] No dev-dependency was added: `conserved/Cargo.toml` has no
      `[dev-dependencies]` entry introduced by this spec, and `stats.rs`
      imports only `conserved::stats`.
- [ ] `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --
      -D warnings` are clean (hard tabs, width 2).
- [ ] `cargo test --workspace` is green — the rest of the crate still passes.

## est

0.75

verify: `bash -c 'set -e; cd /Users/feb/dev/infra/shared; test -f conserved/tests/stats.rs; grep -qxF "mod stats {" conserved/tests/stats.rs; grep -E "^[[:space:]]*use " conserved/tests/stats.rs | grep -qvE "use (conserved|core|std)::" && { echo "unexpected import in tests/stats.rs"; exit 1; } || true; out=$(cargo test -p conserved stats 2>&1) || { echo "$out"; echo "the ticket gate fails"; exit 1; }; for t in empty_is_none single_element_is_its_own_min_median_max odd_length_median_is_the_middle even_length_median_is_the_upper_median median_equals_llm_aggregate_index_for_every_length percentile_at_half_is_median percentile_clamps_p_at_both_ends percentile_is_monotone_in_p percentile_returns_an_element_of_the_input mitosys_percentile_vectors_that_survive nan_is_propagated_positionally unsorted_input_trips_the_debug_assertion nan_in_a_multi_element_slice_trips_the_debug_assertion nan_p_trips_the_debug_assertion; do printf "%s" "$out" | grep -qF "test stats::$t ... ok" || { echo "missing or failing under the ticket gate: stats::$t"; exit 1; }; done; printf "%s" "$out" | grep -qE "test result: ok[.] 1[4-9] passed|test result: ok[.] [2-9][0-9]+ passed" || { echo "the stats target ran fewer than 14 tests under the gate filter"; exit 1; }; grep -c "cfg(debug_assertions)" conserved/tests/stats.rs | grep -qE "^[3-9]$" || { echo "the should_panic tests are not gated on debug_assertions"; exit 1; }; cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace >/dev/null; echo "spec02 ok"'`
