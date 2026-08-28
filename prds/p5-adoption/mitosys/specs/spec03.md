---
complexity: 70
footprint:
  - ../mitosys/Cargo.toml
  - ../mitosys/src/mitosys/engine/util/util.rs
  - ../mitosys/src/mitosys/engine/base/base_types.rs
  - ../mitosys/src/mitosys/engine/identity/lib.rs
  - ../mitosys/src/mitosys/engine/record/oracle.rs
  - ../mitosys/src/mitosys/engine/ingest/
  - ../mitosys/src/mitosys/engine/graph/hnsw.rs
  - ../mitosys/src/mitosys/engine/tick_loop/tick_tasks.rs
  - ../mitosys/src/mitosys/engine/commands/commands_ingest_cmd.rs
  - ../mitosys/src/mitosys/engine/commands/tag.rs
  - ../mitosys/src/mitosys/engine/rpc/server.rs
  - ../mitosys/src/mitosys/engine/vectors/lib.rs
  - ../mitosys/src/mitosys/engine/store_core/
---

# spec03 — `content_hash` moves from SHA-256/hex to `conserved::ContentId` (blake3)

Replace `util::content_hash`'s SHA-256-into-hex-`String` body with
`conserved::ContentId::of(...).to_string()` (or thread `ContentId` itself
through where the caller can take it), behind `store_core`'s `FORMAT_VERSION`
bump — a wipe-and-re-derive break the user already accepted 2026-08-21
(parent PRD `## Answers` §3, restated in this PRD's own `## Decided`), not a
migration and not to be re-escalated. Move the `ed25519:` prefix tolerance
that currently lives inside `util::hex::decode` (`util/util.rs:44`) to the
key-string call sites that actually need it — `ContentId::from_str` refuses
the prefix by design (`p2-content-id`'s requirement 2), so `util::hex` keeps
serving its non-id callers and the id-rendering call sites move to
`ContentId`'s own parsing.

**Call-site count, measured today, not the PRD's "26."** Filtering
`content_hash(` in non-test `.rs` files under `src/` down to actual
invocations of the free function (excluding the `base_types.rs:368` struct
*method* of the same name, and excluding comments) gives **20** real call
sites, listed below by file — not 26. This PRD's own text names 6 of them
(`ingest_worker.rs`, `ingest_intake.rs`, `ingest_direct.rs`,
`ingest_file_watcher.rs`, `engine/identity/lib.rs:48`,
`engine/record/oracle.rs:113`); the remaining 14 are additional sites this
session found and this spec must also cover:

```
engine/record/oracle.rs:113        engine/ingest/ingest_place.rs:411
engine/identity/lib.rs:48          engine/ingest/ingest_file_watcher.rs:189
engine/ingest/ingest_intake.rs:268 engine/ingest/ingest_direct.rs:64
engine/ingest/ingest_worker.rs:207,255,597
engine/graph/hnsw.rs:529           engine/vectors/lib.rs:59
engine/tick_loop/tick_tasks.rs:173 engine/commands/commands_ingest_cmd.rs:82
engine/commands/tag.rs:120         engine/rpc/server.rs:554,1444
engine/base/base_types.rs:430,754,759,770
```

Do not implement against this fixed list either — re-run the `grep` in
Verify and Proof and treat that as the authority, since new call sites can
land before this spec runs. `engine/identity/lib.rs:48` truncates the result
to 16 hex chars for a short id and must keep doing so over `ContentId`'s
rendering; `engine/record/oracle.rs:113` keys the oracle and both id shape
*and* stability across the format bump matter there.

## Acceptance

- [ ] `util::content_hash` calls `conserved::ContentId` internally; `sha2`
      is no longer reachable from that function's body (it may still be a
      dependency of other crates for unrelated hashing — this spec does not
      assert `sha2` leaves the workspace, only that `content_hash`'s own
      algorithm is blake3).
- [ ] `store_core`'s `FORMAT_VERSION` (currently `15`,
      `engine/store_core/lib.rs:101`) is bumped by exactly one, with a
      comment naming the SHA-256→blake3 id break as the reason — matching
      the existing convention at that constant's other bump comments.
- [ ] Every one of the 20 measured call sites (re-verified by grep, not
      copied from this file) compiles and produces a `ContentId`-shaped
      identity; none is left calling a stale SHA-256 helper.
- [ ] `ContentId::from_str`'s `ed25519:`-prefix refusal (p2's decision) is
      not worked around anywhere in the id path; the prefix-tolerant
      `util::hex::decode` keeps serving only its existing non-id (key
      string) callers, unchanged.
- [ ] `cargo test --workspace` passes inside the offline container
      (spec01's mechanism), including any test asserting
      `id == content_hash(text)` (`engine/graph/merge.rs:96`'s invariant) —
      a break here is this spec's, not a pre-existing failure to shrug off.

## Verify and Proof

```sh
cd ../mitosys
grep -rn 'content_hash(' src --include='*.rs' | grep -v '/tests/' | grep -v 'base_types.rs:368'
cargo build --workspace
cargo test --workspace
just check
```


## Addresses corrected 2026-08-28 by the board, measured at `276a400`

`p8d-floor-split` moved the floor into the engine. **`src/mitosys/util/util.rs`
no longer exists** — it is `src/mitosys/engine/util/util.rs`, package
`mitosys-engine-util`. The footprint is rewritten. Line numbers largely
survived the move; these did not:

| this spec says | measured |
|---|---|
| `util/util.rs:44` (the `ed25519:` prefix) | `src/mitosys/engine/util/util.rs:44` — content `reproduced` |
| `FORMAT_VERSION` at `engine/store_core/lib.rs:101` | value `15` correct, line is **106** |
| **20** `content_hash` call sites | **21** |
| `ingest_place.rs:411` | does not exist — there are **two** sites in that file, `:381` and `:429` |
| `ingest_worker.rs:207,255,597` | **206, 254, 599** |

Every other listed site still holds: `oracle:113`, `identity:48`,
`intake:268`, `direct:64`, `file_watcher:189`, `hnsw:529`, `vectors:59`,
`tick_tasks:173`, `commands_ingest_cmd:82`, `tag:120`, `server:554` and
`:1444`, `base_types:430`, `:754`, `:759`, `:770`. The
`grep -v 'base_types.rs:368'` filter is still correct.

Callers now spell it `mitosys_engine_util::content_hash`, or `util::content_hash`
through the alias.
