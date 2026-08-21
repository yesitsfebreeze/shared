mod load_throughput {
	//! An **asserted** floor on `ContentId::of` throughput at 1 B, 1 KiB and
	//! 1 MiB. Not a table that gets printed and ignored: every cell is a
	//! comparison, and missing one fails the test.
	//!
	//! # Where the floors come from
	//!
	//! Measured on an Apple M5 (10 cores), `rustc 1.94.0 (4a4ef493e 2026-03-02)`,
	//! `blake3` at the workspace pin, best-of-7 timed samples after a warm-up
	//! calibration, `ContentId::of` routed through `std::hint::black_box`:
	//!
	//! | input | dev (opt-level 0) | release |
	//! |-------|------------------:|--------:|
	//! | 1 B   | 0.78 MB/s (1.3 us/call) | 21.3 MB/s (47 ns/call) |
	//! | 1 KiB | 56.7 MB/s | 1350 MB/s |
	//! | 1 MiB | 117 MB/s | 2530 MB/s |
	//!
	//! The floors below are the **idle median divided by 5** for release and
	//! **divided by 7** for dev, rounded down to a round number. Dev gets the
	//! wider margin because none of its work is vectorised, so per-core
	//! variation between machines is wider.
	//!
	//! # Why the two profiles are 20x apart, and why dev is the binding column
	//!
	//! This is a property of `blake3` on aarch64: with default features it has
	//! no NEON path, so the compression function is portable Rust that LLVM
	//! vectorises in release and does not touch at `opt-level = 0`. On x86_64
	//! `blake3`'s build script compiles the SSE/AVX assembly regardless of
	//! profile, so a Linux CI box is *faster* than the dev numbers above, not
	//! slower. The dev column is therefore the binding constraint, and floors
	//! safe here are safe there.
	//!
	//! # Why best-of-7 and not median-of-7
	//!
	//! Contention can only ever make a sample slower, never faster. The fastest
	//! of the samples is therefore the least-perturbed estimate of what the
	//! machine can do, which is exactly the claim a floor makes. Measured:
	//! switching median -> best raised the worst contended 1 MiB observation
	//! from 1.45x the floor to 3.2x. Run against twelve competing CPU spinners,
	//! the worst margin observed in any cell was 2.4x (release, 1 KiB) and 4.4x
	//! (dev, 1 MiB) — so a box at half this machine's per-core speed, fully
	//! contended, still clears every floor.
	//!
	//! A future reader lowering a floor has to contradict the paragraphs above
	//! first. The floors are not theatre either: hex-encoding through `of`,
	//! hashing byte-at-a-time, or copying the input per call are all 10x+
	//! regressions, and all six cells catch them.
	//!
	//! No `criterion`: it compares against a saved baseline under `target/` and
	//! *reports* a regression rather than failing an absolute floor, so a fresh
	//! clone with no baseline reports green — the same shape of non-proof this
	//! whole ticket exists to replace.

	use conserved::ContentId;
	use std::hint::black_box;
	use std::time::{Duration, Instant};

	/// `(input length in bytes, floor in MB/s)`.
	///
	/// The 1 B floor is stated in MB/s for uniformity with the other two, but
	/// what it really is is a **latency ceiling**: 4 MB/s at one byte per call
	/// is 250 ns/call, and 0.10 MB/s is 10 us/call. Read it that way — 4 MB/s
	/// is not otherwise an interpretable number for a one-byte input.
	#[cfg(debug_assertions)]
	const FLOORS: [(usize, f64); 3] = [(1, 0.10), (1024, 8.0), (1024 * 1024, 15.0)];
	#[cfg(not(debug_assertions))]
	const FLOORS: [(usize, f64); 3] = [(1, 4.0), (1024, 250.0), (1024 * 1024, 500.0)];

	/// Timed samples per size. Best of these is the reported figure.
	const SAMPLES: usize = 7;
	/// A sample must span at least this long, so that scheduler granularity is
	/// noise rather than signal.
	const MIN_SAMPLE: Duration = Duration::from_millis(25);
	/// Calibration bound, so a pathologically slow machine still terminates.
	const MAX_ITERS: u64 = 1 << 30;

	/// Hashes `input` `iters` times and returns the elapsed wall time.
	///
	/// `black_box` on the input stops the optimiser from constant-folding the
	/// digest; `black_box` on the result stops it from deciding the call is
	/// dead and hoisting it out of the loop. Delete either one and the release
	/// numbers must move; if they do not, the loop is being elided and this
	/// whole file is measuring nothing.
	fn time_hashing(input: &[u8], iters: u64) -> Duration {
		let start = Instant::now();
		for _ in 0..iters {
			black_box(ContentId::of(black_box(input)));
		}
		start.elapsed()
	}

	/// Doubles the iteration count until one sample spans `MIN_SAMPLE`.
	///
	/// This loop *is* the warm-up: by the time it returns, the code path has
	/// been executed at least as many times as the timed samples will execute
	/// it, so caches and branch predictors are in the same state.
	fn calibrate(input: &[u8]) -> u64 {
		let mut iters: u64 = 1;
		loop {
			if time_hashing(input, iters) >= MIN_SAMPLE || iters >= MAX_ITERS {
				return iters;
			}
			iters *= 2;
		}
	}

	/// One test function for all three sizes, deliberately: `cargo test` runs
	/// the test functions in a binary in parallel, so three separate timing
	/// tests would contend with each other on a two-core runner and depress
	/// exactly the numbers being asserted. This file contains no other test.
	#[test]
	fn content_id_throughput_floor() {
		// Every size is measured and reported before any assertion fires, so a
		// run that misses two floors names both instead of stopping at the
		// first.
		let mut failures: Vec<String> = Vec::new();

		for (size, floor) in FLOORS {
			let input = vec![0xa5u8; size];
			let iters = calibrate(&input);

			let mut mbps: Vec<f64> = Vec::with_capacity(SAMPLES);
			for _ in 0..SAMPLES {
				let elapsed = time_hashing(&input, iters);
				let bytes = size as f64 * iters as f64;
				mbps.push(bytes / elapsed.as_secs_f64() / 1e6);
			}
			// `total_cmp`, not `partial_cmp().unwrap()`: a total order has no
			// panic path on a slow machine that produced something odd.
			mbps.sort_by(f64::total_cmp);
			// The maximum, not the mean and not the median — see the module doc.
			let best = mbps[SAMPLES - 1];

			println!(
				"{size:>9} B: {best:>10.2} MB/s  (floor {floor:.2} MB/s, {:.2}x, {iters} iters/sample)",
				best / floor
			);

			if best < floor {
				failures.push(format!("{size} B: {best:.2} MB/s < floor {floor:.2} MB/s"));
			}
		}

		assert!(
			failures.is_empty(),
			"ContentId::of missed its throughput floor:\n  {}",
			failures.join("\n  ")
		);
	}
}
