---
type: memo
memo: scaffold-reset
subject: the pre-split conserved-* scaffold found 2026-08-20 is condemned — it does not compile, contradicts the recorded proposal, and p0 deletes it
status: decided
date: 2026-08-20
---

# scaffold-reset — why the five crates get deleted

On 2026-08-20 this repository was found holding a five-crate scaffold —
`conserved/`, `conserved-core/`, `conserved-alloc/`, `conserved-net/`,
`conserved-deriv/`, `conserved-derive/` — apparently left by an automated run
(`.pi/worktrees/wf-learn-*` are the fingerprints). This memo is the record of
why it goes, so the deletion in `p0-foundation` is an act with a reason, not
a loss.

## It contradicts the proposal it claims to implement

`learnings/shared-crate.md` says, verbatim: one crate to start; roughly
600–800 lines; **"Do not pre-split; let the gate decide."** The scaffold
pre-splits into five, invents an `alloc` shard ("memory allocators") and a
`net` shard ("network sublayer") that appear nowhere in any learning and fail
the admission test's criterion 1 outright — no consumer needs them today.
A `derive` macro crate likewise: `ContentId` needs no derive.

## It does not compile — itemized

- Root `Cargo.toml` carries both `[workspace]` and a `[package]` named
  `conserved-workspace` with no target; `conserved/` (which has sources) is
  not a workspace member; `conserved-deriv` and `conserved-derive` both call
  themselves `conserved-derive`.
- `conserved-alloc`, `conserved-net`, `conserved-deriv` all depend on
  `conserved-core = { path = ".." }` — which resolves to the workspace root,
  not `conserved-core`.
- `conserved-derive/Cargo.toml` is a prose sentence, not TOML, and its
  `lib.rs` is a proc-macro with no `proc-macro = true` anywhere.
- `conserved/src/lib.rs`: `blake3::hash_bytes(bytes)?` (no such function, `?`
  in a non-Result fn), `self.as_ref().unwrap()` on a `[u8; 32]`,
  `std::convert::Error` (does not exist), `s.parse::<[u8; 32]>()` (no
  `FromStr` for arrays), `Display`/`FromStr` used without imports.
- `conserved-core/src/lib.rs` reinvents `Instant` as an incrementing counter
  ("suitable for test/deterministic contexts") — the exact opposite of
  `learnings/clock.md`, where `Instant` is a real timestamp and `SystemClock`
  is the one permitted reader — and pulls `chrono` against a spec that says
  the clock module has **no dependencies**.

## What replaces it

`p0-foundation`: a single `conserved/` crate (edition 2021,
`rust-version = "1.94.0"`, tests at `conserved/tests/`), root manifest
`[workspace]`-only. The split to `conserved-id` happens if and when mitosys's
`dependency_tree.rs` gate objects to `blake3` under `Scope` consumers — the
gate decides (`learnings/shared-crate.md` §"Size and shape"), not a scaffold.

Nothing in the deleted code is worth salvaging; every type it sketched is
specified better in the learnings it ignored.
