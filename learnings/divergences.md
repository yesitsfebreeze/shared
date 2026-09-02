---
type: learning
learning: divergences
subject: four rules contradicted between mitosys and model in both directions; each now has one answer — mitosys's test law family-wide with one dated exemption, packaging was never a conflict, the 1.94.0 pin, [workspace.dependencies]
binds: [mitosys, llm]
status: decided
date: 2026-08-23
code: llm src/node/mod.rs:44, llm src/record/mod.rs:90
---

# Four contradictions, one answer each

The two trees share `rustfmt.toml` byte for byte, share the seventeen rules
(model's `DOGMA.md` is the pre-fold form of mitosys's four laws — the mapping
table in `.mi/docs/DOGMA.md` is one-to-one), and share the board protocol
down to the commit message. And they contradicted each other on four rules,
in both directions, each tree enforcing its own version — so no shared code
could satisfy both.

Decided 2026-08-23 (`.pearde/prds/settle-divergences`). Each answer names the rule
the family keeps and what breaks in the tree that loses.

A mechanical fact narrows what the test-law and dependency answers govern: a
shared crate reaches each tree as a rev-pinned git dependency
([[shared-crate]] §Where it lives), outside both repo roots, so neither
tree's gate scans its files. The answers decide (1) the layout and manifest
shape shared crates carry and (2) which law each gate port in
`.pearde/prds/propagate-gates` enforces.

## Test law — mitosys's, family-wide, one dated exemption

**Kept: mitosys's law.** `<crate>/tests/` + `tests/unit/`, beside-the-module
forbidden, at every crate boundary. It is the executable one — 3 checks with
vacuity thresholds in `mitosys/src/mitosys/gates/tests/source_layout.rs`.
Shared crates already carry it (`shared/shared/tests/`, 10 files); realm
follows it (`shared/AGENTS.md` §divergence table).

**model pays.** Its `AGENTS.md` §Testing rules law (`src/<module>/tests/`,
root `tests/` forbidden) loses. That law was prose with no gate — `just
check` there is a bare `cargo check` — and the tree violates it anyway:
root `tests/peer_churn.rs` is tracked (`model/docs/next-wave.md:74`).

**One dated exemption, 2026-08-23:** the legacy `llm` package (148 test
files in 21 `src/<module>/tests/` dirs) keeps its layout until its planned
crate split — `model/docs/dev-loop.md` §The ceiling names the split, and the
split is the event that removes the exemption. Until then the gate holds the
line model already breaks: no new beside-the-module test dirs, no new root
`tests/` files. The gate is the `source_layout` port in
`.pearde/prds/propagate-gates`.

## Packaging — both rules stand, no conflict

**Kept: both, unchanged.** model's law governs *binaries* — one, never a
second `[[bin]]`. mitosys's 39-crate workspace is about *crates*. A shared
crate is a library, and `model/Cargo.toml` already carries three library
members (`interface`, `reload-sort`, `reload-sum`) beside its one binary —
the law was already being read correctly there. **Nobody pays.**

## Toolchain — mitosys's pin, `1.94.0`

**Kept: the pin.** Already family-wide but for model:
`mitosys/rust-toolchain.toml` and `realm/rust-toolchain.toml` both pin
`1.94.0`, and `shared/Cargo.toml` sets `rust-version = "1.94.0"`. The rule
existed in the family; it just had not propagated.

**model pays.** Its floating stable loses. mitosys's recorded case is the
argument: clippy 0.1.97 added a `collapsible_match` hit that 1.94 does not
have, on code nothing touched — an unpinned toolchain makes "does the gate
pass" a property of whichever stable was current on the machine, not of the
tree.

**Editions stay tree-local.** Cargo editions are per-crate: a 2021-edition
`shared` compiles as a dependency of 2024-edition model unchanged. Only
the pin propagates; the edition row was never a fork.

## Dependencies — mitosys's `[workspace.dependencies]`

**Kept: pinned once in `[workspace.dependencies]`**, gated by
`mitosys/src/mitosys/gates/tests/dependency_tree.rs`. `shared/Cargo.toml`
already adopted it, and a shared crate must pass the gate to be admitted
([[shared-crate]] §The admission test, criterion 3).

**model pays.** Its ~60 per-package version declarations lose:
`model/Cargo.toml` lifts them into `[workspace.dependencies]` now, members
inherit — same versions, same features, same `Cargo.lock`, no behavior
change. The `dependency_tree` port in `.pearde/prds/propagate-gates` then has a
table to assert against.

## The children

Three child PRDs, all on `model/.pearde` — model is the only tree an answer
changes:

| child | carries |
|---|---|
| `adopt-test-law` | the family test law into `model/AGENTS.md` §Testing rules, with the dated `llm`-package exemption |
| `pin-toolchain` | `model/rust-toolchain.toml`, `channel = "1.94.0"` |
| `workspace-deps` | the `[workspace.dependencies]` lift in `model/Cargo.toml` |

No child elsewhere:

- **realm** already conforms — pinned `1.94.0`, mitosys's test layout
  (`shared/AGENTS.md` §divergence table).
- **shared** already adopted all three kept rules — the `shared/tests/`
  layout landed, `[workspace.dependencies]` in `shared/Cargo.toml`,
  `rust-version = "1.94.0"`.
- **mitosys** keeps every kept rule — nothing changes there. The gate ports
  are `.pearde/prds/propagate-gates`' work, not a child of this decision.

## The evidence: LearnOrigin is two enums

model declares `LearnOrigin` **twice**, in one repository:

```
src/node/mod.rs:44     Corpus · Prompt · Correction · Teacher · Peer · ModelOutput
src/record/mod.rs:90   Human · Corpus · Peer · Tool · ModelOutput
```

Same concept, two spellings, different variants, different discriminants.
Under the doctrine both files cite, that is a straight violation — one fact
one home, one concept one name, law 4's *one vocabulary*.

It matters more than an ordinary duplication because of *where* it sits: it
is the vocabulary of **where a training signal came from**, which is
precisely the seam [[two-halves]] identifies as the highest-value coupling
between the two trees. The one place a shared word most needed to mean one
thing is the place it means two.

It survived because model has the words and not the gate. `just check` there
is a bare `cargo check`; nothing reads the doctrine.

## The general lesson

mitosys records the same failure from the other direction, and it is worth
quoting because it is the argument against holding any of the four answers
with prose:

> the copy that used to live beside the laws had already dropped "extends
> the request that triggered it, byte for byte" from rule 5 and "inside the
> boundary, the types are the check" from rule 10, **and read as
> authoritative the whole time.**

Two trees, two independent demonstrations that a shared rule with nothing
running it decays. Law 3: the law on the lowest rung is the one violated
first, silently. So every answer above that a gate could hold names the
gate, and every named gate is `.pearde/prds/propagate-gates`' input — the decision
is not done until something reads it.

## What this beat

- **model migrates now (test law, no exemption).** 148 files move to
  `tests/` + `tests/unit/` with `#[path]` declarations — the largest single
  child PRD on model's board, redone in part when the planned crate split
  lands. The dated exemption gets the same one-law end state and lets the
  migration ride scheduled work.
- **Each tree keeps its own test law; the family law binds shared crates
  only.** Two dialects forever — the exact drift this learning exists to
  record, the `LearnOrigin` lesson ignored.
- **model's test law family-wide.** mitosys and realm move 235+ files across
  51 crates, `source_layout.rs` is rewritten against its own recorded
  history, `shared/tests/` moves under `src/`, and mitosys's code-free
  gates crate has no legal home for its tests. The evidence is against it.
- **Defer the dependency lift to the crate split.** Rejected: model's
  workspace already has 4 members sharing `interface`, the lift is one
  mechanical manifest edit, and deferring leaves the `dependency_tree` port
  with nothing to hold.
