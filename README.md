# infra — shared foundation

Three independent Rust projects, one meta-project.

| project | what it is | workspace |
|---|---|---|
| **llm** | the learner — model, peer mesh, improving loop | `llm/Cargo.toml` |
| **mitosys** | the harness — record, orchestration, plugins, surfaces | `mitosys/Cargo.toml` (39 crates) |
| **realm** | the container orchestrator — ZFS, Linux drivers, net, SSH | `realm/Cargo.toml` (7 crates) |

All three are built on the same laws — one append-only record, every view a
pure fold, one fact one home — and keep independently arriving at the same
shapes. The **learnings/** directory is where those shared shapes are captured:
prose, on disk, that every consumer points at, admitted only when true of more
than one project.

## The learnings

`learnings/` holds what is true of more than one project. The admission rule
(from its `README.md`):

> A document lives here **if and only if it is true of more than one consumer.**
> True of one → it stays in that project.

Every learning carries typed frontmatter (`type: learning`) and a `binds:` field
naming the projects it holds for. The form is shared with mitosys's gene/memo
system, so one parser reads both.

**Current contents** (all at `learnings/`):

| document | subject | status |
|---|---|---|
| `inventory.md` | capability matrix: what each tree has built | decided |
| `two-halves.md` | mitosys = record-half, llm = learner-half of one system | open |
| `divergences.md` | four rules contradicting between llm and mitosys | open |
| `record-shape.md` | llm's record is ahead → port direction reverses | decided |
| `content-addressing.md` | blake3 over SHA-256, `[u8; 32]` over hex `String` | decided |
| `storage.md` | redb over LMDB, sequenced behind mitosys's fold rewrite | decided |
| `clock.md` | both trees read wall clock ~65×; the fix and the ratchet | open |
| `shared-crate.md` | proposal for `conserved` — what goes in, what stays out | partial |

The learnings are prose today and nothing gates them. The goal is to turn them
into executable truth: a shared crate (`conserved`, proposed in
`learnings/shared-crate.md`) that holds `ContentId`, `Clock`, `Scope`/effect
handles, order statistics, and hex — things all three projects need, with one
implementation tested under load.

## The board

The vision below is now claimable work: **`.mi/prds/`** holds the board
(p0-foundation → p1-scope → p2/p3/p4 → p5-adoption), with the specs staying
in `learnings/`. Two memos in `.mi/docs/memos/` record the reset of the
broken crate scaffold and the settled distribution decision.

`scripts/fresh-clone-check.sh` is how p0's fresh-clone criterion is checked: it
clones this repository into a temporary directory and builds and tests there,
so a file that was never committed fails the check instead of passing quietly.

## The vision

1. **Audit** every crate in all three projects — document what patterns repeat.
2. **Distill** the learnings: what is true of more than one project lands in
   `learnings/`. What is also dependency-light and domain-free graduates into
   `conserved` (the shared crate).
3. **Extract** the shared crate — starting with `Scope` (reversible effects,
   262 lines, zero dependencies), then `ContentId`, `Clock`, and order
   statistics. Each extraction proves the mechanism before the next one.
4. **Test under load.** The shared crate must hold its contract at scale.
5. **Adopt across projects.** Every project (`llm`, `mitosys`, `realm`)
   depends on `conserved` for the patterns it provides, replacing hand-rolled
   duplicates.

This is about utility functions and repeated patterns — `ContentId`, `Clock`,
effects, hex, median — not about one-off implementations in any individual
project. The shared crate stays domain-free: no agent, no model, no peer, no
surface.

## Where the shared code lives

The mechanism is **a git dependency pinned by commit rev**: each consumer's
`Cargo.toml` says `conserved = { git = "...", rev = "<commit>" }`, so drift
between the trees requires a visible rev bump. `.mi/docs/memos/distribution.md`
carries the full argument.

How that was reached, since `learnings/shared-crate.md` §"Where it lives"
deliberately records no recommendation:

- **A path dependency to a sibling directory** is eliminated. The user's
  requirement of 2026-08-20 is that the crate be distributable to every Rust
  repo; a path dependency is host-only by construction — it does not exist for
  a clone, nor inside mitosys's dev container, which bind-mounts the repo and
  nothing else.
- **Vendoring into each tree** was considered and not chosen. It builds
  everywhere with zero network, but the sync is manual and must be gated in
  every consumer, or the trees silently compile against different content.
- **A git dependency pinned by commit** is the mechanism. Its cost is that
  cargo fetches at build time.

**Who carries the offline cost: mitosys, not this repo.** mitosys's dev
container has no network, so its offline build story — `cargo vendor` output or
a pre-populated registry cache covering the pinned `conserved` rev — is
follow-up work scoped to mitosys, to be designed before its adoption step
lands. It is not a blocker here.

`Scope` alone (zero dependencies, already working in mitosys, missing in llm)
is the first extraction and will test the mechanism.

## Reading order

New to this meta-project:

1. `learnings/README.md` — the admission rule and philosophy
2. `learnings/inventory.md` — what each tree has built
3. `learnings/two-halves.md` — how they fit together
4. `learnings/shared-crate.md` — the shared-crate proposal
5. `learnings/divergences.md` — what must settle before shared code compiles

Then pick a project:

- `llm/AGENTS.md` — the learner's context
- `mitosys/AGENTS.md` — the harness's context (read `.mi/skills/process/laws.md` first)
- `realm/` — container orchestration (no AGENTS.md yet)
