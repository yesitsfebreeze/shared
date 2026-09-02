---
type: learning
learning: done-means-done
subject: a `done` PRD requires evidence per box, and a box that cannot close is struck with a reason rather than ticked
binds: [mitosys, model, realm, shared]
status: decided
date: 2026-08-24
code: .pearde/prds/done-means-done/prd.md
---

# done-means-done — the family rule for what `state: done` means

`state: done` is the board's only claim that work exists. A PRD in that state
is the board asserting, by its current frontmatter, that every acceptance box
holds. The board cannot tell from the outside whether a `- [ ]` in a
`state: done` PRD is a record-keeping lapse (the box is closed, the tick is
owed) or a real gap (the work was never done). Both are corrosive, and the
rule has to make the two cases distinguishable to a reader with neither the
git log nor the worker's report in hand.

## What evidence a tick requires

A tick is the form `-[x]`. The evidence it carries, with no exception:

- **the verify output, quoted.** The PRD's own `verify:` line, run literally,
  with the output pasted under the box or above it. `cargo test ... → 55
  passed, 0 failed` is the shape; "`cargo test` was green" is not. A box
  without output is not closed; the worker is owed the evidence, the board
  is owed the proof.
- **or the gate that proves it.** When the PRD's verify is a gate (a test
  that runs under `cargo test --workspace` / `just check`), the gate's name
  and the assertion it makes are the evidence. Quoting `cargo test -p
  realm-gates --test done_boxes_are_ticked` passing is enough for any box
  whose truth the gate asserts.

The shape is identical across trees. A tick on a `mitosys` box looks the
same as a tick on a `model` box, a `realm` box, and a `shared` box: a
quoted verify output or a named gate. No tree has its own verb.

## The three forms an unclosable box may take

A box that cannot close — the work was not done, the gate fails, the proof
does not exist — takes one of three forms. All three convert a visible gap
into a recorded one, which is the point: a recorded gap is closable; an
invisible one is not.

- **Strike-with-reason.** The box's `- [ ]` is rewritten to a struck form
  with the reason recorded beside it. The precedent is two boxes on
  `realm/02-linux-driver` (`VolumeDriver::Bind`/`Tmpfs` only-in-tests and
  `realm create --disk N` → `quota=N` owed to `03-zfs-driver`): the box
  stays on the PRD as `- [ ]` with a measurement note explaining why it
  does not close here and which other node owns the closure. Used when
  the work belongs elsewhere and the local PRD cannot move it.
- **`blocked` with `needs:`.** The PRD's `state` flips from `done` to
  `blocked` and the frontmatter carries `needs: <event>`. Used when the
  box is closable as soon as a recorded event lands (a Linux run, a
  feature gate landing, a sibling PRD closing).
- **Child PRD.** A new PRD, `state: open`, on the same tree's board, with
  the unclosable box's work as its body. Used when the work is a real
  follow-on that another node can be claimed against.

A tick on an unclosable box is not a fourth form. It is a defect: it
converts a visible gap into an invisible one, and the gate in specs 02/03/04
exists to make this defect impossible by reading `- [ ]` literally rather
than by trusting the frontmatter.

## Why the rule binds four trees and not one

The rule is the family's, not any tree's: a `done` PRD with a `- [ ]`
acceptance box is the same defect in `mitosys`, `model`, `realm`, and
`shared`. A per-tree fix would settle the four current cases and leave the
rule unwritten, so the next twelve would land the same way. The four gates
in specs 02/03/04 and the child PRDs in specs 05/06/07 make the rule a
gate so it cannot decay back into the unwritten form.

## Cross-tree gate 4 (future)

`shared/learnings/README.md` names four gates the family carries. Gate 4
("named symbols agree across trees") is not yet written; when it lands, it
will read the tick form this learning records against each tree's own tick
form and fail if the verbs diverge. The shape above — quoted verify output
or named gate, no tree-specific verb — is the form gate 4 will require.

## Addition 2026-08-27 — which boxes the gate counts

This learning says a `done` PRD carries no unticked box. It never said which
boxes, and the three gates it shipped answered differently: `realm` whole-file,
`mitosys` and `shared` under `## Acceptance` only. `resources/board/plan.py`
answered from a fourth population — `specs/*.md` — and it is the one the board
dispatches on.

**`shared/learnings/exemptions-name-their-reason.md` settles it: the whole
`prd.md`, every heading, `- [~]` read as a closure.** Decided by the user
2026-08-27, recorded as `.pearde/memos/done-counts-which-boxes.md`.

That document also carries what this one left unwritten:

| clause | where it lands |
|---|---|
| the population a gate counts | whole `prd.md`; `specs/*.md` stay out |
| what an `EXEMPT` entry owes | the PRD, the commit and the removal condition — all three, distinct per entry |
| which trees carry the gate | all four. `model` is the one that does not, today |
| the cost of widening | 25 `## Requirements` boxes over 7 `done` PRDs, 18 mitosys / 7 shared |
| what covers the master board | nothing, and it names the violation that stands |

The three forms above are unchanged by it. They are what closes each of the 25.
