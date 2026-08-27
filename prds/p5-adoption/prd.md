---
state: open
mode: afk
priority: 40
verify: "echo 'proof lives in the consumer trees: just check in ../mitosys, cargo test in ../model and ../realm, each against a conserved dependency'"
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

- [ ] **mitosys** (`../mitosys`) — `util/effect` becomes a re-export or is
      deleted in favour of `conserved::scope`; SHA-256 ids migrate to
      `ContentId` (the `ed25519:` prefix shim lives here, at the call sites
      that need it); `percentile_sorted` (zero callers) deleted;
      clock reads route through `Clock`. Gate: `dependency_tree.rs` accepts
      the crate; `just check` green in the container — which is the final
      proof of the p0 distribution decision.
- [ ] **llm** (`../model`) — gains `Scope` (retiring the DOGMA-13-by-hand
      prose sites in `main.rs`); `rec_now()` and the ~65 wall-clock reads
      route through `Clock` — the content-hash preimage first, it is the
      dangerous one; `grade::measure::aggregate` calls `min_median_max`.
      Note: adopting `conserved` means llm consumes an edition-2021 crate
      from its edition-2024 tree — fine, but record it.
- [ ] **realm** (`../realm`) — smallest surface; adopt `Scope` and `ContentId`
      where applicable. Also the cleanest test of "distributable to any repo",
      since no learning binds it yet — if adoption forces a `conserved`
      change, that change was a missing requirement, not realm's problem.
- [ ] **The ratchets** — per `learnings/clock.md`: a count of direct
      wall-clock reads per tree that may only go down, enforced in each
      consumer's gate, not here. Landed when a new read fails a named check.
- [ ] **The load proof** — the crate must hold its contract at scale
      (README vision §4): `ContentId` hashing throughput and `Scope`
      unwind-under-panic exercised in a bench/test recorded in `conserved`
      itself, before mitosys's record depends on them.

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
