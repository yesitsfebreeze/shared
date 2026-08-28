# learnings — what is true of more than one project

Four trees under `~/dev/` are built on the same laws and keep independently
arriving at the same shapes. Until now a fact true of two of them had two
homes, or one home and a stale copy — `mitosys`'s
`engine/record/stream.rs:23` cites `~/dev/llm`'s event spine in a doc
comment, which is a citation standing where a shared document should be.

This folder is that shared document, and nothing more: prose, on disk, that
every consumer points at.

## The consumers

| tree | what it is | doctrine it runs |
|---|---|---|
| `~/dev/mitosys` | the harness: record, orchestration, plugins, surfaces | four laws (`.mi/skills/process/laws.md`) |
| `~/dev/llm` | the learner: model, peer mesh, improving loop | seventeen rules (`DOGMA.md`) |
| `~/dev/zirkle/kern` | ancestor of mitosys's memory engine | `AGENTS.md` only |
| `~/dev/zirkle/model` | llm's own reference implementation | `AGENTS.md` only |

Four, not two. A bilateral mitosys↔llm bridge would need rebuilding the
first time zirkle is folded in, so nothing here assumes two.

## The admission rule

> A document lives here **if and only if it is true of more than one
> consumer**. True of one → it stays in that project.

That is law 1 applied across trees: *a rule is written once, at the most
general scale where it is true; the specific document carries only what
changes there.* The failure mode this rule exists to prevent is the junk
drawer — a folder that accumulates whatever had no obvious home, which is
a second source of truth wearing a helpful name.

**In:** the laws themselves, the board/worker protocol, the test law, the
toolchain and dependency pins, the record/fold shape, the reload seam, the
storage engine, shared vocabulary.

**Out:** llm's thesis (self-specialization, the precision ladder,
seed/leech, perturbation-pair training), mitosys's ACP/surface/board
internals, anything naming one tree's crate layout.

## The form

Same gene form mitosys memos use — `---` frontmatter declaring a `type:`,
then the fields that type requires — so a document here is readable by the
same parser that reads `.mi/docs/memos/`, rather than being a second
dialect of one idea.

```
---
type: learning
learning: <slug>
subject: <one line: what is true, not what the document is about>
binds: [mitosys, llm]
status: decided | open | partial
date: 2026-08-18
code: <file:line in each bound tree, where there is code>
---
```

`requires` is `subject` and `binds`. Everything else is optional.
`supersedes` / `superseded_by` link between documents.

**Why `learning` and not `memo`.** mitosys's memos plugin already owns the
`memo` type (`src/plugins/memos/.mitosys`), scoped to `.mi/docs/memos/`.
Two owners for one type name is the one-namespace-one-owner problem that
tree already has a gate against. A learning is a memo that binds more than
one consumer — a genuinely different type, read by the same parser. That
is what the open type set is for.

`binds:` is the field a memo does not have and this needs: it is what makes
the drift check below possible at all.

## What is enforced

**Nothing.** No gate reads this folder.

Stated plainly rather than left to be discovered, because law 3 is exactly
that a rule nothing runs is a wish — so every document here is currently a
wish, and should be read as one. Two specific limits stood behind that;
the second has since been lifted:

- The folder sits outside every consumer's tree. mitosys's dev container
  bind-mounts the repo and nothing else (`.mi/SYSTEM.md`, "no bind mount
  beyond the repo itself"), so a check over this folder can only ever run
  on the host — never in the container, never in CI as those are set up
  today.
- The folder **is** version controlled, as of `ab154f7` — `p0-foundation`'s
  first requirement, which quoted the sentence this bullet replaces as its
  reason. So "the record only grows" now has a subject here: a correction
  **shadows** its predecessor instead of overwriting it, and
  `git log -p learnings/<doc>.md` is where the predecessor lives, permanently.

### How a document is corrected

Two rules, and the boundary between them is the whole of it:

1. **An addition or a factual correction is an edit in place.** It reverses no
   decision, and git holds what it replaced. An edit *adds*: it never deletes
   the sentence it corrects — it extends, qualifies or resolves it, so a
   reader sees both the claim and its correction on disk rather than having to
   go to `git log -p` to learn there was one.
2. **A reversal of a decision is a new document**, linked by `supersedes:` /
   `superseded_by:`. `AGENTS.md`'s prohibition is scoped by its own first
   clause — *"edited to erase a **decision**"* — and that is the case the
   supersede ceremony exists for: the record of what a decision beat has to
   survive the decision changing.

Rule 1 is usable by anyone at any time; rule 2's reciprocal links are a
ceremony, and a correction that has to wait for a ceremony is a correction
that does not get made. That asymmetry is deliberate: the cheap path is the
one that only adds.

This section is itself a correction of this README under rule 1. The sentence
it replaced — the claim that this folder was unversioned, naming `git init` as
the one command that would lift the rule — is still readable in
`git log -p learnings/README.md`, which is the rule demonstrating itself.

## What a gate would check, if one is ever written

1. Frontmatter valid for the declared type; every name in `binds:` is a
   real consumer.
2. Every `[[link]]` resolves.
3. Every path in `code:` exists in the tree it names.
4. **For a document whose `code:` spans more than one tree, the named
   symbols agree.**

Check 4 is the one that would earn this folder its existence. It is what
catches [[divergences]]' finding that `LearnOrigin` is two different enums
inside one repo — one document declaring the origin vocabulary, `binds:`
naming both trees, and the check failing the moment they disagree. Without
it, this is a better-organised copy of the drift that is already there.

## Contents

**The audit** — what is actually built, on both sides:

- [[inventory]] — the capability matrix: what each tree has implemented,
  which four capabilities exist on exactly one side, and eight findings
  worth acting on

**What is true of both** — the shared shape:

- [[two-halves]] — mitosys is the record-half and llm the learner-half of one
  system; the three couplings, in order of value, and where shared code
  should live
- [[divergences]] — four rules that contradict between the trees and block
  shared code today, plus the evidence that doctrine without a gate drifts

**Decisions** — recorded with what they beat:

- [[record-shape]] — llm has already built the record mitosys's PRD calls
  decided-not-done, so the port direction reverses
- [[content-addressing]] — blake3 over SHA-256, `[u8; 32]` over hex `String`
- [[storage]] — redb over LMDB, sequenced behind mitosys's fold rewrite
- [[clock]] — both trees read the wall clock ~65 times each against a shared
  law that forbids it; the fix, and the ratchet that makes it affordable.
  Decided 2026-08-21; the type landed, the reads have not moved
- [[shared-crate]] — the concrete proposal for the crate: what goes in,
  what stays out, and where the code lives — the one constraint that had to
  be decided before the first line moved, and was. Decided 2026-08-21; no
  consumer has adopted the crate yet
- [[crate-name]] — the crate is named `shared`, not `conserved`: directory and
  package move together for three lines, live prose is renamed and `prds/` is
  not, and a vendored consumer build is a false green until the new rev is
  published. Decided 2026-08-28
- [[ratchet]] — one measurement discipline arrived at twice: model's floor
  and mitosys's ceiling are the same shape; each tree's missing half is
  filed on its own board
- [[exemptions-name-their-reason]] — a `done_boxes_are_ticked` gate reads the
  whole `prd.md`, and every `EXEMPT` entry names a PRD, a commit and a removal
  condition. Decided 2026-08-27
- [[a-shared-name-is-not-a-shared-function]] — a similarity score is a candidate finder, never a licence to merge: model's three `one_line`, mitosys's calendar pair at two widths, and model's scratch-name builders share a name or a body and not a function. Decided 2026-08-29

## Reading order

New to this: [[inventory]], then [[two-halves]]. Deciding something:
[[divergences]] first — four of its rows block everything else.
