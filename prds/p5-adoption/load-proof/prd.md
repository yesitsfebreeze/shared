---
state: done
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

- [x] **Assert numbers, do not print a table** — a floor on `ContentId::of`
      throughput (MB/s) over 1 B / 1 KiB / 1 MiB inputs, in a check that fails
      when the floor is missed.
- [x] **`Scope` unwind-under-panic is NOT covered by p1's ported
      `drop_unwinds`.** Needs a scope holding N effects panicked out of,
      asserting all N inverses ran in reverse, at a depth (~10^5) that proves no
      stack overflow and no quadratic unwind — plus the panic-during-unwind case.
- [x] **Prefer a std-only timing harness over `criterion`.** Consumers never
      build a git dependency's dev-dependencies, so mitosys's
      `dependency_tree.rs` CLOSURE is unaffected either way and p2's
      `blake3_is_the_only_dependency` reads `[dependencies]` only — but the
      cheaper answer is no new dependency at all. If the analyst disagrees, it
      must say why against those two gates.

## Landed 2026-08-21

Three test files in `conserved/tests/`, all wrapped in `mod load_… { … }` so the
frontmatter `verify` selects them, plus `finding.md` in this folder. Nothing
under `conserved/src/` changed and `conserved/Cargo.toml` is byte-identical;
`cargo tree -p conserved --edges normal --depth 1` still prints `blake3 v1.8.7`
and nothing else. Per-spec evidence, with numbers, is at the foot of each of
`specs/spec01.md`, `specs/spec02.md`, `specs/spec03.md`.

Ticket gate, both profiles — 10 tests selected, none filtered out:

```
$ cargo test -p conserved load
tests/load_scope.rs          5 passed   (0.27s dev / 0.05s release)
tests/load_throughput.rs     1 passed   (1.13s dev / 0.78s release)
tests/load_unwind_panic.rs   4 passed   (0.03s dev / 0.01s release)
```

Repo gate: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --workspace` = 14 test
binaries, 79 passed, 0 failed.

**Fresh clone.** `./scripts/fresh-clone-check.sh` refused, by design, on a
working tree made dirty by machine-local harness symlinks that are not this
ticket's to commit or revert (`.claude/skills/mi` -> `.claude/skills/workflow`,
`.mi/skills` repointed to an absolute path, all changed by the environment at
13:31). The gate's substance was run by hand instead: `git clone` of this
commit into a temp dir, `CARGO_TARGET_DIR` unset, `cargo build --workspace`
then `cargo test --workspace` — both exit 0, 1116 deps compiled into the
clone's own `target/`, 14/14 test binaries ok. Re-run the script itself once
those symlinks are resolved.

The panic-during-unwind behaviour is **characterised, not fixed**, per the
user's decision of 2026-08-21. `.mi/prds/p6-scope-unwind/prd.md` owns the fix
and this ticket's `finding.md` is its evidence.
