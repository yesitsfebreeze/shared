//! `Instant` — the unit, the range, the flooring, the saturation, and the two
//! refusals that are enforced by reading the source rather than by asking.
//!
//! Every test function is wrapped in `mod clock` **and** named with a `clock_`
//! prefix. That is not cosmetic: the ticket's own gate is
//! `cargo test -p shared clock`, which filters on test *names*, not file
//! names, so a test called `unit_is_nanoseconds` in a file called
//! `clock_instant.rs` is reported as "0 tests, 1 filtered out" — green, having
//! run nothing.

mod clock {
	use shared::Instant;
	use std::time::{Duration, SystemTime, UNIX_EPOCH};

	/// `shared/`, from the test binary's manifest directory.
	fn crate_root() -> std::path::PathBuf {
		std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
	}

	/// Every `*.rs` under `shared/src/`.
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

	/// A source file with its line comments removed, so that prose *about* a
	/// hazard is not mistaken for the hazard. Same treatment p2's
	/// `blake3_is_reachable_only_through_content_id` gives its gate.
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

	/// The unit-pinning test the ticket's acceptance names.
	///
	/// `Instant` is **nanoseconds** since 1970-01-01T00:00:00Z. Changing this
	/// is a wire-format change for both trees — `../model`'s postcard rows in
	/// redb and its blake3 commit preimage move — not a refactor. A doc
	/// comment alone demonstrably does not hold the line:
	/// `../model/src/node/transactional.rs:59` documents its `timestamp` as
	/// "unix epoch milliseconds" and line 72 computes `.as_secs()`.
	#[test]
	fn clock_instant_unit_is_unix_nanoseconds() {
		assert_eq!(Instant::EPOCH.as_unix_nanos(), 0);
		assert_eq!(Instant::from_unix_secs(1).as_unix_nanos(), 1_000_000_000);
		assert_eq!(Instant::from_unix_millis(1).as_unix_nanos(), 1_000_000);
		// 2026-08-21T00:00:00Z.
		assert_eq!(
			Instant::from_unix_secs(1_787_270_400).as_unix_millis(),
			1_787_270_400_000
		);
		assert_eq!(
			Instant::from_unix_nanos(1_787_270_400_123_456_789).as_unix_secs(),
			1_787_270_400
		);
	}

	/// The epoch is 1970, not boot and not call number one.
	///
	/// This is the test that fails if someone reintroduces the condemned
	/// scaffold's tick-counter reading of `Instant`
	/// (`.mi/docs/memos/scaffold-reset.md`).
	#[test]
	fn clock_instant_epoch_is_unix_not_boot_and_not_a_counter() {
		assert_eq!(Instant::EPOCH.to_system_time(), UNIX_EPOCH);
		assert_eq!(Instant::from_system_time(UNIX_EPOCH).as_unix_nanos(), 0);
	}

	/// The range the nanosecond decision costs: 1677-09-21T00:12:43.145224192Z
	/// to 2262-04-11T23:47:16.854775807Z. No site in either tree pays it.
	#[test]
	fn clock_instant_range_bounds() {
		assert_eq!(Instant::MIN.as_unix_nanos(), i64::MIN);
		assert_eq!(Instant::MAX.as_unix_nanos(), i64::MAX);
		assert!(Instant::MIN < Instant::EPOCH && Instant::EPOCH < Instant::MAX);
	}

	/// The truncation trap, as its own named test.
	///
	/// Rust's `/` truncates toward zero, so `-1_500_000_000 / 1_000_000_000`
	/// is `-1` where the floor is `-2`. The accessors use `div_euclid`, which
	/// is what keeps them monotone across the epoch.
	#[test]
	fn clock_instant_as_unix_secs_floors_before_the_epoch() {
		assert_eq!(Instant::from_unix_nanos(-1_500_000_000).as_unix_secs(), -2);
		assert_eq!(Instant::from_unix_nanos(-1).as_unix_secs(), -1);
		assert_eq!(
			Instant::from_unix_nanos(-1_500_000_000).as_unix_millis(),
			-1_500
		);
		assert_eq!(Instant::from_unix_nanos(1_999_999_999).as_unix_secs(), 1);
	}

	/// Nothing here wraps, and nothing here panics in debug mode — which is
	/// the mode `cargo test` builds.
	#[test]
	fn clock_instant_conversions_saturate_rather_than_wrap() {
		assert_eq!(Instant::from_unix_secs(i64::MAX), Instant::MAX);
		assert_eq!(Instant::from_unix_secs(i64::MIN), Instant::MIN);
		assert_eq!(Instant::from_unix_millis(i64::MAX), Instant::MAX);
		assert_eq!(Instant::from_unix_millis(i64::MIN), Instant::MIN);
		assert_eq!(
			Instant::MAX.saturating_add(Duration::from_secs(1)),
			Instant::MAX
		);
		assert_eq!(
			Instant::MIN.saturating_sub(Duration::from_secs(1)),
			Instant::MIN
		);
		// A `Duration` is wider than this type's range; it saturates too.
		assert_eq!(
			Instant::EPOCH.saturating_add(Duration::from_secs(u64::MAX / 2)),
			Instant::MAX
		);
	}

	/// `../model`'s `rec_heat` (`src/record/mod.rs:258`) branches on the sign
	/// of this difference, which a `Duration`-returning `duration_since`
	/// cannot express.
	#[test]
	fn clock_instant_signed_nanos_since_is_signed() {
		let earlier = Instant::from_unix_secs(1_787_270_400);
		let later = Instant::from_unix_secs(1_787_270_460);
		assert!(later.signed_nanos_since(earlier) > 0);
		assert_eq!(later.signed_nanos_since(earlier), 60 * 1_000_000_000);
		assert!(earlier.signed_nanos_since(later) < 0);
		assert_eq!(earlier.signed_nanos_since(later), -60 * 1_000_000_000);
		assert_eq!(later.signed_nanos_since(later), 0);
		assert_eq!(Instant::MIN.signed_nanos_since(Instant::MAX), i64::MIN);
		assert_eq!(Instant::MAX.signed_nanos_since(Instant::MIN), i64::MAX);
	}

	/// The bridge mitosys needs, in both directions.
	///
	/// The pre-1970 case is its own assertion because
	/// `SystemTime::duration_since(UNIX_EPOCH)` returns `Err` there, and the
	/// `.unwrap_or_default()` both trees reach for silently yields the epoch.
	#[test]
	fn clock_instant_system_time_round_trip() {
		for instant in [
			Instant::EPOCH,
			Instant::from_unix_secs(1_787_270_400),
			Instant::from_unix_nanos(1_787_270_400_123_456_789),
			Instant::MAX,
		] {
			assert_eq!(
				Instant::from_system_time(instant.to_system_time()),
				instant,
				"{instant:?} did not survive the SystemTime bridge"
			);
		}

		// Pre-1970, on its own: the `Err` branch must carry the sign, not
		// collapse to the epoch.
		let pre = Instant::from_unix_nanos(-1_500_000_000);
		assert_eq!(Instant::from_system_time(pre.to_system_time()), pre);
		assert_ne!(
			Instant::from_system_time(pre.to_system_time()),
			Instant::EPOCH
		);
		assert_eq!(
			Instant::from_system_time(Instant::MIN.to_system_time()),
			Instant::MIN
		);
	}

	/// Derived `Ord` on the `i64` gives this; the test is what stops a later
	/// hand-written `Ord`, or an unsigned representation, from silently
	/// reversing the pre-epoch half of the timeline.
	#[test]
	fn clock_instant_ordering_is_chronological() {
		let mut instants: Vec<Instant> = [3_i64, -1, 0, 2_000_000_000, -86_400, 1]
			.iter()
			.map(|s| Instant::from_unix_secs(*s))
			.collect();
		instants.sort();
		let seconds: Vec<i64> = instants.iter().map(|i| i.as_unix_secs()).collect();
		assert_eq!(seconds, vec![-86_400, -1, 0, 1, 3, 2_000_000_000]);
		assert!(Instant::from_unix_secs(-1) < Instant::EPOCH);
		assert!(Instant::from_unix_nanos(-1) < Instant::EPOCH);
	}

	/// `0` is the epoch, a real instant, and both trees already overload `0`
	/// as "unset" (`until == 0`, `heat_at == 0`, `last_secs == 0`). A
	/// `Default` would let the sentinel and the timestamp wear one spelling,
	/// so the epoch has a name — `Instant::EPOCH` — and the sentinel stays the
	/// consumer's `Option`.
	#[test]
	fn clock_instant_has_no_default() {
		let source = std::fs::read_to_string(crate_root().join("src").join("clock.rs"))
			.expect("clock.rs is readable");
		let code = code_only(&source);

		assert!(
			!code.contains("impl Default for Instant"),
			"Instant must not implement Default"
		);

		// The derive list that immediately precedes `pub struct Instant`.
		let struct_at = code
			.find("pub struct Instant(")
			.expect("Instant is declared in clock.rs");
		let derives = code[..struct_at]
			.rfind("#[derive(")
			.map(|i| code[i..struct_at].to_string())
			.expect("Instant carries a derive list");
		assert!(
			!derives.contains("Default"),
			"Instant's derive list must not contain Default, got `{}`",
			derives.trim()
		);
		// `SystemClock` may derive `Default` — a unit struct's default is
		// itself and carries no ambiguity. Only `Instant` is checked here.

		// And the refusal is written down, not merely observed.
		assert!(
			source.lines().any(|l| {
				let t = l.trim_start();
				(t.starts_with("//!") || t.starts_with("///")) && t.contains("Instant::EPOCH")
			}),
			"the refusal paragraph naming Instant::EPOCH must be present"
		);
	}

	/// The name collision, enforced rather than trusted.
	///
	/// `shared::Instant` is the **wall** clock: comparable and storable
	/// across processes and machines. std's `time::Instant` is **monotonic**:
	/// opaque, process-local, unserializable. The ~47 monotonic sites across
	/// the two trees — profilers, deadlines — are the other kind and must not
	/// be converted, so this crate never imports the std type at all.
	///
	/// Line comments are stripped first: prose *about* the collision (this
	/// module's refusal 2 spells the hazard out) is not an import.
	#[test]
	fn clock_src_never_uses_std_time_instant() {
		let src = crate_root().join("src");
		let mut files = Vec::new();
		rust_sources(&src, &mut files);
		assert!(
			files.len() >= 4,
			"expected at least lib.rs, scope.rs, content_id.rs, clock.rs"
		);

		for path in files {
			let text = std::fs::read_to_string(&path).expect("a readable source file");
			let code = code_only(&text);
			assert!(
				!code.contains("std::time::Instant"),
				"{} names std::time::Instant — that is the monotonic clock",
				path.display()
			);
			assert!(
				!code.contains("use std::time::Instant"),
				"{} imports the monotonic Instant",
				path.display()
			);
		}
	}

	/// The `SystemTime` bridge on the two paths that panic elsewhere in the
	/// family: before the epoch, and past the range.
	#[test]
	fn clock_instant_from_system_time_never_panics_at_the_edges() {
		assert_eq!(
			Instant::from_system_time(UNIX_EPOCH - Duration::from_secs(1)),
			Instant::from_unix_secs(-1)
		);
		// `SystemTime`'s own `+` panics on overflow, so the far-future case is
		// built with `checked_add`; on a platform that cannot represent it
		// there is nothing for `from_system_time` to saturate.
		if let Some(far) = UNIX_EPOCH.checked_add(Duration::from_secs(u64::MAX / 2)) {
			assert_eq!(Instant::from_system_time(far), Instant::MAX);
		}
		// And a real reading lands somewhere sane.
		let now = Instant::from_system_time(SystemTime::now());
		assert!(now > Instant::EPOCH);
	}
}
