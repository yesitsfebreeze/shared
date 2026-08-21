//! `ContentId` — one hash, one spelling.
//!
//! A content id is the blake3 digest of some bytes, held as `[u8; 32]` and
//! rendered as exactly 64 lowercase hex characters. `learnings/content-addressing.md`
//! settles the two choices this module encodes: blake3 over SHA-256, and
//! `[u8; 32]` over a hex `String`. Today mitosys hashes SHA-256 into a hex
//! `String` and llm hashes blake3 into `[u8; 32]`, so an id computed on one
//! side does not exist on the other. This type is the single spelling both
//! sides can hold.
//!
//! Hex encoding and decoding are **private to this module**, reachable only
//! through [`Display`](fmt::Display) and [`FromStr`]. There is no public
//! `encode`/`decode` for two callers to spell differently, and nothing
//! hex-shaped is re-exported from the crate root. One canonical rendering and
//! one canonical parse means one spelling per id.
//!
//! `blake3` enters the crate here and is reachable through this module alone.
//! The integration test `blake3_is_reachable_only_through_content_id` enforces
//! that by reading `conserved/src/` rather than by asking nicely.
//!
//! # What this type refuses
//!
//! - **No `ed25519:` prefix tolerance.** `../mitosys/src/mitosys/util/util.rs`
//!   decodes hex with `let s = s.strip_prefix("ed25519:").unwrap_or(s);`, so
//!   two spellings of one id parse to the same value there. That tolerance
//!   does **not** port. Prefix stripping is the caller's business at the call
//!   site, and the mitosys-side shim belongs to the adoption node (p5), not to
//!   this crate. A reader who deletes the rejection must delete this paragraph
//!   first — and then explain what an id with a signature-scheme prefix is.
//!
//! - **No `Default`.** `[u8; 32]` has one, and `../model`'s
//!   `impl Default for Record` (`src/record/mod.rs:180`) leans on it. A zero
//!   `ContentId` is not the hash of anything: it is a hole shaped like an id,
//!   which is exactly what this type exists to make inexpressible. p5 carries
//!   the model-side fix; this crate does not supply the hole.
//!
//! # Two departures from `learnings/content-addressing.md` §"The shape"
//!
//! - **`Debug` is hand-written, not derived and not omitted.** The learning's
//!   derive list leaves `Debug` out, but `../model`'s `Record` is
//!   `#[derive(Clone, Debug)]` (`src/record/mod.rs:152`) and its wire types are
//!   `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`
//!   (`src/utils/transport/codec.rs:59`) — a `ContentId` that is not `Debug`
//!   cannot stand where `[u8; 32]` stands in either. A *derived* `Debug` would
//!   print 32 decimal integers, which is a second spelling of the id in every
//!   log line, so this one prints the hex.
//!
//! - **[`ContentId::from_bytes`] exists.** The learning lists only `of` and
//!   `as_bytes`, which is a one-way type. `../model` stores ids as redb
//!   `&[u8]` keys (`src/utils/fs/mod.rs:23-67`) and reads them back, and the
//!   `serde` feature's `Deserialize` needs the same constructor. Without it the
//!   type cannot cross a storage boundary at all.

use std::fmt;
use std::str::FromStr;

/// The blake3 digest of some bytes: 32 bytes, rendered as 64 lowercase hex.
///
/// Construct with [`ContentId::of`] (hash some bytes) or
/// [`ContentId::from_bytes`] (adopt a digest that already exists, e.g. one read
/// back out of storage).
///
/// Two refusals, repeated here from the module documentation because they are
/// the surface a caller meets:
///
/// - **No `Default`.** A zero id is not the hash of anything, and this type
///   exists to make that value inexpressible.
/// - **No `ed25519:` prefix tolerance on parse.** mitosys's hex decode strips
///   that prefix, so one id has two spellings there. Stripping it is the
///   caller's business at the call site.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentId([u8; 32]);

impl ContentId {
	/// The blake3 digest of `bytes`.
	pub fn of(bytes: &[u8]) -> Self {
		Self(*blake3::hash(bytes).as_bytes())
	}

	/// Adopt 32 bytes that are already a digest.
	///
	/// This is the inverse of [`ContentId::as_bytes`], and the constructor a
	/// storage or wire boundary needs on the way back in.
	pub fn from_bytes(bytes: [u8; 32]) -> Self {
		Self(bytes)
	}

	/// The 32 digest bytes, borrowed as a fixed-size array — not a slice, so a
	/// caller cannot lose the length on the way out.
	pub fn as_bytes(&self) -> &[u8; 32] {
		&self.0
	}
}

/// Why a string is not a [`ContentId`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContentIdParseError {
	/// The input was not exactly 64 bytes long. Checked before any hex
	/// decoding, so a short non-hex string reports its length rather than its
	/// first bad character.
	WrongLength {
		/// The length that was offered, in bytes.
		got: usize,
	},
	/// The input was 64 bytes long but held something outside `0-9a-f`.
	NotHex {
		/// Byte index of the first offending character.
		at: usize,
	},
}

impl fmt::Display for ContentIdParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::WrongLength { got } => {
				write!(f, "a content id is 64 lowercase hex characters, got {got}")
			}
			Self::NotHex { at } => {
				write!(f, "not a lowercase hex character at byte {at}")
			}
		}
	}
}

impl std::error::Error for ContentIdParseError {}

/// The 16 characters a content id may be spelled with. Uppercase is absent on
/// purpose: see [`nibble`].
const HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";

/// Encode 32 bytes as 64 lowercase hex characters. Private: the only way out is
/// [`Display`](fmt::Display).
fn encode_hex(bytes: &[u8; 32]) -> [u8; 64] {
	let mut out = [0u8; 64];
	for (i, b) in bytes.iter().enumerate() {
		out[i * 2] = HEX_DIGITS[usize::from(b >> 4)];
		out[i * 2 + 1] = HEX_DIGITS[usize::from(b & 0x0f)];
	}
	out
}

/// One hex character to one nibble.
///
/// `A-F` is rejected rather than case-folded. A canonical rendering with a
/// canonical parse means one spelling per id; accepting uppercase would
/// reintroduce the two-spellings problem this type exists to close.
fn nibble(c: u8) -> Option<u8> {
	match c {
		b'0'..=b'9' => Some(c - b'0'),
		b'a'..=b'f' => Some(c - b'a' + 10),
		_ => None,
	}
}

/// Decode exactly 64 lowercase hex characters into 32 bytes. Private: the only
/// way in is [`FromStr`].
///
/// The length is checked *before* the loop, so `WrongLength` always wins over
/// `NotHex` on a short non-hex string — the caller learns the shape problem
/// first. No prefix is stripped and no case is folded.
fn decode_hex(s: &str) -> Result<[u8; 32], ContentIdParseError> {
	let src = s.as_bytes();
	if src.len() != 64 {
		return Err(ContentIdParseError::WrongLength { got: src.len() });
	}
	let mut out = [0u8; 32];
	for (i, pair) in src.chunks_exact(2).enumerate() {
		let hi = nibble(pair[0]).ok_or(ContentIdParseError::NotHex { at: i * 2 })?;
		let lo = nibble(pair[1]).ok_or(ContentIdParseError::NotHex { at: i * 2 + 1 })?;
		out[i] = (hi << 4) | lo;
	}
	Ok(out)
}

impl fmt::Display for ContentId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let hex = encode_hex(&self.0);
		// `encode_hex` only ever emits bytes from `HEX_DIGITS`, so this is
		// ASCII by construction; the `map_err` is there so no path in this
		// module unwraps.
		f.write_str(std::str::from_utf8(&hex).map_err(|_| fmt::Error)?)
	}
}

impl fmt::Debug for ContentId {
	/// Hand-written so a log line spells an id the same way every other surface
	/// does. `#[derive(Debug)]` would print 32 decimal integers.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "ContentId({self})")
	}
}

impl FromStr for ContentId {
	type Err = ContentIdParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		decode_hex(s).map(Self)
	}
}

// ---------------------------------------------------------------------------
// serde — behind the optional `serde` feature, off by default.
// ---------------------------------------------------------------------------

/// Visits the human-readable form: a 64-character lowercase hex string.
#[cfg(feature = "serde")]
struct HexVisitor;

#[cfg(feature = "serde")]
impl serde::de::Visitor<'_> for HexVisitor {
	type Value = ContentId;

	fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("a content id as 64 lowercase hex characters")
	}

	fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
		// Straight through `FromStr`: one rule, and one place it lives. The
		// rejections tested against `FromStr` — uppercase, wrong length, an
		// `ed25519:` or `0x` prefix — are therefore the deserializer's too.
		v.parse().map_err(E::custom)
	}
}

/// Visits the binary form: 32 elements of a fixed-size tuple.
#[cfg(feature = "serde")]
struct BytesVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for BytesVisitor {
	type Value = ContentId;

	fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("32 bytes")
	}

	fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
		let mut out = [0u8; 32];
		for (i, slot) in out.iter_mut().enumerate() {
			*slot = match seq.next_element()? {
				Some(b) => b,
				// A short frame must never quietly become an id.
				None => return Err(serde::de::Error::invalid_length(i, &"32 bytes")),
			};
		}
		Ok(ContentId(out))
	}
}

/// A `ContentId` is a 64-character lowercase hex **string** in human-readable
/// formats and the **32 raw bytes** in binary ones — byte-identical to a bare
/// `[u8; 32]`, so the type can be substituted for one on an existing wire or
/// in an existing key without moving a single byte.
///
/// Two things are deliberate here.
///
/// The binary form is a fixed 32-length **tuple**, not `serialize_bytes`.
/// `learnings/content-addressing.md:77` says "bytes on a binary wire", and
/// under serde `[u8; 32]` is a fixed-size tuple: postcard writes it as 32
/// bytes with no length prefix, while `serialize_bytes` would write a varint
/// length first — 33 bytes. Taking the sentence literally would silently move
/// `../model`'s redb keys and its peer frames, which is the representation
/// drift this whole type exists to prevent. `binary_encoding_is_identical_to_u8_32`
/// and `struct_substitution_is_wire_compatible` pin it.
///
/// The JSON form is a **deliberate** divergence from `[u8; 32]`'s
/// array-of-numbers. `../model`'s MCP surface already hands ids out as hex
/// strings by hand (`src/mcp/tests/fold.rs:334`); this makes an existing
/// convention typed rather than inventing a new one, and keeps a record
/// readable by eye where it is already JSON.
///
/// The branch is on `is_human_readable()`, never on a format name — a third
/// format gets the right form without this module learning its name.
#[cfg(feature = "serde")]
impl serde::Serialize for ContentId {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		if serializer.is_human_readable() {
			let hex = encode_hex(&self.0);
			let rendered = std::str::from_utf8(&hex).map_err(serde::ser::Error::custom)?;
			serializer.serialize_str(rendered)
		} else {
			use serde::ser::SerializeTuple;
			let mut tuple = serializer.serialize_tuple(32)?;
			for byte in &self.0 {
				tuple.serialize_element(byte)?;
			}
			tuple.end()
		}
	}
}

/// The mirror of [`Serialize`](serde::Serialize): hex string in, through
/// [`FromStr`], from human-readable formats; a fixed 32-length tuple from
/// binary ones. See that impl's documentation for why the binary form is a
/// tuple and not `serialize_bytes`.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ContentId {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		if deserializer.is_human_readable() {
			deserializer.deserialize_str(HexVisitor)
		} else {
			deserializer.deserialize_tuple(32, BytesVisitor)
		}
	}
}
