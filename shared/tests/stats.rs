//! Order statistics — the pin on the median decision.
//!
//! Everything lives inside `mod stats { … }` on purpose. The ticket's gate is
//! `cargo test -p shared stats`, and that argument is a **test name**
//! filter, not a target filter: cargo hands it to every test binary, which
//! matches it against each test's full path. A test called
//! `even_length_median_is_the_upper_median` does not contain the string
//! `stats`, so without this module the gate would run zero tests and report
//! green. Inside it, every test reports as `stats::<name>` and the filter
//! selects it.

mod stats {
	use shared::stats::{median, min_median_max, percentile};

	/// `None` means empty input and only empty input.
	#[test]
	fn empty_is_none() {
		let empty: &[f64] = &[];
		for p in [0.0, 0.5, 1.0] {
			assert_eq!(
				percentile(empty, p),
				None,
				"percentile(&[], {p}) must be None"
			);
		}
		assert_eq!(median(empty), None);
		assert_eq!(min_median_max(empty), None);
	}

	#[test]
	fn single_element_is_its_own_min_median_max() {
		let xs = [42.0];
		assert_eq!(min_median_max(&xs), Some((42.0, 42.0, 42.0)));
		assert_eq!(median(&xs), Some(42.0));
		for p in [0.0, 0.25, 0.5, 0.75, 1.0] {
			assert_eq!(percentile(&xs, p), Some(42.0), "percentile(&[42.0], {p})");
		}
	}

	/// Deliberately **not** the pin: on odd lengths all three candidate
	/// definitions — upper median, lower median and interpolating — agree, so
	/// this input cannot discriminate between them. Test 4 is the pin.
	#[test]
	fn odd_length_median_is_the_middle() {
		assert_eq!(median(&[1.0, 2.0, 3.0]), Some(2.0));
	}

	/// **The pin.** One input, three candidate answers, one assert each.
	#[test]
	fn even_length_median_is_the_upper_median() {
		let xs = [1.0, 2.0, 3.0, 4.0];
		assert_eq!(
			median(&xs),
			Some(3.0),
			"the upper median wins: `sorted[n / 2]`, llm's live expression at five call sites"
		);
		assert_ne!(
			median(&xs),
			Some(2.5),
			"2.5 is the interpolating median, rejected: it invents a value that is not in the sample"
		);
		assert_ne!(
			median(&xs),
			Some(2.0),
			"2.0 is mitosys `percentile_sorted`'s ceil-rank lower median, rejected: zero callers"
		);
	}

	/// The adoption proof. `xs[n / 2]` is llm's `grade::measure::aggregate`
	/// expression copied verbatim: adopting this crate moves no recorded
	/// number, at every length and not merely at four.
	#[test]
	fn median_equals_llm_aggregate_index_for_every_length() {
		for n in 1..=16usize {
			let xs: Vec<f64> = (1..=n).map(|i| i as f64).collect();
			assert_eq!(
				median(&xs),
				Some(xs[n / 2]),
				"median must equal llm's sorted[n / 2] at n = {n}"
			);
		}
	}

	/// The crate cannot carry two definitions internally.
	#[test]
	fn percentile_at_half_is_median() {
		for n in 1..=16usize {
			let xs: Vec<f64> = (1..=n).map(|i| i as f64).collect();
			assert_eq!(percentile(&xs, 0.5), median(&xs), "at n = {n}");
		}
	}

	#[test]
	fn percentile_clamps_p_at_both_ends() {
		let xs: Vec<f64> = (1..=10).map(|i| i as f64).collect();
		for p in [0.0, -1.0, f64::NEG_INFINITY] {
			assert_eq!(percentile(&xs, p), Some(1.0), "low end at p = {p}");
		}
		for p in [1.0, 2.0, f64::INFINITY] {
			assert_eq!(percentile(&xs, p), Some(10.0), "high end at p = {p}");
		}
	}

	#[test]
	fn percentile_is_monotone_in_p() {
		let xs: Vec<f64> = (1..=10).map(|i| i as f64).collect();
		let mut previous = f64::NEG_INFINITY;
		for step in 0..=100 {
			let p = f64::from(step) / 100.0;
			let got = percentile(&xs, p).expect("non-empty slice");
			assert!(
				got >= previous,
				"percentile dropped at p = {p}: {got} < {previous}"
			);
			previous = got;
		}
	}

	/// "Never interpolates", as a property rather than a single vector.
	#[test]
	fn percentile_returns_an_element_of_the_input() {
		let xs: Vec<f64> = (1..=10).map(|i| i as f64).collect();
		for step in 0..=100 {
			let p = f64::from(step) / 100.0;
			let got = percentile(&xs, p).expect("non-empty slice");
			assert!(
				xs.contains(&got),
				"percentile(p = {p}) returned {got}, not an element of the input"
			);
		}
	}

	/// mitosys's own unit-test vectors, split into what survives and what this
	/// ticket deliberately overturns. p5-adoption reads this to know exactly
	/// which mitosys assertion it must rewrite.
	#[test]
	fn mitosys_percentile_vectors_that_survive() {
		let xs: Vec<f64> = (1..=10).map(|i| i as f64).collect();
		assert_eq!(percentile(&xs, 0.0), Some(1.0), "unchanged from mitosys");
		assert_eq!(percentile(&xs, 1.0), Some(10.0), "unchanged from mitosys");
		assert_eq!(percentile(&xs, 0.95), Some(10.0), "unchanged from mitosys");
		assert_eq!(
			percentile(&xs, 0.5),
			Some(6.0),
			"overturned deliberately: mitosys's percentile_sorted asserts 5.0 here; the upper-median decision moves it to 6.0"
		);
	}

	/// A single-element slice is vacuously sorted, so this pins the
	/// release-build behaviour without tripping the debug assertion: NaN comes
	/// back verbatim, never sanitised and never collapsed into `None`.
	#[test]
	fn nan_is_propagated_positionally() {
		let got = median(&[f64::NAN]).expect("NaN is not empty; None means empty only");
		assert!(got.is_nan());
		let (a, b, c) = min_median_max(&[f64::NAN]).expect("NaN is not empty; None means empty only");
		assert!(a.is_nan() && b.is_nan() && c.is_nan());
	}

	#[cfg(debug_assertions)]
	#[test]
	#[should_panic(expected = "sorted")]
	fn unsorted_input_trips_the_debug_assertion() {
		let _ = median(&[2.0, 1.0]);
	}

	/// `a <= b` is false whenever either side is NaN, so the sortedness
	/// assertion is the NaN check for any slice of length >= 2.
	#[cfg(debug_assertions)]
	#[test]
	#[should_panic(expected = "sorted")]
	fn nan_in_a_multi_element_slice_trips_the_debug_assertion() {
		let _ = median(&[1.0, f64::NAN, 2.0]);
	}

	/// A NaN `p` is a contract violation, not a silent `0.0`.
	#[cfg(debug_assertions)]
	#[test]
	#[should_panic(expected = "NaN")]
	fn nan_p_trips_the_debug_assertion() {
		let _ = percentile(&[1.0, 2.0], f64::NAN);
	}
}
