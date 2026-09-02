# goal

`Instant` behind p2's **existing optional `serde` feature** — encoded exactly as
the bare `i64` it replaces, in every format — so `../model` can put the type
where its timestamps already live (postcard into redb, postcard into a blake3
commit preimage, JSON out of the MCP surface) without moving a single byte.

est: 0.75h

## Ordering: this spec lands AFTER p2-content-id/spec03

**It adds no dependency and changes no line of `conserved/Cargo.toml`.** That is
only true if p2's spec03 has already landed the optional `serde` dependency, the
`[features] default = []` table, and the `serde_json` / `postcard`
dev-dependencies. If p2 has not landed, **stop and report to the orchestrator** —
do not add `serde` here. Two tickets adding the same optional dependency to the
same `[dependencies]` table is exactly the collision the board's footprints exist
to prevent, and p2's manifest gate test
(`blake3_is_the_only_dependency`, amended by p2 spec03) will fail on whichever
edit lands second.

Everything this spec needs from the manifest, p2 already wrote:

```toml
serde = { version = "1", optional = true, default-features = false }
[features]
default = []
serde = ["dep:serde"]
[dev-dependencies]
serde_json = "1"
postcard = { version = "1", features = ["alloc", "use-std"] }
```

Also assumed: spec01 and spec02 of this ticket have landed.

## Why `Instant` needs serde at all

The same argument p2 spec03 made for `ContentId`, from the same tree, on the
fields right beside the ids:

1. **`../model/src/record/log.rs:268-285`** — `StoredRecord { …, created: i64,
   until: i64, heat_at: i64 }`, `#[derive(Serialize, Deserialize)]`, encoded
   with `postcard::to_stdvec` (line 305) and keyed into redb. If `Instant`
   cannot stand where `i64` stands, `rec_now()` — the read this whole ticket
   exists to remove — cannot be replaced at its only destination.
2. **`../model/src/node/transactional.rs:47-62`** — `Commit { …, timestamp: u64 }`,
   postcard-encoded and **blake3-hashed** ("the content hash of this commit:
   blake3 of the canonical postcard", line 99), with the value produced by a
   live `SystemTime::now()` at line 72. This is the *second* content hash in
   `../model` fed by a live clock read; `learnings/clock.md` names only
   `rec_now()`. Record the second one in the module doc comment — a learning is
   not edited here, but the crate can say what it found.
3. **`../model/src/daemon/leases.rs:85-93`** — `Lease { expires_at: i64,
   last_access: i64 }`, `#[derive(Serialize, Deserialize)]`, and its clock is
   `unix_now()` at line 364.

## The decision this spec makes: transparent, not split

**`Instant` serializes as its `i64` nanoseconds, in human-readable and binary
formats alike.** No `is_human_readable()` branch.

**Rejected: a decimal string in human-readable formats** (the shape p2 chose for
`ContentId`). It is the defensible option — nanoseconds since the epoch is
~1.8e18, past JSON's 2^53 exact-integer range, so a JavaScript client reading
the number loses precision. It is rejected because:

- Every field `Instant` replaces is **already** a JSON number today
  (`created`, `until`, `heat_at`, `expires_at`, `timestamp`). A string form
  would be a wire change for `../model`'s existing JSON, which is the opposite
  of what this type is for.
- The p2 precedent does not transfer. `[u8; 32]` in JSON is an array of 32
  integers and `../model`'s MCP surface already hands ids out as hex by hand,
  so hex made an existing convention typed. There is no comparable existing
  convention for timestamps, and a string of the same digits is not more
  readable — `Instant`'s readability problem is that it is not a date, and
  spec01 refused a date formatter with a reason.

The 2^53 hazard is real and belongs to the JSON consumer: state it in the doc
comment and name the escape hatch — a surface that must hand a timestamp to a
browser converts at the boundary with `as_unix_millis()` (which is exact to
2^53 until the year 285,000), rather than the whole family changing
representation for one surface.

## Files

- `conserved/src/clock.rs` — hand-written `Serialize`/`Deserialize` for
  `Instant` under `#[cfg(feature = "serde")]`, in the module that owns the type.
  No `#[derive(Serialize)]`, no `serde/derive` feature.
- `conserved/tests/clock_serde.rs` — **new**. Whole file behind
  `#![cfg(feature = "serde")]`.

**`conserved/Cargo.toml` is not touched. `conserved/src/lib.rs` is not touched.**
Nor is `../mitosys`, `../model`, `../realm`, `learnings/`, or any other
ticket's directory.

## Acceptance

Every test function lives in `conserved/tests/clock_serde.rs` and is named with a
`clock_` prefix, so `cargo test -p conserved clock --features serde` reaches it.

- [x] `git diff --stat conserved/Cargo.toml` shows no change from this spec, and
      `cargo tree -p conserved --edges normal --depth 1` with no features is
      unchanged (`blake3` only, no `serde`).
- [x] p2's amended manifest gate test still passes unmodified. If this spec had
      to edit it, the ordering precondition above was violated.
- [x] `cargo build -p conserved` with **no** features compiles, and `Instant`
      has no `Serialize`/`Deserialize` impl in that build (the `#[cfg]` is on the
      impls, not on a stub).
- [x] `clock_serde_binary_encoding_is_identical_to_i64` — for
      `Instant::EPOCH`, `Instant::from_unix_secs(1_787_270_400)`,
      `Instant::from_unix_nanos(-1_500_000_000)`, `Instant::MIN` and
      `Instant::MAX`: `postcard::to_stdvec(&i)? == postcard::to_stdvec(&i.as_unix_nanos())?`.
      This is the test that stops `../model`'s redb rows and its blake3 commit
      preimage from moving under it; the comment says so.
- [x] `clock_serde_struct_substitution_is_wire_compatible` — postcard-encodes
      `struct A { created: i64, until: i64, heat: f32 }` and
      `struct B { created: Instant, until: Instant, heat: f32 }` holding the same
      values, and asserts the two byte strings are **equal**. This is the exact
      substitution p5 will perform on `StoredRecord`
      (`../model/src/record/log.rs:268`), proven here rather than discovered
      there.
- [x] `clock_serde_postcard_round_trips` —
      `postcard::from_bytes::<Instant>(&postcard::to_stdvec(&i)?)? == i` for all
      five values above.
- [x] `clock_serde_json_is_a_number_not_a_string` —
      `serde_json::to_string(&Instant::from_unix_secs(1_787_270_400))` is exactly
      `"1787270400000000000"` with **no** surrounding quotes, and
      `serde_json::to_string(&Instant::from_unix_nanos(-1))` is `"-1"`.
- [x] `clock_serde_json_round_trips_including_negative_and_extremes` —
      `serde_json::from_str::<Instant>` round-trips all five values, `Instant::MIN`
      and `Instant::MAX` included (a `u64`-shaped deserializer fails here).
- [x] `clock_serde_json_rejects_non_integers` —
      `serde_json::from_str::<Instant>` is `Err` for `"\"1787270400\""` (a
      string), `"1.5"`, `"null"`, `"[1]"`, and `"9223372036854775808"` (i64::MAX
      + 1). One representation, and only that one.
- [x] `clock_serde_deserialize_uses_from_unix_nanos` — deserializing `0` yields
      `Instant::EPOCH` and deserializing `1_000_000_000` yields
      `Instant::from_unix_secs(1)`, i.e. the wire integer is **nanoseconds** and
      the deserializer did not silently reinterpret it as seconds. The unit is
      pinned on the wire, not only in the type.
- [x] `FixedClock` and `SystemClock` gain **no** serde impls. A clock is not
      data; only the instant it produced is. The doc comment states the refusal.
- [x] The impls' doc comment states in one sentence: the encoding is the i64
      nanoseconds, identical in every format, deliberately transparent so a
      field that was `i64` can become `Instant` without a wire change — plus the
      2^53 JSON note and the `as_unix_millis()` escape hatch.
- [x] A note in the same doc comment records the one place transparency does
      **not** hold: `../model`'s `Commit.timestamp` is `u64`, and postcard
      varint-encodes `u64` and `i64` differently (zigzag), so substituting
      `Instant` there changes the commit's blake3 hash. That is p5's decision to
      make deliberately; the crate's job is to have said it out loud first.
- [x] `cargo test -p conserved` (no features) passes and
      `cargo test -p conserved --features serde` passes, both with 0 failures.
- [x] `cargo clippy -p conserved --all-targets --features serde -- -D warnings`
      is clean, and so is the same command without `--features serde`.
- [x] spec02's `clock_system_clock_is_the_only_reader` and p2's
      `blake3_is_reachable_only_through_content_id` both still pass: the serde
      impls live in `clock.rs`, not a new module.
- [x] `cargo fmt --all --check` passes.

verify: `cargo fmt --all --check && cargo test -p conserved && cargo test -p conserved --features serde && cargo test -p conserved --features serde clock && cargo test -p conserved --features serde clock 2>&1 | grep -qE "[1-9][0-9]* passed" && cargo clippy -p conserved --all-targets --features serde -- -D warnings && cargo tree -p conserved --edges normal --depth 1`

## Notes from the implementation

Every box above was run; the output is quoted in the implementer's report.

The ordering precondition held: p2's spec03 had landed, so the optional
`serde` dependency, `[features] default = []` and the `serde_json` /
`postcard` dev-dependencies were already in place. This spec added nothing —
`git diff --stat conserved/Cargo.toml` is empty and `conserved/src/lib.rs` was
not touched by the spec03 commit. p2's `blake3_is_the_only_dependency` passes
unmodified (`18 passed` in `tests/content_id.rs`).

On "`Instant` has no `Serialize`/`Deserialize` impl in the no-feature build":
the `#[cfg(feature = "serde")]` sits on the two `impl` items themselves — the
only two occurrences of that attribute in `clock.rs` — and with the feature
off `cargo tree -p conserved --edges normal --depth 1` shows serde is not
linked at all, so the impls cannot exist to be named. `cargo build -p
conserved` with no features compiles, and `cargo test -p conserved` with no
features runs `tests/clock_serde.rs` as `0 tests` because the file carries
`#![cfg(feature = "serde")]`.

`clock_serde_is_not_implemented_for_the_clocks` is an eighth test the spec did
not name. The "no serde for the clocks" box is a *compile-time* refusal, which
a passing test cannot demonstrate by construction, so it is asserted the only
way it can be: by reading `clock.rs` and failing if an impl for `SystemClock`
or `FixedClock` ever appears.
