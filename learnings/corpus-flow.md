---
type: learning
learning: corpus-flow
subject: mitosys produces correction signal as exhaust; model reads it through its existing learn door, with LearnOrigin filtering at learn_trains and an upstream-ModelOutput-ancestry gate keeping the door safe from self-training
binds: [mitosys, model]
status: decided
date: 2026-08-24
code: mitosys src/mitosys/engine/record/door.rs:55, model src/node/mod.rs:48, model src/node/mod.rs:69
---

# Corpus flow — mitosys's record becomes model's corpus

Decided 2026-08-24 (`prds/corpus-flow`). The seam already named by
[[two-halves]] §The seam (the corpus-signal row) and §Sequencing step 7
("admitted after the origin vocabulary and the record shape converge") is
now admitted: steps 4 (`model/prds/one-learn-origin`) and 6
(`mitosys/prds/record-shape-port`) are both `state: open`, and this
document settles direction, mechanism, vocabulary admission, refusal
enforcement, and the filtering rule.

## Direction and mechanism

**Model reads. mitosys does not write model's door, and no third daemon
moves records between them.**

The mechanism: **model's learn door is the only guarded entry point, and
a reader on that door is the cheaper shape** — the record is append-only
and every view is a pure fold ([[two-halves]] §A), so model can pull
admitted records through the existing door rather than having a writer
bypass it. The reader is `model::record::fold` over
`mitosys::engine::record::door`'s replay door — the read door
`engine/record/door.rs:55`'s `doors(log)` already exposes, the same one a
Rust plugin reads through and the Lua host republishes as `mito.record`
(`engine/record/door.rs:57`). Each record that crosses is one
`LearnOrigin`-tagged `Entry` from the replay stream; the receiving
`Node::learn` path on model's side calls `learn_trains` first and only
then schedules a training step.

The doors on each side, named:

| side | site | role |
|---|---|---|
| producer | `mitosys/src/mitosys/engine/record/door.rs:55` (`doors(log)`) | the read door; the replay door that returns `Entry`s through the `replay` impl |
| producer | `mitosys/src/mitosys/engine/record/door.rs:59` (constructor `doors`) | the *one* construction site for both Rust plugins and the Lua republish — the diagram-vs-program distinction the door file already enforces |
| consumer | `model/src/node/mod.rs:48` (`LearnOrigin`) | the data doctrine; six variants, one of them refused |
| consumer | `model/src/node/mod.rs:69` (`learn_trains`) | the door-side check; returns `false` for `ModelOutput` |

mitosys never classifies an `AgentTurn` outcome as `ModelOutput` —
mitosis has no agent loop by design (`CLAUDE.md` §"What this repo is",
"*no agent loop; intelligence lives in the agents*"), so its `AgentTurn`
events are *by construction* either `Prompt`, `Correction`, `Teacher`,
or `Peer`. The producer-side invariant: **no `AgentTurn` outcome
ever carries `LearnOrigin::ModelOutput`**, so the door-side check
(`learn_trains(ModelOutput) == false`) is the second line of defense
for a value the producer never mints.

## The admitted vocabulary

The node six — `Corpus · Prompt · Correction · Teacher · Peer ·
ModelOutput` — from `model/src/node/mod.rs:48`. Per
[[two-halves]] §"One `LearnOrigin`" this is the canonical set, and the
mapping of `Human` and `Tool` is settled:

| current `record/mod.rs:94` variant | site | canonical | survives as |
|---|---|---|---|
| `Human = 0` | `src/record/mod.rs:94` | retired | a stored `0` decodes as `Prompt` — the least a `Human` byte proves |
| `Corpus = 1` | `src/record/mod.rs:94` | `Corpus` | byte `1` keeps its meaning |
| `Peer = 2` | `src/record/mod.rs:94` | `Peer` | byte `2` keeps its meaning |
| `Tool = 3` | `src/record/mod.rs:94` | retired | a stored `3` decodes as `Teacher` — an external answer, human-verifiable at its source |
| `ModelOutput = 4` | `src/record/mod.rs:94` | `ModelOutput` (refused) | byte `4` keeps its meaning |

`Human` → `Prompt` and `Tool` → `Teacher` are byte-level collapses
[[two-halves]] §"One `LearnOrigin`" already authorized, and
`model/prds/one-learn-origin` carries the format consequence.
This learning does not change the mapping; it adopts it as the
producer-side classifier. A `Human`-tagged event arriving at the door
is tagged `Prompt`; a `Tool`-tagged event is tagged `Teacher`. The
`Corpus` byte keeps its identity on disk and at the door.

## The `ModelOutput` refusal rule

`learn_trains` (`model/src/node/mod.rs:69`) returns `false` for
`ModelOutput` and `true` for every other variant — paper §11.2
("self-training amplifies systematic error", measured 2% gain from
self-training vs 85% from a teacher). This is the **door-side
enforcement**, and it is the only place the door refuses; the
`Node::learn` caller checks `learn_trains` before tokenizing or
scheduling a step.

The **producer-side invariant** is what makes this safe rather than
circular: mitosys never classifies an `AgentTurn` outcome as
`ModelOutput`, because it has no model output to classify — the
`AgentTurn` events in `engine/record` are already the discriminator
between *what an agent proposed*, *what a human accepted*, and *what a
gate then confirmed*. The provenance is on the record; the door reads
it; the refusal sits where it has always sat.

Both lines stand. The door-side check is the structural guarantee that
*no value tagged `ModelOutput` reaches the gradient update path*. The
producer-side invariant is the empirical claim that *mitosys never
mints such a value*. Together they make the refusal a property of the
record rather than a property of the reader.

## The filtering rule

**A `Correction` whose upstream `Prompt` carried no `ModelOutput`
ancestry is admitted; provenance sufficient to prove that ancestry
must already be on the record.** A gate pass on a vacuous scan is
noise; a gate pass on a discriminator the record already carries is
signal.

The rule applies one step at a time, walking the provenance chain
backwards from the `Correction`:

1. The `Correction` entry carries its `upstream` pointer to the `Prompt`
   it corrected (mitosis already records this — every `Correction`
   event names the prompt it fixed).
2. The `Prompt` entry is read; if it carries no `ModelOutput`
   ancestry, the `Correction` is admitted.
3. "No `ModelOutput` ancestry" means: the `Prompt` itself is not a
   `ModelOutput`, and none of its referenced `AgentTurn` events
   are `ModelOutput`. The chain terminates at a `Prompt` typed
   directly as a user keystroke (a `Human` byte, decoding as `Prompt`)
   or at a `Teacher` (an external answer, verified at its source).

This filter is why the seam's value is asymmetric in model's favour
([[two-halves]] §A): the discrimination "did a model participate in
producing this signal" is what model needs to filter on and would
otherwise have to reconstruct. The record already carries it.

Three classes of `Correction` are filtered out by this rule and never
reach `learn_trains`:

- `Correction`s whose upstream `Prompt` was itself a model output
  (a user copy-pasted a model's reply and corrected it — the model's
  error is already baked in; admitting the correction does not
  unbake it).
- `Correction`s whose upstream `Prompt` referenced a `ModelOutput`
  `AgentTurn` (a tool was given a model's prior answer as context —
  the chain is tainted).
- `Correction`s with missing or unresolvable upstream provenance
  (the record shape has not converged — handled by
  `mitosys/prds/record-shape-port`).

The remaining class — `Correction`s over `Prompt`s that trace back to
a human keystroke or a `Teacher` — is what model admits.

## Why a reader, not a writer

A writer — mitosys pushing records into a door it does not own —
bypasses the only guarded entry point. A reader — model pulling
through the door — sees every record `learn_trains` would have refused
refused *by the door*, with the refusal counted at the door
(`Node::learn` returns `Err(LearnRefusal::ModelOutput)` at
`src/node/mod.rs:319`, named, never silent). A writer cannot
add that count without re-implementing it; a reader gets it for free.

The cheaper shape is also the right shape on memory: a reader folds
over the existing record, never copies, never duplicates. The
`engine/record` design — append-only, every view a fold — is the
mechanism that makes the read the natural operation.

## Alternatives considered

A decision with no rejected road cannot be told apart from an
assumption. Per `pearde/references/memo.md` §"Alternatives is not
optional", the rejected roads are recorded here, one per fork, with the
reason each lost.

### Direction and mechanism

- **A third daemon that tails both stores and mirrors records.**
  Rejected: a third process is a third gate runner, a third board, and
  a third toolchain pin — exactly the cost `shared`'s "where it
  lives" decision already measured and refused ([[shared-crate]] §Where
  it lives). Two consumers and one daemon is three things to keep
  honest, and the mirror has to know both door shapes; the reader
  knows only the door shape it is reading.
- **mitosys writing model's door directly (a writer).** Rejected: a
  writer bypasses the only guarded entry point — model's `learn_trains`
  is the door's structural guarantee, and a writer that does not pass
  through it cannot inherit it. The provenance claim "this signal was
  never a model output" is not provable without the door's check.

### Admitted vocabulary

- **Widening to admit `Tool` as a first-class learnable origin.**
  Rejected: `Tool` is exactly the byte the seam's mapping retires
  ([[two-halves]] §"One `LearnOrigin`"), because every external tool
  answer is verifiable at its source — `Teacher`'s definition
  (`model/src/node/mod.rs:55`). Two names for one concept reintroduces
  the duplication the seam already collapsed.
- **Narrowing to refuse `Teacher`.** Rejected: `Teacher` is the
  external-answer origin (paper §4.1, Table 4); refusing it discards
  the verifiable-by-source signal class, which is precisely the
  signal the corpus-flow exists to admit. A `Correction` whose
  upstream was a verified `Teacher` is *more* trustworthy than one
  whose upstream was a `Prompt`, not less.

### Filtering

- **Admit every `Correction` regardless of upstream `ModelOutput`
  ancestry.** Rejected: the door-side `learn_trains` check is a
  guarantee at one level of the chain; without the upstream check,
  the `Correction`'s upstream `Prompt` may itself be `ModelOutput`
  and the discrimination the filter exists to make is lost. Admitting
  every `Correction` reduces the door's refusal to a per-event
  nit, which the producer-side invariant already handles; the value
  the filter adds is the chain-level check.
- **Require a manual review before any record crosses.** Rejected:
  this is what the synthetic teacher in `model/src/teacher/` already
  does, by manufacturing synthetic signal because no real signal
  crosses — and the harness where a human corrects an agent produces
  correction as exhaust ([[two-halves]] §A). A manual review gate
  defeats the exhaust production. The provenance chain is the
  mechanical substitute; the human's role is the original correction,
  not the gate.

## Cross-references

- [[two-halves]] §A — the coupling this implements
- [[two-halves]] §"One `LearnOrigin`" — the byte-level mapping this
  learning adopts as the producer-side classifier
- [[two-halves]] §Sequencing step 7 — the deferral this admits
- [[record-shape]] — the upstream provenance chain depends on
  per-event content ids; `record-shape-port` is the carrier
- [[divergences]] §The evidence — the duplicate `LearnOrigin` is what
  the seam collapsed; `one-learn-origin` is the format consequence
- `prds/corpus-flow/` — the master PRD
- `mitosys/prds/corpus-flow-producer/` — the producing half (child)
- `model/prds/corpus-flow-consumer/` — the consuming half (child)
