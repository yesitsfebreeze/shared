//! `Instant` under serde — the bare `i64` nanoseconds, identically in every
//! format, so a field that is `i64` today can become an `Instant` without a
//! wire change.
//!
//! The whole file is behind the optional `serde` feature p2 admitted.
//!
//! Wrapped in `mod clock` and every test named with a `clock_` prefix: the
//! gate `cargo test -p conserved --features serde clock` filters on test
//! *names*, not file names.

#![cfg(feature = "serde")]

mod clock {
	use conserved::Instant;
	use serde::{Deserialize, Serialize};

	/// The five values every encoding test runs: the epoch, a 2026 instant, a
	/// pre-epoch instant, and both ends of the range.
	fn cases() -> Vec<Instant> {
		vec![
			Instant::EPOCH,
			Instant::from_unix_secs(1_787_270_400),
			Instant::from_unix_nanos(-1_500_000_000),
			Instant::MIN,
			Instant::MAX,
		]
	}

	/// The test that stops `../model`'s redb rows
	/// (`src/record/log.rs:268-285`, postcard) and its blake3 commit preimage
	/// (`src/node/transactional.rs:99`) from moving under it: an `Instant`
	/// encodes as exactly the bytes its `i64` would.
	#[test]
	fn clock_serde_binary_encoding_is_identical_to_i64() {
		for instant in cases() {
			let as_instant = postcard::to_stdvec(&instant).expect("serializes");
			let as_i64 = postcard::to_stdvec(&instant.as_unix_nanos()).expect("serializes");
			assert_eq!(
				as_instant, as_i64,
				"{instant:?} does not encode as its i64 nanoseconds"
			);
		}
	}

	/// The exact substitution p5 will perform on `StoredRecord`
	/// (`../model/src/record/log.rs:268`), proven here rather than discovered
	/// there.
	#[test]
	fn clock_serde_struct_substitution_is_wire_compatible() {
		#[derive(Serialize, Deserialize)]
		struct A {
			created: i64,
			until: i64,
			heat: f32,
		}

		#[derive(Serialize, Deserialize)]
		struct B {
			created: Instant,
			until: Instant,
			heat: f32,
		}

		let created = 1_787_270_400_123_456_789_i64;
		let until = -1_500_000_000_i64;
		let a = A {
			created,
			until,
			heat: 0.5,
		};
		let b = B {
			created: Instant::from_unix_nanos(created),
			until: Instant::from_unix_nanos(until),
			heat: 0.5,
		};

		let encoded_a = postcard::to_stdvec(&a).expect("serializes");
		let encoded_b = postcard::to_stdvec(&b).expect("serializes");
		assert_eq!(encoded_a, encoded_b, "the substitution moves the wire");

		// And the bytes A wrote decode as B, which is what an in-place
		// migration actually needs.
		let decoded: B = postcard::from_bytes(&encoded_a).expect("A's bytes are B's bytes");
		assert_eq!(decoded.created.as_unix_nanos(), created);
		assert_eq!(decoded.until.as_unix_nanos(), until);
	}

	#[test]
	fn clock_serde_postcard_round_trips() {
		for instant in cases() {
			let encoded = postcard::to_stdvec(&instant).expect("serializes");
			let decoded: Instant = postcard::from_bytes(&encoded).expect("round-trips");
			assert_eq!(decoded, instant);
		}
	}

	/// A number, not a string. Every field this type replaces is already a
	/// JSON number today, and a string form would be the wire change the type
	/// exists to avoid.
	#[test]
	fn clock_serde_json_is_a_number_not_a_string() {
		let encoded =
			serde_json::to_string(&Instant::from_unix_secs(1_787_270_400)).expect("serializes");
		assert_eq!(encoded, "1787270400000000000");
		assert!(!encoded.contains('"'), "{encoded} is quoted");
		assert_eq!(
			serde_json::to_string(&Instant::from_unix_nanos(-1)).expect("serializes"),
			"-1"
		);
	}

	/// Both ends of the range included: a `u64`-shaped deserializer fails
	/// here, and so does one that clamps.
	#[test]
	fn clock_serde_json_round_trips_including_negative_and_extremes() {
		for instant in cases() {
			let encoded = serde_json::to_string(&instant).expect("serializes");
			let decoded: Instant = serde_json::from_str(&encoded).expect("round-trips");
			assert_eq!(decoded, instant, "{encoded} did not round-trip");
		}
	}

	/// One representation, and only that one.
	#[test]
	fn clock_serde_json_rejects_non_integers() {
		for bad in [
			"\"1787270400\"",
			"1.5",
			"null",
			"[1]",
			"9223372036854775808",
			"{}",
			"true",
		] {
			assert!(
				serde_json::from_str::<Instant>(bad).is_err(),
				"{bad} must not deserialize"
			);
		}
	}

	/// The unit is pinned on the wire, not only in the type: the integer that
	/// arrives is nanoseconds, and the deserializer does not quietly
	/// reinterpret it as seconds.
	#[test]
	fn clock_serde_deserialize_uses_from_unix_nanos() {
		let epoch: Instant = serde_json::from_str("0").expect("deserializes");
		assert_eq!(epoch, Instant::EPOCH);
		let one_second: Instant = serde_json::from_str("1000000000").expect("deserializes");
		assert_eq!(one_second, Instant::from_unix_secs(1));
		assert_eq!(one_second.as_unix_secs(), 1);

		// Postcard says the same thing, so the unit is not a JSON accident.
		let encoded = postcard::to_stdvec(&1_000_000_000_i64).expect("serializes");
		let decoded: Instant = postcard::from_bytes(&encoded).expect("deserializes");
		assert_eq!(decoded, Instant::from_unix_secs(1));
	}

	/// A clock is not data — only the instant it produced is. This is a
	/// compile-time refusal, so it is asserted by reading the source: no
	/// `Serialize`/`Deserialize` impl names `SystemClock` or `FixedClock`.
	#[test]
	fn clock_serde_is_not_implemented_for_the_clocks() {
		let source = std::fs::read_to_string(
			std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("src")
				.join("clock.rs"),
		)
		.expect("clock.rs is readable");
		let code: String = source
			.lines()
			.map(|line| match line.find("//") {
				Some(i) => &line[..i],
				None => line,
			})
			.collect::<Vec<_>>()
			.join("\n");
		for forbidden in [
			"Serialize for SystemClock",
			"Deserialize for SystemClock",
			"Serialize for FixedClock",
			"Deserialize<'de> for FixedClock",
			"Deserialize for FixedClock",
		] {
			assert!(
				!code.contains(forbidden),
				"a clock is not data: found `{forbidden}`"
			);
		}
		// The impls that do exist live in clock.rs, beside the type, not in a
		// module of their own — which is what keeps p2's
		// `blake3_is_reachable_only_through_content_id` and spec02's
		// `clock_system_clock_is_the_only_reader` true.
		assert!(code.contains("impl serde::Serialize for Instant"));
	}
}
