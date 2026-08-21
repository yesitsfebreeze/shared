---
state: specced
est: 1.75h
mode: afk
priority: 30
verify: "cargo test -p conserved stats"
---

# P4 — order statistics: one definition of median

Purpose: `learnings/shared-crate.md` §4. mitosys has `percentile_sorted` with
zero callers; llm's `grade::measure::aggregate` computes min / upper-median
(index `n/2`) / max by hand and is the thing that would call it. Two
definitions of median across a family that intends to share a grade envelope
is a regression that reads as a real one. Blocked on `p0-foundation`.

## Requirements

- [ ] **The functions** — over already-sorted `&[f64]`, returning `Option`
      on empty input:
      `percentile(sorted, p)`, `median(sorted)`, `min_median_max(sorted)`.
- [ ] **ONE definition, stated** — which median (the interpolating one or
      llm's upper-median `n/2`) is a decision to make against both call
      sites, written in the doc comment with the rejected alternative named,
      and pinned by a test on an even-length slice — the input where the
      definitions disagree.
- [ ] **Sortedness is the caller's contract** — stated in the docs; debug
      assertion, not a hidden sort. No dependencies, no allocation.

## Acceptance

Tests cover empty, single, even and odd lengths, and NaN handling is stated
and pinned rather than left to float comparison accidents.
