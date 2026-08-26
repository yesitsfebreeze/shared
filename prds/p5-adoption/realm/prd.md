---
state: specced
mode: afk
priority: 43
est: 6h
repo: realm
verify: "cd ../realm && just check"
complexity: 35
blast-radius: mid
---

# P5c — realm: adopt what there is a call site for, record what there is not

Purpose: the cleanest test of "distributable to any repo" — realm has no
AGENTS.md, no board and no learning binding it. If adoption forces a `conserved`
change, that change was a missing requirement, not realm's problem.

**The parent's wording does not survive contact with the tree.** It says "adopt
`Scope` and `ContentId` where applicable". `ContentId` is applicable **nowhere**:
`grep` for `blake3|sha2|Sha256` across realm's `src/` and manifests returns
**zero hits**, as does `median|percentile`. Adopting either would fail admission
criterion 1 — *both trees need it today, not speculatively*. Record the refusal;
do not manufacture a call site.

## Requirements

- [ ] **`Scope` has three real sites**, all hand-rolled reverse-order undo:
      `src/drivers/linux/src/overlay.rs` (`unmount_all`, line 385 — "Undo every
      mount `assemble` made, innermost first"),
      `src/drivers/linux/src/zfs_volumes.rs:139` ("Leave nothing behind: undo
      this call's own work"), `src/net/src/lib.rs:437` ("Cleanup netns on
      failure").
- [ ] **`Clock`: 3 non-test wall-clock reads** — `drivers/linux/src/state.rs:140`,
      `cli/src/lib.rs:1025`, `cli/src/state.rs:61`. Everything else is monotonic
      (`net/src/dns.rs` TTL cache, `zfs/src/cli.rs:437` deadline,
      `drivers/linux/src/lib.rs:1038-1054` timeout) and must NOT be converted.
- [ ] **Record the `ContentId`/`stats` refusal** with the grep evidence, so the
      next reader does not re-litigate it.

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
