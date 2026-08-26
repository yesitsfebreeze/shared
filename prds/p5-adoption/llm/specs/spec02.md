---
complexity: 15
footprint:
  - ../model/Cargo.toml
  - ../model/src/node/transactional.rs
---

# spec02 — the second live-clock-into-a-content-hash: `Commit::new`

`Commit::new` (`../model/src/node/transactional.rs:66`) reads
`SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()` into
`timestamp: u64`, and `Commit::content_hash()` (same file, lines 104-109)
blake3-hashes the postcard encoding of the whole commit with `signature` set
to `None` — `timestamp` included. That hash is `parent_heads`' own id and a
**network-visible** commit id (`Commit::sign`/`verify_signature` sign and
verify it too). Same defect class as spec01's `rec_now()`, in the more
expensive place, per `learnings/clock.md`. This is the accepted
`Commit.timestamp` u64 -> i64 persisted-id break from the PRD's `##
Decided` section — wipe and re-derive, not a migration.

`Commit` is referenced opaquely (never `.timestamp`) from
`node/federation.rs`, `utils/transport/codec.rs`, and
`utils/ident/delegate.rs`; this spec's footprint is contained to
`transactional.rs` itself, and those three files only need to keep
compiling, not change.

## Acceptance

- [ ] `Commit::new` reads the wall clock through `conserved::SystemClock`
      (adding the `conserved` dependency to `model/Cargo.toml` if spec01
      has not already landed it — the same git-pinned dependency, added
      idempotently) instead of calling `SystemTime::now()` directly; the
      bare `.unwrap()` on `duration_since(UNIX_EPOCH)` — the third spelling
      of the pre-epoch hazard the PRD names — is gone, because
      `conserved::Instant::from_system_time` saturates instead of
      panicking.
- [ ] `Commit.timestamp` is `i64`, not `u64`. A test pins that
      `content_hash()`'s postcard bytes actually change for the same
      logical instant versus the old `u64` encoding — postcard varint-codes
      a `u64` and zigzag-codes an `i64` differently, so this is a wire
      format change, not only a type change, and the PRD says to record
      that, not assume it.
- [ ] The field's doc comment ("When the commit was created (unix epoch
      milliseconds)") is corrected to state the type's actual unit now
      that it is `Instant`-backed. The old comment never matched the code
      (`.as_secs()`, not milliseconds) — `learnings/clock.md` names this
      exact drift between the comment and the code as the argument for
      pinning the unit by type rather than by doc comment; this spec is
      where that drift closes.
- [ ] `node/federation.rs`, `utils/transport/codec.rs`,
      `utils/ident/delegate.rs` compile unchanged — none of them read
      `.timestamp`, so this is a build-only check on those three files, not
      a rewrite.
- [ ] The monotonic-clock inventory (`grade/probe.rs`, `grade/inproc.rs`,
      `loop/harness.rs`, `grade/measure.rs`, `node/refine.rs`, and the rest
      of the 65 counted in `learnings/clock.md`) is untouched: none of them
      may become `conserved::Instant` or `conserved::Clock`.

## Verify and Proof

```sh
cd ../model && cargo build -p llm \
  && cargo test -p llm --lib node::transactional:: \
  && ! rg "conserved::(Instant|Clock)" src/grade src/loop src/node/refine.rs
```
