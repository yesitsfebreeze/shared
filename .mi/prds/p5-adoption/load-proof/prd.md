---
state: specced
mode: afk
priority: 42
est: 3.25h
repo: shared
verify: "cargo test -p conserved load -- --include-ignored"
---

# P5b — the load proof: the crate holds its contract at scale

Purpose: the one requirement of p5 that lands in **this** repo. `ContentId`
hashing throughput and `Scope` unwind-under-panic, exercised in a bench or test
recorded in `conserved` itself, before mitosys's record depends on them.

This node exists because the parent's `verify:` was a bare `echo` that exits 0
unconditionally — it reported the load proof green having never written it.
Blocked on **p1 + p2 only**, not p3/p4.

## Requirements

- [ ] **Assert numbers, do not print a table** — a floor on `ContentId::of`
      throughput (MB/s) over 1 B / 1 KiB / 1 MiB inputs, in a check that fails
      when the floor is missed.
- [ ] **`Scope` unwind-under-panic is NOT covered by p1's ported
      `drop_unwinds`.** Needs a scope holding N effects panicked out of,
      asserting all N inverses ran in reverse, at a depth (~10^5) that proves no
      stack overflow and no quadratic unwind — plus the panic-during-unwind case.
- [ ] **Prefer a std-only timing harness over `criterion`.** Consumers never
      build a git dependency's dev-dependencies, so mitosys's
      `dependency_tree.rs` CLOSURE is unaffected either way and p2's
      `blake3_is_the_only_dependency` reads `[dependencies]` only — but the
      cheaper answer is no new dependency at all. If the analyst disagrees, it
      must say why against those two gates.
