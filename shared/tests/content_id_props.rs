//! `ContentId` — the round-trip properties the ticket's acceptance names.
//!
//! Wrapped in `mod content_id` for the same reason as `tests/content_id.rs`:
//! the ticket gate `cargo test -p shared content_id` filters on test
//! *names*, so without the wrapper these report as `display_parse_round_trip`
//! and friends and the gate runs none of them while exiting 0.
//!
//! Every property runs at proptest's default case count. There is no
//! `proptest_config` here shrinking the coverage to look fast.

mod content_id {
	use proptest::prelude::*;
	use shared::ContentId;

	fn is_lowercase_hex(s: &str) -> bool {
		s.bytes()
			.all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
	}

	/// A string worth handing to `FromStr`: mostly junk, but often the right
	/// length and often the right alphabet, so both arms of the `iff` below
	/// are actually exercised.
	fn candidate_string() -> impl Strategy<Value = String> {
		prop_oneof![
			".*",
			"[0-9a-f]{0,80}",
			"[0-9a-fA-F]{64}",
			"[0-9a-f]{64}",
			"[[:ascii:]]{64}",
		]
	}

	proptest! {
		#[test]
		fn display_is_always_64_lowercase_hex(input in prop::collection::vec(any::<u8>(), 0..4096)) {
			let rendered = ContentId::of(&input).to_string();
			prop_assert_eq!(rendered.len(), 64);
			prop_assert!(is_lowercase_hex(&rendered));
		}

		#[test]
		fn display_parse_round_trip(input in prop::collection::vec(any::<u8>(), 0..4096)) {
			let id = ContentId::of(&input);
			let parsed = id.to_string().parse::<ContentId>();
			prop_assert!(parsed.is_ok());
			prop_assert_eq!(parsed.unwrap(), id);
		}

		#[test]
		fn parse_display_round_trip(bytes in any::<[u8; 32]>()) {
			let id = ContentId::from_bytes(bytes);
			let rendered = id.to_string();
			let parsed = rendered.parse::<ContentId>();
			prop_assert!(parsed.is_ok());
			let parsed = parsed.unwrap();
			prop_assert_eq!(parsed.as_bytes(), &bytes);
			// The canonical form is a fixed point: no id has two spellings.
			prop_assert_eq!(parsed.to_string(), rendered);
		}

		#[test]
		fn of_is_deterministic(input in prop::collection::vec(any::<u8>(), 0..4096)) {
			prop_assert_eq!(ContentId::of(&input), ContentId::of(&input));
			let copy = input.clone();
			prop_assert_eq!(ContentId::of(&input), ContentId::of(&copy));
		}

		#[test]
		fn distinct_inputs_distinct_ids(
			a in prop::collection::vec(any::<u8>(), 0..512),
			b in prop::collection::vec(any::<u8>(), 0..512),
		) {
			prop_assume!(a != b);
			// A failure here is a blake3 collision, not a flake. Treat it as
			// a hash failure and do not add a retry.
			prop_assert_ne!(ContentId::of(&a), ContentId::of(&b));
		}

		/// One mutation away from canonical is not canonical. The `ed25519:`
		/// case is drawn like every other mutation — it is not a special case
		/// in this crate, which is exactly the point.
		#[test]
		fn mutated_hex_is_rejected(
			input in prop::collection::vec(any::<u8>(), 0..512),
			which in 0usize..7,
			pos in 0usize..64,
			filler in prop::sample::select(vec!['g', 'z', 'G', 'Z', '!', '-', ' ']),
		) {
			let valid = ContentId::of(&input).to_string();
			let mutant = match which {
				0 => {
					// Uppercasing a digit is a no-op, so only mutate a letter.
					prop_assume!(valid.as_bytes()[pos].is_ascii_alphabetic());
					let mut s = valid.clone();
					s.replace_range(pos..pos + 1, &valid[pos..pos + 1].to_uppercase());
					s
				}
				1 => {
					let mut s = valid.clone();
					s.replace_range(pos..pos + 1, "");
					s
				}
				2 => format!("{valid}0"),
				3 => {
					let mut s = valid.clone();
					s.replace_range(pos..pos + 1, &filler.to_string());
					s
				}
				4 => format!("ed25519:{valid}"),
				5 => format!("0x{valid}"),
				_ => format!("{valid}\n"),
			};
			prop_assert_ne!(&mutant, &valid);
			prop_assert!(mutant.parse::<ContentId>().is_err(), "{} parsed", mutant);
		}

		/// Parsing succeeds **iff** the string is the canonical spelling, and
		/// when it succeeds the value renders back to the identical string.
		#[test]
		fn arbitrary_strings_never_parse_unless_canonical(s in candidate_string()) {
			let canonical = s.len() == 64 && is_lowercase_hex(&s);
			match s.parse::<ContentId>() {
				Ok(id) => {
					prop_assert!(canonical, "{} parsed but is not canonical", s);
					prop_assert_eq!(id.to_string(), s);
				}
				Err(_) => prop_assert!(!canonical, "{} is canonical but did not parse", s),
			}
		}
	}
}
