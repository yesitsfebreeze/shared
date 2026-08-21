---
type: learning
learning: record-shape
subject: llm has already built the content-addressed bitemporal append-only record that mitosys's PRD §4 calls decided-not-done, so the port direction for the record reverses — mitosys takes from llm, not the other way round
binds: [mitosys, llm]
status: decided
date: 2026-08-18
code: llm src/record/mod.rs:208, llm src/record/log.rs:412, mitosys src/mitosys/engine/record/stream.rs:52
---

# The record: llm is ahead, and the port direction reverses

mitosys's `PRD.md` §4 commits the memory graph to converging on the shape
file state already has in git: *one event per step, appended, never edited,
merkle-chained and provenance-carrying, folded — never stored — into
whatever view a consumer needs.* `.mi/SYSTEM.md` is honest that this has not
happened: *"Design settled, sequencing not executed... Treat it as decided,
not done."*

**llm has done it.** Not a sketch — a working implementation, in
`src/record/`, 5.3k lines.

## Side by side

| | mitosys `engine/record` | llm `src/record` |
|---|---|---|
| event identity | `seq: i64`, assigned by the store | `rec_id()` = **blake3 of a fixed-order preimage** — recomputable by anyone holding the content |
| time | `time: String`, RFC3339 | `created: i64` unix seconds, plus `until` |
| encoding | JSON (`serde_json`) | postcard preimage + typed `Record` |
| correction | append a new event | **bitemporal supersede** — closes in place without re-hashing; the closed record still answers under its birth id |
| fold entry points | `project_messages`, `reconstruct_requests` | `replay(as_of, kinds)`, `replay_window`, `as_of(id, t)`, `recover(id)` |
| self-check | — | `LogStats::balances()` |
| retention | compaction machine | heat decay, **lazily on read** — "decay is never a sweep" |
| append result | — | `AppendReceipt` with a printable `line()` |

mitosys's `Event` does carry one content address, but of the *call*
(`hash(kind, target, canonical(args))`), not of the event. There is no
per-event id, no `prev` link, no chain.

## The three ideas mitosys should take

**1. The preimage discipline.** `rec_preimage` lists every field that
*defines* a record, in a fixed order, and names exactly what is excluded —
`id`, `address`, `heat`, `heat_at`, `signature`, and `until` — with the
reason stated where the code is: *"the id covers the claim, not its
lifespan."* That single decision is what makes bitemporal supersede
possible without re-hashing, and it is the piece mitosys's design has not
worked out.

**2. Supersede over append-a-correction.** mitosys corrects by appending a
new event that shadows the old one in the fold. llm closes the old record
in place by setting `until`, and the birth id still answers. Both satisfy
"the record only grows"; llm's additionally answers *"what did this look
like at time t"* (`as_of(id, t)`) without folding the whole log.

**3. Lazy decay.** Nothing walks the store on a timer; the cost is one
`exp` on the records a query already touched. mitosys's compaction is a
sweep. For a store that is meant to become a disposable fold, a retention
policy with no background job is strictly better — there is no timer to
resume after a crash.

## What llm should take back

- **`Scope`** (`mitosys/util/effect`, 262 lines, imports nothing). llm holds
  DOGMA 13 by hand, one site at a time, with prose comments in `main.rs`.
- **`gates/`**, which is the subject of [[divergences]].
- **The event/record split as separate crates.** llm's record reaches into
  `crate::events`, `crate::grade::baseline`, `crate::node` and
  `libp2p::Multiaddr` for its durable form, and `event.rs` documents the
  contortion this forces: `SystemEvent` cannot derive `Serialize` because
  `OpCompleted.kind` is a `&'static str`, so decoding "could only give it
  back by leaking." A record crate that knows nothing about what is being
  recorded — mitosys's rule for `engine/record`, stated as *"what it must
  never know"* — does not have that problem.

## Consequence for sequencing

[[two-halves]] listed the record convergence as step 5 of 5. That still
holds, but its *direction* flips: the target shape is llm's, and mitosys's
`engine/record` is the one that changes. Which also means mitosys's fold
rewrite — the one [[storage]] sequences the redb swap behind — has a
working reference implementation to port from rather than a design to
execute from scratch.

Two things that do **not** transfer:

- llm's record is entangled with its domain (see above), so what ports is
  the *shape* — preimage, supersede, lazy decay, replay-as-fold — not the
  file.
- llm's `Record` carries a triple projection (`s`/`p`/`o` term ids,
  `ONT_ANY` wildcard) from the zirkle ontology specs. mitosys's graph has
  its own vocabulary in `engine/base`. The triple layer is llm's, not
  shared.
