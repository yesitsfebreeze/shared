---
state: done
est: 2.5h
mode: afk
priority: 10
verify: "cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && git rev-parse --git-dir && ./scripts/fresh-clone-check.sh"
---

# P0 — foundation: a repo that can hold the crate

Purpose: make this directory a place code can actually live before any code
moves. Today it is not a git repository, the workspace manifest is invalid
(one `Cargo.toml` carrying both `[workspace]` and a `[package]` named
`conserved-workspace`), and five pre-split crates from a runaway automated run
sit where the proposal says one crate belongs. `cargo build` does not pass.

## Requirements

- [x] **git init** — `git init` + an initial commit of the board, the
      learnings, and the reset workspace. The board protocol claims by
      commit, and `learnings/README.md` already notes "the record only grows"
      has no subject without version control. Without this, no node on this
      board is claimable.
      Evidence (from `shared-classify/prd.md`'s Classification table):
      `git log --reverse --oneline | head -1` → `ab154f7 foundation: the
      reset workspace, the board, the learnings` (the repo's first commit,
      2026-08-21). The shared tree is under version control with the board
      and learnings in the same initial commit.
- [x] **Condemn the scaffold** — delete `conserved-alloc/`, `conserved-net/`,
      `conserved-deriv/`, `conserved-derive/`, `conserved-core/`, and the
      current `conserved/src/lib.rs`. None of it compiles (see
      `.mi/docs/memos/scaffold-reset.md` for the itemized evidence) and the
      pre-split directly contradicts the proposal's "one crate to start — do
      not pre-split; let the gate decide". The memo is the record; the
      deletion is the act.
      Evidence (from `shared-classify/prd.md`'s Classification table):
      negative evidence — `find . -maxdepth 2 -name 'conserved-alloc' -o
      -name 'conserved-net' -o -name 'conserved-deriv' -o -name
      'conserved-derive' -o -name 'conserved-core'` returns nothing on
      2026-08-24 (run at `/Users/feb/dev/infra/shared`). The five condemned
      crates do not exist on disk.
- [x] **One crate, one manifest** — `conserved/` with a valid `Cargo.toml`
      (edition 2021, `rust-version = "1.94.0"`), an empty-but-compiling
      `src/lib.rs`, tests at `conserved/tests/` (the mitosys shape, per
      `AGENTS.md` §divergences — the shared crate resolves each dimension
      explicitly). Root manifest becomes `[workspace]` only, `resolver = "2"`,
      member `conserved`.
      Evidence (from `shared-classify/prd.md`'s Classification table):
      `head -5 /Users/feb/dev/infra/shared/Cargo.toml` → `[workspace] /
      resolver = "2" / members = ["conserved"]`. `ls
      /Users/feb/dev/infra/shared/conserved/tests/` → `clock_instant.rs,
      clock_serde.rs, clock_source.rs, content_id_props.rs,
      content_id_serde.rs, content_id.rs, load_scope.rs, load_throughput.rs,
      load_unwind_panic.rs, scope.rs, smoke.rs, stats.rs` (12 test files).
      The root manifest is `[workspace]`-only and `conserved` is the single
      member.
- [x] **Distribution decision — put to the user, not guessed.** The user
      settled the *requirement* 2026-08-20: the crate must be distributable
      to all other Rust repos, which rules out path-dependency-only (option 3
      of `learnings/shared-crate.md` §"Where it lives"). The *mechanism* —
      git dependency pinned by commit vs. vendored copy with recorded source
      hash — is still open, and the deciding constraint is mitosys's offline
      dev container (a git dep needs a vendored registry cache to build
      there). Frame both in `.mi/docs/memos/distribution.md`, escalate,
      record the answer there with status `decided`.
      Evidence (from `shared-classify/prd.md`'s Classification table):
      `grep '^status:\|^decided:' /Users/feb/dev/infra/shared/.mi/docs/memos/
      distribution.md` → `status: decided` / `decided: 2026-08-21`. The
      distribution memo carries the decision (`Option A, git dependency
      pinned by commit rev`); the `## Answers` section on this PRD already
      names the mechanism as settled.

## Acceptance

`cargo build --workspace && cargo test --workspace` passes from a fresh clone
of this repository, and the distribution memo carries a decision, not options.

## Questions

*One round, put to the user 2026-08-21 and answered the same day. Written back
2026-08-28 — the fork lived in `.mi/docs/memos/distribution.md` and in the
fourth requirement above, never under this heading, which left an `## Answers`
section with no question above it. The requirement itself is the round's
authority: "put to the user, not guessed."*

1. **The crate must be distributable to every other Rust repo — path-only is
   already ruled out. Which mechanism: a git dependency pinned by commit rev,
   or a vendored copy with a recorded source hash?** The deciding constraint is
   mitosys's offline dev container, which has no network at build time and so
   needs a vendored registry cache to resolve a git dep at all.

   - **(a) Git dependency pinned by commit rev. (recommended)** One source of
     truth, one rev every consumer pins, and an update is a rev bump. Cost:
     mitosys's offline container needs a `cargo vendor` or a pre-populated
     registry cache — scoped as mitosys-side follow-on work, designed before
     p5's mitosys adoption step, and not a reason to reopen the mechanism here.
   - **(b) Vendored copy with a recorded source hash.** Builds offline
     everywhere with no container work. Cost: every consumer carries a copy,
     and "which rev is this" becomes a hash to check rather than a rev to read.

## Answers

1. **Distribution mechanism** — settled 2026-08-21: **Option A, git dependency
   pinned by commit rev**. `.mi/docs/memos/distribution.md` carries
   `status: decided`; do not re-escalate this. mitosys's offline dev container
   is explicitly scoped as mitosys-side follow-up work (a `cargo vendor` /
   pre-populated registry cache, designed before p5's mitosys adoption step),
   not a reason to reopen the mechanism here.
