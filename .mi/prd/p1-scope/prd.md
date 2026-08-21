---
state: claimed
claim: impl-p1-scope
est: 3h
mode: afk
priority: 20
verify: "cargo test -p conserved scope && sh -c 'if grep -nE \"^[[:space:]]*use[[:space:]]\" conserved/src/scope.rs | grep -vE \"use (crate|super|self|std)::\"; then echo \"scope.rs imports an external crate\"; exit 1; fi; echo \"scope pulls nothing in\"'"
---

# P1 — Scope/Handle: the first extraction, proving the mechanism

Purpose: port mitosys's reversible-effect scope — `util/effect`, 262 lines,
imports nothing, unwinds in reverse on drop — into `conserved`, as-is. Spec is
`learnings/shared-crate.md` §3; the source of truth is the working mitosys
implementation (`../mitosys`, `src/mitosys/util/effect`), which its own tests
already pin. llm has no implementation — it cites DOGMA 13 in prose comments
in `main.rs` and holds the rule by hand at each site.

This is deliberately the first move because it cannot fail in an interesting
way. What it actually tests is the **mechanism p0 chose**: does the
dependency resolve from a consumer tree, on a fresh clone, in mitosys's
offline container, under both gates. Learn that on 262 lines, not on the
record. Blocked on `p0-foundation`.

## Requirements

- [x] **Port the module** — code and its tests, moved not rewritten. Renames
      only where the crate boundary forces them; every deviation from the
      mitosys source recorded at its site.
- [x] **Zero dependencies** — the verify line counts edges; `Scope` must not
      drag `blake3` in transitively (it cannot yet — p2 has not landed — keep
      it that way when it does: modules stay independent).
- [ ] **Prove the mechanism once** — from one consumer tree (mitosys is the
      natural first, since it is the strict one), depend on `conserved` via
      the p0-decided mechanism and compile. Not adoption — one crate, one
      `cargo build`, recorded here with the commit. Full adoption is p5.

## Acceptance

`conserved::scope` compiles with no dependencies, its ported tests pass, and
one consumer tree has built against the crate through the real distribution
mechanism at least once.
