---
type: learning
learning: gates
subject: four gates every tree carries (source_layout, one_vocabulary, dependency_tree, board_is_tracked), two bind mitosys+model, one waits on a recorded flake, the rest are mitosys's alone
binds: [mitosys, model, realm]
status: decided
date: 2026-08-23
code: mitosys src/mitosys/gates/lib.rs
---

# gates — the subset every tree carries

mitosys makes its rules executable: `src/mitosys/gates/` is a crate with no
code, its tests each refusing one class of violation with a named exemption
list. model and realm run none — their gate commands compile and nothing reads
the doctrine. This learning classifies mitosys's gates: which every family
tree carries, which bind two trees, and which are mitosys's alone. The child
PRDs (`model/prds/adopt-gates`, `realm/.mi/prds/15-gates`) cite this
classification instead of restating it.

## The count

Seventeen, recounted off `ls mitosys/src/mitosys/gates/tests/` on 2026-08-23.
`gates/lib.rs` records three prior undercounts and orders exactly that
recount — a count off the directory, never an increment off the previous
sentence. The four tables below cover all seventeen files: 4 + 3 + 2 + 8.

## Carried by every tree — the family subset

The subset `model/prds/adopt-gates` and `realm/.mi/prds/15-gates` land:

| gate | evidence |
|---|---|
| `source_layout` | the test law, decided family-wide 2026-08-23 (`shared/learnings/divergences.md` §Test law), with one dated exemption for model's legacy `llm` package until its crate split |
| `one_vocabulary` | one concept, one name, one owner — the gate class that would have caught `LearnOrigin` declared twice in model (`src/node/mod.rs:44`, `src/record/mod.rs:90`). Mechanical form per tree: mitosys checks command-namespace ownership; model and realm check that no public type name is declared twice outside an exemption list |
| `dependency_tree` | `[workspace.dependencies]` decided family-wide (divergences §Dependencies). realm's table already exists (`realm/Cargo.toml`); model's arrives via `model/prds/workspace-deps` |
| `board_is_tracked` | every tree carries a board, and the gate's own doc records the family incident: 77 untracked `prd.md` in mitosys, 51 in model — a `git clone` produced a repository with no board |

## Bind mitosys + model

Named, not ported by the child PRDs — model's board ports them when it has a
seam to hold:

| gate | evidence |
|---|---|
| `oracle_seam` | model constructs LLM clients in at least two non-test files (`model/src/teacher/mod.rs:395`, `model/src/improve/llm.rs:319` and `:679`) with no single recording seam. realm has no model client |
| `read_path_never_repairs` | both trees carry a record and a read path — model's `src/record` and its grade/overview reads. realm has no record |
| `tests_do_not_share_a_counter` | tree-agnostic in shape (no test measures process-global state another test moves); expensive to port — port when a tree records the flake it prevents |

## Excluded here — clock-discipline's work

`tests_wait_on_a_budget` and `write_path_reads_no_clock` port nowhere from
this decision. They are `clock-discipline`'s work, per the PRD constraint and
`shared/learnings/clock.md`.

## mitosys's alone

| gate | why it stays |
|---|---|
| `composition_wiring` | the plugin/composition machinery is mitosys's |
| `core_boundary` | the `src/mitosys/` boundary memo |
| `recall_is_recorded` | the recall effect exists only in mitosys's engine |
| `removal_policy` | the engine's one removal predicate |
| `retired_words` | shape portable, content is one tree's retirement history — each tree grows its own when it retires a word |
| `runtime_reads_the_record` | `.mi/` markdown parser allowances |
| `append_authority` | one writer on `.mi/journal`; model's ledger is multi-signer by design |
| `service_seams` | the plugin service inventory |

## The port rules

Every port, in every tree — from `gates/Cargo.toml`, `source_layout.rs`, and
the PRD's constraints:

- Copy the shape, not the file. A gate asserts against the tree it lives in,
  walked from its own workspace root.
- Every gate carries an exemption list with a reason per entry, shrink-only.
- Every scan has a vacuity threshold: a gate that finds nothing to check fails
  rather than passes (`source_layout.rs` asserts `checked > 50`).
- The gates crate has no code and links nothing it guards — `[dependencies]`
  empty, tooling under `[dev-dependencies]` only
  (`mitosys/src/mitosys/gates/Cargo.toml` states the reason: a guard that
  linked what it guards could be made to pass by changing its subject).
- A gate is reachable from the tree's own gate command: mitosys `just check`,
  model `just verify` (`cargo test --workspace`), realm `just test` / CI.

## Where nothing changes

- shared's workspace already conforms — `conserved/tests/`,
  `[workspace.dependencies]`, `rust-version = "1.94.0"` — and shared crates
  are admitted through mitosys's gates (`shared/learnings/shared-crate.md`
  §admission). No child PRD lands on shared's board.
- mitosys keeps all seventeen. Nothing changes there.
