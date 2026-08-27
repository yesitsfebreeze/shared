---
type: learning
learning: plugin-core
subject: membrane stays in mitosys and the shared-crate admission test stands unwidened — no second tree hosts plugins today, and the revisit trigger is a second host or a second pin of the WIT world's version, not a date
binds: [mitosys, model]
status: decided
date: 2026-08-23
code: mitosys src/mitosys/api/plugin, mitosys src/mitosys/api/plugin/world/wit/mitosys-plugin.wit, model interface/src/lib.rs
---

# plugin-core — membrane stays in mitosys, the admission test stands

mitosys is extracting its plugin core as **membrane**
(`mitosys/.mi/docs/memos/membrane-is-the-core.md`, claimed at
`mitosys/prds/p8-membrane/`). Measured 2026-08-23, four of its crates carry
7,428 impl lines with zero domain dependencies — the most domain-free code in
the family. Decided 2026-08-23 (`prds/membrane-home`): it is **not**
`conserved`'s next admission.

## The decision

- membrane stays in mitosys.
- [[shared-crate]] §The admission test stands as written — no widening, no
  stated exception. This document is the cross-reference; `shared-crate.md`
  is unedited.
- Criterion 1 fails today ("both trees need it, today, not speculatively"):
  only mitosys has plugins. The failure is a fact about the family, not a gap
  in the rule.
- Criteria 2 and 3 pass, measured 2026-08-23: 7,428 impl lines, zero domain
  edges, four third-party deps. Criterion 4 is unanswerable while there is
  one implementation.
- Reopen on the named triggers below, not on a date.

## The measurement, 2026-08-23

| crate | impl lines | domain dependencies |
|---|---|---|
| `api/plugin` | 5,695 | 0 |
| `api/plugin/wasm` | 1,498 | 0 |
| `api/plugin/world` | 62 | 0 |
| `util/effect` | 173 | 0 |

## No second consumer exists

model rejects the stack in its own docs, verified 2026-08-23:

- `model/docs/freenet.md:163` — "we don't execute third-party code … Skip
  Wasmtime/fuel entirely". `model/docs/freenet-data-prd.md:106` — "No
  Wasmtime, no fuel".
- model's reload seam is not a plugin host. `model/interface/src/lib.rs` +
  `model/reload-fixture/seam.rs`: a same-rustc dylib swap of model's own
  substrate functions inside its improving loop — `ABI_VERSION`, layout
  fingerprint, host-owned state, POD by pointer. No manifest, no
  registration, no kinds, no service table, no WIT. It swaps *itself*; it
  hosts nothing.
- [[two-halves]] §C names the sharing direction as the reverse: "llm's reload
  seam is mitosys's plugin host" — model's proven swap is a gift *to*
  mitosys, not a demand for membrane.
- [[shared-crate]] §What stays out already classified the reload seam: "a
  *seam*, not a utility … deserves its own crate, on its own schedule, once
  the swap work actually starts." A future `interface` crate is a separate
  admission with its own PRD; it is not membrane.
- Zero mentions of membrane or a plugin host on `model/prds/` (grep,
  2026-08-23).

realm ships a plugin, never hosts one:

- `realm/prds/prd.md:28-34` plans realm *as* a mitosys plugin and forbids
  a hard dependency on mitosys; the `kind: realm` transport stays in the
  mitosys repo (option b, recorded there).
- Authoring a plugin never needs the membrane crate — the WIT world is the
  contract: `mitosys/src/plugins/tally` is C compiled against
  `api/plugin/world`'s interface description "and nothing else".

Membrane is needed only by a tree that wants to **host** plugins, and only
mitosys has a host role.

## The sharp edge stays one repo's

The versioned WIT world is the PRD's named sharp edge, and it cuts only
under admission:

- `api/plugin/world` is one file, `package mitosys:plugin@0.1.0` — version
  read off the package line, host and guest bindings generated from it.
- Today one repo bumps it and mitosys's `tests/world.rs` holds it.
- Cross-repo it becomes: commit in `shared/`, push, rev-bump in mitosys,
  plus the no-network dev container's vendoring cost [[shared-crate]]
  §Where it lives already prices — all to protect a consumer pinned to an
  old version who does not exist.

## The revisit trigger

Reopen as a new master-board PRD when either event occurs:

1. A second tree wants to **host** plugins — compose `api/plugin`'s runtime,
   not merely ship a plugin.
2. A second repo needs to pin `api/plugin/world`'s package version — e.g.
   realm's plugin moving out of the mitosys repo.

On either event criterion 1 passes, and criteria 2–3 already pass —
admission becomes mechanical. Until then mitosys owns the world's version
and bumps it in-tree.

## What breaks in the tree that loses

Per `settle-divergences`' standard ([[divergences]]): no tree loses code.
The family loses the claim that its most domain-free 7,428 lines are family
property — they stay one tree's, and a second host arriving pays a one-time
repo move of an already-bounded, already-gated crate at whatever rev
membrane then carries. Priced and accepted.

## Child PRDs: none

No tree gains or loses work, so this decision spawns **zero child PRDs**:

- mitosys's extraction is already claimed at `mitosys/prds/p8-membrane/` —
  that board owns it.
- model adopts nothing.
- realm adopts nothing.

`settle-divergences` is decided and no longer blocks either outcome: the
test law is settled family-wide ([[divergences]]), and under this answer no
shared crate exists to carry it.

## What this beat

- **Admit membrane and widen the rule, naming model's reload seam as the
  second consumer.** The named candidate affirmatively rejects the stack:
  model's own docs skip Wasmtime twice, its reload seam swaps itself and
  hosts nothing, and [[two-halves]] §C says the value flows the other way —
  model's proven swap informs mitosys's host. Widening criterion 1 to admit
  a consumer that has said no in writing is not a widening, it is a
  deletion.
- **Admit membrane as a stated exception.** An exception with zero consumers
  buys nothing — there is no tree waiting to be unblocked — and costs
  criterion 1 its meaning: it becomes the precedent every future "will need
  it soon" candidate cites. [[divergences]]' lesson is the argument: a rule
  weakened in prose decays silently.
- **Admit it now while the API is hot.** The world is about to churn:
  `membrane-is-the-core.md` §What is not done names the missing `judge`
  export, and `p8f`–`p8j` rewrite `plugin.rs` semantics — plugin tree,
  isolation realms, quiescence, swap discipline — across five serialized
  waves. Freezing a churning API behind a cross-repo rev pin mid-flight is
  the maximum-cost moment to admit it. And nothing waits:
  `mitosys/prds/p8-membrane/prd.md` §Out of scope excludes the home question
  explicitly, and `p8a` names, addresses and gates the crate in-tree — own
  manifest, zero domain edges, gated — exactly the shape that later moves to
  `shared/` as a rev-pinned git dependency with zero call-site rework, the
  mechanism `conserved` proved once (`9fff8ea`, [[shared-crate]] §Landed).
