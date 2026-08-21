# goal

`ContentId([u8; 32])` — blake3 in, 64 lowercase hex out — with hex encode/decode
private behind `Display`/`FromStr`, `blake3` entering the crate here and
reachable through this module alone, and the fixed test vectors that make a
silent algorithm swap fail a named test.

est: 2.0h

## What p0 leaves behind (assumed, not created here)

- Package `conserved` at `conserved/`, `edition = "2021"`,
  `rust-version = "1.94.0"`, an empty-but-compiling `conserved/src/lib.rs`.
- Root `Cargo.toml` is `[workspace]` only, `resolver = "2"`, one member:
  `conserved`.
- Tests live at `conserved/tests/` (the mitosys shape — `AGENTS.md`
  §divergences). No `src/**/tests/`, no `#[cfg(test)]` module beside the code.
- The five pre-split `conserved-*` directories and the old
  `conserved/src/lib.rs` stub are deleted. This spec writes into a crate whose
  `[dependencies]` table is empty or absent.
- `rustfmt.toml` at the repo root already sets `hard_tabs = true`,
  `tab_spaces = 2`; write to it, do not fight it.

If p0 landed a different package name or test root, stop and re-spec — the
ticket's own `verify` (`cargo test -p conserved content_id`) names both.

## Files

- `conserved/Cargo.toml` — add `[dependencies] blake3 = "1"`.
- `conserved/src/content_id.rs` — **new**. The whole module.
- `conserved/src/lib.rs` — `mod content_id;` + `pub use content_id::{ContentId, ContentIdParseError};`.
- `conserved/tests/content_id.rs` — **new**. Vectors, rejections, encapsulation gate.

Touches nothing else. Does **not** touch `../mitosys`, `../model`, `../realm`,
`learnings/`, or any other ticket's directory.

## The surface

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentId([u8; 32]);

impl ContentId {
	pub fn of(bytes: &[u8]) -> Self;          // blake3
	pub fn from_bytes(bytes: [u8; 32]) -> Self;
	pub fn as_bytes(&self) -> &[u8; 32];
}

impl fmt::Display for ContentId;              // exactly 64 lowercase hex
impl fmt::Debug for ContentId;                // hand-written: ContentId(<64 hex>)
impl FromStr for ContentId;                   // Err = ContentIdParseError

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContentIdParseError {
	WrongLength { got: usize },               // anything but 64 chars
	NotHex { at: usize },                     // byte index of the first offender
}
impl fmt::Display for ContentIdParseError;
impl std::error::Error for ContentIdParseError;
```

Three departures from the shape printed in `learnings/content-addressing.md`
§"The shape", each forced by a real call site — record each one in the module
doc comment, not only here:

- **`Debug` is hand-written, not derived and not omitted.** The learning's
  derive list omits it, but `../model`'s `Record` is `#[derive(Clone, Debug)]`
  (`src/record/mod.rs:152`) and its wire types are `#[derive(Debug, Clone,
  PartialEq, Serialize, Deserialize)]` (`src/utils/transport/codec.rs:59`) —
  a `ContentId` that is not `Debug` cannot be substituted for `[u8; 32]` in
  either. Derived `Debug` would print 32 decimal integers; print the hex.
- **`from_bytes` exists.** The learning lists only `of` and `as_bytes`, which
  is a one-way type: `../model` stores ids as redb `&[u8]` keys
  (`src/utils/fs/mod.rs:23-67`) and reads them back, and spec03's
  `Deserialize` needs the same constructor. Without it the type cannot cross a
  storage boundary at all.
- **No `Default`.** `[u8; 32]` has one and `../model`'s `impl Default for
  Record` (`src/record/mod.rs:180`) uses it. A zero `ContentId` is not the
  hash of anything, which is exactly the value law 3's first rung says the
  type must make inexpressible. p5 carries the mitosys/model-side fix; this
  crate does not supply the hole. State the refusal in the doc comment.

## Acceptance

- [x] `conserved/Cargo.toml` `[dependencies]` contains exactly one entry,
      `blake3 = "1"` — the same version string `../model/Cargo.toml:19`
      already declares, with blake3's default features, so no build behaviour
      new to that tree enters with the crate.
- [x] `ContentId` derives `Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord`
      and is a newtype over `[u8; 32]` whose field is private.
- [x] `ContentId::of(b"")` renders as
      `af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262`.
- [x] `ContentId::of(b"abc")` renders as
      `6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85`.
- [x] `ContentId::of(b"hello world")` renders as
      `d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24`.
- [x] `ContentId::of(b"conserved")` renders as
      `8d369871266d2453da564f5748e5a3070f25068aa5be7db442dd2c2b1b31f08e`.
- [x] `ContentId::of(&(0u8..32).collect::<Vec<u8>>())` renders as
      `e528e95798037df410543d9f31e396ecdd458d71b157d6014398bae32fb56c65`.
- [x] A test named `sha256_swap_would_fail` asserts
      `ContentId::of(b"abc").to_string()` is **not**
      `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`
      (SHA-256 of the same input — what `../mitosys/src/mitosys/util/util.rs:9`
      returns today) and **not**
      `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
      for `b""`. The comment above it names the swap it exists to catch.
- [x] `Display` output is exactly 64 bytes and every byte is in `0-9a-f` for
      each vector above (asserted, not eyeballed).
- [x] `Debug` of `ContentId::of(b"abc")` is
      `ContentId(6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85)`
      — no decimal byte array anywhere in the output.
- [x] `from_bytes(b).as_bytes() == &b` and
      `ContentId::of(x).as_bytes()` returns a `&[u8; 32]`, not a slice.
- [x] `FromStr` accepts each vector's own `Display` output and returns a
      `ContentId` equal to the original.
- [x] `FromStr` rejects, each as its own named case with the stated error:
      63 hex chars (`WrongLength { got: 63 }`), 65 hex chars
      (`WrongLength { got: 65 }`), the empty string (`WrongLength { got: 0 }`),
      the uppercase spelling of a valid id (`NotHex`), a valid id with a
      trailing newline or surrounding spaces (`WrongLength`), `0x` + 64 hex
      (`WrongLength`), and a 64-char string containing `g` (`NotHex { at: .. }`
      pointing at the `g`).
- [x] A test named `ed25519_prefix_is_rejected` asserts
      `"ed25519:<64 valid hex>".parse::<ContentId>()` is `Err`. This is the
      one behaviour `../mitosys/src/mitosys/util/util.rs` `hex::decode`
      (`let s = s.strip_prefix("ed25519:").unwrap_or(s);`) has and this type
      deliberately does not.
- [x] The module doc comment records the refusal in prose: prefix stripping is
      the caller's business at the call site, and the mitosys-side shim is
      p5's, not this crate's. A reader who deletes the check must first delete
      that paragraph.
- [x] Hex encode and decode are **private** to `conserved/src/content_id.rs` —
      no `pub fn encode`/`decode`, no `pub mod hex`, nothing hex-shaped
      re-exported from `lib.rs`. `cargo doc` for the crate shows `ContentId`
      and `ContentIdParseError` and no hex utility.
- [x] A test named `blake3_is_reachable_only_through_content_id` reads every
      `*.rs` under `conserved/src/` and asserts that the only file mentioning
      `blake3` is `content_id.rs`. It fails if a second module reaches for the
      hasher. (Same shape as mitosys's own
      `src/mitosys/gates/tests/dependency_tree.rs`: a gate that reads the tree
      rather than a comment asking nicely.)
- [x] A test named `blake3_is_the_only_dependency` reads
      `conserved/Cargo.toml` and asserts the `[dependencies]` table names
      `blake3` and nothing else. (spec03 amends this test — deliberately, in
      one commit, with the reason written in it.)
- [x] No `unsafe`. No `unwrap`/`expect` on any path reachable from `FromStr`.
- [x] `cargo clippy -p conserved --all-targets -- -D warnings` is clean.

## Notes for the implementer

- `blake3::hash(bytes)` returns a `Hash`; `*h.as_bytes()` is the `[u8; 32]`.
  Do not go via `to_hex()` and back — the hex path here is ours.
- Decode with a nibble table, the same shape mitosys's `hex_nibble` uses,
  minus the prefix strip and with the length checked **before** the loop so
  `WrongLength` always wins over `NotHex` on a short non-hex string.
- Reject uppercase by simply not accepting `A-F` in the nibble table. A
  canonical rendering with a canonical parse means one spelling per id; case
  folding would reintroduce the two-spellings problem the whole ticket is
  about.
- **p1 interaction:** this spec moves `conserved`'s normal-dependency edge
  count from 0 to 1, which breaks p1-scope's `cargo tree ... | grep -qx 1`
  gate if p1 is still open. That edge is already recorded as missing in
  `.mi/gantt/plan.md` §Edges. Land after p1, or expect that failure and read
  it correctly.

verify: `cargo fmt --all --check && cargo clippy -p conserved --all-targets -- -D warnings && cargo test -p conserved --test content_id`
