---
type: learning
learning: divergences
subject: four rules contradict between mitosys and llm and must be settled before one line of shared code compiles; llm's duplicate LearnOrigin is the evidence that shared words without a shared gate drift
binds: [mitosys, llm]
status: open
date: 2026-08-18
code: llm src/node/mod.rs:44, llm src/record/mod.rs:90
---

# Four contradictions, and why words alone will not hold

The two trees share `rustfmt.toml` byte for byte, share the seventeen rules
(llm's `DOGMA.md` is the pre-fold form of mitosys's four laws — the mapping
table in `.mi/docs/DOGMA.md` is one-to-one), and share the board protocol
down to the commit message: llm's git log reads
`claim <path>: <uuid>` / `release <path>: <uuid>`, the same lock mitosys
uses.

And they contradict each other on four rules, in both directions. These are
not preferences. Both trees enforce their version, so shared code cannot
satisfy both.

## The four

| | llm | mitosys |
|---|---|---|
| **test law** | `src/<module>/tests/`, root `tests/` **forbidden** | `<crate>/tests/` and `tests/unit/`, beside-the-module **forbidden** |
| **packaging** | "one binary, always — never add a second `[[bin]]`" | `mi` + `mi-memory`, 39 crates |
| **toolchain** | edition 2024, **no pin** | edition 2021, pinned `1.94.0` |
| **dependencies** | per-package versions | pinned once in `[workspace.dependencies]`, gated by `dependency_tree.rs` |

The test law is a direct contradiction and it is the blocking one: a shared
crate must put its tests *somewhere*, and each tree's gate rejects the
other's answer.

Two of the four resolve almost for free:

- **Toolchain.** `~/dev/zirkle/model` already carries a
  `rust-toolchain.toml`; llm is the only tree in the family without one.
  The rule exists in the family, it just did not propagate. mitosys states
  the reason well enough to lift verbatim: an unpinned toolchain makes
  "does the gate pass" a property of whichever stable happened to be
  current on the machine that ran it, not of the tree.
- **Packaging.** Not actually a conflict. llm's law is *one binary*; a
  shared crate is a library. llm already has three workspace members
  (`interface`, `reload-sort`, `reload-sum`) and one binary, so the law is
  already being read correctly there.

## The evidence: LearnOrigin is two enums

llm declares `LearnOrigin` **twice**, in one repository:

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

It survived because llm has the words and not the gate. `just check` there
is a bare `cargo check`; nothing reads the doctrine.

## The general lesson

mitosys records the same failure from the other direction, and it is worth
quoting because it is the argument against solving this with prose:

> the copy that used to live beside the laws had already dropped "extends
> the request that triggered it, byte for byte" from rule 5 and "inside the
> boundary, the types are the check" from rule 10, **and read as
> authoritative the whole time.**

Two trees, two independent demonstrations that a shared rule with nothing
running it decays — and this folder is a third instance waiting to happen
unless something reads it. Law 3: the law on the lowest rung is the one
violated first, silently.

**So the first thing to share is not code and not prose. It is
`src/mitosys/gates/`.** Porting it to llm costs little, and it surfaces the
duplicate `LearnOrigin` as a failing check rather than as a claim in a
document.

## Open

Every row in the table above is undecided. Each needs a decision recorded
here — naming what it beat — before the first shared crate is extracted.
The test law is the one to settle first, because nothing compiles into both
trees until it is.
