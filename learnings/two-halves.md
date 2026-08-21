---
type: learning
learning: two-halves
subject: mitosys is the record-half and llm is the learner-half of one system; the join is law 1, and the three couplings in order of value are corpus, grading, and the reload seam
binds: [mitosys, llm]
status: open
date: 2026-08-18
code: mitosys src/mitosys/engine/record/, llm src/record/
---

# Two halves of one system

Measured 2026-08-18, both trees at that day's HEAD: **zero shared source.**
Every `.rs` in both trees hashed — 238 files in llm, 351 in mitosys, 76k and
108k lines — and **not one file is identical**. What is shared is doctrine,
process, and shape, arrived at twice.

## The structure

- **mitosys** is the record and the harness: memory, provenance,
  orchestration, plugins, surfaces. It has no intelligence *by design* —
  "no agent loop; intelligence lives in the agents."
- **llm** is the learner and the substrate: a model, a peer mesh, an
  improving loop. It has no harness, so it hand-rolled a chat window, a
  config loader, a daemon and a teacher to have somewhere to live.

Neither is complete alone, and law 1 is already the join: both are built on
one append-only record with every view a pure fold over it. That is not a
convenient similarity to exploit later — it is load-bearing on both sides
today.

## The three couplings, in order of value

### A. mitosys's record is llm's corpus

The strongest one, and asymmetric in llm's favour.

llm's hardest problem is where human-sourced signal comes from. It built
2.4k lines of `src/teacher/` to manufacture it off local ollama, and says
why plainly: *"The learner's signal has to come from somewhere
human-sourced. The chat agent's door is a person typing and the loop
harness's door is a synthetic stream of blake3 digests — deterministic, but
not language."*

mitosys **generates** that signal continuously and already stores it with
provenance: every agent turn, every tool result and its outcome, every board
claim and release, every gate pass and fail, every memo naming what it beat.

llm's one door already has the exact origins to receive it —
`LearnOrigin::Prompt` (a person typed), `Correction` (a person fixed an
answer), `Teacher` (an external answer, human-verifiable at its source).
Correction signal is the most valuable kind and the hardest to buy; a
harness where a human corrects an agent produces it as exhaust.

The refusal rule is what makes this safe rather than circular:
`ModelOutput` is refused at the door because self-training amplifies the
model's own error. mitosys's record already distinguishes what an agent
proposed from what a human accepted from what a gate then confirmed —
precisely the discrimination llm needs to filter on, and would otherwise
have to reconstruct.

### B. each tree has the gate the other lacks

| | llm | mitosys |
|---|---|---|
| perf/behavioural grading | `src/grade/` (8.0k lines): `grade.json` timing envelopes, measure/probe/report, pass·regression·fail | none — conformance replay only |
| structural gates | `just check` is a bare `cargo check` | `source_layout.rs`, `dependency_tree.rs`, one-namespace-one-crate, conformance replay |
| improving loop | `src/improve/` (5.8k lines): measure → judge → build → swap → verify → keep-or-rollback | none |

Exactly complementary, which makes this the cheapest real exchange
available and the one to do first. See [[storage]] for why it is also a
*prerequisite* rather than a parallel nice-to-have.

### C. llm's reload seam is mitosys's plugin host

`interface/` + `src/reload/` is a **proven** dylib swap at a tick with host
state untouched: ABI version constant, a layout fingerprint checked at
load, last-good retained on a broken load, and the macOS dyld
generation-path rule handled (never load the same path twice).

mitosys's `api/surface/abi.rs` loads once and never swaps.

Sharing this gives mitosys hot-reloadable plugins and saves llm from
inventing a plugin host. llm's `docs/decision-record.md` §1 — what may
cross the seam, what may not, and why same-rustc layout identity beat both
a full C ABI and an interface-crate-as-shared-dylib — is the best single
artifact either tree has on this subject, and it is true of both.

## Where shared code should live

**mitosys owns the shared crates; llm depends on them.** Not a third
repository — another board, another gate runner, another toolchain pin, for
two consumers. And not copy-and-sync, which is what is happening now: the
`~/dev/llm` citation in `mitosys`'s `engine/record/stream.rs:23` is a
dependency written as a comment.

llm's one-binary law is untouched by this; these are libraries, not
binaries.

**The test for what qualifies already exists and is mechanical:** if a
crate can pass mitosys's `gates/tests/dependency_tree.rs`, it is shareable.
The gate that keeps a database and a UI toolkit out of mitosys's core is
the same gate that keeps candle and libp2p out of anything llm shares back.
No judgment call is needed — the gate answers.

First candidates, all dependency-light today: `util/effect`
(install-with-inverse, imports nothing), the event spine, `engine/record`'s
fold and replay, the reload `interface` + loader, content-addressed ids.

## What must not converge

llm's thesis — self-specialization, the precision ladder, seed/leech,
perturbation-pair training — and mitosys's ACP/surface/board stack.
Convergence that reaches into either produces one slow project instead of
two fast ones.

## Sequencing

Both trees are mid-flight: mitosys's engine merge/fold rewrite is "decided,
not done", llm is on phase-8 real-peers. Nothing here should stall either,
and the repositories should not be merged.

1. Settle the four contradictions in [[divergences]].
2. Port `gates/` to llm — cheapest high-value move, and it finds llm's
   duplicate `LearnOrigin` rather than anyone taking it on faith.
3. Extract `util/effect` as the first shared crate. It imports nothing,
   which is why it was mitosys's first extraction and why it should be the
   first shared one: it proves the mechanism on something that cannot fail
   interestingly.
4. Port the grade harness into mitosys as a plugin-grading gate.
5. Then the record convergence, and only then llm behind `LlmFunc`.

That last step is the easy one and worth not mistaking for the goal:
mitosys's `engine/model` is already the right seam — two closure types,
`Fn(&str) -> String` and `Fn(&str) -> Result<Vec<f32>, String>`, no HTTP,
no vendor, imports nothing. llm satisfying it means it can back mitosys's
`reason`/`embed` with no wiring change anywhere. The plumbing is trivial;
the co-evolution loop lives in steps 1–4.
