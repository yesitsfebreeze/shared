---
state: done
est: 4.5h
mode: afk
priority: 30
verify: "cargo test -p conserved content_id"
---

# P2 — ContentId: one hash, one spelling

Purpose: the type that makes the family able to share a record at all. Spec is
`learnings/content-addressing.md` (the decision: blake3 over SHA-256,
`[u8; 32]` over hex `String`) and `learnings/shared-crate.md` §1. Today
mitosys hashes SHA-256 into a hex `String`, llm hashes blake3 into `[u8; 32]`;
an id computed on one side does not exist on the other. Drift is silent —
admission criterion 4 in its strongest form. Blocked on `p0-foundation`.

## Requirements

- [x] **The type** — `ContentId([u8; 32])`, `of(&[u8])`, `as_bytes()`,
      `Display` as exactly 64 lowercase hex, `FromStr` rejecting everything
      else (wrong length, uppercase, prefixes, non-hex). Property tests on
      the round-trip.
- [x] **hex moves in, private** — encode/decode behind `Display`/`FromStr`,
      not a public utility two callers spell differently. mitosys's tolerance
      of an `ed25519:` prefix on decode does **not** port: prefix stripping is
      the caller's business at the call site, and the adoption node (p5)
      carries the mitosys-side shim. Record this refusal in the doc comment.
- [x] **The one dependency** — `blake3` enters the crate here and is reachable
      only through this module. If mitosys's `dependency_tree.rs` gate objects
      to `Scope` consumers pulling blake3 transitively, split at the obvious
      line (`conserved` / `conserved-id`) **then** — the gate decides, not
      this node. Do not pre-split.

## Acceptance

Round-trip property tests pass; a test vector fixed in the learning's terms
(known input → known 64-hex digest) pins the algorithm so a silent hash swap
fails a named test.

## Deviations

1. **`[dependencies]` inherits from the workspace.** spec01 prints
   `blake3 = "1"` in `conserved/Cargo.toml`; the manifest says
   `blake3 = { workspace = true }`, with `blake3 = "1"` pinned in the root
   `[workspace.dependencies]`. p0 committed this crate to mitosys's dependency
   convention in two places — the root manifest's own comment ("blake3 enters
   in p2, behind ContentId", written directly under `[workspace.dependencies]`)
   and `lib.rs` §"The four divergences, resolved here". Following the spec
   literally would have contradicted the landed convention. The acceptance is
   unweakened: `blake3_is_the_only_dependency` asserts the `[dependencies]`
   table *and* that the workspace pins `blake3 = "1"`, the version
   `../model/Cargo.toml:19` declares. spec03's `serde` line is inherited the
   same way, carrying `optional = true` at the member.

2. **`serde` also appears under `[dev-dependencies]`, with `features =
   ["derive"]`.** The `ContentId` impls are hand-written as spec03 requires;
   the derive is needed only for the two throwaway structs in
   `struct_substitution_is_wire_compatible`. A dev-dependency is not a normal
   edge, so the default `cargo tree -p conserved --edges normal` still shows
   exactly one: blake3.

3. **All three test files are wrapped in `mod content_id { … }`.** The ticket's
   own verify, `cargo test -p conserved content_id`, filters on **test
   names**, not file names or targets. Unwrapped, a `#[test] fn` in
   `tests/content_id.rs` reports `running 0 tests … filtered out` and exits 0
   — a gate that is green having run nothing. The wrapper makes each test
   report as `content_id::<name>`. Same convention p1 applied to
   `tests/scope.rs`.

4. **`blake3_is_reachable_only_through_content_id` strips `//` line comments
   before searching.** `lib.rs`'s crate documentation narrates blake3's
   arrival in prose. Prose *about* the dependency is not a module reaching for
   the hasher, and the gate should not force the crate doc to go quiet about
   its own one dependency.

5. **The two refusals are repeated on the `ContentId` struct doc.** spec01
   puts the module behind `mod content_id;` (private) with a `pub use`, so its
   module doc comment — where the `ed25519:` and `Default` refusals live in
   full — does not render in `cargo doc`. A short version of each is therefore
   also on the type a caller actually reads.
