---
complexity: 65
footprint:
  - ../mitosys/Cargo.toml
  - ../mitosys/src/mitosys/util/effect/
  - ../mitosys/src/mitosys/api/plugin/
  - ../mitosys/src/mitosys/api/plugin/lua/
  - ../mitosys/src/mitosys/api/surface/
  - ../mitosys/src/mitosys/api/agentic/
  - ../mitosys/src/mitosys/api/agentic/pool/
  - ../mitosys/src/mitosys/api/service/
  - ../mitosys/src/mitosys/engine/record/
  - ../mitosys/src/mitosys/engine/layers/
  - ../mitosys/src/mitosys/engine/channel/
  - ../mitosys/src/mitosys/engine/genome/
  - ../mitosys/src/plugins/memory/
  - ../mitosys/src/mitosys/gates/tests/dependency_tree.rs
---

# spec02 — `util/effect` becomes `conserved::scope`, teardown behaviour change stated at the site

Make `mitosys-util-effect` depend on `conserved` (git, pinned by rev) instead
of carrying its own copy of `effect.rs`, re-exporting `Disposer`/`Scope`
(`Disposer` is the real type name — the mitosys README and
`learnings/shared-crate.md` both say `Handle`, which is wrong) under the same
`mitosys_util_effect::effect` path so every re-exporting crate's use-site is
unchanged. Record, in the adoption commit, that this changes teardown
behaviour: `conserved::scope::Scope::close` now runs every inverse even when
one panics (resuming the first panic reached afterwards, keeping `held()`
true throughout, and exposing `Scope::failed()`), where mitosys's own
`util/effect` abandoned the remaining inverses on the first panic and
reported `held() == []` while some were still owed
(`shared/prds/p5-adoption/load-proof/finding.md`, measured against
`conserved` at `main`/`a3d8bcc`). `shared/prds/p1-scope/specs/spec01.md`'s
byte-for-byte `verify:` diff no longer holds, by design — this spec is what
supersedes it.

**Re-export site count, measured today, not the PRD's "10+" list.** `grep -rl
mitosys_util_effect src --include='*.rs'` (excluding
`util/effect/{lib.rs,tests/effect.rs}` themselves) currently returns **11**
sites: `api/plugin`, `api/plugin/lua`, `api/surface`, `api/agentic`,
`api/agentic/pool`, `api/service`, `engine/record`, `engine/layers`,
`engine/channel`, `engine/genome`, and `src/plugins/memory/plugin.rs`. The
PRD's named list of 10 includes `api/engine`, which no longer exists — it was
folded into `src/plugins/memory` on 2026-08-21 per this repo's own
`CLAUDE.md` — and omits `engine/genome` and `plugins/memory`, both of which
re-export today. Do not implement against the PRD's fixed list; re-run the
`grep` below and treat its output as the list, since it can drift again
before this spec runs.

## Acceptance

- [ ] `grep -rl mitosys_util_effect ../mitosys/src --include='*.rs'` returns
      only `util/effect`'s own files plus re-exporting crates whose `//! May
      import:` layer doc also names `mitosys_util_effect` — no crate imports
      it undeclared.
- [ ] `mitosys-util-effect/Cargo.toml` depends on `conserved` (git, pinned
      rev) and no longer contains a local copy of `effect.rs`'s logic; the
      type re-exported is spelled `Disposer`, not `Handle`, at every site
      that names it (source and doc comments).
- [ ] `src/mitosys/gates/tests/dependency_tree.rs` accepts the new edge:
      `conserved` gains an entry in `OWNERS` naming the crate(s) that declare
      it, and gains an entry in `CLOSURE` — both lists stay sorted — and
      `cargo test -p mitosys-gates --test dependency_tree` passes.
- [ ] Every non-test call site under `src/mitosys/` matching `catch_unwind`
      is checked for a dependency on the old abandon-on-panic behaviour
      around a `Scope`/`Disposer` `close()` — measured today at
      `api/surface/abi.rs`, `api/agentic/agent.rs`,
      `engine/tick_loop/tick_trainer.rs`, `engine/tick_loop/tick.rs` — and
      the finding (depends / does not depend) for each is recorded in the
      adoption commit message, not silently left for a future reader.
- [ ] Every non-test call site matching `.held(` is checked for reliance on
      the old "may lie during an active unwind" behaviour — measured today
      at `api/plugin/plugin.rs` — same recording requirement.
- [ ] `Scope::failed()` is adopted at at least one site currently swallowing
      a teardown failure, or the commit states there is none.
- [ ] `cargo build --workspace` and `cargo test --workspace` pass inside the
      offline container (spec01's mechanism).

## Verify and Proof

```sh
cd ../mitosys
grep -rl mitosys_util_effect src --include='*.rs'
cargo test -p mitosys-gates --test dependency_tree
cargo build --workspace
cargo test --workspace
just check
```
