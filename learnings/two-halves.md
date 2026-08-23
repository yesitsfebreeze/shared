---
type: learning
learning: two-halves
subject: mitosys and model are two halves of one system; the seam is the origin vocabulary plus four ported shapes, each type with one owner; the merge is a seven-step sequence, every step carried
binds: [mitosys, llm]
status: decided
date: 2026-08-23
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

## The seam

Which types cross, which tree owns each, how each crosses. Decided
2026-08-23 (`prds/two-halves-merge`).

| type | owner | crosses as |
|---|---|---|
| `LearnOrigin` | model | one declaration, one home (`model/prds/one-learn-origin`); moves into `conserved` only when mitosys ships signal — [[shared-crate]] criterion 1 admits today's need, never a speculative one |
| `ContentId`, `Clock`, `Scope`/`Disposer`, order stats | shared | `conserved`, landed — a rev-pinned git dependency in each tree ([[shared-crate]] §Where it lives) |
| record shape — preimage, bitemporal supersede, lazy decay, replay-as-fold | model | ported shape, not file: mitosys's `engine/record` adopts it, direction per [[record-shape]] (`mitosys/prds/record-shape-port`) |
| grade envelope — `Baseline`/`Grade`/`normalized_ms`/`pass_window` | model | ported shape per [[ratchet]]; the carrier is mitosys `prds/p9-perf-floor` |
| reload seam — `interface` + loader | model | its own crate when mitosys's swap work starts, not before ([[shared-crate]] §What stays out) |
| `LlmFunc`/`EmbedFunc` (`mitosys/src/mitosys/engine/model/lib.rs:48`) | mitosys | model satisfies the two closure types; zero wiring change on the mitosys side (`model/prds/back-llmfunc`) |
| corpus signal — mitosys record → model door | mitosys produces, model admits | a future master-board PRD, admitted after the origin vocabulary and the record shape converge (steps 4 and 6) |

### One `LearnOrigin`

model declares the seam's one word twice: `src/node/mod.rs:48` (Corpus ·
Prompt · Correction · Teacher · Peer · ModelOutput) and `src/record/mod.rs:94`
(Human · Corpus · Peer · Tool · ModelOutput, `#[repr(u8)]`, `try_from_u8`).
The canonical set is the node six — the doctrine set (paper §4.1, Table 4)
both doors claim to share:

**Corpus · Prompt · Correction · Teacher · Peer · ModelOutput**

Two constraints bound the choice, both already decided by the couplings:

- Prompt, Correction and Teacher stay distinct variants — coupling A filters
  on exactly that discrimination, and Correction is the signal mitosys
  produces as exhaust.
- `ModelOutput` stays a variant and stays refused: `learn_trains` returns
  `false` for it, and both doors count the refusal.

The mapping, every variant of both current spellings:

| current variant | site | canonical |
|---|---|---|
| `Corpus` | `src/node/mod.rs:48` | `Corpus` |
| `Prompt` | `src/node/mod.rs:48` | `Prompt` |
| `Correction` | `src/node/mod.rs:48` | `Correction` |
| `Teacher` | `src/node/mod.rs:48` | `Teacher` |
| `Peer` | `src/node/mod.rs:48` | `Peer` |
| `ModelOutput` | `src/node/mod.rs:48` | `ModelOutput`, refused |
| `Human = 0` | `src/record/mod.rs:94` | retired; a stored `0` decodes as `Prompt` — the least a `Human` byte proves |
| `Corpus = 1` | `src/record/mod.rs:94` | `Corpus`; byte `1` keeps its meaning |
| `Peer = 2` | `src/record/mod.rs:94` | `Peer`; byte `2` keeps its meaning |
| `Tool = 3` | `src/record/mod.rs:94` | retired; a stored `3` decodes as `Teacher` — an external answer, verifiable at its source |
| `ModelOutput = 4` | `src/record/mod.rs:94` | `ModelOutput`; byte `4` keeps its meaning, still refused |

The `u8` discriminants are data on disk. A retired discriminant decodes by
the table above and is never reassigned. The code and format consequences —
the decode mapping in `try_from_u8`, or a format-version bump — are
`model/prds/one-learn-origin`'s work.

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

Six things stay apart — convergence that reaches into any of them produces
one slow project instead of two fast ones:

- model's thesis — self-specialization, the precision ladder, seed/leech,
  perturbation-pair training.
- mitosys's ACP/surface/board stack.
- The event spine — mitosys's is `std` (`Arc`/`Mutex`/`Condvar`), model's is
  `tokio::sync::broadcast`; sharing it picks a runtime for both trees.
- The triple projection (`s`/`p`/`o` term ids, `ONT_ANY`) — model's
  vocabulary, not shared ([[record-shape]]).
- Each tree's fold — domain-entangled on both sides; the record shape ports,
  the fold does not.
- The repositories themselves.

## Sequencing

The merge as it stands on 2026-08-23, one carrier per step:

1. **Divergences** — done. [[divergences]] is decided; the model children
   `adopt-test-law`, `pin-toolchain`, `workspace-deps` are open on
   `model/prds`.
2. **Gates port** — carried by master `prds/propagate-gates` (analyzing).
3. **First shared crate** — done. `conserved` landed ([[shared-crate]]
   §Landed); consumer adoption is held under shared `p5-adoption`.
4. **Origin vocabulary** — carried by model child
   `model/prds/one-learn-origin`. No prerequisite; lands now.
5. **Grade shape into mitosys** — carried by mitosys `prds/p9-perf-floor`
   under master `prds/one-ratchet` ([[ratchet]]).
6. **Record convergence** — direction per [[record-shape]]; carried by
   mitosys child `mitosys/prds/record-shape-port`, sequenced with the fold
   rewrite `p6k10` (`mitosys/prds/p6-rust-core/p6k-kern-merge/
   p6k10-production-fold`). Unblocks master `prds/storage-convergence`.
7. **model behind `LlmFunc`/`EmbedFunc`** — carried by model child
   `model/prds/back-llmfunc`. Last on purpose: the plumbing is small and it
   is not the goal. Then corpus flow — a future master-board PRD, admitted
   when steps 4 and 6 hold.
