---
state: specced
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

- [ ] **The type** — `ContentId([u8; 32])`, `of(&[u8])`, `as_bytes()`,
      `Display` as exactly 64 lowercase hex, `FromStr` rejecting everything
      else (wrong length, uppercase, prefixes, non-hex). Property tests on
      the round-trip.
- [ ] **hex moves in, private** — encode/decode behind `Display`/`FromStr`,
      not a public utility two callers spell differently. mitosys's tolerance
      of an `ed25519:` prefix on decode does **not** port: prefix stripping is
      the caller's business at the call site, and the adoption node (p5)
      carries the mitosys-side shim. Record this refusal in the doc comment.
- [ ] **The one dependency** — `blake3` enters the crate here and is reachable
      only through this module. If mitosys's `dependency_tree.rs` gate objects
      to `Scope` consumers pulling blake3 transitively, split at the obvious
      line (`conserved` / `conserved-id`) **then** — the gate decides, not
      this node. Do not pre-split.

## Acceptance

Round-trip property tests pass; a test vector fixed in the learning's terms
(known input → known 64-hex digest) pins the algorithm so a silent hash swap
fails a named test.
