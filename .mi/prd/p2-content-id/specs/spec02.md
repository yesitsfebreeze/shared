# goal

The property tests the ticket's acceptance names: for arbitrary input, a
`ContentId` renders as exactly 64 lowercase hex and parses back to itself, and
anything that is not that canonical spelling is rejected.

est: 1.0h

## What this assumes

- spec01 has landed: `ContentId`, `from_bytes`, `Display`, `FromStr`,
  `ContentIdParseError`, all public from `conserved`.
- p0's test root: `conserved/tests/`, not `src/**/tests/`.
- `proptest` is the family's property-test crate — `../model/Cargo.toml:80`
  already declares `proptest = "1.11"`. Match it rather than introducing a
  second generator into the family.

## Files

- `conserved/Cargo.toml` — add `[dev-dependencies] proptest = "1.11"`.
- `conserved/tests/content_id_props.rs` — **new**. Nothing else.

## Acceptance

- [ ] `proptest` is declared under `[dev-dependencies]`, never
      `[dependencies]`, and `cargo tree -p conserved --edges normal` is
      unchanged by this spec (still exactly one edge out of `conserved`:
      `blake3`).
- [ ] `blake3_is_the_only_dependency` (spec01's manifest gate) still passes
      unmodified — it asserts the `[dependencies]` table, and a dev-dependency
      must not make it fail. If it does, the gate was reading the wrong
      table; fix the gate, not the manifest.
- [ ] `display_is_always_64_lowercase_hex` — for any `Vec<u8>` up to 4 KiB,
      `ContentId::of(&input).to_string()` has `len() == 64` and every byte is
      in `0-9a-f`.
- [ ] `display_parse_round_trip` — for any `Vec<u8>`,
      `ContentId::of(&x).to_string().parse::<ContentId>().unwrap()`
      equals `ContentId::of(&x)`.
- [ ] `parse_display_round_trip` — for any `[u8; 32]`, rendering
      `ContentId::from_bytes(b)` and parsing it back yields a value whose
      `as_bytes()` is `&b`; and re-rendering the parsed value produces the
      *identical* string (canonical form is a fixed point, so no id has two
      spellings).
- [ ] `of_is_deterministic` — for any `Vec<u8>`, two calls to
      `ContentId::of` agree, and `ContentId::of(&x) == ContentId::of(&x.clone())`.
- [ ] `distinct_inputs_distinct_ids` — for any two `Vec<u8>` with
      `prop_assume!(a != b)`, the ids differ. (A collision here is a hash
      failure, not a flake; the test says so in a comment.)
- [ ] `mutated_hex_is_rejected` — start from a valid rendering and apply one
      mutation drawn by proptest: uppercase one hex digit, drop one char,
      append one char, replace one char with a non-hex char, prepend
      `ed25519:`, prepend `0x`, append `\n`. Every mutant parses to `Err`.
      The `ed25519:` case is drawn like every other mutation — it is not a
      special case in this crate, which is the point.
- [ ] `arbitrary_strings_never_parse_unless_canonical` — for any `String`,
      parsing succeeds **iff** the string is 64 chars long and every char is
      in `0-9a-f`; when it succeeds, re-rendering returns the same string.
- [ ] Every property runs at proptest's default case count (no
      `#![proptest_config(...)]` shrinking the coverage to look fast).
- [ ] `cargo clippy -p conserved --all-targets -- -D warnings` is clean.

verify: `cargo test -p conserved --test content_id_props && cargo tree -p conserved --edges normal`
