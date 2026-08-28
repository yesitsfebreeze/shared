//! `Clock`, `SystemClock`, `FixedClock` — one trait, one reader of the
//! operating system, and the deterministic fold that is the reason the ticket
//! exists.
//!
//! Wrapped in `mod clock` and every test named with a `clock_` prefix: the
//! ticket's gate `cargo test -p shared clock` filters on test *names*, not
//! file names, so an unprefixed test here would be silently filtered out and
//! the gate would pass having run nothing.

mod clock {
	use shared::{Clock, FixedClock, Instant, SystemClock};
	use std::sync::Arc;
	use std::time::{Duration, SystemTime, UNIX_EPOCH};

	/// 2026-08-21T00:00:00Z, the same fixed point the `Instant` tests use.
	const T2026: i64 = 1_787_270_400;

	fn crate_root() -> std::path::PathBuf {
		std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
	}

	fn rust_sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
		for entry in std::fs::read_dir(dir).expect("src/ is readable") {
			let path = entry.expect("a readable directory entry").path();
			if path.is_dir() {
				rust_sources(&path, out);
			} else if path.extension().is_some_and(|e| e == "rs") {
				out.push(path);
			}
		}
	}

	/// A source file with its line comments stripped, so that prose *about* a
	/// clock read is not counted as one.
	fn code_only(text: &str) -> String {
		text
			.lines()
			.map(|line| match line.find("//") {
				Some(i) => &line[..i],
				None => line,
			})
			.collect::<Vec<_>>()
			.join("\n")
	}

	/// `SystemClock` reads the wall clock, not a counter.
	///
	/// The condemned `conserved-core` scaffold
	/// (`.mi/docs/memos/scaffold-reset.md`) shipped an `Instant` that was a
	/// tick counter; such a clock returns 0 or 1 on its first call and fails
	/// here immediately.
	#[test]
	fn clock_system_clock_returns_a_real_timestamp_not_a_counter() {
		let now = SystemClock.now();
		// 2026-01-01T00:00:00Z .. 2100-01-01T00:00:00Z.
		assert!(
			now.as_unix_secs() > 1_767_225_600,
			"{now:?} is before 2026 — a counter, not a clock"
		);
		assert!(
			now.as_unix_secs() < 4_102_444_800,
			"{now:?} is after 2100 — not a wall-clock reading"
		);
	}

	/// Pins `SystemClock` to the *wall* clock rather than to any other source:
	/// its reading must lie between two `SystemTime::now()` readings taken on
	/// either side of it.
	#[test]
	fn clock_system_clock_agrees_with_system_time() {
		let before = Instant::from_system_time(SystemTime::now());
		let reading = SystemClock.now();
		let after = Instant::from_system_time(SystemTime::now());
		assert!(
			before <= reading && reading <= after,
			"{reading:?} is not between {before:?} and {after:?}"
		);
	}

	/// `SystemTime::now()` appears in exactly one file — `clock.rs` — and
	/// exactly once in it, and the monotonic `Instant::now()` appears nowhere.
	///
	/// **This is not p5's ratchet.** The count-of-clock-reads gate described in
	/// `learnings/clock.md` §"The fix" is a *consumer* gate over `../mitosys`
	/// and `../model`, it enumerates the ~65 reads in each tree, and it belongs
	/// to the adoption node. This test reads `shared/src/` and nothing else:
	/// it keeps *this crate* honest about having one reader. Do not delete the
	/// consumer ratchet believing this one already covers it, and do not grow
	/// this one into it.
	///
	/// Line comments are stripped first — the module documents the hazard in
	/// prose, and prose about a clock read is not a clock read.
	#[test]
	fn clock_system_clock_is_the_only_reader() {
		let src = crate_root().join("src");
		let mut files = Vec::new();
		rust_sources(&src, &mut files);
		assert!(files.len() >= 4, "expected at least four source files");

		let mut readers: Vec<String> = Vec::new();
		let mut monotonic: Vec<String> = Vec::new();
		let mut reads_in_clock_rs = 0usize;
		for path in files {
			let text = std::fs::read_to_string(&path).expect("a readable source file");
			let code = code_only(&text);
			let name = path
				.file_name()
				.expect("a named file")
				.to_string_lossy()
				.into_owned();
			if code.contains("SystemTime::now()") {
				readers.push(name.clone());
				if name == "clock.rs" {
					reads_in_clock_rs = code.matches("SystemTime::now()").count();
				}
			}
			if code.contains("Instant::now()") {
				monotonic.push(name);
			}
		}
		readers.sort();
		assert_eq!(
			readers,
			vec!["clock.rs".to_string()],
			"the operating system's clock is read outside clock.rs"
		);
		assert_eq!(
			reads_in_clock_rs, 1,
			"clock.rs must read the OS clock exactly once, found {reads_in_clock_rs}"
		);
		// The monotonic reading is refusal 5, and this crate makes none: the
		// stronger form of "at most clock.rs" is "nowhere at all".
		assert!(
			monotonic.is_empty(),
			"a monotonic Instant::now() appears in {monotonic:?}"
		);
	}

	/// A fixed clock answers the same instant every time it is asked — across
	/// the whole range, including before the epoch.
	#[test]
	fn clock_fixed_clock_is_constant() {
		for t in [
			Instant::EPOCH,
			Instant::from_unix_secs(T2026),
			Instant::from_unix_nanos(-1_500_000_000),
			Instant::MAX,
		] {
			let c = FixedClock::new(t);
			assert_eq!(c.now(), t);
			for _ in 0..100 {
				assert_eq!(c.now(), t, "a fixed clock moved");
			}
		}
	}

	#[test]
	fn clock_fixed_clock_round_trips_its_instant() {
		for t in [
			Instant::EPOCH,
			Instant::from_unix_secs(T2026),
			Instant::from_unix_nanos(-1),
			Instant::MIN,
		] {
			assert_eq!(FixedClock::new(t).instant(), t);
		}
	}

	/// A fold that stamps each input with the clock, in the shape of
	/// `../model`'s `rec_preimage` — which ends
	/// `out.extend_from_slice(&r.created.to_le_bytes())`.
	///
	/// Written against `&dyn Clock` on purpose: that is what proves `Clock` is
	/// object-safe, which is what p5 needs to hold one behind a pointer.
	fn fold(inputs: &[&str], clock: &dyn Clock) -> Vec<u8> {
		let mut out = Vec::new();
		for input in inputs {
			out.extend_from_slice(input.as_bytes());
			out.extend_from_slice(&clock.now().as_unix_nanos().to_le_bytes());
		}
		out
	}

	/// The ticket's acceptance criterion, made runnable: *serialize the
	/// record, fold from empty, compare*.
	///
	/// (a) the same fixed clock yields byte-identical output; (b) a different
	/// fixed clock yields different output, so (a) is not vacuous — the clock
	/// really is an input to the fold. Two `SystemClock` folds are
	/// deliberately **not** compared: that would be a flaky test, and the
	/// claim is not that the system clock is unstable but that a fixed one is
	/// available.
	#[test]
	fn clock_fold_is_reproducible_under_a_fixed_clock() {
		let inputs = ["alpha", "beta", "gamma"];
		let t = Instant::from_unix_secs(T2026);
		let clock = FixedClock::new(t);

		let first = fold(&inputs, &clock);
		let second = fold(&inputs, &clock);
		assert_eq!(
			first, second,
			"a fold under a fixed clock is not reproducible"
		);
		assert!(!first.is_empty());

		let t2 = t.saturating_add(Duration::from_secs(1));
		assert_ne!(t2, t);
		let elsewhere = fold(&inputs, &FixedClock::new(t2));
		assert_ne!(
			first, elsewhere,
			"the clock is not actually an input to the fold"
		);
	}

	/// The spellings `../model`'s `Arc`-held substrate needs, proven here
	/// rather than discovered in the adoption node.
	#[test]
	fn clock_trait_is_object_safe_and_shareable() {
		fn assert_send_sync<T: Send + Sync + 'static>() {}
		assert_send_sync::<SystemClock>();
		assert_send_sync::<FixedClock>();

		let c: Arc<dyn Clock + Send + Sync> = Arc::new(SystemClock);
		assert!(c.now() > Instant::EPOCH);

		let b: Box<dyn Clock> = Box::new(FixedClock::new(Instant::EPOCH));
		assert_eq!(b.now(), Instant::EPOCH);
	}

	/// `&C`, `Box<C>` and `Arc<C>` are clocks, so no consumer re-derives the
	/// forwarding impl.
	#[test]
	fn clock_blanket_impls_forward() {
		let t = Instant::from_unix_secs(T2026);
		let by_ref: &dyn Clock = &FixedClock::new(t);
		assert_eq!(by_ref.now(), t);
		assert_eq!(Clock::now(&&FixedClock::new(t)), t);
		// Bound to their pointer types and called through `Clock::now`, so the
		// call resolves at `Box<C>` / `Arc<C>` — the blanket impls — rather than
		// auto-dereferencing to `FixedClock::now`.
		let owned: Box<FixedClock> = Box::new(FixedClock::new(t));
		assert_eq!(Clock::now(&owned), t);
		let counted: Arc<FixedClock> = Arc::new(FixedClock::new(t));
		assert_eq!(Clock::now(&counted), t);

		// And through the pointer forms of the trait object, which is the
		// shape a consumer actually stores.
		let boxed: Box<dyn Clock> = Box::new(FixedClock::new(t));
		assert_eq!(boxed.now(), t);
		let shared: Arc<dyn Clock + Send + Sync> = Arc::new(FixedClock::new(t));
		assert_eq!(shared.now(), t);
	}

	/// The test that fails if `SystemClock::now()` ever bypasses
	/// [`Instant::from_system_time`] and does its own
	/// `duration_since(..).unwrap()`. spec01 supplies the saturating
	/// behaviour; this pins the clock to it.
	#[test]
	fn clock_system_clock_never_panics_on_conversion() {
		assert_eq!(
			Instant::from_system_time(UNIX_EPOCH - Duration::from_secs(1)),
			Instant::from_unix_secs(-1)
		);
		// `SystemTime`'s own `+` panics on overflow, so the far-future point
		// is built with `checked_add`; where the platform cannot represent it
		// there is nothing to saturate.
		if let Some(far) = UNIX_EPOCH.checked_add(Duration::from_secs(u64::MAX / 2)) {
			assert_eq!(Instant::from_system_time(far), Instant::MAX);
		}
		// The live path, which must not panic either.
		let reading = SystemClock.now();
		assert!(reading > Instant::EPOCH && reading < Instant::MAX);
	}

	/// The one dependency contract, restated for this file: nothing in the
	/// clock reaches for a date library.
	#[test]
	fn clock_uses_no_date_library() {
		let src = crate_root().join("src");
		let mut files = Vec::new();
		rust_sources(&src, &mut files);
		for path in files {
			let text = std::fs::read_to_string(&path).expect("a readable source file");
			assert!(
				!code_only(&text).contains("chrono"),
				"{} reaches for chrono",
				path.display()
			);
		}
	}
}
