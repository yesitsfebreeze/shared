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
  - ../mitosys/src/builtins/memory/
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
`engine/channel`, `engine/genome`, and `src/builtins/memory/plugin.rs`. The
PRD's named list of 10 includes `api/engine`, which no longer exists — it was
folded into `src/plugins/memory` on 2026-08-21 per this repo's own
`CLAUDE.md` — and omits `engine/genome` and `plugins/memory`, both of which
re-export today. Do not implement against the PRD's fixed list; re-run the
`grep` below and treat its output as the list, since it can drift again
before this spec runs.

## Acceptance

- [x] `grep -rl mitosys_util_effect ../mitosys/src --include='*.rs'` returns
      only `util/effect`'s own files plus re-exporting crates whose `//! May
      import:` layer doc also names `mitosys_util_effect` — no crate imports
      it undeclared. **One crate did import it undeclared and now does not.**
      13 files: `util/effect/{lib.rs,tests/effect.rs}` plus the 11 re-exporters
      the spec lists. Checked file by file — every one of the 11 carries
      `mitosys_util_effect` inside its `//! Layer: … May import:` block —
      except `api/service/lib.rs`, which has done
      `pub use mitosys_util_effect::effect;` at line 53 while its layer doc
      (line 9) named five crates and not this one. The doc line now declares
      it; the edge itself is unchanged and pre-existing.
- [x] `mitosys-util-effect/Cargo.toml` depends on `conserved` (git, pinned
      rev) and no longer contains a local copy of `effect.rs`'s logic; the
      type re-exported is spelled `Disposer`, not `Handle`, at every site
      that names it (source and doc comments).
      `conserved = { git = "https://github.com/yesitsfebreeze/shared.git",
      rev = "70d7e15cd21c6017ec928c63697d0c7f42f53a20" }` in
      `[workspace.dependencies]`, `conserved.workspace = true` in the crate.
      **The rev was checked onto the remote before vendoring** —
      `git -C shared branch -r --contains 70d7e15` lists `origin/main`, and
      `conserved/src/` is byte-identical between `origin/main` and the shared
      tree's local HEAD (`git diff --stat 70d7e15 HEAD -- conserved/src/
      conserved/Cargo.toml` is empty; the three unpushed commits touch only
      `conserved/tests/done_boxes_are_ticked.rs`).
      `effect.rs` is 144 lines of module doc over one line of code:
      `pub use conserved::scope::{Closed, Disposer, Scope, Undo};`.
      `Handle` survives in exactly three places, all tombstones saying the
      name was always wrong (`effect.rs` and the crate README);
      `api/plugin/wasm/host.rs`'s `Handle` is the wasm guest's resource type,
      a different concept its own doc distinguishes, and is untouched.
- [x] `src/mitosys/gates/tests/dependency_tree.rs` accepts the new edge:
      `conserved` gains an entry in `OWNERS` naming the crate(s) that declare
      it, and gains an entry in `CLOSURE` — both lists stay sorted — and
      `cargo test -p mitosys-gates --test dependency_tree` passes.
      `OWNERS` gains one row, `("conserved", &["mitosys-engine-util",
      "mitosys-util-effect"])` — two owners because spec03 declares it from
      the engine floor too. `CLOSURE` gains **four**: `arrayvec`, `blake3`,
      `conserved`, `constant_time_eq` — exactly the four `cargo` reported
      (`Adding arrayvec v0.7.8 / blake3 v1.8.7 / conserved v0.1.0 /
      constant_time_eq v0.4.2`), matching the PRD's prediction.
      `cargo test -p mitosys-gates --test dependency_tree`:
      `8 passed; 0 failed`, including
      `every_third_party_dependency_has_the_owners_recorded` and
      `the_whole_dependency_closure_is_the_recorded_one`.
- [x] Every non-test call site under `src/mitosys/` matching `catch_unwind`
      is checked for a dependency on the old abandon-on-panic behaviour
      around a `Scope`/`Disposer` `close()`. Six sites in the four measured
      files, **none depends** — not one of them wraps a `Scope` or a
      `Disposer`:
      `abi.rs:322` wraps the `dlopen`ed surface entry point, `abi.rs:337` the
      ABI version probe, `abi.rs:374` the `entry_point!` macro's own guest
      body, `agent.rs:918` a settle callback (`cb(report)`),
      `tick_trainer.rs:82` a gnn training run, `tick.rs:132` a tick task.
      The findings are recorded in `util/effect/effect.rs`'s module doc
      (§ "What was checked in the tree before this landed") as well as in the
      commit message, so a future reader meets them at the code rather than
      in a log.
- [x] Every non-test call site matching `.held(` is checked for reliance on
      the old "may lie during an active unwind" behaviour. Two, both in
      `api/plugin/plugin.rs` — `Context::held` at :330 (`self.scope.held()`)
      and `Runtime::fiber_view` at :654 (`ctx.held()`, the census read) —
      and **neither depends**: both are introspection reached from outside a
      close, on a `Context` the runtime still holds, so neither could ever
      have observed the old `[]`-during-unwind. Recorded in `effect.rs`'s
      module doc beside the `catch_unwind` finding.
- [~] `Scope::failed()` is adopted at at least one site currently swallowing
      a teardown failure, **or the commit states there is none** — this is the
      second branch, taken on measurement. There is no such site. The tree
      holds exactly one `Scope` (`api/plugin`'s `Context::scope`,
      `Scope::new()` at `plugin.rs:850`) and closes it at exactly two places,
      `Runtime::deactivate` (:703) and the refused-apply rollback (:880).
      Neither swallows: both let the panic reach the caller, and the refusal
      path has already reported through `Runtime::on_refusal`'s sink before
      the close runs. The six `catch_unwind` sites above wrap no teardown.
      Routing a teardown report anywhere new would mean a third `Notice`
      variant — that type is documented as "a plugin's inactivity has two
      shapes" — or a `tracing` row in `api/plugin`, which owns no `tracing`
      edge today; both are decisions larger than this box, and inventing one
      to avoid saying "none" is what the second branch exists to prevent.
      Stated in `effect.rs`'s module doc and in the crate README.
- [x] `cargo build --workspace` and `cargo test --workspace` pass inside the
      offline container (spec01's mechanism), and the container is where this
      box earns its keep: **`conserved` compiled from `vendor/` with no git
      access and no credential.**
      `docker compose exec dev cargo build --workspace --offline` — exit 0,
      `Finished dev profile ... in 1m 05s`, with
      `Compiling conserved v0.1.0 (https://github.com/yesitsfebreeze/shared.git?rev=70d7e15c…)`
      in the log.
      `docker compose exec dev cargo test --workspace --offline` — exit 0,
      **2138 passed; 0 failed; 21 ignored**.
      Both under `CARGO_NET_OFFLINE=true`, and measured inside the container
      afterwards: `/usr/local/cargo/git/` is **empty** — no `db/`, no
      `checkouts/` — and there is no `~/.gitconfig`. So the git dependency was
      resolved entirely from `vendor/conserved-0.1.0`, which is spec01's
      prediction turned into a fact: the credential requirement collapses to
      one machine, once per rev bump, whoever runs `just vendor`.
      `just vendor` was fully idempotent over the rest of the tree —
      `git status vendor/` shows **four new directories and nothing else
      changed** (`arrayvec-0.7.8` 180K, `blake3-1.8.7` 1.8M,
      `conserved-0.1.0` 200K, `constant_time_eq-0.4.2` 172K), taking it from
      356MB/585 to 358MB/589 with the stub count unmoved at 123. None of the
      four contains a CR byte, so `.gitattributes`' `vendor/** -text` has
      nothing new to protect and was not touched.

## Verify and Proof

```sh
cd ../mitosys
grep -rl mitosys_util_effect src --include='*.rs'
cargo test -p mitosys-gates --test dependency_tree
cargo build --workspace
cargo test --workspace
just check
```


## Addresses corrected 2026-08-28 by the board, measured at `276a400`

`src/plugins/` **does not exist.** A tree-wide rename moved every plugin
directory to `src/builtins/` earlier that day; `src/` now holds `builtins/`,
`mitosys/` and `surfaces/`. The footprint entry and the body reference are
rewritten to `src/builtins/memory/plugin.rs` (package `mitosys-memory`).

Everything else in this spec was **`reproduced`** against the current tree and
needs no change: `src/mitosys/util/effect/` is still correct — that crate did
**not** move under `p8d-floor-split`, because it is membrane's temporal half.
The 11 re-export sites reproduce exactly (`grep -rl mitosys_util_effect src
--include='*.rs'` gives 13 files, minus `util/effect/{lib.rs,tests/effect.rs}`),
as do the four `catch_unwind` files and `api/plugin/plugin.rs`'s `.held(` at
lines 330 and 616.
