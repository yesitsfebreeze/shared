---
state: open
mode: afk
priority: 45
est: 18h
repo: model
verify: "cd ../model && cargo test --workspace"
---

# P5e — llm: the content-hash preimage first, it is the dangerous one

Purpose: `../model` gains `Scope`, routes its wall-clock reads through `Clock`
starting with the content-hash preimage, and calls `min_median_max`.

## Requirements

- [ ] **A unit mismatch p2/p3's specs did not catch.** `rec_now()`
      (`src/record/mod.rs:239`) returns unix **seconds**; `Record.created: i64
      // unix seconds`; `rec_preimage` ends
      `out.extend_from_slice(&r.created.to_le_bytes())`. `conserved::Instant` is
      unix **nanoseconds**. p3's `clock_serde_struct_substitution_is_wire_compatible`
      proves the *bytes* are identical — it cannot prove the *meaning* is, and
      here it is not. Substituting rewrites every `rec_id` in the store. See the
      parent's `## Questions`.
- [ ] **The second live-clock-into-a-content-hash**, which `learnings/clock.md`
      does not name: `src/node/transactional.rs:72` —
      `SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()` into
      `timestamp: u64`, and `Commit::content_hash()` postcard-encodes the struct
      with `signature: None` and blake3s it. Same defect class as `rec_now()`, in
      a struct whose hash is a **network-visible** commit id
      (`parent_heads: Vec<[u8; 32]>`). Note the `.unwrap()` — a third spelling of
      the pre-epoch hazard. `timestamp` is `u64`; postcard zigzags `i64`
      differently, so `Instant` there moves bytes as well as units.
      `parent_heads` and `result_id` are `ContentId` substitution sites.
- [ ] **The monotonic trap is concentrated here.** 65 non-test `Instant::now()`,
      13 files importing `tokio::time::Instant`: `grade/probe.rs` (11),
      `grade/inproc.rs` (7), `loop/harness.rs` (5), `grade/measure.rs` (5),
      `node/refine.rs` (4). **None** of these may become `conserved::Instant`.
- [ ] **`aggregate`** (`src/grade/measure.rs:203`) sorts `u64` in place then
      indexes `len()/2`. `min_median_max` takes an already-sorted `&[f64]`, so
      adoption is sort → cast → call → cast back; the `u64`→`f64` round trip is
      exact for ms durations and must be **recorded, not assumed**.
- [ ] **`impl Default for Record`** (`src/record/mod.rs:180`) sets
      `id: [0u8; 32]`. `ContentId` has no `Default` by p2's deliberate refusal —
      closing that hole is this node's job.
- [ ] **DOGMA-13-by-hand**: the prose site is `src/main.rs:126`; the real
      inverse-of-a-checkpoint work is around `Mode::ResetNode` / `boot_live`.
- [ ] **Record** that an edition-2024 tree (`../model/Cargo.toml`, no pin)
      consumes an edition-2021 crate.

## Held — 2026-08-21

**Not dispatchable from this board yet.** The user's instruction is to finish the
shared repo's own tools first and reconcile the consumer implementations later,
once everything here is tested and works. This node is fully specified and ready;
do not start it, and do not write into the consumer trees, until that hold lifts.

## Decided — persisted-id break accepted

The user accepted this break explicitly on 2026-08-21, one version bump: `Record.created` seconds -> `Instant` nanoseconds (rewrites every `rec_id`), plus `Commit.timestamp` u64 -> i64.
Wipe and re-derive, not a migration. It does not need re-escalating when this
node runs.
