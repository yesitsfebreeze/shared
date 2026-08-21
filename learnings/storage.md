---
type: learning
learning: storage
subject: redb replaces LMDB/heed as the graph store, sequenced after mitosys's fold rewrite — the sequencing is the whole decision, because a disposable store makes redb's two remaining costs vanish
binds: [mitosys, llm]
status: partial
date: 2026-08-18
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

`legacy.rs` — 934 lines of old-format readers — does not touch it; it reads
raw files. The real surface is four table handles in one file, all the same
shape:

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

- **After the fold rewrite**, the swap is "delete the store, refold from
  the journal." redb's format churn stops being a risk *permanently* — a
  format break becomes just another refold, which is a thing the system
  does anyway — and the maturity objection evaporates because nothing
  durable is at stake.
- **Before it**, the storage engine changes underneath a rewrite that is
  changing what storage *means*. Two moving parts, and whichever breaks
  gets blamed on the other.

Migration, the usual blocker, is already gone either way.
`store_core/README.md`: `FORMAT_VERSION` gates every open, a mismatched
store is *rejected and wiped by the operator, never migrated*, and **"no
migration code exists to rot."** A storage swap therefore costs exactly one
wipe — what the tree already charges for a format bump.

One knot unties itself as a side effect: `open_membrane_db` currently opens
the LMDB named database under the retired `kern` name, because renaming it
would silently open every existing store *empty* (`create_database` on a
missing name succeeds). redb tables are declared fresh, so the hazard has
nowhere left to live.

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
what llm's `src/grade/` does and exactly what mitosys lacks.

Which reorders the plan: **sharing the grade harness is a prerequisite for
this decision, not a parallel task.** The first shared tool decides the
first shared crate.

## Housekeeping when this lands

llm pins `redb = "4.1.0"` and has no `rust-toolchain.toml`; 4.2.0 shipped
2026-08-17. If both trees share a store, redb belongs in mitosys's
`[workspace.dependencies]`, pinned once — the same reasoning as the serde
pin, and for a stronger reason: two versions of an embedded database mean
two on-disk formats that look identical.
