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
