# goal

`ContentId` serde support behind an **optional `serde` feature, off by
default**: a 64-char lowercase hex string in human-readable formats, and an
encoding byte-identical to `[u8; 32]` in binary ones — closing the gap between
`learnings/content-addressing.md` §"The shape" and this ticket's "the one
dependency" without changing the crate's default dependency contract.

est: 1.5h

## The gap this closes, and why it is settled here

`learnings/content-addressing.md:77` states plainly: *"Serde: bytes on a binary
wire, hex string in JSON, so a record stays readable by eye where it is already
JSON."* The board's own audit (`.mi/gantt/plan.md` §"Lost work") records that
no ticket owned it and that it reads as a contradiction with p2's *"the one
dependency — `blake3` enters the crate here"* and `learnings/shared-crate.md`
§"Size and shape" (*"one dependency (`blake3`)"*). It is not a contradiction
once the dependency is optional, and the call sites decide it rather than
taste:

1. **The default dependency contract does not move.** With `default = []`,
   `cargo tree -p conserved --edges normal` still shows exactly one edge out
   of `conserved`, and it is `blake3`. Every sentence in the learnings about
   "one dependency" stays literally true of the crate as built by default.
2. **Enabling it adds nothing new to either consumer's closure.**
   `../mitosys/Cargo.toml:191` already pins
   `serde = { version = "1", features = ["derive", "rc"] }`, and `serde`,
   `serde_core` and `serde_derive` are already recorded in mitosys's
   `src/mitosys/gates/tests/dependency_tree.rs` `CLOSURE`
   (lines 225-227). `../model/Cargo.toml:65` declares
   `serde = { version = "1", features = ["derive"] }`. Admission criterion 3
   ("it passes mitosys's `dependency_tree.rs` gate") is a mechanical test, and
   it passes: no crate that was not already there arrives. (Enabling a
   feature is not a direct declaration, so that gate's `OWNERS` row for
   `serde` is unchanged. The `conserved` and `blake3` rows are p5's commit.)
3. **Without it, `../model` cannot adopt the type at all.** Its ids live
   *inside* serde structs, on the wire — `SyncSummary { keys: Vec<[u8; 32]> }`,
   `src/utils/transport/codec.rs:59-67` — and on disk —
   `VersionRecord { model_id, parents: Vec<[u8; 32]>, checksum: [u8; 32] }`
   serialized with postcard and keyed into redb, `src/utils/fs/mod.rs:185-205`.
   A `ContentId` that cannot stand where `[u8; 32]` stands leaves the drift
   this ticket exists to close exactly where it was.

The one thing the learning's prose does *not* settle, and this spec does:
**"bytes on a binary wire" means the 32 raw bytes, not `serialize_bytes`.**
Under serde, `[u8; 32]` is a fixed-size tuple, and postcard writes it as 32
bytes with no length prefix; `serializer.serialize_bytes` would write a varint
length first (33 bytes in postcard). Taking the sentence literally would
silently change `../model`'s redb keys and its peer wire — a hash-shaped
representation drift, which is the failure mode this whole node exists to
prevent. Serialize as a fixed 32-length tuple, and pin the equality with a
test.

## What this assumes

- spec01 has landed: `ContentId`, `from_bytes`, `as_bytes`, `Display`,
  `FromStr` with its rejection rules, and the `blake3_is_the_only_dependency`
  manifest gate test.
- p0's manifest: package `conserved`, edition 2021, `rust-version = "1.94.0"`
  (the `dep:` feature syntax used below needs cargo 1.60+; 1.94 is fine).

## Files

- `conserved/Cargo.toml` — optional `serde`, a `[features]` table,
  dev-dependencies `serde_json` and `postcard`.
- `conserved/src/content_id.rs` — hand-written `Serialize`/`Deserialize`
  under `#[cfg(feature = "serde")]`, in the module that owns the type.
- `conserved/tests/content_id.rs` — amend the manifest gate test (see below).
- `conserved/tests/content_id_serde.rs` — **new**. Whole file behind
  `#![cfg(feature = "serde")]`.

## The manifest

```toml
[dependencies]
blake3 = "1"
serde = { version = "1", optional = true, default-features = false }

[features]
default = []
serde = ["dep:serde"]

[dev-dependencies]
serde_json = "1"
postcard = { version = "1", features = ["alloc", "use-std"] }
```

No `serde/derive`: the impls are hand-written, because the human-readable
split cannot be derived. Add `features = ["std"]` to the optional `serde` line
only if the build actually demands it, and say so in a comment if you do.
`serde_json` and `postcard` match `../model/Cargo.toml:65,48` — the two
formats the consumers actually use, so the test exercises the real encoders,
not a stand-in.

## Acceptance

- [ ] `cargo build -p conserved` with **no** features compiles, and
      `cargo tree -p conserved --edges normal --depth 1` lists `blake3` and
      no `serde`.
- [ ] spec01's manifest gate test is amended (not deleted) to assert
      `[dependencies]` holds exactly `blake3` unconditional and `serde`
      carrying `optional = true`, and that `[features]` has `default = []`.
      Its comment names this spec and the reason the second entry is allowed.
      A third dependency, or `serde` losing `optional`, fails it.
- [ ] `serde_json::to_string(&ContentId::of(b"abc"))` is
      `"\"6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85\""`
      — a JSON string, not an array of numbers.
- [ ] `serde_json::from_str::<ContentId>` round-trips every spec01 vector, and
      **rejects** the same inputs `FromStr` rejects: uppercase, 63/65 chars,
      `0x`-prefixed, non-hex, and — as its own named case —
      `"\"ed25519:<64 hex>\""`. The deserializer goes through `FromStr`, so
      there is one rule and one place it lives.
- [ ] `serde_json::from_str::<ContentId>("[1,2,3]")` and `from_str::<ContentId>("5")`
      are `Err` — the human-readable form is the hex string and only that.
- [ ] A test named `binary_encoding_is_identical_to_u8_32` asserts, for at
      least three ids including `ContentId::of(b"")` and one all-`0xff` value,
      that `postcard::to_stdvec(&id)? == postcard::to_stdvec(id.as_bytes())?`
      and that both are exactly 32 bytes long. This is the test that stops
      `../model`'s redb keys and gossip frames from moving under it.
- [ ] `postcard::from_bytes::<ContentId>(&postcard::to_stdvec(&id)?)` equals
      `id`, for the same values.
- [ ] `postcard::from_bytes::<ContentId>` on 31 bytes is `Err`, and on 33
      bytes either errs or consumes exactly 32 — a short frame must never
      silently produce an id.
- [ ] A test named `struct_substitution_is_wire_compatible` postcard-encodes
      a `struct A { keys: Vec<[u8; 32]>, n: u64 }` and a
      `struct B { keys: Vec<ContentId>, n: u64 }` holding the same values and
      asserts the two byte strings are equal — the substitution `../model`'s
      `SyncSummary` will perform in p5, proven here rather than discovered
      there.
- [ ] The `Serialize`/`Deserialize` impls branch on
      `serializer.is_human_readable()` / `deserializer.is_human_readable()`,
      not on a format name, and both branches are exercised by tests.
- [ ] The doc comment on the impls states the split in one sentence and states
      that the JSON form is a **deliberate** divergence from `[u8; 32]`'s
      array-of-numbers — `../model`'s MCP surface already hands ids out as hex
      strings by hand (`src/mcp/tests/fold.rs:334`), so this makes an existing
      convention typed rather than inventing one.
- [ ] `cargo test -p conserved` (no features) passes and
      `cargo test -p conserved --features serde` passes.
- [ ] `cargo clippy -p conserved --all-targets --features serde -- -D warnings`
      is clean, and so is the same command without `--features serde`.
- [ ] spec01's `blake3_is_reachable_only_through_content_id` still passes: the
      serde impls live in `content_id.rs`, not a new module.

verify: `cargo test -p conserved --features serde && cargo test -p conserved && cargo clippy -p conserved --all-targets --features serde -- -D warnings && cargo tree -p conserved --edges normal --depth 1`
