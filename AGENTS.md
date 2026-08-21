# AGENTS.md — infra meta-project

This repository is a collection of three independent Rust projects that share a
common foundation. It is **not** a monorepo — each project is its own Cargo
workspace, its own tests, its own conventions. What joins them is the
`learnings/` directory — shared knowledge, and the seed of a shared crate.

## Hard rule — check learnings and siblings before implementing

Before implementing ANY non-trivial feature, type, or utility in any of the
three projects:

1. **Check `learnings/`** — does a document bind this project? If yes, the
   learning holds. Read it.
2. **Check sibling projects** — does mitosys already have a `Scope` type
   llm's AGENTS.md mentions it lacks? Does realm already have the CLI parser
   shape llm is about to write?
3. **Check for duplication in the target project** — run
   `rg --type rust 'fn (your pattern)'` across all three `src/` trees. A
   second implementer of the same thing should be a shared crate, not a second
   implementation.

**Violation:** a crate in any of the three projects independently invents a
type, pattern, or utility that already exists in another project in this repo,
with nothing in `learnings/` recording the decision. The purpose of this rule
is to detect that before the implementation, not after.

## The three projects

```
infra/
  llm/              the learner — model, peer mesh, improving loop
                    AGENTS.md at llm/AGENTS.md
                    one binary, always. Edition 2024, no pin yet.

  mitosys/          the harness — record, orchestration, plugins, surfaces
                    AGENTS.md at mitosys/AGENTS.md
                    39 crates in three families (util/, engine/, api/).
                    Edition 2021, pinned 1.94.0, tests at <crate>/tests/
                    and <crate>/tests/unit/. Read
                    .mi/skills/process/laws.md first — four laws that govern
                    everything.

  realm/            container orchestration — ZFS, Linux drivers, net, SSH
                    No AGENTS.md yet. 7 crates (cli/, core/, drivers/linux/,
                    net/, ssh/, zfs/).

learnings/          shared knowledge — prose, on disk, admitted only when
                    true of more than one of the above.
```

### Divergences between the projects

The three projects share philosophy but differ in convention. These are
documented in `learnings/divergences.md` and must be respected when writing
code that targets a specific project:

| rule | llm | mitosys | realm |
|---|---|---|---|
| **test law** | `src/<module>/tests/`, root `tests/` forbidden | `<crate>/tests/` + `tests/unit/`, beside-the-module forbidden | follows mitosys pattern |
| **packaging** | one binary, always | `mi` + `mi-memory`, 39 crates | workspace with 7 members |
| **toolchain** | edition 2024, **no pin** | edition 2021, pinned `1.94.0` | edition 2021, **no pin** |
| **dependencies** | per-package versions | pinned once in `[workspace.dependencies]`, gated | per-package versions |

A shared crate must choose one convention per dimension. `conserved` (proposed
in `learnings/shared-crate.md`) resolves each one explicitly in its own crate
manifest and test layout.

## The learnings — how they work

A learning is a markdown file in `learnings/` with YAML frontmatter:

```yaml
---
type: learning
learning: <slug>
subject: <one line — what is true, not what the document is about>
binds: [llm, mitosys]     # which projects this holds for
status: decided | open | partial
date: <YYYY-MM-DD>
code: <project file:line, where applicable>
---
```

### When to create a learning

Create a learning when you discover something true of **more than one** of the
projects in this repo:

- A repeated pattern (both llm and mitosys have cosine distance)
- A contradictory rule (the test laws disagree)
- A shared decision (both will use blake3)
- A dependency pin both consume (both pin `serde`)

Do **not** create a learning for something true of only one project — it
stays in that project's own docs. This is the admission rule documented in
`learnings/README.md`.

### The status ladder

| status | meaning |
|---|---|
| **open** | found and argued, not yet settled. Implements nothing. |
| **partial** | agreed in principle, not fully specified or not yet extracted. |
| **decided** | agreed; the record of what it beat. May still need extraction. |

`supersedes` / `superseded_by` links between documents record when a later
learning replaces an earlier one. No document here is ever edited to erase a
decision — corrections are new documents that supersede the old one.

## The shared crate

`learnings/shared-crate.md` proposes **`conserved`** — a crate all three
projects depend on, holding domain-free utilities:

| thing | dependency | why it qualifies |
|---|---|---|
| `ContentId` | `blake3` | both projects hash; they hash differently and drift silently |
| `Clock`/`Instant` | none | both read wall clock ~65×; law forbids it but the type doesn't |
| `Scope`/`Handle` | none | mitosys has it, llm cites DOGMA 13 in prose — one works, one wishes |
| order statistics | none | both compute median; two definitions across a shared grade envelope |
| `hex` | none | both encode/decode hex; mitosys tolerates an `ed25519:` prefix |

The admission test (from `learnings/shared-crate.md`):

> A thing belongs in this crate when **all four** hold:
> 1. Both trees need it, today, not speculatively.
> 2. It is domain-free — no agent, no model, no peer, no surface.
> 3. It passes mitosys's `dependency_tree.rs` gate.
> 4. One implementation is genuinely better than two.

The first move is `Scope` alone — 262 lines, zero dependencies, already
working in mitosys, missing in llm. It proves the mechanism.

## Workflow for pattern extraction

When you find a repeated pattern (same type, same shape, same bug in two
places):

1. **Capture** the pattern in `learnings/` as a new document. Bind it to the
   projects where it appears. Status: `open`.
2. **Research** the pattern against the shared-crate admission test (above).
   Does it qualify? If no, it stays as prose in learnings — still valuable.
3. **If it qualifies**, open a learning with status `partial`, naming both
   implementations and the extraction path.
4. **Extract** — move one implementation into `conserved`, replace the other
   with a dependency. Never extract blind — the cargo build and `just check`
   in all affected projects must pass.
5. **Close** — update the learning status to `decided`, link to the commit.

The research step is where deep work lives: testing under load, measuring the
contract, deciding the names and the API surface. Do it before writing code.

## Key documents to read

| document | where | what it is |
|---|---|---|
| **The board** | `.mi/prds/` | claimable work for extracting `conserved` — p0 first, everything blocks on it |
| Scaffold reset memo | `.mi/docs/memos/scaffold-reset.md` | why the pre-split conserved-* crates are condemned |
| Distribution memo | `.mi/docs/memos/distribution.md` | how the crate reaches consumers — **decided**: a git dependency pinned by commit rev; mitosys carries the offline cost |
| Learnings admission rule | `learnings/README.md` | what belongs in learnings, what does not |
| Shared crate proposal | `learnings/shared-crate.md` | the full `conserved` plan |
| Capability matrix | `learnings/inventory.md` | what each project has built |
| Two halves | `learnings/two-halves.md` | how mitosys and llm fit together |
| Divergences | `learnings/divergences.md` | four contradictions blocking shared code |
| llm working context | `llm/AGENTS.md` | one binary, candle, libp2p, redb |
| mitosys working context | `mitosys/AGENTS.md` | four laws, 39 crates, gates |
| mitosys laws | `mitosys/.mi/skills/process/laws.md` | four laws that do not bend |
