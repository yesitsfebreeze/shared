//! `ContentId` — fixed vectors, the rejection table, and the two gates that
//! keep `blake3` where this ticket put it.
//!
//! Wrapped in `mod content_id` on purpose: the ticket's own verify is
//! `cargo test -p conserved content_id`, and that filter matches **test
//! names**, not file names. Without the wrapper every test here reports as
//! `roundtrip_is_a_fixed_point` and the filter reports `0 tests … filtered
//! out` — green having run nothing. The wrapper makes each one
//! `content_id::<name>`, so the gate selects them. Same convention as
//! `tests/scope.rs`.

mod content_id {
	use conserved::{ContentId, ContentIdParseError};

	/// Input → its blake3 digest, spelled the one way this crate spells it.
	/// A silent algorithm swap breaks every row.
	const VECTORS: &[(&[u8], &str)] = &[
		(
			b"",
			"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
		),
		(
			b"abc",
			"6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85",
		),
		(
			b"hello world",
			"d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24",
		),
		(
			b"conserved",
			"8d369871266d2453da564f5748e5a3070f25068aa5be7db442dd2c2b1b31f08e",
		),
	];

	fn zero_to_thirty_one() -> Vec<u8> {
		(0u8..32).collect()
	}

	#[test]
	fn fixed_vectors_render_as_expected() {
		for (input, expected) in VECTORS {
			assert_eq!(
				ContentId::of(input).to_string(),
				*expected,
				"vector {input:?} moved"
			);
		}
		assert_eq!(
			ContentId::of(&zero_to_thirty_one()).to_string(),
			"e528e95798037df410543d9f31e396ecdd458d71b157d6014398bae32fb56c65"
		);
	}

	/// The test the ticket's acceptance exists for: if someone swaps blake3
	/// back out for SHA-256 — what `../mitosys/src/mitosys/util/util.rs:9`
	/// returns today — these two digests appear and this named test fails.
	#[test]
	fn sha256_swap_would_fail() {
		assert_ne!(
			ContentId::of(b"abc").to_string(),
			"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
		);
		assert_ne!(
			ContentId::of(b"").to_string(),
			"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
		);
	}

	#[test]
	fn display_is_exactly_64_lowercase_hex() {
		let mut inputs: Vec<Vec<u8>> = VECTORS.iter().map(|(i, _)| i.to_vec()).collect();
		inputs.push(zero_to_thirty_one());
		for input in inputs {
			let rendered = ContentId::of(&input).to_string();
			assert_eq!(rendered.len(), 64, "{rendered} is not 64 bytes");
			for b in rendered.bytes() {
				assert!(
					b.is_ascii_digit() || (b'a'..=b'f').contains(&b),
					"{} is not a lowercase hex character in {rendered}",
					b as char
				);
			}
		}
	}

	/// A derived `Debug` prints 32 decimal integers, which is a second
	/// spelling of the id in every log line. This one prints the hex.
	#[test]
	fn debug_prints_hex_not_a_decimal_byte_array() {
		let rendered = format!("{:?}", ContentId::of(b"abc"));
		assert_eq!(
			rendered,
			"ContentId(6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85)"
		);
		assert!(
			!rendered.contains('['),
			"{rendered} looks like a byte array"
		);
		assert!(
			!rendered.contains(','),
			"{rendered} looks like a byte array"
		);
	}

	#[test]
	fn from_bytes_and_as_bytes_are_inverses() {
		let bytes: [u8; 32] = *ContentId::of(b"conserved").as_bytes();
		assert_eq!(ContentId::from_bytes(bytes).as_bytes(), &bytes);

		// A fixed-size borrow, not a slice: the length cannot be lost on the
		// way out.
		let id = ContentId::of(b"abc");
		let borrowed: &[u8; 32] = id.as_bytes();
		assert_eq!(borrowed.len(), 32);
	}

	#[test]
	fn parse_accepts_its_own_rendering() {
		let mut inputs: Vec<Vec<u8>> = VECTORS.iter().map(|(i, _)| i.to_vec()).collect();
		inputs.push(zero_to_thirty_one());
		for input in inputs {
			let id = ContentId::of(&input);
			let parsed: ContentId = id.to_string().parse().expect("its own rendering parses");
			assert_eq!(parsed, id);
			assert_eq!(parsed.to_string(), id.to_string());
		}
	}

	const VALID: &str = "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85";

	fn err(s: &str) -> ContentIdParseError {
		match s.parse::<ContentId>() {
			Ok(id) => panic!("{s:?} must not parse, but produced {id}"),
			Err(e) => e,
		}
	}

	#[test]
	fn parse_rejects_63_characters() {
		assert_eq!(
			err(&VALID[..63]),
			ContentIdParseError::WrongLength { got: 63 }
		);
	}

	#[test]
	fn parse_rejects_65_characters() {
		assert_eq!(
			err(&format!("{VALID}0")),
			ContentIdParseError::WrongLength { got: 65 }
		);
	}

	#[test]
	fn parse_rejects_the_empty_string() {
		assert_eq!(err(""), ContentIdParseError::WrongLength { got: 0 });
	}

	/// Uppercase is a second spelling of the same id, which is the problem the
	/// whole ticket is about. `A-F` is rejected rather than case-folded.
	#[test]
	fn parse_rejects_uppercase() {
		assert!(matches!(
			err(&VALID.to_uppercase()),
			ContentIdParseError::NotHex { .. }
		));
	}

	#[test]
	fn parse_rejects_surrounding_whitespace() {
		assert_eq!(
			err(&format!("{VALID}\n")),
			ContentIdParseError::WrongLength { got: 65 }
		);
		assert_eq!(
			err(&format!(" {VALID} ")),
			ContentIdParseError::WrongLength { got: 66 }
		);
	}

	#[test]
	fn parse_rejects_an_0x_prefix() {
		assert_eq!(
			err(&format!("0x{VALID}")),
			ContentIdParseError::WrongLength { got: 66 }
		);
	}

	#[test]
	fn parse_rejects_a_non_hex_character() {
		// `g` at byte 3, everything before it valid — so the error points at
		// the offender, not at the start.
		let mut s = String::from(VALID);
		s.replace_range(3..4, "g");
		assert_eq!(err(&s), ContentIdParseError::NotHex { at: 3 });
	}

	/// The one behaviour `../mitosys/src/mitosys/util/util.rs` has and this
	/// type deliberately does not: it strips an `ed25519:` prefix before
	/// decoding (`let s = s.strip_prefix("ed25519:").unwrap_or(s);`), so one
	/// id has two spellings there. Prefix stripping is the caller's business
	/// at the call site; the mitosys-side shim is p5's.
	#[test]
	fn ed25519_prefix_is_rejected() {
		assert!(format!("ed25519:{VALID}").parse::<ContentId>().is_err());
	}

	/// `WrongLength` wins over `NotHex`: the length is checked before the
	/// decode loop, so a short non-hex string reports its shape problem first.
	#[test]
	fn wrong_length_wins_over_not_hex() {
		assert_eq!(err("zzz"), ContentIdParseError::WrongLength { got: 3 });
	}

	// ----- the gates -------------------------------------------------------

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

	/// `blake3` enters the crate in `content_id.rs` and is reachable through
	/// that module alone. Same shape as mitosys's own
	/// `src/mitosys/gates/tests/dependency_tree.rs`: a gate that reads the
	/// tree rather than a comment asking nicely. If a second module reaches
	/// for the hasher, this fails.
	///
	/// Line comments are stripped before the search: prose *about* the
	/// dependency (`lib.rs` narrates it) is not a module reaching for it.
	#[test]
	fn blake3_is_reachable_only_through_content_id() {
		let src = crate_root().join("src");
		let mut files = Vec::new();
		rust_sources(&src, &mut files);
		assert!(files.len() >= 3, "expected lib.rs, scope.rs, content_id.rs");

		let mut mentions: Vec<String> = Vec::new();
		for path in files {
			let text = std::fs::read_to_string(&path).expect("a readable source file");
			let code: String = text
				.lines()
				.map(|line| match line.find("//") {
					Some(i) => &line[..i],
					None => line,
				})
				.collect::<Vec<_>>()
				.join("\n");
			if code.contains("blake3") {
				mentions.push(
					path
						.file_name()
						.expect("a named file")
						.to_string_lossy()
						.into_owned(),
				);
			}
		}
		mentions.sort();
		assert_eq!(
			mentions,
			vec!["content_id.rs".to_string()],
			"blake3 escaped content_id.rs"
		);
	}

	/// Read the `[dependencies]` table of `conserved/Cargo.toml` and return
	/// each entry as `(name, the rest of the line)`.
	fn dependency_entries(manifest: &str) -> Vec<(String, String)> {
		let mut entries = Vec::new();
		let mut inside = false;
		for line in manifest.lines() {
			let line = line.trim();
			if line.starts_with('[') {
				inside = line == "[dependencies]";
				continue;
			}
			if !inside || line.is_empty() || line.starts_with('#') {
				continue;
			}
			let (name, rest) = line.split_once('=').expect("a key = value line");
			entries.push((name.trim().to_string(), rest.trim().to_string()));
		}
		entries
	}

	fn section(manifest: &str, header: &str) -> Vec<String> {
		let mut out = Vec::new();
		let mut inside = false;
		for line in manifest.lines() {
			let line = line.trim();
			if line.starts_with('[') {
				inside = line == header;
				continue;
			}
			if inside && !line.is_empty() && !line.starts_with('#') {
				out.push(line.to_string());
			}
		}
		out
	}

	/// The crate's dependency contract, asserted against the manifest rather
	/// than trusted.
	///
	/// `blake3` is the one unconditional dependency: p2's whole "one
	/// dependency" claim. It is inherited from `[workspace.dependencies]`,
	/// which is the dependency convention `lib.rs` commits this crate to
	/// (mitosys's, per `AGENTS.md` §divergences), so the version pin is
	/// asserted at the workspace manifest below.
	#[test]
	fn blake3_is_the_only_dependency() {
		let manifest =
			std::fs::read_to_string(crate_root().join("Cargo.toml")).expect("the crate manifest");
		let entries = dependency_entries(&manifest);
		let names: Vec<&str> = entries.iter().map(|(n, _)| n.as_str()).collect();
		assert_eq!(
			names,
			vec!["blake3"],
			"[dependencies] must name blake3 and nothing else"
		);

		let workspace = std::fs::read_to_string(crate_root().join("..").join("Cargo.toml"))
			.expect("the workspace manifest");
		assert!(
			section(&workspace, "[workspace.dependencies]").contains(&"blake3 = \"1\"".to_string()),
			"blake3 must be pinned at \"1\" — the version ../model/Cargo.toml:19 declares"
		);
	}
}
