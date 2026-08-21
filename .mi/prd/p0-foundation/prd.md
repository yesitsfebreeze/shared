---
state: claimed
claim: impl-p0-foundation
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

- [ ] **git init** — `git init` + an initial commit of the board, the
      learnings, and the reset workspace. The board protocol claims by
      commit, and `learnings/README.md` already notes "the record only grows"
      has no subject without version control. Without this, no node on this
      board is claimable.
- [ ] **Condemn the scaffold** — delete `conserved-alloc/`, `conserved-net/`,
      `conserved-deriv/`, `conserved-derive/`, `conserved-core/`, and the
      current `conserved/src/lib.rs`. None of it compiles (see
      `.mi/docs/memos/scaffold-reset.md` for the itemized evidence) and the
      pre-split directly contradicts the proposal's "one crate to start — do
      not pre-split; let the gate decide". The memo is the record; the
      deletion is the act.
- [ ] **One crate, one manifest** — `conserved/` with a valid `Cargo.toml`
      (edition 2021, `rust-version = "1.94.0"`), an empty-but-compiling
      `src/lib.rs`, tests at `conserved/tests/` (the mitosys shape, per
      `AGENTS.md` §divergences — the shared crate resolves each dimension
      explicitly). Root manifest becomes `[workspace]` only, `resolver = "2"`,
      member `conserved`.
- [ ] **Distribution decision — put to the user, not guessed.** The user
      settled the *requirement* 2026-08-20: the crate must be distributable
      to all other Rust repos, which rules out path-dependency-only (option 3
      of `learnings/shared-crate.md` §"Where it lives"). The *mechanism* —
      git dependency pinned by commit vs. vendored copy with recorded source
      hash — is still open, and the deciding constraint is mitosys's offline
      dev container (a git dep needs a vendored registry cache to build
      there). Frame both in `.mi/docs/memos/distribution.md`, escalate,
      record the answer there with status `decided`.

## Acceptance

`cargo build --workspace && cargo test --workspace` passes from a fresh clone
of this repository, and the distribution memo carries a decision, not options.

## Answers

1. **Distribution mechanism** — settled 2026-08-21: **Option A, git dependency
   pinned by commit rev**. `.mi/docs/memos/distribution.md` carries
   `status: decided`; do not re-escalate this. mitosys's offline dev container
   is explicitly scoped as mitosys-side follow-up work (a `cargo vendor` /
   pre-populated registry cache, designed before p5's mitosys adoption step),
   not a reason to reopen the mechanism here.
