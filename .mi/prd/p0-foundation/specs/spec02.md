# goal

One crate, one manifest: a `[workspace]`-only root and a single member,
`conserved`, that is empty but compiles, formats, lints and tests clean — and
that resolves every dimension `AGENTS.md` §divergences leaves contradictory.

The crate compiles for the strictest consumer: **edition 2021, rustc 1.94.0**
(mitosys). No dependencies enter here — `blake3` arrives in p2, behind
`ContentId`, and only if the gate lets it.

## Files and dirs

- `Cargo.toml` (root) — rewritten: `[workspace]` only. The current file
  carries both `[workspace]` and a `[package]` named `conserved-workspace`
  with no target, which is why `cargo build` does not pass today.

  ```toml
  [workspace]
  resolver = "2"
  members = ["conserved"]

  [workspace.package]
  version = "0.1.0"
  edition = "2021"
  rust-version = "1.94.0"

  # Dependency convention (mitosys's, per AGENTS.md §divergences): every
  # external dependency is pinned once here and inherited by members.
  # Empty at p0 — blake3 enters in p2, behind ContentId.
  [workspace.dependencies]
  ```

- `conserved/Cargo.toml` — new. `name = "conserved"`, `version.workspace`,
  `edition.workspace`, `rust-version.workspace`, a one-line `description`, and
  an empty `[dependencies]`.
- `conserved/src/lib.rs` — new. Crate-level doc comment only: what the crate
  is, that it is deliberately empty at p0, and the four resolutions below.
  `#![forbid(unsafe_code)]`. No `pub` items, no `mod`s.
- `conserved/tests/` — new directory, with one integration test
  (`conserved/tests/smoke.rs`) that references the crate (`use conserved as _;`)
  so it proves linkage, not just that a test binary ran.

Not touched: `rustfmt.toml`, `.editorconfig` (both already correct — hard
tabs, width 2; Rust written here must be tab-indented or `cargo fmt --check`
fails).

## The four divergences, resolved here

| dimension | resolution | where it is visible |
|---|---|---|
| test law | `conserved/tests/` (mitosys shape); no beside-the-module tests | the directory + `smoke.rs` |
| toolchain | edition 2021, pinned `rust-version = "1.94.0"` | `[workspace.package]` |
| packaging | one crate, no pre-split | `members = ["conserved"]` |
| dependencies | pinned once in `[workspace.dependencies]`, inherited | root manifest |

## Acceptance

- [x] Root `Cargo.toml` contains no `[package]` section and no
      `conserved-workspace` anywhere; it declares `resolver = "2"` and
      `members = ["conserved"]`.
- [x] `cargo metadata --no-deps` reports the single package `conserved` with
      `"edition":"2021"` and `"rust_version":"1.94.0"`.
- [x] `conserved/src/lib.rs` exists, is doc-comment-and-attributes only (no
      `pub fn`, no `pub struct`, no `mod`), and `cargo build --workspace`
      passes.
- [x] `conserved/tests/smoke.rs` exists, names the crate, and
      `cargo test --workspace` runs at least one passing test. There is no
      root-level `tests/` directory and no `conserved/src/*/tests/`.
- [x] `cargo fmt --all --check` passes against the repo's `rustfmt.toml`
      (hard tabs, width 2) — the code was not written with spaces.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [x] No dependency was added: `conserved/Cargo.toml`'s `[dependencies]` is
      empty and `cargo metadata` resolves zero external packages.

## est

0.75

verify: `sh -c 'set -e; if grep -q "^\[package\]" Cargo.toml; then echo "root manifest still carries a package"; exit 1; fi; if grep -q "conserved-workspace" Cargo.toml; then echo "conserved-workspace survives"; exit 1; fi; cargo metadata --format-version 1 --no-deps | grep -q "\"rust_version\":\"1.94.0\""; cargo metadata --format-version 1 --no-deps | grep -q "\"edition\":\"2021\""; test -f conserved/tests/smoke.rs; test ! -d tests; cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; cargo build --workspace; cargo test --workspace'`
