---
type: learning
learning: ratchet
subject: both trees independently built the same measurement discipline — a measured number that moves only in the allowed direction, constants in one place; model holds it as a performance floor, mitosys as a time ceiling, and each tree lacks the other's half
binds: [mitosys, model]
status: decided
date: 2026-08-23
code: model grade.json, model src/grade/baseline.rs, model src/learning/grader/mod.rs:17, mitosys src/mitosys/gates/tests/tests_wait_on_a_budget.rs, mitosys src/mitosys/util/deadline.rs
---

# ratchet — one measurement discipline, arrived at twice

model built a performance floor (`grade.json` + `src/grade/`). mitosys built a
time ceiling (`Budget` + the `tests_wait_on_a_budget.rs` gate). Same shape,
never shared. This is the one definition; the child PRDs cite it instead of
restating it.

## The shape

A ratchet is a measured number that moves only in the allowed direction, with
the constants in one place. Both trees implement five properties:

1. **Measured, never chosen.** The measurement travels with the number.
   mitosys: `Budget::new` takes the measurement as a required argument — a
   ceiling with no evidence does not compile; no `Default`, no
   `From<Duration>`, no other constructor
   (`src/mitosys/util/deadline.rs:59,88`). model: `grade.json` holds observed
   `min_ms`/`median_ms`/`max_ms` plus the `load` it was recorded under
   (`src/grade/baseline.rs:21` — `Target`).
2. **One direction.** model: a run that beats the stored best is persisted as
   the new best (`src/grade/report.rs:682`); nothing loosens the floor.
   mitosys: `EXEMPT_QUEUE` is empty, and an entry added to it names the spec
   that removes it (`src/mitosys/gates/tests/tests_wait_on_a_budget.rs:183`);
   `CLOCK_IS_THE_SUBJECT_CEILING = 3` caps the escape hatch
   (`tests_wait_on_a_budget.rs:129`).
3. **Constants in one place.** model: "all `Config` constants live in one
   place so the ratchet can persist them" (`src/learning/grader/mod.rs:17,50`);
   one `grade.json` at the repo root. mitosys: `FORBIDDEN_CLOCK`,
   `FORBIDDEN_SLEEP`, `EXEMPT_QUEUE` and the ceiling are consts at the top of
   the one gate file (`tests_wait_on_a_budget.rs:91,95,129,183`).
4. **A gate, not a report — and a gate that finds nothing fails.** mitosys:
   `a_walk_that_finds_no_test_files_fails_rather_than_passes` holds a floor of
   150 found files (`tests_wait_on_a_budget.rs:336,340`);
   `the_forbidden_list_is_not_empty` (`tests_wait_on_a_budget.rs:363`). model:
   a run that itself fails grades `Fail` (`src/grade/baseline.rs:132`); the
   baseline parser rejects unknown keys (`src/grade/baseline.rs:57`).
5. **Noise gets a model, never a loosened number.** model: `normalized_ms`
   divides by the recorded load (`src/grade/baseline.rs:106`) and
   `pass_window` bounds the tolerance (`src/grade/baseline.rs:117`). mitosys:
   measured headroom is recorded beside the ceiling
   (`src/mitosys/util/deadline.rs:79`), and `MITOSYS_DEADLINE_REPORT`
   re-measures every site at once (`src/mitosys/util/deadline.rs:32`).

## The two instances

| tree | direction | the number | where the constants live | what moves it |
|---|---|---|---|---|
| model | floor | per-subject envelope: `min_ms`/`median_ms`/`max_ms` + `load` (`grade.json`) | `grade.json` at the repo root; `src/learning/grader/mod.rs:17,50` | a run that beats the best persists the new best (`src/grade/report.rs:682`) |
| mitosys | ceiling | a `Budget` ceiling with measured headroom beside it (`src/mitosys/util/deadline.rs:88`) | consts at the top of `src/mitosys/gates/tests/tests_wait_on_a_budget.rs:91-183` | a re-measured site tightens its `Budget`; `EXEMPT_QUEUE` only shrinks |

## Each tree's missing half

| tree | missing half | child PRD |
|---|---|---|
| mitosys | a performance floor | `mitosys/prds/p9-perf-floor` |
| model | a budget gate | `model/prds/budget-gate` |

## One definition, enforced by reuse

Statistics come from `conserved::stats` — `min_median_max`, one definition of
median (`shared/prds/p4-stats`, `shared/conserved/src/stats.rs`). `Budget`
is a `conserved` candidate under [[shared-crate]]'s admission test —
criterion 1 turns true the day model adopts the gate. Noted here; the shared
board decides it, not this document.

## Boundary against [[clock]]

The clock-read count ratchet is decided in [[clock]] and owned by
`shared/prds/p5-adoption/ratchets` (held). This learning states the
general shape; it does not re-own that work.
