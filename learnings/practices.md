---
type: learning
learning: practices
subject: this family has independently invented four practices that already have names and literature elsewhere — and is missing three that its own doctrine is unusually well set up to adopt
binds: [mitosys, model, realm, shared]
status: proposed
date: 2026-08-25
---

# What was invented here, and what it is called elsewhere

Naming a practice is not pedantry — it buys the literature, the failure modes
other people already hit, and tooling that does not have to be written here.
Four things in this workspace are re-inventions, and all four are good ones.

| built here | called elsewhere | what the name buys |
|---|---|---|
| `learnings/ratchet.md` — a measured number that moves only in the allowed direction | **fitness function** (Ford, Parsons & Kua, *Building Evolutionary Architectures*) | the vocabulary for the whole class: triggered vs. continuous, atomic vs. holistic. `ratchet.md` records that model holds a performance floor and mitosys a time ceiling and each lacks the other's half — that is exactly the atomic/holistic split |
| `mitosys-gates`, `gates/one_vocabulary.rs` — tests that read the source tree rather than link it | **architecture fitness functions**, the ArchUnit family | Java/C# have ArchUnit and NetArchTest. Rust has nothing equivalent, which is why these were hand-built — worth knowing the ecosystem gap is real and not an oversight |
| `.mi/docs/memos/*.md` with `status: decided` / `decided: <date>` | **Architecture Decision Records** (Nygard, 2011) | `npryce/adr-tools` (5.6k★) and `thomvaill/log4brains` (1.6k★) exist. The memo format here is already an ADR in all but name and does not need replacing — but the index-gate problem it hit is the same one every ADR corpus hits |
| `done-means-done.md` — a tick requires quoted evidence; an unclosable box is struck with a reason | **auditable Definition of Done** | rarer than it should be. Most DoD is a checklist with no evidence requirement. The evidence-quoting variant is the strong form and is worth keeping as the family's own |

# The three worth adding

## 1. Deterministic simulation testing — the largest fit

Run the whole system on a seeded, virtual clock and a simulated network, so a
distributed bug reproduces exactly from its seed. FoundationDB's method; the
reason `spacejam/sled` (9.1k★) surfaces under both the formal-methods and
fuzzing sweeps.

This workspace is unusually ready for it and does not know it. `realm` drives
containers, ZFS and nftables — its CI already documents six suites that cannot
run on a runner for want of a real pool or a reachable sshd, and two more that
are known-red because GitHub blocks outbound ICMP. Those are precisely the
tests DST replaces with something that runs anywhere, deterministically. And
`shared/learnings/clock.md` plus `shared/tests/clock_source.rs` mean the
clock is *already* an injectable seam — the single hardest precondition, and
it is met.

Rust tooling: `antithesishq/bombadil` (1.4k★), or the hand-rolled seeded
approach `sled` uses.

## 2. Mutation testing — the ratchet applied to the ratchets

`cargo-mutants` (1.3k★, active 1d ago) mutates the code and reports which
mutations no test catches. This family checks that gates *exist* — `gates.md`
enumerates the four every tree carries — but nothing checks that a gate has
any *power*. A gate whose assertion is vacuous passes forever and reads green.

The vacuity risk is already known here: `one_vocabulary.rs`'s doc comment
names "the vacuity threshold" as one of two things ported from mitosys. That
threshold is currently a human judgement. Mutation testing is the mechanical
form of the same question.

## 3. Machine-checked laws

mitosys runs on "four laws" (`.mi/skills/process/laws.md`); model runs on
"seventeen rules" (`DOGMA.md`). Both are prose that humans enforce. Some
subset of any such list is a state-machine claim, and those can be checked:
`quint` (1.6k★, TLA+ lineage with a readable syntax), `p-org/P` (3.7k★), or
`creusot-rs` (1.8k★) for verifying Rust functions directly.

Not all of them — most laws are about process, not state. But a law like the
record's append-only ordering is a safety property a model checker settles in
an afternoon and a test suite only samples.

# Already covered — do not re-recommend

**Property-based testing.** `proptest` is in the lockfiles and
`shared/tests/content_id_props.rs` uses it correctly, at default case
counts, with an explicit note that there is no `proptest_config` shrinking
coverage to look fast. That is the discipline most proptest adopters skip.
`hypothesis` (8.9k★) and `fast-check` (5.1k★) lead their languages, but this
tree is Rust and already has the right tool used the right way.
