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

- [x] `util::content_hash` calls `conserved::ContentId` internally; `sha2`
      is no longer reachable from that function's body. The whole body is now
      `conserved::ContentId::of(s.as_bytes()).to_string()` — one line, no
      `Sha256`, no `hex::encode`. Signature unchanged (`&str -> String`), and
      `ContentId`'s `Display` is 64 lowercase hex, so every call site's type
      and shape carry over.
      `sha2` stays declared by `mitosys-engine-util`, for `digest()` — the
      raw-32-byte SHA-256 whose only callers are `store_core`'s
      `canonical_digest`/`map_digest` (`lib.rs:562,569`), a changed-or-not
      marker for the pack and **not an identity**. Its doc comment used to
      claim it was "deliberately the *same* function underneath" as
      `content_hash`; that sentence is now false and has been rewritten rather
      than left standing.
- [x] `store_core`'s `FORMAT_VERSION` is bumped by exactly one, 15 -> 16, at
      `engine/store_core/lib.rs:106` (the board's corrected line), with a
      trailing `// v16: content_hash moved SHA-256 -> blake3
      (conserved::ContentId), so every persisted doc/entity id, dedup key and
      oracle key rehashes — a wipe-and-re-derive break the user accepted
      2026-08-21, not a migration (…spec03)` in the same one-line-per-version
      shape as v15, v14, v13 and v11 beneath it.
- [x] Every one of the measured call sites (re-verified by grep, not copied
      from this file) compiles and produces a `ContentId`-shaped identity;
      none is left calling a stale SHA-256 helper. The board's corrected count
      of **21 was reproduced exactly** before the pin — including
      `ingest_place.rs` at `:381` and `:429` and `ingest_worker.rs` at
      `206, 254, 599` — and **20 of them moved to blake3**. None needed
      editing: the function's name, signature and rendered shape are
      unchanged, so the algorithm moved underneath them.
      **The twenty-first is `engine/vectors/lib.rs`, and it deliberately did
      not move** (see "The address stays SHA-256" below). It is the one site
      of the 21 that was never an identity, and after the pin the grep returns
      20 call sites plus the definition. Every remaining one is a doc id,
      entity id, dedup key, cache key or oracle key — exactly the population
      the parent PRD scoped.
      `engine/identity/lib.rs:48`'s 16-char truncation still holds — blake3
      renders 64 lowercase hex the same as SHA-256 did — and it is a
      process-lifetime build id, never persisted.
- [x] `ContentId::from_str`'s `ed25519:`-prefix refusal (p2's decision) is
      not worked around anywhere in the id path; the prefix-tolerant
      `util::hex::decode` keeps serving only its existing non-id (key string)
      callers, unchanged — **and that set is measured as empty.**
      `grep -rn 'hex::decode\|hex::encode' src --include='*.rs' | grep -v
      '/tests/'` returned exactly one line before this change,
      `util/util.rs:15` inside `content_hash` itself, and returns none after
      it. So `hex::decode` has never had a non-test caller in this tree and
      `hex::encode` has just lost its only one; the box holds trivially,
      because there is nothing left to work around with. Nothing calls
      `ContentId::from_str` at all — mitosys holds ids as rendered `String`s
      everywhere, so no id in this tree is parsed back.
      `util::hex` is left byte-identical rather than deleted (this node's
      deletion spec named one function and this is not it), with a doc block
      added above it saying it is not the id path and must not become one.
      **Reported, not chased:** a `pub mod` with zero callers is dead weight
      the next reader will have to re-derive.
- [x] `cargo test --workspace` passes inside the offline container
      (spec01's mechanism). Measured twice, because the store break was
      re-scoped mid-node:
      **before the pin** — `docker compose exec dev cargo test --workspace
      --offline` under `CARGO_NET_OFFLINE=true`: 2138 passed; 0 failed;
      21 ignored, exit 0.
      **after the pin** — host `just check` 2140 passed; 0 failed; 21 ignored,
      exit 0, and container `just check` (the PRD's own `verify:`, under
      `CARGO_NET_OFFLINE=true`) **2139 passed; 0 failed; 21 ignored, exit 0**.
      `merge.rs:96`'s `id == content_hash(text)` invariant holds untouched —
      both sides call the same function, so moving the algorithm moved both.

      **Three things did break, all this spec's, all owned rather than
      shrugged off.** The first `cargo test --workspace --no-fail-fast` after
      the swap was 2133/6/21, in three targets:

      1. `engine/base`'s two frozen-digest tests
         (`source_id_is_byte_identical_for_every_scheme_that_carries_no_position`,
         `a_declared_position_is_a_fourth_hashed_component`) — seven hardcoded
         SHA-256 `source_id()`s. **Re-based, not recomputed from the failure
         output**: each of the seven was re-derived by hand-typing the key
         string `source_id()` documents (`scheme \0 object_id \0 section`,
         plus `\0 position` when non-zero) and hashing it with blake3 outside
         this tree; two of the seven matched the assertion's `left` value,
         which is what makes the other five trustworthy. The test's comment
         now says the baseline moved once, on 2026-08-28, for the algorithm
         and not for the layout — which is still what it guards.
      2. `engine/commands`'s **determinism conformance**, all four tests, and
         this one is the finding rather than the chore. It did not fail a
         byte-compare: `fold::replay` **refused the fixture** —
         *"vector c2b2950f… named at seq 3 is corrupt"*. `vectors::address`
         is `content_hash` over the f32 bit patterns, so the five committed
         blobs were named by a rule that no longer exists, and
         `Vectors::get`'s re-derive-on-read answered `Corrupt`. Fixed by the
         sanctioned deliberate act: `record_the_determinism_fixture`
         (`#[ignore]`d recorder) re-recorded all four JSON files and all five
         blobs off the same texts, instants and producers. Recorded in that
         file's module doc, not only in the commit.
      3. Two gates, one real and one a homonym —
         `membrane_boundary::every_membrane_dependency_is_allowed` (a membrane
         crate declaring a non-membrane dep: a real gate doing its job, closed
         with a scoped `EXEMPT` row) and
         `tests_do_not_share_a_counter::no_test_measures_shared_state_against_its_verdict`
         (its inventory scans every test file for the literal `failed(` and
         attributed spec02's new `Scope::failed()` to `commands_exit.rs`'s
         process-global `FAILED` latch).

## The address stays SHA-256 — user decision, 2026-08-28

`vectors::address` was the twenty-first `content_hash` call site and it is
**pinned to SHA-256** rather than following the id to blake3. The decision is
the user's, taken on this spec's own measurement, and it is recorded here
because the code now holds two hash functions and that reads as an oversight
unless the reason travels with it.

**What the measurement was.** Replaying the pre-adoption determinism recording
through the post-adoption code — the exact shape of a user's first run — was
not a byte-compare failure. `fold::replay` **refused the fixture**: *"vector
c2b2950f… named at seq 3 is corrupt"*. `store_core`'s `FORMAT_VERSION` wipes
the packed graph and reaches neither the journal nor `.mi/vectors`, so the
fold replayed an old journal whose `document`/`chunks` fields name SHA-256
addresses, `Vectors::get` re-derived them under blake3, and the store refused
byte-perfect blobs as `Corrupt`. The workspace came up **empty**. That is a
total loss of reachability, not the bounded doc-id break the parent PRD
scoped.

**Why an id may move and an address may not.** The fold recomputes an entity id
from the *text* the event carries — measured, not assumed: the same replay
produced `b79d408a…` = `blake3(text)` where the recording held
`101c80aa…` = `sha256(text)`, both verified against an oracle outside the tree.
So the graph is re-minted whole and nothing inside it dangles. A vector address
is different in kind: it is the name of a file the journal does not contain, and
nothing can rename bytes the fold never sees. An append-only log cannot be
rewritten to say a new name.

**Implemented as `hex::encode(digest(..))`**, which is byte-for-byte what
`content_hash` computed before it became blake3 — so no address in any existing
store moved. Confirmed against a pre-adoption blob with an oracle outside this
tree: unpacking `vectors/c2/b295…vec` out of git and taking SHA-256 over the hex
of every `to_bits` reproduces `c2b2950f…`, the name the file already had.
`vectors/tests/vectors.rs` now asserts the equality **and** an `assert_ne!`
against `content_hash`, so the next attempt to "unify the two hashes" fails a
test instead of stranding every journal.

### Still broken, reported and deliberately not worked around

Pinning the address makes the fold succeed; it does not close everything.
**An id that one event stores as a literal reference to another entity still
dangles, and it dangles silently.** A `memory.recall` event names the ids it
recalled as text; `retrieval::score::apply_recall` skips ids the graph does not
hold. On the same replay, two entities came back `heat 1.0 / access 1` where
the recording had `1.96 / 2`, and the delivered ranking moved with them — with
no error, no log and no refusal. `Ingest::replaces` is the same shape and that
fixture does not exercise it.

So a user's first run after this lands: the pack wipes on `FORMAT_VERSION`, the
fold **succeeds**, the graph rebuilds whole under blake3 ids — and whatever
recall heat and access counts the store had accumulated are silently dropped,
changing retrieval ranking with nothing saying why. That is smaller than
"comes up empty" and worse in kind, because it is silent. It is a finding for
the board, not something this spec worked around.


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
