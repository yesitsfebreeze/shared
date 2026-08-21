//! `ContentId` under serde — the hex string in JSON, and the 32 raw bytes on a
//! binary wire, byte-identical to a bare `[u8; 32]`.
//!
//! The whole file is behind the optional `serde` feature.
//!
//! Wrapped in `mod content_id` for the same reason as the other two test
//! files: the ticket gate `cargo test -p conserved content_id` filters on test
//! *names*, not file names, so without the wrapper it would run none of these
//! and still exit 0.

#![cfg(feature = "serde")]

mod content_id {
	use conserved::ContentId;
	use serde::{Deserialize, Serialize};

	const ABC_HEX: &str = "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85";

	const VECTORS: &[(&[u8], &str)] = &[
		(
			b"",
			"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
		),
		(b"abc", ABC_HEX),
		(
			b"hello world",
			"d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24",
		),
		(
			b"conserved",
			"8d369871266d2453da564f5748e5a3070f25068aa5be7db442dd2c2b1b31f08e",
		),
	];

	// ----- human-readable: JSON --------------------------------------------

	#[test]
	fn json_is_a_hex_string_not_an_array_of_numbers() {
		let encoded = serde_json::to_string(&ContentId::of(b"abc")).expect("serializes");
		assert_eq!(encoded, format!("\"{ABC_HEX}\""));
		assert!(!encoded.contains('['), "{encoded} is an array");
	}

	#[test]
	fn json_round_trips_every_vector() {
		for (input, hex) in VECTORS {
			let id = ContentId::of(input);
			let encoded = serde_json::to_string(&id).expect("serializes");
			assert_eq!(encoded, format!("\"{hex}\""));
			let decoded: ContentId = serde_json::from_str(&encoded).expect("round-trips");
			assert_eq!(decoded, id);
		}
	}

	/// The deserializer goes through `FromStr`, so there is one rule and one
	/// place it lives: everything the parser rejects, JSON rejects too.
	#[test]
	fn json_rejects_what_from_str_rejects() {
		for bad in [
			ABC_HEX.to_uppercase(),
			ABC_HEX[..63].to_string(),
			format!("{ABC_HEX}0"),
			format!("0x{ABC_HEX}"),
			format!("{}g{}", &ABC_HEX[..3], &ABC_HEX[4..]),
			String::new(),
			format!("{ABC_HEX}\\n"),
		] {
			let json = format!("\"{bad}\"");
			assert!(
				serde_json::from_str::<ContentId>(&json).is_err(),
				"{json} must not deserialize"
			);
		}
	}

	/// The one tolerance `../mitosys/src/mitosys/util/util.rs` has and this
	/// type does not — asserted on the serde path too, because that is where
	/// a stored id would arrive from.
	#[test]
	fn json_rejects_an_ed25519_prefix() {
		let json = format!("\"ed25519:{ABC_HEX}\"");
		assert!(serde_json::from_str::<ContentId>(&json).is_err());
	}

	/// The human-readable form is the hex string and only that. An
	/// array-of-numbers is what a bare `[u8; 32]` would produce, and it is not
	/// accepted here.
	#[test]
	fn json_rejects_arrays_and_numbers() {
		assert!(serde_json::from_str::<ContentId>("[1,2,3]").is_err());
		assert!(serde_json::from_str::<ContentId>("5").is_err());
		assert!(serde_json::from_str::<ContentId>("null").is_err());
	}

	// ----- binary: postcard -------------------------------------------------

	fn binary_cases() -> Vec<ContentId> {
		vec![
			ContentId::of(b""),
			ContentId::of(b"abc"),
			ContentId::from_bytes([0xff; 32]),
			ContentId::from_bytes([0x00; 32]),
		]
	}

	/// The test that stops `../model`'s redb keys and gossip frames from
	/// moving under it. `serialize_bytes` would prepend a varint length —
	/// 33 bytes in postcard — and a fixed 32-tuple does not.
	#[test]
	fn binary_encoding_is_identical_to_u8_32() {
		for id in binary_cases() {
			let as_id = postcard::to_stdvec(&id).expect("serializes");
			let as_array = postcard::to_stdvec(id.as_bytes()).expect("serializes");
			assert_eq!(as_id, as_array, "{id} does not encode as its [u8; 32]");
			assert_eq!(as_id.len(), 32, "{id} encoded to {} bytes", as_id.len());
		}
	}

	#[test]
	fn postcard_round_trips() {
		for id in binary_cases() {
			let encoded = postcard::to_stdvec(&id).expect("serializes");
			let decoded: ContentId = postcard::from_bytes(&encoded).expect("round-trips");
			assert_eq!(decoded, id);
		}
	}

	#[test]
	fn postcard_rejects_a_short_frame() {
		let encoded = postcard::to_stdvec(&ContentId::of(b"abc")).expect("serializes");
		assert!(
			postcard::from_bytes::<ContentId>(&encoded[..31]).is_err(),
			"31 bytes must never silently produce an id"
		);
	}

	#[test]
	fn postcard_consumes_exactly_32_bytes() {
		let id = ContentId::of(b"abc");
		let mut long = postcard::to_stdvec(&id).expect("serializes");
		long.push(0xaa);
		match postcard::take_from_bytes::<ContentId>(&long) {
			Ok((decoded, rest)) => {
				assert_eq!(decoded, id);
				assert_eq!(rest, &[0xaa], "the id must consume exactly 32 bytes");
			}
			Err(_) => { /* erring on a 33-byte frame is equally acceptable */ }
		}
	}

	/// The substitution `../model`'s `SyncSummary`
	/// (`src/utils/transport/codec.rs:59-67`) will perform in p5, proven here
	/// rather than discovered there: swapping `[u8; 32]` for `ContentId`
	/// inside a serde struct does not move a byte of the encoding.
	#[test]
	fn struct_substitution_is_wire_compatible() {
		#[derive(Serialize, Deserialize)]
		struct A {
			keys: Vec<[u8; 32]>,
			n: u64,
		}

		#[derive(Serialize, Deserialize)]
		struct B {
			keys: Vec<ContentId>,
			n: u64,
		}

		let ids = binary_cases();
		let a = A {
			keys: ids.iter().map(|id| *id.as_bytes()).collect(),
			n: 4_242,
		};
		let b = B {
			keys: ids.clone(),
			n: 4_242,
		};

		let encoded_a = postcard::to_stdvec(&a).expect("serializes");
		let encoded_b = postcard::to_stdvec(&b).expect("serializes");
		assert_eq!(encoded_a, encoded_b, "the substitution moves the wire");

		// And the bytes A wrote decode as B, which is what an in-place
		// migration actually needs.
		let decoded: B = postcard::from_bytes(&encoded_a).expect("A's bytes are B's bytes");
		assert_eq!(decoded.keys, ids);
		assert_eq!(decoded.n, 4_242);
	}

	/// Both branches of `is_human_readable()` are exercised above; this one
	/// states the difference in a single assertion so the split cannot quietly
	/// collapse to one form.
	#[test]
	fn the_two_forms_differ() {
		let id = ContentId::of(b"abc");
		let json = serde_json::to_string(&id).expect("serializes");
		let binary = postcard::to_stdvec(&id).expect("serializes");
		assert_eq!(json.len(), 66, "64 hex characters and two quotes");
		assert_eq!(binary.len(), 32);
		assert_ne!(json.as_bytes(), binary.as_slice());
	}
}
