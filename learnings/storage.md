---
type: learning
learning: storage
subject: redb replaces LMDB/heed as the graph store, sequenced after mitosys's fold rewrite — the sequencing is the whole decision, because a disposable store makes redb's two remaining costs vanish
binds: [mitosys, llm]
status: decided
date: 2026-08-23
code: mitosys src/mitosys/engine/store_core/lib.rs, llm src/utils/fs/mod.rs
---

# One storage engine: redb

mitosys stores its graph in LMDB through `heed`; llm stores its registry,
history and event log in `redb`. Two embedded key-value stores across two
trees that intend to share a record is two on-disk formats that look
identical from a distance.

**Decision: redb.** Not yet executed — the sequencing below is the
substance of this document, and doing it early costs more than waiting.

## The coupling is one file

`heed` appears in exactly two files across all of mitosys:

```
7 mentions  src/mitosys/engine/store_core/lib.rs
8 mentions  src/mitosys/gates/tests/dependency_tree.rs   <- the gate that records it
```

`legacy.rs` — 1535 lines of frozen old-format decoders — does not touch it;
it decodes bincode rows and opens no container. The real surface is four
table handles in one file, all the same shape:

```rust
env.create_database::<Str, Bytes>(&mut wtxn, Some(MEMBRANE_DB))
//                   + COLD_DB, COLD_VEC_DB, META_DB
```

String keys, byte values, no `DUPSORT`, no custom comparators. That is
`TableDefinition<&str, &[u8]>` in redb, close to transliteration.

The values do not move at all. The workspace pin note says it outright —
*"bincode bytes ARE the persisted store"* (`bincode` held at 2) — so this
swaps the KV engine underneath an encoding that stays byte-identical.

## What redb buys

1. **`map_size` disappears.** `EnvOpenOptions::new().map_size(MAP_SIZE)` at
   two call sites is a hardcoded ceiling on how large a store may ever
   grow; `MDB_MAP_FULL` is what happens when someone reaches it. redb
   requires no such declaration. Strongest single argument — an entire bug
   class stops existing.
2. **No C toolchain.** Serves llm's stated constraint ("single static
   binary") directly, makes cross-compilation ordinary, and removes one
   reason mitosys's dev container is load-bearing.
3. **mmap stops constraining where a store may live.** LMDB is hostile to
   network filesystems and awkward on some container storage.
4. **Savepoints**, which are worth a look given that layers' whole thesis
   is rollback-by-dropping-a-ref.
5. One storage vocabulary across the family.

## What it costs — less than expected

The obvious objection was LMDB's zero-copy mmap reads. That is **not** the
trade: redb documents zero-copy reads via `AccessGuard`, and MVCC with
non-blocking concurrent readers against a single writer — the same
concurrency model mitosys has now. Two real costs remain:

- **Maturity.** LMDB is fifteen years of production hardening. redb 4.2.0
  released 2026-08-17, and its on-disk format has broken across major
  versions before.
- **Read throughput on the cold and vector tiers.** Genuinely unknown until
  measured, and the one place a graph-plus-vector store would feel a
  regression.

## The sequencing is the decision

mitosys's store is **not yet a disposable fold**. `PRD.md` §4 commits it to
becoming one; `.mi/SYSTEM.md` is explicit that the merge/fold rewrite is
*"decided, not done."*

That distinction converts both remaining costs:

- **Once "delete the store, refold from the journal" is proven**, redb's
  format churn stops being a risk *permanently* — a format break becomes
  just another refold, which is a thing the system does anyway — and the
  maturity objection evaporates because nothing durable is at stake. The
  proof lands inside the swap node itself, as its first acceptance box
  (step 6 below).
- **Before the fold rewrite**, the storage engine changes underneath a
  rewrite that is changing what storage *means*. Two moving parts, and
  whichever breaks gets blamed on the other.

The fold rewrite now has named state, so "after the fold rewrite" is spelled
as nodes. The sequence, one owned carrier per step, states as of 2026-08-23:

1. `p6k10b-vector-home` (mitosys, **specced**) — vectors leave the journal
   for a digest-referenced store outside it (`p6k10` §Answers 1). A refold
   stops needing an embedder, and the DiskANN indexes rebuild from the
   vector store. Without this step, "delete the store, refold" is defined
   only for a store with no vectors.
2. `p6k10c-journaled-ingest` (mitosys, **specced**, needs b) — a
   disk-ingested file becomes a record entry, addressable by `seq`. The
   journal becomes the store's full preimage; a wipe stops losing ingested
   files.
3. `p6k10d-production-fold` (mitosys, **specced**, needs b + c) — the fold
   lands as production code with one production caller (boot tail-replay),
   and the store carries a "folded through seq N" watermark. **The full
   demotion of the store to a checkpoint — the invalidation witnesses, the
   restore floor, the boot-authority change — is a declared follow-on of
   `p6k10d`, not part of it** (`p6k10` §Answers 3). The swap does not wait
   for the demotion: the swap node proves refold-from-empty itself.
4. `record-shape-port` (mitosys, **open**) — the row layout moves:
   per-event preimage id, bitemporal `until`, lazy heat decay
   ([[two-halves]] §Sequencing step 6). **This is the on-disk version bump
   — `FORMAT_VERSION` 15 → 16** (`store_core/lib.rs:101` is
   `const FORMAT_VERSION: u8 = 15`). The engine swap itself bumps nothing:
   the version byte identifies a row layout, the rows stay bincode
   byte-identical, and an LMDB environment and a redb file are mutually
   unopenable containers that identify themselves at open. Shape port
   before swap means the operator wipes once, not twice — one wipe covers
   both.
5. `p8l-nucleus` (mitosys, **blocked**, wave 6 of `p8-membrane`) — the
   memory engine becomes an executable plugin, with the store's on-disk
   behavior explicitly unchanged in that node. `p8-membrane/prd.md` §Out of
   scope sequences the redb swap after `p8l`; after it, the swap changes
   one plugin's store instead of the core's.
6. **The swap** — mitosys child `prds/redb-swap`, under master
   `prds/storage-convergence`: heed → redb inside `store_core`, the seam to
   `store` untouched. Four table handles become
   `TableDefinition<&str, &[u8]>`; the bincode values do not move.
   `MAP_SIZE` with both `.map_size()` call sites
   (`store_core/lib.rs:27,29,680,1298`) and the `kern` named-database
   hazard (`MEMBRANE_DB_LEGACY`, `lib.rs:49`) are deleted by construction.
   `redb` lands in mitosys `[workspace.dependencies]`, pinned once.
   Refold-from-empty is the node's first acceptance box; before/after
   retrieval and cold-tier numbers are measured against a recorded
   envelope — the grade shape `p9-perf-floor` carries (mitosys, open,
   [[two-halves]] §Sequencing step 5).

## What reads the old format during the overlap

Between now and the end of step 6, the v15/LMDB format has four readers:

| reader | fate |
|---|---|
| `store_core` v15 over heed | the only LMDB reader; deleted at the end of the swap's commit series |
| `store_core/legacy.rs` (1535 lines, frozen pre-v15 decoders) | decodes bincode rows, opens no container — zero `heed` mentions; engine-agnostic, untouched by the swap |
| the admin scan, `engine/commands/commands_admin.rs:1113` | a stat walk over `.pi/kern` and `.mi` data dirs, opens nothing; its "set `MITOSYS_MEMORY_DIR` to pin" hint becomes a lie for LMDB stores post-swap — the swap child updates the hint |
| DiskANN indexes beside `data.mdb` | separate files, no LMDB dependency; rebuilt from the vector store (step 1) |

## The data, named

Verified 2026-08-23. Every store on this machine is the **legacy `.pi/kern`
layout**, pre-journal — no `.mi/journal` and no `.mi/memory` exists in any
tree, and the default open path is `.mi/memory`
(`src/plugins/memory/memory.rs:322`), so **nothing reads these stores
today** without a `MITOSYS_MEMORY_DIR` pin:

| store | size |
|---|---|
| `mitosys/.pi/kern/data` | 48M |
| `model/.pi/kern/data` | 160M |
| `realm/.pi/kern/data` | 3.3M |
| `shared/.pi/kern/data` | 2.9M |
| `/Users/feb/dev/archive/ui/.pi/kern/data` | 2.8M — retired with the tree (`prds/ui-disposition`, done) |

Disposition: the `.pi` → `.mi` move stranded these stores, not this swap;
the swap deletes their last *reader* from HEAD, never the files. The
recovery route is version control — the last pre-swap commit builds the v15
reader, pinned at any of these directories — the same reasoning
`ui-disposition` used to retire `ui` checkpointed. No export tool: the
standing policy is *rejected and wiped, never migrated*
(`store_core/README.md` §Invariants), and no migration code exists to rot.
The swap child names these five paths so the wipe is an act, not an
accident.

## What this beat

- **Stay on heed.** Loses `map_size` relief and keeps a C dependency in a
  family whose other half wants a single static binary. Its real argument
  is maturity, which the fold rewrite neutralises.
- **Abstract behind a storage trait, support both.** Rejected: two
  implementations where one is wanted is the law 4 violation, not the fix.
  The coupling is already one file behind a crate boundary that
  `dependency_tree.rs` enforces — **the crate is the abstraction, and the
  gate already keeps the option open at zero cost.**
- **Swap now, before the fold rewrite.** Rejected on sequencing above.

## How to decide it honestly

On numbers, not preference — and this is the tie-in to [[two-halves]]:
baseline the retrieval and cold-tier paths on heed, port the one file,
re-measure. Pass/regression/fail against a recorded envelope is exactly
what model's `src/grade/` does and exactly what mitosys lacks.

Which reorders the plan: **sharing the grade harness is a prerequisite for
executing this decision, not a parallel task.** The carrier is mitosys
`prds/p9-perf-floor` (step 6 above).

## Housekeeping when this lands

- model already runs redb (`src/utils/fs/mod.rs`) and pins `redb = "4.1.0"`
  (`model/Cargo.toml:56`). When the swap child pins redb in mitosys's
  `[workspace.dependencies]`, model bumps its pin to match — one Cargo
  line, admitted to model's board at that moment; a child placed now would
  carry an unknown version. `model/prds/workspace-deps` (open) does not
  cover the bump: that lift keeps `Cargo.lock` unchanged.
