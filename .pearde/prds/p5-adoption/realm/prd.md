---
state: done
mode: afk
priority: 43
est: 6h
repo: realm
verify: "cd ../realm && just check"
complexity: 35
blast-radius: mid
needs:
  - "@realm/16-fmt-gate"
footprint:
  - ../realm/Cargo.toml
  - ../realm/Cargo.lock
  - ../realm/src/cli/Cargo.toml
  - ../realm/src/cli/src/lib.rs
  - ../realm/src/cli/src/state.rs
  - ../realm/src/cli/tests/unit/lib.rs
  - ../realm/src/cli/tests/unit/state.rs
  - ../realm/src/drivers/linux/Cargo.toml
  - ../realm/src/drivers/linux/src/overlay.rs
  - ../realm/src/drivers/linux/src/state.rs
  - ../realm/src/drivers/linux/src/zfs_volumes.rs
  - ../realm/src/drivers/linux/tests/unit/overlay.rs
  - ../realm/src/drivers/linux/tests/unit/state.rs
  - ../realm/src/drivers/linux/tests/unit/zfs_volumes.rs
  - ../realm/src/gates/tests/dependency_tree.rs
  - ../realm/src/net/Cargo.toml
  - ../realm/src/net/src/lib.rs
  - ../realm/src/net/tests/unit/lib.rs
commit: { realm: c862aec + c239677, shared: 795f1df + ad1b3b4 }
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

- [~] **`Scope` has three real sites**, all hand-rolled reverse-order undo:
      `src/drivers/linux/src/overlay.rs` (`unmount_all`, line 385 — "Undo every
      mount `assemble` made, innermost first"),
      `src/drivers/linux/src/zfs_volumes.rs:139` ("Leave nothing behind: undo
      this call's own work"), `src/net/src/lib.rs:437` ("Cleanup netns on
      failure").

      **Struck — the adoption landed in `../realm`, a tree this board does not
      own and cannot re-verify as its own.** Commit `c862aec` ("p5-adoption/realm
      — realm adopts conserved's Scope, Clock and id vocabulary", 2026-08-26),
      with `c239677` after it.

      **The line numbers in this box are pre-adoption and must not be read as
      current.** They were the 2026-08-21 hand-rolled-undo sites; `c862aec`
      rewrote exactly those call sites, which moved them. Measured in realm
      2026-08-28 — three files and three imports `reproduced`, the three line
      numbers `refuted`:
      ```
      src/drivers/linux/src/overlay.rs:80     use conserved::scope::{Scope, Undo};
      src/drivers/linux/src/overlay.rs:422    let scope = Scope::new();
      src/drivers/linux/src/overlay.rs:450    let scope = Scope::new();
      src/drivers/linux/src/zfs_volumes.rs:47   use conserved::scope::{Scope, Undo};
      src/drivers/linux/src/zfs_volumes.rs:237  let scope = Scope::new();
      src/net/src/lib.rs:51     use conserved::scope::{Scope, Undo};
      src/net/src/lib.rs:1122   let scope = Scope::new();
      ```
      The current sites are `overlay.rs:422` and `:450`, `zfs_volumes.rs:237`
      and `net/src/lib.rs:1122` — **four** `Scope::new()` call sites across the
      three files this box names, not three: `overlay.rs` took two. The box's
      wording is left as written, because a strike records a bar this board did
      not clear rather than moving the bar.
- [~] **`Clock`: 3 non-test wall-clock reads** — `drivers/linux/src/state.rs:140`,
      `cli/src/lib.rs:1025`, `cli/src/state.rs:61`. Everything else is monotonic
      (`net/src/dns.rs` TTL cache, `zfs/src/cli.rs:437` deadline,
      `drivers/linux/src/lib.rs:1038-1054` timeout) and must NOT be converted.

      **Struck — the conversion is realm's and the count that keeps it is
      realm's ratchet, not this board's.** The **3** this box names is the
      *before* count, measured 2026-08-21. After `c862aec` it is **0**:
      `refuted` by the landing commit `e0e4cdd`, whose own message records
      *"realm holds 0, not 3"*. All three named sites now read through
      `conserved::SystemClock`, measured in realm 2026-08-28:
      ```
      src/drivers/linux/src/state.rs:212  conserved::SystemClock.now().as_unix_secs().max(0) as u64
      src/cli/src/state.rs:93             let now = conserved::SystemClock.now().as_unix_secs()...
      src/cli/src/lib.rs:1337             conserved::SystemClock.now().as_unix_secs().max(0) as u64
      ```
      Zero direct wall-clock reads remain. What holds it at zero is
      `../realm/src/gates/tests/clock_read_ratchet.rs`, commit `01254aa`:
      `RATCHET_CEILING = 0` at `:55` and `WALL_CLOCK_READS = ["SystemTime::now"]`
      at `:41`, so any new `SystemTime::now` in realm turns that gate red. Re-run
      in realm 2026-08-28:
      ```
      $ cargo test -p realm-gates --test clock_read_ratchet
      running 3 tests
      test monotonic_reads_are_never_counted ... ok
      test a_walk_that_scans_nothing_fails_rather_than_passes ... ok
      test wall_clock_reads_may_only_decrease ... ok

      test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```
      The monotonic sites this box protects are protected by construction:
      `Instant::now` is absent from `WALL_CLOCK_READS`, guarded by
      `monotonic_reads_are_never_counted` at `:223`.
- [~] **Record the `ContentId`/`stats` refusal** with the grep evidence, so the
      next reader does not re-litigate it.

      **Struck — the refusal is recorded, and the evidence is a grep over
      `../realm`, which only realm's own tree can produce.** Re-run there
      2026-08-28 across `src/` and the root manifest, 74 `.rs`/`.toml` files
      plus `Cargo.toml` = **75 files**, `reproduced`:
      ```
      $ cd ../realm
      $ grep -rnE 'blake3|sha2|Sha256' --include='*.rs' --include='*.toml' src/ Cargo.toml
      EXIT=1
      $ grep -rnE 'median|percentile' --include='*.rs' --include='*.toml' src/ Cargo.toml
      EXIT=1
      ```
      Both exit 1 — **no match, no output**. `ContentId` and `stats` have no
      call site in realm, so adopting either would fail admission criterion 1
      (*both trees need it today, not speculatively*). This is the record the
      box asks for, quoted here rather than pointed at. Nothing in this
      repository can close it, because nothing here can re-run a grep over
      realm's tree — which is why it is struck, not ticked.

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

## Blocked — 2026-08-26

The work is done and verified. Three boxes stay open, none of them this
node's to close.

| box | what closes it |
|---|---|
| `spec01` — `cargo fmt --all --check` clean | `@realm/16-fmt-gate`. Red on `HEAD` before this run, on `src/gates/tests/{done_boxes_are_ticked,one_vocabulary}.rs` and `src/cli/tests/unit/lib.rs` — files outside this footprint. Every file this node touched is `rustfmt --check` clean |
| `spec02` — same command | the same PRD, the same three files |
| `spec01` — the footprint box | nothing. The footprint was wrong and is corrected in the frontmatter above, per `@references/parts/commits.md`: a path the worker wrote outside its footprint is a wrong footprint, committed with the rest and said out loud |

**The footprint deviation, named.** Four files beyond the specs' declared
paths: `src/cli/tests/unit/lib.rs`, `src/drivers/linux/tests/unit/state.rs`,
`src/net/tests/unit/lib.rs` and `src/gates/tests/dependency_tree.rs`. The
first three were forced — realm's `source_layout` gate forbids inline
`mod tests`, so the unit tests those specs' acceptance boxes demand cannot
live in the source files the specs listed. The fourth is the `OWNERS` entry
the `dependency_tree` gate required once `conserved` entered the tree. Each
was recorded by the implementer rather than argued away.

**Verified on a real Linux kernel**, not merely cross-checked: unprivileged
`rust:1-bookworm`, `89 / 35 / 41 / 9 / 2`. The 89-vs-92 gap is three
pre-existing `cfg(not(linux))` refusal tests, diffed name by name.

**One finding is filed, not fixed:** the `conserved` pin resolves from a
**private** remote, so this tree no longer builds on a machine without the
user's credentials. `@infra/shared-remote-is-private`, which names
`@shared/p5-adoption/mitosys` as the PRD it would otherwise get wrong.


## Frontmatter repaired — 2026-08-28, by the board

`  - "@realm/16-fmt-gate"` sat as a bare list item directly under
`blast-radius: mid`, with its own key missing. **Any YAML reader saw no
dependency at all** — the edge existed only as an indented string under a
scalar. Found by `@shared/exemptions-name-their-reason`'s analyst.

Restored as `needs:`, which is the relation the body already describes: the
`spec01` acceptance row at line 87 defers its `cargo fmt --all --check` result
to `@realm/16-fmt-gate`, naming files outside this node's footprint that were
red on `HEAD` before the run. That is a dependency, not an ordering hint.

This node is `state: done`, so nothing about dispatch moves — the repair is to
the record. Recorded rather than done silently because the key was **inferred
from the body**, not read from anywhere: if the original author meant `after:`,
this is the line to correct.
