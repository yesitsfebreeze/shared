//! Order statistics over an already-sorted `&[f64]` — **one** definition of
//! median, for the whole family.
//!
//! Three functions, no dependencies, no allocation, no sort:
//! [`percentile`], [`median`] and [`min_median_max`]. Each returns `None` on
//! empty input and `Some` on everything else — `Option` here means *empty
//! input and nothing else*.
//!
//! # The percentile convention
//!
//! ```text
//! index = min(floor(p * n), n - 1)      p clamped to [0, 1], n = sorted.len()
//! ```
//!
//! The sorted slice is `n` equal buckets covering `[0, 1)`; `percentile(p)`
//! returns the element whose bucket contains `p`, with `p = 1` pinned to the
//! last element. It is monotone in `p`, total, and always returns an element
//! that is actually in the input — it never invents a value.
//!
//! # The median decision, and the two rejected alternatives
//!
//! At `p = 0.5` the convention above is `floor(n / 2)`, which is `n / 2` in
//! integer arithmetic — so [`median`] is **the upper median**, and it is
//! *defined as* `percentile(sorted, 0.5)` rather than as a parallel
//! computation that could drift away from it.
//!
//! Two other definitions were considered and **rejected**:
//!
//! - **The interpolating median** — the mean of the two central elements,
//!   `2.5` on `[1.0, 2.0, 3.0, 4.0]`. Rejected: nothing in the family
//!   implements it, it returns a value that is not a measurement that ever
//!   happened, and it forces a rounding decision at the one live call site,
//!   which aggregates `u64` milliseconds and has no such decision today.
//! - **The lower median** — mitosys's `percentile_sorted`, a nearest-rank
//!   `ceil(p * n)` selection, `2.0` on `[1.0, 2.0, 3.0, 4.0]`. Rejected:
//!   `percentile_sorted` has zero callers outside its own unit test, while the
//!   upper median is called from five sites in llm's
//!   `grade::measure::aggregate`, is computed a second time and independently
//!   by `LatencyStats::from`, is documented in public prose, is pinned by a
//!   test named for it, and is *persisted* — it becomes the stored best that
//!   every later run is compared against. Moving it would silently shift every
//!   recorded best by one element of the sample.
//!
//! So `percentile(s, 0.5)`, `median(s)` and llm's hand-written `sorted[n / 2]`
//! are one thing, at every length. Adopting this module changes no recorded
//! number. mitosys's own assertion `percentile_sorted(&xs, 0.5) == Some(5.0)`
//! on `1.0..=10.0` becomes `Some(6.0)`; its `p = 0.0`, `p = 1.0` and
//! `p = 0.95` vectors on that slice are unchanged.
//!
//! # Sortedness is the caller's contract
//!
//! These functions **never sort**. Passing an unsorted slice is a bug at the
//! call site, and a hidden sort would hide it (and would allocate). Ascending
//! order is asserted with `debug_assert!` only: debug builds panic, release
//! builds stay total, deterministic and O(1).
//!
//! # NaN
//!
//! These functions **never compare elements**: `percentile` indexes, the
//! minimum is `sorted[0]` and the maximum is `sorted[n - 1]`. Therefore:
//!
//! - A NaN in the input is **propagated positionally** — returned verbatim if
//!   and only if it sits at the selected index. It is never sanitised, and it
//!   can never make a *non*-NaN element come out wrong.
//! - A NaN is **never collapsed into `None`**. `None` would mean "empty", and
//!   at the consuming call site an empty sample already means "no run
//!   succeeded"; folding a broken measurement into that signal would turn a
//!   data bug into a fabricated measurement.
//! - A slice of length >= 2 holding a NaN is not ascending-sorted under any
//!   total order, so it violates the caller's contract — and the sortedness
//!   `debug_assert` catches it for free, because `a <= b` is false whenever
//!   either side is NaN. The sortedness assertion *is* the NaN check.
//! - A NaN `p` is a contract violation too, caught by its own
//!   `debug_assert` in [`percentile`] and handled in release by an explicit
//!   branch — not by the accident that `(f64::NAN * n as f64) as usize`
//!   saturates to `0`.

/// The caller's sortedness contract, checked in debug builds only.
///
/// `windows` does not allocate; the O(n) walk exists only where
/// `debug_assertions` is on. `a <= b` is false when either side is NaN, so
/// this is also the NaN check for any slice of length >= 2.
fn debug_assert_sorted(sorted: &[f64]) {
	debug_assert!(
		sorted.windows(2).all(|w| w[0] <= w[1]),
		"shared::stats: the slice must be sorted ascending and NaN-free; sortedness is the caller's contract, never a hidden sort"
	);
}

/// The element at percentile `p` of an already-sorted slice, or `None` if the
/// slice is empty.
///
/// `index = min(floor(p * n), n - 1)`, with `p` clamped to `[0, 1]`. The
/// result is always an element of the input — this never interpolates.
///
/// # Contract
///
/// `sorted` must be ascending and NaN-free, and `p` must not be NaN. Both are
/// `debug_assert!`ed and neither is repaired: this function does not sort.
///
/// ```
/// use shared::stats::percentile;
///
/// let xs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
/// assert_eq!(percentile(&xs, 0.0), Some(1.0));
/// assert_eq!(percentile(&xs, 0.5), Some(6.0));
/// assert_eq!(percentile(&xs, 0.95), Some(10.0));
/// assert_eq!(percentile(&xs, 1.0), Some(10.0));
/// assert_eq!(percentile(&[], 0.5), None);
/// ```
#[must_use]
pub fn percentile(sorted: &[f64], p: f64) -> Option<f64> {
	debug_assert!(
		!p.is_nan(),
		"shared::stats::percentile: `p` must not be NaN; a NaN `p` is a bug at the call site"
	);
	debug_assert_sorted(sorted);
	if sorted.is_empty() {
		return None;
	}
	if p.is_nan() {
		// Explicit, not the saturating cast: in release a NaN `p` selects the
		// first element, deterministically.
		return sorted.first().copied();
	}
	let n = sorted.len();
	// `as usize` truncates toward zero, which is `floor` for a non-negative
	// finite `p`. `.min(n - 1)` keeps `p = 1.0` on the last element.
	let index = ((p.clamp(0.0, 1.0) * n as f64) as usize).min(n - 1);
	sorted.get(index).copied()
}

/// The median of an already-sorted slice, or `None` if the slice is empty.
///
/// This is **the upper median**, defined as `percentile(sorted, 0.5)` — which
/// is `sorted[n / 2]`, llm's `grade::measure::aggregate` expression, at every
/// length. The two other definitions are named and **rejected** in the module
/// documentation: the *interpolating* median (`2.5` below — invents a value
/// that is not in the sample) and mitosys's `percentile_sorted` lower median
/// (`2.0` below — nearest rank, zero callers).
///
/// # Contract
///
/// `sorted` must be ascending and NaN-free; this is a `debug_assert!`, never
/// a hidden sort.
///
/// ```
/// use shared::stats::median;
///
/// // The discriminating input: the three definitions disagree here.
/// assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), Some(3.0)); // upper median: chosen
/// assert_ne!(median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5)); // interpolating: rejected
/// assert_ne!(median(&[1.0, 2.0, 3.0, 4.0]), Some(2.0)); // lower median: rejected
///
/// // Odd lengths do not discriminate — all three definitions agree.
/// assert_eq!(median(&[1.0, 2.0, 3.0]), Some(2.0));
/// assert_eq!(median(&[]), None);
/// ```
#[must_use]
pub fn median(sorted: &[f64]) -> Option<f64> {
	percentile(sorted, 0.5)
}

/// The minimum, [`median`] and maximum of an already-sorted slice, or `None`
/// if the slice is empty.
///
/// The whole grade envelope in one call: the minimum is the first element and
/// the maximum is the last, so nothing is compared and nothing is scanned.
/// The median is [`median`]'s, never re-derived here.
///
/// ```
/// use shared::stats::min_median_max;
///
/// assert_eq!(min_median_max(&[100.0, 200.0, 300.0, 400.0]), Some((100.0, 300.0, 400.0)));
/// assert_eq!(min_median_max(&[42.0]), Some((42.0, 42.0, 42.0)));
/// assert_eq!(min_median_max(&[]), None);
/// ```
#[must_use]
pub fn min_median_max(sorted: &[f64]) -> Option<(f64, f64, f64)> {
	Some((*sorted.first()?, median(sorted)?, *sorted.last()?))
}
