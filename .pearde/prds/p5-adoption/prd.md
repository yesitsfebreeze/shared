---
state: claimed
mode: afk
priority: 40
verify: "sh prds/p5-adoption/probe/ledger.sh"   # was an echo until 2026-08-28; the probe exits 1 on any red or missing proof
complexity: 14
blast-radius: low
claim: impl-p5-adoption 2026-08-28 23:35
---

# P5 — adoption: every consumer, one implementation

Purpose: the extraction is not done when the crate compiles — it is done when
the duplicates are gone. Each consumer replaces its hand-rolled version with a
`conserved` dependency through the p0-decided mechanism, and the ratchets that
keep the drift from regrowing land in the consumers' own gates. The work
happens in the consumer trees under their own laws and boards; this node is
the cross-tree ledger. Blocked on p1–p4 per module — adopt each module as it
lands, do not wait for all four.

## Requirements

- [~] ~~**mitosys** (`../mitosys`) — `util/effect` becomes a re-export or is
      deleted in favour of `conserved::scope`; SHA-256 ids migrate to
      `ContentId` (the `ed25519:` prefix shim lives here, at the call sites
      that need it); `percentile_sorted` (zero callers) deleted;
      clock reads route through `Clock`. Gate: `dependency_tree.rs` accepts
      the crate; `just check` green in the container — which is the final
      proof of the p0 distribution decision.~~ — struck 2026-08-28: five of
      six clauses hold, measured by `probe/ledger.sh` — `util/effect` is
      `pub use conserved::scope::{Closed, Disposer, Scope, Undo};` at
      `src/mitosys/util/effect/effect.rs:79` (79 lines, no implementation
      left); SHA-256 ids are on `conserved::ContentId` at
      `src/mitosys/engine/util/util.rs:81` (`digest` at `:109` stays SHA-256
      as a documented non-identity); the `ed25519:` shim is at
      `src/mitosys/engine/util/util.rs:113`; `percentile_sorted` has 0
      occurrences in non-test source; `src/mitosys/gates/tests/dependency_tree.rs:122`
      accepts `"conserved"` and `cargo test -p mitosys-gates --test dependency_tree`
      is `ok. 8 passed; 0 failed`; the container `just check` is the child's
      record at mitosys commit `2d04000d` — 2139/0/21, EXIT=0, empty
      `/usr/local/cargo/git/` (`prds/p5-adoption/mitosys/prd.md` § Done
      2026-08-28). The sixth clause does not hold: `now_nanos`/`now_ms`/`now_secs`
      at `src/mitosys/engine/util/util.rs:206-222` read `SystemTime::now`
      directly — 41 direct wall reads in non-test source, held under
      `RATCHET_CEILING = 48`. Owner: `mitosys/prds/adopt-conserved`
      (`state: open`), its box "Clock reads route through `conserved::Clock`".
- [~] ~~**llm** (`../model`) — gains `Scope` (retiring the DOGMA-13-by-hand
      prose sites in `main.rs`); `rec_now()` and the ~65 wall-clock reads
      route through `Clock` — the content-hash preimage first, it is the
      dangerous one; `grade::measure::aggregate` calls `min_median_max`.
      Note: adopting `conserved` means llm consumes an edition-2021 crate
      from its edition-2024 tree — fine, but record it.~~ — struck
      2026-08-28: four clauses hold, measured by `probe/ledger.sh` — `Scope`
      is at the boot path `src/daemon/mod.rs:803` and `src/main.rs:136`
      points at it; `rec_now()` is `SystemClock.now().as_unix_secs()` at
      `src/record/mod.rs:263` with `Record.created: Instant` at
      `src/record/mod.rs:170`, so the content-hash preimage is on `Clock`;
      `grade::measure::aggregate` calls `conserved::stats::min_median_max`
      at `src/grade/measure.rs:226`; the edition note is `model/Cargo.toml:34`.
      The "~65 wall-clock reads route through `Clock`" clause cannot hold as
      written: those reads were monotonic `Instant::now` (74 in non-test
      source today), which the llm child's box 3
      (`prds/p5-adoption/llm/prd.md`) forbids converting to
      `conserved::Instant`; the real wall-clock count went 15 → ceiling 10
      (`gates/tests/clock_read_ratchet.rs:82`, `RATCHET_CEILING = 10`), 15
      `SystemTime::now` lines by plain grep. Owner of the remainder: model's
      ratchet and `model/prds/adopt-conserved` (`state: open`).
- [x] **realm** (`../realm`) — smallest surface; adopt `Scope` and `ContentId`
      where applicable. Also the cleanest test of "distributable to any repo",
      since no learning binds it yet — if adoption forces a `conserved`
      change, that change was a missing requirement, not realm's problem.
      Verified 2026-08-28, `sh prds/p5-adoption/probe/ledger.sh`:
      ```
      ok    Scope adopted at 4 sites: src/net/src/lib.rs:1 src/drivers/linux/src/zfs_volumes.rs:1 src/drivers/linux/src/overlay.rs:2
      ok    ContentId and stats have no call site in realm (grep for blake3|sha2|Sha256 and median|percentile both exit 1) — refusal recorded, admission criterion 1
      ok    realm's adoption forced no change to the crate: shared commits 795f1df + ad1b3b4 touch prds/ only — 'distributable to any repo' held
      ok    cd ../realm && just check — exit 0 (vendor-check rev agreement + content match, cargo fmt --check, cargo check --workspace)
      ```
      `ContentId` has no applicable site in realm — the two greps exit 1 —
      so "where applicable" is `Scope` only, the refusal the child
      `prds/p5-adoption/realm/prd.md` records. The `just check` line is the
      probe's gate on `conserved-vendor-check: ok   content match`.
- [x] **The ratchets** — per `learnings/clock.md`: a count of direct
      wall-clock reads per tree that may only go down, enforced in each
      consumer's gate, not here. Landed when a new read fails a named check.
      Verified 2026-08-28, `sh prds/p5-adoption/probe/ledger.sh` — the named
      check is `wall_clock_reads_may_only_decrease`, run in each tree:
      ```
      ok    mitosys cargo test -p mitosys-gates --test write_path_reads_no_clock — ok. 7 passed; 0 failed; wall_clock_reads_may_only_decrease + monotonic_reads_are_never_counted
      ok    model   cargo test -p gates --test clock_read_ratchet — ok. 3 passed; 0 failed; wall_clock_reads_may_only_decrease + monotonic_reads_are_never_counted
      ok    realm   cargo test -p realm-gates --test clock_read_ratchet — ok. 3 passed; 0 failed; wall_clock_reads_may_only_decrease + monotonic_reads_are_never_counted
      ok    mitosys RATCHET_CEILING = 48  (../mitosys/src/mitosys/gates/tests/write_path_reads_no_clock.rs:228)
      ok    model RATCHET_CEILING = 10  (../model/gates/tests/clock_read_ratchet.rs:82)
      ok    realm RATCHET_CEILING = 0  (../realm/src/gates/tests/clock_read_ratchet.rs:55)
      ```
      The probe's `run_gate` passes only on `wall_clock_reads_may_only_decrease ... ok`
      and `monotonic_reads_are_never_counted ... ok` both present in each
      tree's output.
- [x] **The load proof** — the crate must hold its contract at scale
      (README vision §4): `ContentId` hashing throughput and `Scope`
      unwind-under-panic exercised in a bench/test recorded in `conserved`
      itself, before mitosys's record depends on them.
      Verified 2026-08-28, `sh prds/p5-adoption/probe/ledger.sh`
      (`cargo test -p shared load -- --include-ignored`):
      ```
           Running tests/load_scope.rs (target/debug/deps/load_scope-ff3f78fd490caa17)
      test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
           Running tests/load_throughput.rs (target/debug/deps/load_throughput-84804d089c2b865a)
      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
           Running tests/load_unwind_panic.rs (target/debug/deps/load_unwind_panic-acea399ebc32c684)
      test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
      ok    cargo test -p shared load -- --include-ignored: every load_* binary green
      ```

## Acceptance

`rg` across all three trees finds no second implementation of scope, content
hashing, clock reads outside `SystemClock`, or median; each tree's own gate
is what enforces that, and this node names where each proof lives.


## Refined 2026-08-21 — six children

This node held six separable contracts across four repositories, ~52h measured.
It is now a parent; work flows to the leaves. Order (no all-four barrier, per
the node's own "adopt each module as it lands"):

`ratchets` (blocked on nothing — lands first, per `learnings/clock.md`'s
"make it visible first") -> `load-proof` (needs p1+p2 only) ->
`realm` || `mitosys` || `llm` -> `close` (last, per AGENTS.md's Close step).

## Questions

Three forks the children cannot settle. `mitosys`, `llm` and `realm` are held
until 1 and 2 are answered.

1. **Does this repo get a real git remote?** p1's proof pinned
   `git = "file:///Users/feb/dev/infra/shared"` and recorded the URL as the
   provisional half; this node was named as the swap. `git remote -v` is empty.
   A `file://` URL cannot work inside mitosys's container or for realm on any
   other machine. *Recommendation: name or create the remote before the consumer
   children start; all three pin `{ git = <url>, rev = <sha> }` at the same rev.*
2. **Are cross-tree adoption commits this board's to make?** mitosys and
   `../model` each run their own `.mi/prds` board and laws; realm has neither.
   This node's own text says "the work happens in the consumer trees under their
   own laws and boards; this node is the cross-tree ledger."
   *Recommendation: each consumer child produces a branch plus a node on that
   tree's own board, and this board records where each proof lives.*
3. **Two deliberate persisted-id breaks — are both accepted, one version bump each?** mitosys: SHA-256 hex doc ids ->
   blake3, behind `store_core`'s `FORMAT_VERSION` wipe. llm: `Record.created`
   seconds -> `Instant` nanoseconds, rewriting every `rec_id`, plus
   `Commit.timestamp` u64 -> i64. Both are wipe-and-re-derive, not migrations.
   *Recommendation: accept both explicitly, one version bump each, recorded in
   the child's prd — but the user says so, not an analyst.*

## Answers — 2026-08-21

1. **Remote**: `https://github.com/inner-zirkle/shared`. Added as `origin`
   locally; **not pushed** — publishing is the user's call. When the consumer
   children eventually run, all three pin `{ git = "https://github.com/inner-zirkle/shared", rev = <sha> }`
   at the same rev, replacing p1's provisional `file:///Users/feb/dev/infra/shared`.
2. **No work lands in the sibling repos yet.** The user's instruction: finish
   this shared repo's tools first, then reconcile the implementations once
   everything is tested and works. So `ratchets`, `mitosys`, `llm` and `realm`
   are **held** — they are fully written up and ready, but nothing is dispatched
   into `../mitosys`, `../model` or `../realm` from this board. In scope now:
   `load-proof` and `close`, both of which land here.
3. **Both persisted-id breaks accepted, one version bump each**, recorded in the
   `mitosys` and `llm` children for when they run. mitosys: SHA-256 hex doc ids
   -> blake3 behind `store_core`'s `FORMAT_VERSION` wipe. llm: `Record.created`
   seconds -> `Instant` nanoseconds (rewriting every `rec_id`) plus
   `Commit.timestamp` u64 -> i64. Wipe and re-derive, not migrations.

## Answers — 2026-08-26, the hold lifts

4. **The hold of Answer 2 is lifted, in full** (user decision, 2026-08-26).
Its condition was "finish this shared repo's tools first, then reconcile the
implementations once everything is tested and works." That condition is now
met and measured: `p0-foundation`, `p1-scope`, `p2-content-id`, `p3-clock`,
`p4-stats`, `p6-scope-unwind`, and this node's own `load-proof` and `close`
children are all `state: done`. Nothing is left in this repository but the
four children the hold itself parked.

`ratchets`, `realm`, `llm` and `mitosys` are dispatchable from this board from
now on, in the order this node's `## Refined` section already fixed:

```
ratchets  ->  realm || llm || mitosys  ->  close
```

`close` is already `done` and is re-opened only if an adoption changes what it
recorded.

5. **Answer 1's remote is stale — use the repository's own.** Answer 1
(2026-08-21) pinned `https://github.com/inner-zirkle/shared`. Measured
2026-08-26: `git remote -v` in this repository reports
`https://github.com/yesitsfebreeze/shared.git` (reproduced; the repositories
moved off the `inner-zirkle` organization on 2026-08-23). Every consumer child
pins **that** URL at a common `rev`, not the one Answer 1 names.

Two things a consumer child must check before it pins, neither settled here:

- `git -C ../shared log origin/main..HEAD` — this repository has commits that
  have never been pushed, and a `rev` that is not on the remote cannot be
  fetched by mitosys's container or by realm on another machine. Pushing is
  the user's act, not the board's.
- `../mitosys` has **no** git remote at all (`git remote get-url origin` →
  empty, reproduced). That does not block it consuming `shared`, but it does
  mean mitosys's own adoption commit lives only on this machine.

## Ledger — 2026-08-28

Where each proof lives. Measured by `sh prds/p5-adoption/probe/ledger.sh`
(read-only in all four trees; the implementer's run: exit 0, 0 `FAIL`,
6 `GAP`). The `measured` column quotes that run's lines. `GAP` is a clause
that does not hold as written, with its owner — a strike-with-reason under
`learnings/done-means-done.md`, not a tick.

| box | holds? | where the proof lives | measured |
|---|---|---|---|
| 1 **mitosys** | struck — five of six clauses hold; the `Clock` clause does not | re-export: `src/mitosys/util/effect/effect.rs:79`. `ContentId`: `src/mitosys/engine/util/util.rs:81`; `digest` stays SHA-256 at `:109`; `ed25519:` shim at `:113`. `percentile_sorted`: 0 occurrences. Gate: `src/mitosys/gates/tests/dependency_tree.rs:122`, `cargo test -p mitosys-gates --test dependency_tree`. Container `just check`: mitosys commit `2d04000d`, recorded in `prds/p5-adoption/mitosys/prd.md` § Done 2026-08-28 (not re-runnable from this tree). The unadopted clause: `src/mitosys/engine/util/util.rs:206-222` (`now_nanos`/`now_ms`/`now_secs` read `SystemTime::now`); owner `mitosys/prds/adopt-conserved` (`state: open`), box "Clock reads route through `conserved::Clock`" | `util/effect is a re-export: effect.rs:79, 79 lines, no implementation left` · `content_hash is conserved::ContentId (engine/util/util.rs:81) — SHA-256 ids migrated; digest (util.rs:109) stays SHA-256 as a documented non-identity` · `ed25519: shim lives at engine/util/util.rs:113` · `percentile_sorted: 0 occurrences in non-test mitosys source` · `dependency_tree.rs accepts the crate (line 122)` · `cargo test -p mitosys-gates --test dependency_tree — ok. 8 passed; 0 failed` · `just check in the container: ... 2d04000d — 2139/0/21, EXIT=0, empty /usr/local/cargo/git/` · `GAP mitosys has NOT adopted Clock: now_nanos/now_ms/now_secs (engine/util/util.rs:206-222) read SystemTime::now; 41 direct wall reads in non-test source, held by the ratchet` |
| 2 **llm** (`../model`) | struck — `Scope`, `rec_now()`, `min_median_max`, the edition note hold; "~65 wall-clock reads" cannot hold as written | `rec_now()`: `src/record/mod.rs:263` (`SystemClock.now().as_unix_secs()`); `Record.created: Instant` at `src/record/mod.rs:170` — the content-hash preimage is on `Clock`. `min_median_max`: `src/grade/measure.rs:226`. `Scope`: `src/daemon/mod.rs:803`; `src/main.rs:136` points at it. Edition note: `model/Cargo.toml:34`. The "~65" clause: those reads were monotonic `Instant::now` (74 today), which the llm child's box 3 (`prds/p5-adoption/llm/prd.md`) forbids converting; the real wall-clock count went 15 → ceiling 10 in `gates/tests/clock_read_ratchet.rs:82`. Owner of the remainder: model's ratchet and `model/prds/adopt-conserved` (`state: open`) | `rec_now() reads SystemClock (record/mod.rs:263); Record.created: Instant (record/mod.rs:170) — the content-hash preimage is on Clock` · `grade::measure::aggregate calls min_median_max (grade/measure.rs:226)` · `Scope adopted at boot: daemon/mod.rs:803; main.rs's DOGMA-13 prose site now points at it (main.rs:136)` · `edition-2024 tree consuming an edition-2021 crate is recorded (model/Cargo.toml:34)` · `GAP the box's "~65 wall-clock reads" were monotonic: 74 Instant::now in non-test source today and none may become conserved::Instant (llm child box 3); the real wall-clock count went 15 -> ceiling 10 (gates/tests/clock_read_ratchet.rs), 15 SystemTime::now lines by plain grep` |
| 3 **realm** | yes | `Scope::new()` at 4 sites: `src/net/src/lib.rs` ×1, `src/drivers/linux/src/zfs_volumes.rs` ×1, `src/drivers/linux/src/overlay.rs` ×2. `ContentId`/`stats` refusal: `grep -rE 'blake3\|sha2\|Sha256'` and `grep -rE 'median\|percentile'` over `src/` + `Cargo.toml` both exit 1. No forced crate change: `shared` commits `795f1df` + `ad1b3b4` touch `prds/` only. Gate: `cd ../realm && just check` | `Scope adopted at 4 sites: src/net/src/lib.rs:1 src/drivers/linux/src/zfs_volumes.rs:1 src/drivers/linux/src/overlay.rs:2` · `ContentId and stats have no call site in realm (grep for blake3\|sha2\|Sha256 and median\|percentile both exit 1) — refusal recorded, admission criterion 1` · `realm's adoption forced no change to the crate: shared commits 795f1df + ad1b3b4 touch prds/ only — 'distributable to any repo' held` · `cd ../realm && just check — exit 0 (vendor-check rev agreement + content match, cargo fmt --check, cargo check --workspace)` |
| 4 **the ratchets** | yes | the named check is `wall_clock_reads_may_only_decrease`, beside `monotonic_reads_are_never_counted`, in each tree's own gate: mitosys `src/mitosys/gates/tests/write_path_reads_no_clock.rs` (`RATCHET_CEILING = 48` at `:228`), model `gates/tests/clock_read_ratchet.rs` (`RATCHET_CEILING = 10` at `:82`), realm `src/gates/tests/clock_read_ratchet.rs` (`RATCHET_CEILING = 0` at `:55`) | `mitosys cargo test -p mitosys-gates --test write_path_reads_no_clock — ok. 7 passed; 0 failed; wall_clock_reads_may_only_decrease + monotonic_reads_are_never_counted` · `model cargo test -p gates --test clock_read_ratchet — ok. 3 passed; 0 failed; wall_clock_reads_may_only_decrease + monotonic_reads_are_never_counted` · `realm cargo test -p realm-gates --test clock_read_ratchet — ok. 3 passed; 0 failed; wall_clock_reads_may_only_decrease + monotonic_reads_are_never_counted` · `mitosys RATCHET_CEILING = 48 (...write_path_reads_no_clock.rs:228)` · `model RATCHET_CEILING = 10 (...clock_read_ratchet.rs:82)` · `realm RATCHET_CEILING = 0 (...clock_read_ratchet.rs:55)` |
| 5 **the load proof** | yes | `shared/tests/load_scope.rs`, `shared/tests/load_throughput.rs`, `shared/tests/load_unwind_panic.rs`; `cargo test -p shared load -- --include-ignored`. (The child `prds/p5-adoption/load-proof` recorded 10 on 2026-08-21; `p6-scope-unwind` added three.) | `Running tests/load_scope.rs` `test result: ok. 5 passed; 0 failed; 0 ignored` · `Running tests/load_throughput.rs` `test result: ok. 1 passed; 0 failed; 0 ignored` · `Running tests/load_unwind_panic.rs` `test result: ok. 7 passed; 0 failed; 0 ignored` · `cargo test -p shared load -- --include-ignored: every load_* binary green` |

The `## Acceptance` sentence, clause by clause:

| acceptance clause | verdict | where | owner of the remainder |
|---|---|---|---|
| scope | holds | no second `Scope`/`Disposer` type in any tree — `scope: no second Scope/Disposer type in any tree` | none |
| content hashing | holds in mitosys and realm; open in model | mitosys is on `ContentId` (`src/mitosys/engine/util/util.rs:81`); realm has no hash site. model still carries a local `blake3_hash`/`content_id`/`rec_id` in six files: `src/record/mod.rs`, `src/utils/fs/mod.rs`, `src/utils/algebra/mod.rs`, `src/version/ledger.rs`, `src/node/hot_swap.rs`, `src/node/transactional.rs` — `GAP content hashing: mitosys is on ContentId; model still carries local blake3 copies` | `model/prds/adopt-conserved` (`state: open`), its `ContentId` substitution boxes |
| clock reads outside `SystemClock` | held by the ratchets, counts descending | realm 0 (ceiling 0); model ≤ 10 (ceiling 10); mitosys ≤ 48 (ceiling 48) — each held by its `wall_clock_reads_may_only_decrease`, the mechanism `learnings/clock.md` chose — `GAP clock reads outside SystemClock: realm 0 (ceiling 0); model ceiling 10; mitosys ceiling 48 — held by the ratchets (box 4)` | mitosys: `mitosys/prds/adopt-conserved`; model: `model/prds/adopt-conserved`; realm: none |
| median | holds | no second implementation in any tree — `median: no second implementation in any tree` | none |

Pins. mitosys pins `conserved` at rev
`70d7e15cd21c6017ec928c63697d0c7f42f53a20` (2026-08-28); model and realm pin
`9a342e1e849dd5775cbadfe6b32e275a076e5f09` (2026-08-23). Both revs are on
`origin/main` — `git branch -r --contains` reports `origin/main` for each, so a
clone can fetch them. Each of the three consumers carries
`vendor/conserved-0.1.0` with a source replacement in `.cargo/config.toml`,
so an offline or container build does not depend on the remote. That is two
revs, not the one Answer 1 asked for — both are pre-rename, so nothing is
broken; each tree's `rename-conserved-to-shared` node re-pins all three. This
repository's crate is `shared` since `dfc98fb`, 5 commits ahead of
`origin/main` and unpushed, while every pinned rev holds a package named
`conserved` — the ledger says `conserved` for that reason.
`learnings/shared-crate.md` and `learnings/clock.md` are both
`status: decided`.
