---
complexity: 5           # pure documentation + a regression grep; no source touched
footprint:
  - prds/p5-adoption/realm/specs/spec03.md
---
<!-- This spec's own file IS the deliverable: the record the PRD's third
     requirement asks for. It touches no file under ../realm — there is
     nothing to adopt, so there is nothing to edit there. Footprint is this
     file alone; the grep commands below read ../realm/src but write nothing
     to it. -->

# spec03 — record the `ContentId`/`stats` refusal, regression-guarded

The parent `p5-adoption/prd.md` asks every consumer to "adopt `Scope` and
`ContentId` where applicable." For realm, `ContentId` (and, by the same
admission test, `conserved::stats`) is **applicable nowhere**: realm hashes
nothing into a persisted id and computes no percentile/median. Do not
manufacture a call site to give either a home. This spec is the permanent
record of that refusal, with the evidence, so a later reader does not
re-litigate it by re-running the search from scratch — and a check that fails
loudly the day the tree changes underneath this record.

Note on where this record lives: this PRD's own rules forbid editing
`prd.md`'s body outside its `## Questions` section, and forbid writing
outside this PRD's own folder. The refusal is therefore recorded here, in
this PRD's own `specs/`, not in `../realm` (nothing there needs a comment
pointing at code that does not exist) and not in `learnings/` (that folder is
for lessons about the shared crate itself, not a per-consumer decision about
one tree).

## The evidence, reproduced at analysis time

```sh
$ cd ../realm && grep -rniE "blake3|sha2|Sha256|ContentId" src/ Cargo.toml
$ cd ../realm && grep -rniE "median|percentile" src/
```

Both commands returned **zero matches** (checked against `src/`, the root
`Cargo.toml`, and every crate's `Cargo.toml` under `src/*/`). Reproduced
(realm tree, commit at analysis time — re-run before trusting this on a
later commit; that is exactly what the acceptance check below re-does).

Why zero matches settles it, per the parent's own admission criterion 1
("both trees need it today, not speculatively"):

- **No hashing.** realm's persisted ids are all either operator-supplied
  workspace names (`valid_workspace_name`, `cli/src/lib.rs`) or
  ZFS/dataset/snapshot names built by string formatting
  (`zfs_volumes.rs::owned_snapshot_name`, `clone_name`) — never a digest of
  content. There is no preimage anywhere for `ContentId` to wrap.
- **No statistics.** realm reports raw metrics (`oom_kills`,
  `drivers/linux/src/lib.rs`) and timestamps; nothing aggregates a
  distribution, so there is no caller for `min_median_max`/`percentile`.

## Acceptance

- [x] `grep -rniE "blake3|sha2|Sha256|ContentId" ../realm/src ../realm/Cargo.toml`
      (run from `../shared`) exits with no matches.

      Re-run 2026-08-26, from `/Users/feb/dev/infra/shared`, **after**
      `spec01` and `spec02` had landed `conserved` in every manifest:

      ```
      $ grep -rniE "blake3|sha2|Sha256|ContentId" ../realm/src ../realm/Cargo.toml
      $ echo $?
      1
      ```

      No output, exit 1 — grep's "no lines selected". Reproduced.

- [x] `grep -rniE "median|percentile" ../realm/src` exits with no matches.

      ```
      $ grep -rniE "median|percentile" ../realm/src
      $ echo $?
      1
      ```

      No output, exit 1. Reproduced.

- [x] If either grep above finds a match when this spec is next run, the
      check **fails** rather than silently passing — that is the signal that
      this refusal is stale and the requirement needs re-litigating with a
      real call site named, not a re-statement of "still nothing here."

      The `## Verify and Proof` block below is the check, and it fails by
      construction: each line is `! grep …`, so a match makes the command
      exit non-zero and the block stops before `echo "spec03 ok"`. Run whole
      on 2026-08-26 from `../realm`, it printed exactly:

      ```
      spec03 ok: no ContentId or stats call site in realm
      ```

      Negative control, so the check is known to fire rather than be assumed
      to: substituting `blake3|Scope` for the first pattern — `Scope` matches
      in `../realm/src` now that `spec01` has landed — makes the `! grep`
      exit 1 and the block stop before the `echo`. Reproduced 2026-08-26.

## One thing changed under this record, and it is not a call site

`spec01` and `spec02` add `conserved` to realm's manifests, and `conserved`
depends on `blake3`. `blake3` therefore now appears in `../realm/Cargo.lock`
(lines 126 and 252) as a transitive dependency of the `conserved` package.

`Cargo.lock` is in neither acceptance grep, and that is correct rather than
convenient: the refusal is about **call sites**, and the admission test the
parent PRD sets is "both trees need it today". realm reaches for
`conserved::scope` and `conserved::clock`; nothing in `../realm/src` names
`ContentId`, and `conserved`'s own `default = []` feature set does not change
that. A hash function reachable through a dependency graph is not a call site.

The day `../realm/src` names `ContentId`, the first grep matches and this
record fails — which is the whole point of it being a check and not a
paragraph.

## Verify and Proof

```sh
cd ../realm
! grep -rniE "blake3|sha2|Sha256|ContentId" src/ Cargo.toml src/*/Cargo.toml
! grep -rniE "median|percentile" src/
echo "spec03 ok: no ContentId or stats call site in realm"
```
