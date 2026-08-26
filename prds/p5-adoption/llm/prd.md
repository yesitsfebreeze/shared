---
state: done
mode: afk
priority: 45
est: 18h
repo: model
verify: "cd ../model && cargo test --workspace"
complexity: 62
actual: 1h40m
commit: de74214
commit-model: 5397c508
blast-radius: high
---

# P5e — llm: the content-hash preimage first, it is the dangerous one

Purpose: `../model` gains `Scope`, routes its wall-clock reads through `Clock`
starting with the content-hash preimage, and calls `min_median_max`.

## Requirements

- [x] **A unit mismatch p2/p3's specs did not catch.** `rec_now()`
      (`src/record/mod.rs:239`) returns unix **seconds**; `Record.created: i64
      // unix seconds`; `rec_preimage` ends
      `out.extend_from_slice(&r.created.to_le_bytes())`. `conserved::Instant` is
      unix **nanoseconds**. p3's `clock_serde_struct_substitution_is_wire_compatible`
      proves the *bytes* are identical — it cannot prove the *meaning* is, and
      here it is not. Substituting rewrites every `rec_id` in the store. See the
      parent's `## Questions`.
- [x] **The second live-clock-into-a-content-hash**, which `learnings/clock.md`
      does not name: `src/node/transactional.rs:72` —
      `SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()` into
      `timestamp: u64`, and `Commit::content_hash()` postcard-encodes the struct
      with `signature: None` and blake3s it. Same defect class as `rec_now()`, in
      a struct whose hash is a **network-visible** commit id
      (`parent_heads: Vec<[u8; 32]>`). Note the `.unwrap()` — a third spelling of
      the pre-epoch hazard. `timestamp` is `u64`; postcard zigzags `i64`
      differently, so `Instant` there moves bytes as well as units.
      `parent_heads` and `result_id` are `ContentId` substitution sites.
- [x] **The monotonic trap is concentrated here.** 65 non-test `Instant::now()`,
      13 files importing `tokio::time::Instant`: `grade/probe.rs` (11),
      `grade/inproc.rs` (7), `loop/harness.rs` (5), `grade/measure.rs` (5),
      `node/refine.rs` (4). **None** of these may become `conserved::Instant`.
- [x] **`aggregate`** (`src/grade/measure.rs:203`) sorts `u64` in place then
      indexes `len()/2`. `min_median_max` takes an already-sorted `&[f64]`, so
      adoption is sort → cast → call → cast back; the `u64`→`f64` round trip is
      exact for ms durations and must be **recorded, not assumed**.
- [x] **`impl Default for Record`** (`src/record/mod.rs:180`) sets
      `id: [0u8; 32]`. `ContentId` has no `Default` by p2's deliberate refusal —
      closing that hole is this node's job.
- [x] **DOGMA-13-by-hand**: the prose site is `src/main.rs:126`; the real
      inverse-of-a-checkpoint work is around `Mode::ResetNode` / `boot_live`.
- [x] **Record** that an edition-2024 tree (`../model/Cargo.toml`, no pin)
      consumes an edition-2021 crate.

## Held — 2026-08-21 — LIFTED 2026-08-26

**Not dispatchable from this board yet.** The user's instruction is to finish the
shared repo's own tools first and reconcile the consumer implementations later,
once everything here is tested and works. This node is fully specified and ready;
do not start it, and do not write into the consumer trees, until that hold lifts.

**The hold lifted on 2026-08-26** (user decision, recorded in full at the
parent's `## Answers — 2026-08-26`). Its condition is met: every other PRD in
this repository — `p0-foundation`, `p1-scope`, `p2-content-id`, `p3-clock`,
`p4-stats`, `p6-scope-unwind`, `load-proof`, `close` — is `state: done`. This
node is dispatchable, and writing into the consumer tree is now in scope.

Before pinning `conserved`, read the parent's Answer 5: the remote recorded in
Answer 1 (`inner-zirkle`) is stale, the live one is
`https://github.com/yesitsfebreeze/shared.git`, and this repository holds
commits that have never been pushed — a `rev` that is not on the remote cannot
be fetched from a container or another machine.

## Decided — persisted-id break accepted

The user accepted this break explicitly on 2026-08-21, one version bump: `Record.created` seconds -> `Instant` nanoseconds (rewrites every `rec_id`), plus `Commit.timestamp` u64 -> i64.
Wipe and re-derive, not a migration. It does not need re-escalating when this
node runs.

## Answers — 2026-08-26

**Q1** — *`spec04` box 1 names four acquisitions for the scope. Should the two
swarm listener binds be registered on it, when Rust's `Drop` already frees
those ports exactly once and no test could tell the two mechanisms apart?*

**Close box 1 on the three filesystem acquisitions.** The scope carries what
`Drop` does not — a directory and two files a failed boot would otherwise
leave behind — and the spec's line is read as naming the acquisitions rather
than mandating a second teardown mechanism per acquisition. The `boot`
restructure is NOT funded, here or as a node of its own; a second mechanism
for a property ownership already guarantees is not work this board wants.

The `## Blocked` section below is kept as the record of what was asked and
why. The block lifted with this answer and the node is `done`.

## Blocked — 2026-08-26, LIFTED the same day by the answer above

Written while one box was open. Every requirement in `## Requirements` was met
and 19 of the 20 acceptance boxes across the four specs were closed against
runs; the twentieth could not be closed from inside this node. It is closed
now, by the user's answer above, and all 20 stand.

| open box | spec | what closes it |
|---|---|---|
| `boot`'s fallible acquisitions register on a `conserved::Scope` — including the **discovery** and **data-plane listener binds** | `spec04` box 1 | a decision from the user, below |

No `needs:` is written: nothing on any board closes this box. It waits on an
answer, not on a PRD, and the frontmatter contract makes `needs:` optional
precisely so a state is not faked with a dependency that does not exist.

Three of the four acquisitions the box names are registered and unwound
(registry file, identity file, plus the data directory the box does not name).
The two swarm listeners are not, for two reasons recorded in full in
`specs/spec04.md`: their teardown is what Rust's `Drop` already does exactly
once on the failure path, so a test could not tell the scope from the drop;
and an `Undo` (`Box<dyn FnOnce() + Send>`) must own the swarm it frees, while
`boot` moves that same swarm into `Routing::new` — with a `.next().await`
between the bind and the hand-off, so the `Arc<Mutex<Option<_>>>` escape does
not compile.

The code is landed and the tree is green; `unblock` re-runs only this box.

## Question for the user

**`spec04` box 1 names four acquisitions for the scope. Should the two swarm
listener binds be registered on it, when Rust's `Drop` already frees those
ports exactly once and no test could tell the two mechanisms apart?**

1. **Close box 1 on the three filesystem acquisitions.** The scope carries what
   `Drop` does not — a directory and two files a failed boot would otherwise
   leave behind — and the spec's line is read as naming the acquisitions
   rather than mandating a second teardown mechanism per acquisition. This
   node goes `done` as it stands. *(recommended)*
2. **Fund the restructure as its own node.** `boot`'s event loop moves to an
   async mutex (or the bind/hand-off is split so the swarm can be owned by an
   undo), and all four acquisitions go on the scope. Real work with its own
   contract, in `../model`, and a redesign of a boot path to satisfy a
   property ownership already guarantees.
3. **Register them, accepting the redundancy, inside this node.** Widen
   `spec04`'s footprint to the boot event loop now. Fastest path to a literal
   `[x]`, and the one that most risks a check written from the answer — the
   test for it would pass with the scope removed.
