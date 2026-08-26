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

- [x] `Commit::new` reads the wall clock through `conserved::SystemClock`
      (adding the `conserved` dependency to `model/Cargo.toml` if spec01
      has not already landed it — the same git-pinned dependency, added
      idempotently) instead of calling `SystemTime::now()` directly; the
      bare `.unwrap()` on `duration_since(UNIX_EPOCH)` — the third spelling
      of the pre-epoch hazard the PRD names — is gone, because
      `conserved::Instant::from_system_time` saturates instead of
      panicking.
- [x] `Commit.timestamp` is `i64`, not `u64`. A test pins that
      `content_hash()`'s postcard bytes actually change for the same
      logical instant versus the old `u64` encoding — postcard varint-codes
      a `u64` and zigzag-codes an `i64` differently, so this is a wire
      format change, not only a type change, and the PRD says to record
      that, not assume it.
- [x] The field's doc comment ("When the commit was created (unix epoch
      milliseconds)") is corrected to state the type's actual unit now
      that it is `Instant`-backed. The old comment never matched the code
      (`.as_secs()`, not milliseconds) — `learnings/clock.md` names this
      exact drift between the comment and the code as the argument for
      pinning the unit by type rather than by doc comment; this spec is
      where that drift closes.
- [x] `node/federation.rs`, `utils/transport/codec.rs`,
      `utils/ident/delegate.rs` compile unchanged — none of them read
      `.timestamp`, so this is a build-only check on those three files, not
      a rewrite.
- [x] The monotonic-clock inventory (`grade/probe.rs`, `grade/inproc.rs`,
      `loop/harness.rs`, `grade/measure.rs`, `node/refine.rs`, and the rest
      of the 65 counted in `learnings/clock.md`) is untouched: none of them
      may become `conserved::Instant` or `conserved::Clock`.

## Verify and Proof

```sh
cd ../model && cargo build -p llm \
  && cargo test -p llm --lib node::transactional:: \
  && ! rg "conserved::(Instant|Clock)" src/grade src/loop src/node/refine.rs
```

## Evidence — implemented 2026-08-26

**Box 1.** `Commit::new` is `let now = SystemClock.now().as_unix_secs();`. The
bare `.unwrap()` on `duration_since(UNIX_EPOCH)` is gone from this function —
`Instant::from_system_time` saturates and carries the sign, so a machine whose
clock is wrong no longer aborts the process here. The `conserved` dependency
was already landed by spec01; nothing was added twice.

`SystemTime` stays imported in this file: `tmp_registry_path` (a `#[cfg(test)]`
helper making a unique temp path from `.as_nanos()`) still uses it. That is a
uniqueness source, not a wall-clock read into a hash, and is out of this spec's
contract.

**Box 2 — measured, not assumed.**
`the_timestamp_type_change_moves_the_commit_id_on_the_wire` serializes the real
`Commit` beside an `OldCommit` struct holding the pre-spec layout with
`timestamp: u64`, at the SAME logical second (`1_700_000_000`), and asserts
both the postcard bytes and the blake3 of them differ. So the break is
demonstrated at the byte level rather than argued from postcard's
documentation. The test also asserts `content_hash()` is still deterministic
for one commit, so it cannot pass by everything simply hashing differently
every time.

`a_commit_timestamp_can_say_before_the_epoch` runs the reason the type is
signed: `timestamp: -1` is now a representable, hashable instant distinct from
`+1`.

**Box 3.** The doc comment said "unix epoch milliseconds" while the code wrote
`.as_secs()`. It now says unix epoch **seconds**, names `SystemClock` as the
reader and `Instant::as_unix_secs` as the unit, and says why the type is
signed. `commit_new_stamps_unix_seconds_from_the_one_clock_reader` checks the
stamp against a live `SystemClock` read within 5 s, so the comment and the code
now disagree loudly rather than silently.

**Reading recorded — the unit did not move.** Boxes 2 and 3 read together as
"`i64`, `Instant`-backed". The field is `i64` **seconds**, sourced from
`Instant::as_unix_secs()` — not nanoseconds. The PRD's `## Decided` names one
accepted break for this field, `u64 -> i64`; changing the unit as well would be
a second break the user did not accept, and box 2's own test (varint versus
zigzag *at the same logical second*) only means anything if the number stays
the same. Recorded here rather than assumed silently.

**Box 4.** `git diff --name-only` names none of `src/node/federation.rs`,
`src/utils/transport/codec.rs`, `src/utils/ident/delegate.rs`; `grep -c
'\.timestamp'` returns 0 in all three. They compile as they stand — build-only,
no rewrite.

**Box 5.** `grep -rn 'conserved::(Instant|Clock)' src/grade src/loop
src/node/refine.rs` → no matches. None of the 65 monotonic sites moved.

## Verify — run 2026-08-26

```
$ cargo build -p llm
    Finished `dev` profile [unoptimized + debuginfo] target(s)

$ cargo test -p llm --lib node::transactional::
running 11 tests
test node::transactional::tests::commit_new_stamps_unix_seconds_from_the_one_clock_reader ... ok
test node::transactional::tests::a_commit_timestamp_can_say_before_the_epoch ... ok
test node::transactional::tests::the_timestamp_type_change_moves_the_commit_id_on_the_wire ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1560 filtered out; finished in 0.19s

$ grep -rn 'conserved::(Instant|Clock)' src/grade src/loop src/node/refine.rs
no matches
```
