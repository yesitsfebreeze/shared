---
state: done
mode: afk
priority: 46
est: 2h
repo: shared
verify: "grep -q '^status: decided' learnings/shared-crate.md && grep -q '^status: decided' learnings/clock.md"
---

# P5f — close: flip the learnings this board settled

Purpose: `AGENTS.md`'s "Close" step. No node on this board flipped the
learnings it settled; the gantt audit already records this under "Lost work".
Per `AGENTS.md:149` this links the landing commits, so it runs **last**.

## Requirements

- [x] **`learnings/shared-crate.md`** — `partial` → `decided`, linking the
      landing commits.
- [x] **`learnings/clock.md`** — `open` → `decided`, and its `code:` line plus
      §"Honest scope" gain `../model/src/node/transactional.rs:72`, the second
      live-clock-into-a-content-hash the learning never named.
- [x] **`README.md:39-40`** renders both statuses and goes stale with them.
- [x] **Decide the correction mechanism and say which was chosen.**
      `learnings/README.md`'s own rule is that corrections are new documents, not
      edits that erase. So `clock.md`'s incompleteness is either an edit here or
      a superseding learning — and if the `llm` node needs the correction
      earlier than this node runs, it cannot wait for this node.

## Landed

Specs ran in the order the ticket required: spec01 -> spec02 -> spec03 ->
spec05 -> spec04. All five `verify:` lines pass, plus the ticket gate,
`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings`
and `cargo test --workspace` (79 passed).

**The mechanism chosen** (spec01, `learnings/README.md` §"What is enforced" ->
§"How a document is corrected"): *correction-by-edit in place for additions and
factual corrections; supersede-by-new-document for reversals of a decision.*
The folder has been version controlled since `ab154f7`, so a correction now
shadows its predecessor in `git log -p` rather than overwriting it — which is
the condition the README itself named as the one that would lift the ceremony.
An edit adds; it never deletes the sentence it corrects.

Deviations:

- **spec03 box 6 is `[~]`.** Its second clause ("`Clock` and order statistics
  are not yet extracted") went stale between speccing and running: p3 and p4
  landed both in the crate. The intent — a `decided` that cannot be misread as
  "the extraction is finished" — is met by §"What is still outstanding", which
  names what actually is: no consumer adoption in any of the three trees, no
  ratchets anywhere, nothing pushed to the remote.
- **`learnings/README.md` §"Contents" was also corrected** (in spec04's step,
  outside every spec's Files list): it listed [[clock]] and [[shared-crate]]
  under "**Open** — found, argued, not yet settled", which this ticket made
  false. Both moved under "**Decisions**", each carrying what is still
  outstanding. Same staleness as `README.md`'s status table, one directory
  over.
- **spec01 also adjusted one lead-in sentence** in §"What is enforced" ("Two
  specific limits behind that:" -> "Two specific limits stood behind that; the
  second has since been lifted:"), because the bullet it introduces is no
  longer a limit. The rules themselves are a `###` subsection of that section.
