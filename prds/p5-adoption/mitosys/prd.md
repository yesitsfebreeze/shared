---
state: claimed
claim: impl-p5-mitosys-spec01 2026-08-28T04:20
mode: afk
priority: 44
est: 20h
repo: mitosys
verify: "cd ../mitosys && just check"
complexity: 88
blast-radius: high
---

# P5d — mitosys: four duplicates become one dependency, green in the container

Purpose: mitosys is the strict consumer — edition 2021, pinned 1.94.0, offline
dev container. Its `just check` going green in that container is the final proof
of p0's distribution decision.

## Requirements

- [ ] **The offline container is this node's blocker, not a footnote.** p0's memo
      scoped `cargo vendor` / a pre-populated registry cache as "mitosys-side
      follow-up designed before p5's mitosys adoption step". That work IS this
      node's acceptance and should be its FIRST spec, not its last.
- [ ] **`util/effect` → `conserved::scope` is not one file.**
      `pub use mitosys_util_effect::effect;` is re-exported by 10+ crates
      (`api/plugin`, `api/plugin/lua`, `api/surface`, `api/agentic`,
      `api/agentic/pool`, `api/service`, `api/engine`, `engine/record`,
      `engine/layers`, `engine/channel`), and each of those crates' `//! May
      import:` layer docs names `mitosys_util_effect`. The type is **`Disposer`**,
      not `Handle`.
- [ ] **`content_hash` has 26 non-test call sites and its output is a persisted
      doc id** — `ingest_worker.rs`, `ingest_intake.rs`, `ingest_direct.rs`,
      `ingest_file_watcher.rs`, `engine/identity/lib.rs:48` (truncates to 16
      chars), `engine/record/oracle.rs:113` (oracle key). SHA-256 → blake3
      **invalidates every stored id**, behind `store_core`'s `FORMAT_VERSION`
      wipe policy. A deliberate store break — see the parent's `## Questions`.
- [ ] **The `ed25519:` prefix shim lives here**, at the call sites that need it.
      p2 deliberately refused to port it into the crate.
- [ ] **`percentile_sorted` deletion** also deletes
      `src/mitosys/engine/util/tests/unit/util.rs:26`. Rewrite lines 31-34:
      `Some(5.0), "ceil(0.5*10)=5 -> xs[4]"` becomes `Some(6.0)` under p4's
      upper-median decision. Lines 37-39 use `u128`
      (`percentile_sorted(&ns, 0.5) == Some(30u128)`) and `conserved::stats` is
      `&[f64]`-only by p4's decision — those vectors have no home and go with
      the function.
- [ ] **Gate**: `dependency_tree.rs` accepts the crate (p1 proved it moves
      exactly two lines: OWNERS + CLOSURE); `just check` green in the container.

- [ ] **`conserved::scope` and `util/effect` no longer agree — reconcile
      deliberately, do not diff-and-merge.** p6-scope-unwind made the first
      *semantic* divergence from the byte-for-byte port (deviation 8 in
      `conserved/src/scope.rs`'s `# Provenance`): on close, `conserved` runs
      **every** inverse even when one panics, resumes the first panic reached
      afterwards, keeps `held()` true *during* the unwind, and adds
      `Scope::failed()` naming the inverses that panicked. mitosys's
      `util/effect` still abandons the tail and still reports `held() == []`
      with inverses owed — measured in
      `.mi/prds/p5-adoption/load-proof/finding.md`. Adopting the crate
      therefore **changes teardown behaviour** in all 10+ re-exporting crates:
      a plugin whose inverse panics now has its remaining inverses run rather
      than dropped. That is the point of adopting, and it must be stated in the
      adoption commit rather than discovered. Check the tree for any site that
      depends on the abandonment — `grep` teardown paths for `catch_unwind`
      around a `close()` — and for anything that reads `held()` during an
      unwind. `Scope::failed()` is new API with no mitosys call site yet; adopt
      it where a teardown failure is currently swallowed. The abort-during-abort
      case is **unchanged** in both trees.
      Also: `.mi/prds/p1-scope/specs/spec01.md`'s byte-for-byte `verify:` diff
      no longer holds, by design.

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

The user accepted this break explicitly on 2026-08-21, one version bump: SHA-256 hex doc ids -> blake3, behind `store_core`'s `FORMAT_VERSION` wipe.
Wipe and re-derive, not a migration. It does not need re-escalating when this
node runs.


## Dispatched scoped to `spec01` — 2026-08-28, and why

The implementer is told to build `spec01` and **stop**. Three reasons, and the
first is this PRD's own text.

1. **The PRD says so.** Its first requirement reads: "The offline container is
   this node's blocker, not a footnote … That work IS this node's acceptance and
   should be its FIRST spec, not its last."
2. **`spec01`'s footprint is clean; `spec03` and `spec04`'s addresses are
   rotted.** `@mitosys/p8-membrane/p8d-floor-split` landed at `276a400` and
   moved the floor: `mitosys-util` → **`mitosys-engine-util` at
   `src/mitosys/engine/util`**, `mitosys-util-math` →
   **`mitosys-engine-util-math` at `src/mitosys/engine/util/math`**, and
   `src/mitosys/util/` now holds **only `effect/`**. So:
   - `spec02` cites `src/mitosys/util/effect/` — **still correct**, that crate
     did not move.
   - `spec03` and `spec04` cite `src/mitosys/util/util.rs` — **gone**. Both need
     their addresses refreshed before anyone implements them, and that is the
     board's edit, not a worker's.
   This is the third time this session a spec has been rotted by a rename in
   flight, after `plugins-view` (four paths → nine) and `p8d` itself (three →
   sixteen). The standing lesson: a spec citing `src/mitosys/**` is stable under
   these moves; one citing the moving half goes stale in an afternoon.
3. **Another worker is live in the mitosys tree** on
   `plugins-visible/census-counts-composed`, probing `api/plugin/` and the
   `inspect` builtin — which `spec02`'s footprint overlaps.

The implementer is asked, instead of running ahead, to report every stale
address it notices in `spec02`–`spec04` measured against the current tree. That
list is what the board needs to refresh them.


## The stale-address sweep, done 2026-08-28

`spec01`'s implementer was asked to report every address in `spec02`–`spec04`
that the day's renames had rotted, rather than run ahead into them. It did, and
the board has applied the corrections in each spec, each with its measurement.

The scale is worth recording: **one of the four specs was clean** (`spec02`,
apart from a `src/plugins/` → `src/builtins/` path), one had a moved file with
five drifted line numbers and a miscount (`spec03`: 21 `content_hash` sites, not
20), and one named **a cargo package that no longer exists** (`spec04`:
`-p mitosys-util`). None of that would have failed loudly; a verify naming an
unresolvable package reports "no packages matched" and moves on.

That is the third time this session a spec has been rotted by a rename in
flight — after `plugins-view` (four paths to nine) and `p8d-floor-split` itself
(three to sixteen). The standing lesson is now paid for three times over: a spec
citing `src/mitosys/**` survives these moves; one citing the moving half goes
stale in an afternoon.
