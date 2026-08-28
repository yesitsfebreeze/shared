---
complexity: 8
footprint:
  - prds/p5-adoption/prd.md
  - prds/p5-adoption/probe/ledger.sh
---

# spec01 — the ledger: where each proof lives, measured, written into `prd.md`

This node is the cross-tree ledger. Its `## Acceptance` says "this node names
where each proof lives" — and today nothing in `prd.md` does. This spec runs
the probe the analyst left at `probe/ledger.sh` (read-only in all four trees;
it exits 1 when any proof it relies on is missing or red) and writes its
findings into `prd.md` as a `## Ledger — 2026-08-28` section: one row per
requirement box, one row per clause of the acceptance sentence, each naming
the file, line, gate or commit the proof lives at and quoting the measured
output. Nothing in the consumer trees is written; nothing in frontmatter moves.

## What already stands

The analyst ran the probe on 2026-08-28 and it exited 0 with these facts.
Re-run it; if a line has changed, the ledger records what the re-run says,
not this list.

- **Pins.** All three consumers pin `conserved` by rev, and both revs are on
  `origin/main`: mitosys `70d7e15cd21c6017ec928c63697d0c7f42f53a20`
  (2026-08-28), model and realm `9a342e1e849dd5775cbadfe6b32e275a076e5f09`
  (2026-08-23). Each carries `vendor/conserved-0.1.0` plus a source
  replacement in `.cargo/config.toml`. That is two revs, not the one Answer 1
  asked for — both are pre-rename, so nothing is broken, and each tree's
  `rename-conserved-to-shared` node re-pins it. This repo's crate is `shared`
  since `dfc98fb`, unpushed, so every pinned rev still holds a package named
  `conserved`; the ledger's text says `conserved` for that reason.
- **Box 5 (load proof)** — `cargo test -p shared load -- --include-ignored`:
  `load_scope` 5 passed, `load_throughput` 1 passed, `load_unwind_panic`
  7 passed, 0 failed. (The child recorded 10 on 2026-08-21; p6-scope-unwind
  added three.)
- **Box 4 (ratchets)** — the named check is `wall_clock_reads_may_only_decrease`
  in all three trees, beside `monotonic_reads_are_never_counted`:
  - mitosys `cargo test -p mitosys-gates --test write_path_reads_no_clock` —
    7 passed; 0 failed; `RATCHET_CEILING = 48` at `:228`
  - model `cargo test -p gates --test clock_read_ratchet` — 3 passed;
    0 failed; `RATCHET_CEILING = 10` at `:82`
  - realm `cargo test -p realm-gates --test clock_read_ratchet` — 3 passed;
    0 failed; `RATCHET_CEILING = 0` at `:55`
- **Box 3 (realm)** — `Scope::new()` at 4 sites across the three files the
  child names (`overlay.rs` ×2, `zfs_volumes.rs`, `net/src/lib.rs`);
  `grep -rE 'blake3|sha2|Sha256'` and `grep -rE 'median|percentile'` over
  `src/` + `Cargo.toml` both exit 1 (no `ContentId`/`stats` call site — the
  refusal the child records); the two `shared` commits the child lists as
  forced by the adoption, `795f1df` + `ad1b3b4`, touch `prds/` only — the
  crate needed no change, so "distributable to any repo" held;
  `cd ../realm && just check` exits 0 (vendor-check `rev agreement` +
  `content match`, `cargo fmt --all --check`, `cargo check --workspace`).
- **Box 1 (mitosys)** — five of six clauses hold: `util/effect/effect.rs:79`
  is `pub use conserved::scope::{Closed, Disposer, Scope, Undo};` (79 lines,
  no implementation left); `engine/util/util.rs:81` is
  `conserved::ContentId::of(s.as_bytes()).to_string()` so SHA-256 ids have
  migrated (`digest` at `:109` stays SHA-256 as a documented non-identity);
  the `ed25519:` shim is `engine/util/util.rs:113`; `percentile_sorted` has 0
  occurrences; `gates/tests/dependency_tree.rs:122` accepts `"conserved"` and
  that gate is 8 passed; 0 failed. The container `just check` is the child's
  record at `2d04000d` — 2139/0/21, EXIT=0, empty `/usr/local/cargo/git/` —
  and is not re-runnable from this tree. **The sixth clause does not hold:**
  `now_nanos`/`now_ms`/`now_secs` at `engine/util/util.rs:206-222` read
  `SystemTime::now` directly, 41 wall reads in non-test source, held under
  ceiling 48. Owner: `mitosys/prds/adopt-conserved` (`state: open`), whose
  box reads "Clock reads route through `conserved::Clock`".
- **Box 2 (llm)** — `rec_now()` is `SystemClock.now().as_unix_secs()`
  (`record/mod.rs:263`), `Record.created: Instant` (`:170`);
  `grade/measure.rs:226` calls `conserved::stats::min_median_max`; `Scope` is
  at the boot path `daemon/mod.rs:803` and `main.rs:136` points at it; the
  edition note is `model/Cargo.toml:34`. **The "~65 wall-clock reads" clause
  does not hold as written:** those were monotonic `Instant::now` (74 today),
  which the llm child's box 3 forbids converting; the real wall-clock count
  went 15 → ceiling 10. Owner of the remainder: model's ratchet and
  `model/prds/adopt-conserved` (`state: open`).
- **Acceptance sentence** — scope: no second `Scope`/`Disposer` type in any
  tree; median: no second implementation in any tree; content hashing:
  mitosys is on `ContentId`, model still carries local `blake3_hash` copies
  in `record/mod.rs`, `utils/fs/mod.rs`, `utils/algebra/mod.rs`,
  `version/ledger.rs`, `node/hot_swap.rs`, `node/transactional.rs` (owner:
  `model/prds/adopt-conserved`); clock reads outside `SystemClock`: realm 0,
  model ≤ 10, mitosys ≤ 48, each held by its ratchet.
- `learnings/shared-crate.md` and `learnings/clock.md` are `status: decided`.

## What is left

Write the `## Ledger — 2026-08-28` section into `prd.md`, below
`## Answers — 2026-08-26, the hold lifts`. Two tables and one paragraph:

1. `| box | holds? | where the proof lives | measured |` — five rows, one per
   `## Requirements` box, `holds?` being `yes` / `struck` and `measured`
   quoting the probe line(s) for that box verbatim (the counts, the
   `N passed; 0 failed`, the file:line).
2. `| acceptance clause | verdict | where | owner of the remainder |` — four
   rows: scope, content hashing, clock reads, median.
3. The pin paragraph: the two revs, that both are on `origin/main`, the
   vendor copies, and that this repo's crate is `shared` (unpushed) while
   every pinned rev holds `conserved`.

Do not tick or strike a box in this spec — `spec02` does that from this
ledger. Do not edit frontmatter. Do not write into `../mitosys`, `../model`
or `../realm`.

## Acceptance

- [ ] `sh prds/p5-adoption/probe/ledger.sh` exits 0 on the implementer's run, and its output is what the ledger quotes — every number in the ledger's `measured` column appears in that output
- [ ] `prd.md` has a `## Ledger — 2026-08-28` heading with a five-row box table whose `where the proof lives` column names a file:line, a gate test name, or a commit sha for every row — no row says only "the child PRD"
- [ ] the ledger's box-1 row names `mitosys/prds/adopt-conserved` as the owner of the unadopted `Clock` clause, and its box-2 row names the llm child's box 3 (monotonic `Instant::now`, not wall-clock) as the reason the "~65" clause cannot hold as written
- [ ] the ledger's acceptance table has a `content hashing` row naming the six model files still carrying a local `blake3_hash`/`content_id`/`rec_id` and `model/prds/adopt-conserved` as their owner, and a `clock reads` row naming the three ceilings (48 / 10 / 0)
- [ ] the pin paragraph names both revs (`70d7e15c…` for mitosys, `9a342e1e…` for model and realm) and says both are on `origin/main`
- [ ] no `- [ ]` box in `prd.md` is ticked or struck by this spec — the count of open boxes under `## Requirements` is the same before and after (five)

## Verify and Proof

```sh
sh prds/p5-adoption/probe/ledger.sh
grep -q '^## Ledger — 2026-08-28' prds/p5-adoption/prd.md
grep -q 'mitosys/prds/adopt-conserved' prds/p5-adoption/prd.md
grep -q 'model/prds/adopt-conserved' prds/p5-adoption/prd.md
grep -q '70d7e15cd21c6017ec928c63697d0c7f42f53a20' prds/p5-adoption/prd.md
grep -q '9a342e1e849dd5775cbadfe6b32e275a076e5f09' prds/p5-adoption/prd.md
grep -q 'wall_clock_reads_may_only_decrease' prds/p5-adoption/prd.md
test "$(grep -cE '^[[:space:]]*- \[ \] \*\*' prds/p5-adoption/prd.md)" -eq 5
```
