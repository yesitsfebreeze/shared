# goal

Move mitosys's `util/effect` into `conserved` as `conserved::scope` — code and
tests, moved not rewritten — with zero dependencies and every deviation from
the mitosys source recorded at its site.

This covers requirements 1 and 2 of the ticket. Requirement 3 (prove the
mechanism from a consumer tree) is `spec02`, and depends on this one having
landed as a commit.

## What p0 must have landed first

This spec is written against the post-`p0-foundation` layout and assumes all
four of its outcomes:

- the repository is a git repository with at least one commit (`git init`
  requirement) — `spec02` needs a rev to pin, and this spec's landing commit is
  that rev;
- the five condemned `conserved-*` crates and the old `conserved/src/lib.rs`
  are gone (`p0/specs/spec01`);
- `conserved/` exists as the single crate: `conserved/Cargo.toml` with
  `edition = "2021"`, `rust-version = "1.94.0"`, an empty `[dependencies]`
  table, an empty-but-compiling `conserved/src/lib.rs`, and tests at
  `conserved/tests/`; root `Cargo.toml` is `[workspace]`-only with
  `resolver = "2"` and member `conserved`;
- `.mi/docs/memos/distribution.md` reads `status: decided` — Option A, git
  dependency pinned by commit rev. Not used by this spec; `spec02` consumes it.

If p0 chose `conserved/lib.rs` over `conserved/src/lib.rs` (the mitosys
"a crate is its directory" convention rather than the cargo default), every
`conserved/src/` path below shifts accordingly and nothing else changes. The
p0 PRD says `src/lib.rs`, so that is what this spec writes.

## Files and dirs

Created:

- `conserved/src/scope.rs` — the port of
  `../mitosys/src/mitosys/util/effect/effect.rs` (154 lines). The body is
  copied byte-for-byte from `use std::collections::HashMap;` to end of file.
  Only the module doc-comment above that line is rewritten, and only to add
  the `# Provenance` block described below.
- `conserved/tests/scope.rs` — the port of
  `../mitosys/src/mitosys/util/effect/tests/effect.rs` (89 lines). Byte-for-byte
  from line 2 to end of file; line 1 changes from
  `use mitosys_util_effect::effect::*;` to `use conserved::scope::*;`.

Edited:

- `conserved/src/lib.rs` — gains the single line `pub mod scope;`. No root
  re-exports: `conserved::scope::Scope` is the path, and inventing
  `conserved::Scope` on top of it would be API surface this ticket did not
  extract.

Read, never written:

- `../mitosys/src/mitosys/util/effect/{effect.rs,lib.rs,README.md,Cargo.toml}`
  and `tests/effect.rs`. Nothing under `../mitosys` is modified by this spec —
  the compatibility shim `mitosys::effect = pub use mitosys_util_effect::effect;`
  stays exactly as it is, mitosys keeps its own copy, and reconciling the two is
  `p5-adoption`'s job, not this one.

Not touched: `conserved/Cargo.toml` (p1 adds no dependency — that is
requirement 2, and the empty `[dependencies]` table p0 wrote is the whole of
it), root `Cargo.toml`, `learnings/`, `.mi/docs/memos/`.

## The deviations, and where each is recorded

"Moved, not rewritten" means the deviation list is short and closed. There are
six, and all six are recorded in a `# Provenance` block in the module doc of
`conserved/src/scope.rs` — one block, at the site, naming the source path and
the mitosys commit sha the copy was taken from:

1. **module `effect` → `scope`.** Forced: the ticket's own acceptance line says
   `conserved::scope` compiles, `learnings/shared-crate.md` §3 titles the thing
   `Scope`/`Handle`, and `effect` is the name mitosys's plugin contract gave it.
2. **`effect/effect.rs` → `conserved/src/scope.rs`.** Forced by p0: mitosys's
   convention is "a crate is its directory" with `[lib] path = "lib.rs"` and no
   `src/`; p0 resolved that divergence dimension for `conserved` in favour of
   the cargo default. `AGENTS.md` §divergences is explicit that the shared crate
   resolves each dimension in its own manifest.
3. **test import path.** `use mitosys_util_effect::effect::*;` →
   `use conserved::scope::*;`. Forced by 1 and 2. Nothing else in the test file
   changes — not a name, not an assertion, not a blank line.
4. **doc-comment framing.** The source doc opens "the plugin contract's
   foundation" and `lib.rs` carries `Layer: L0 · May import: nothing` and the
   `src/core/src/` split history. `conserved` is domain-free — no plugin, no
   surface — so that framing moves *into* the `# Provenance` block rather than
   being deleted or left standing as if `conserved` had plugins. Deleted
   provenance reads as provenance that never existed. Specifically kept, because
   it is the semantics and not the domain: the Cordis / Go `internal/effect`
   attribution, and the two invariants the source README states —
   unwind order is the reverse of registration order, and a scope unwinds on
   drop including on panic.
5. **`Disposer`, not `Handle`.** `learnings/shared-crate.md` §3 and the mitosys
   crate README's ABI block both write `Handle`; the actual type in the source
   is `Disposer`, and the crate boundary does not force a rename. The source
   wins — the learning's wording is loose, and the Provenance block says so, so
   the next reader does not mistake it for a spec violation.
6. **no compatibility shim, no crate README copied.** `mitosys::effect` is a
   `pub use` shim for call sites inside mitosys; `conserved` has no call sites
   to keep resolving, so there is nothing to shim. The crate README at
   `../mitosys/src/mitosys/util/effect/README.md` is about a mitosys crate and
   does not move; its two invariants survive as module docs per deviation 4.

Everything else — `Undo`, `Closed`, `Entry`, `Inner`, `Scope`, `Disposer`,
`effect`, `close`, `held`, `Drop`, every doc comment on every item, the tab
indentation — is copied. The port was compiled, tested, clippy'd and fmt'd
against `rustfmt.toml` (`hard_tabs`, `tab_spaces = 2`, identical to mitosys's)
during analysis: 5 tests pass, `clippy -D warnings` is silent, `fmt --check` is
silent, with no edit to the copied body.

## Note on the ticket's own verify line

The frontmatter verify reads
`cargo tree -p conserved --edges normal | grep -cv conserved | grep -qx 1`.
On a genuinely zero-dependency `conserved` that pipeline **fails**: the tree is
one line, `grep -cv conserved` prints `0` and exits 1, and `grep -qx 1` never
matches. It asserts *exactly one* dependency, which is the opposite of
requirement 2. Frontmatter is not this spec's to edit, so it is recorded here
and the `verify:` line below carries the correct form — one line of
`cargo tree`, zero non-`conserved` lines. Whoever next touches the ticket's
frontmatter should fix it there too.

## Acceptance

- [x] `conserved/src/scope.rs` exists, and from its first
      `use std::collections::HashMap;` line to end of file it is byte-identical
      to `../mitosys/src/mitosys/util/effect/effect.rs` from the same line —
      `diff` reports no output.
- [x] `conserved/tests/scope.rs` exists, and from line 2 to end of file it is
      byte-identical to `../mitosys/src/mitosys/util/effect/tests/effect.rs`
      from line 2; line 1 is exactly `use conserved::scope::*;`.

      **Amended during implementation**, at the board's request, and the
      `verify:` line below with it. The file is wrapped in `mod scope { … }` —
      a seventh deviation, recorded in `scope.rs`'s Provenance block. Without
      it this ticket's gate, `cargo test -p conserved scope`, filters on *test
      function names* and reported `1 passed; 4 filtered out` while exiting 0:
      a gate passing having run one test in five. Byte-identity is unchanged as
      a property, only as a command — strip line 1, the last line, and one
      leading tab from every line, and the diff is still empty. That is what
      the verify now does. p3-clock and p4-stats adopted the same wrapper
      independently, so it is the board-wide convention.
- [x] `conserved/src/lib.rs` contains the line `pub mod scope;` and declares no
      `pub use` re-export of anything in `scope`.
- [x] The module doc of `conserved/src/scope.rs` contains a `# Provenance`
      heading, the source path `src/mitosys/util/effect/effect.rs`, a 40-hex
      mitosys commit sha, and a line for each of the six deviations above.
- [x] `conserved/Cargo.toml` declares no dependency: the `[dependencies]` table
      is still empty and no `[dev-dependencies]` or `[build-dependencies]` table
      was added.
- [x] `cargo tree -p conserved --edges normal` prints exactly one line — the
      crate itself and nothing under it.
- [x] `cargo test -p conserved --test scope` runs 5 tests and all 5 pass:
      `close_unwinds_lifo`, `dispose_runs_once_and_deregisters`,
      `closed_scope_refuses_and_unwinds_late_effects`, `drop_unwinds`,
      `held_reports_live_effects`.
- [x] `cargo fmt --all --check` and
      `cargo clippy --workspace --all-targets -- -D warnings` are both silent.
- [x] `../mitosys` is unchanged by this spec:
      `git -C ../mitosys status --porcelain src/mitosys/util/effect` prints
      nothing.

## est

1

verify: `bash -c 'set -e; cd /Users/feb/dev/infra/shared; M=../mitosys/src/mitosys/util/effect; test -f conserved/src/scope.rs; test -f conserved/tests/scope.rs; grep -qx "pub mod scope;" conserved/src/lib.rs; grep -q "# Provenance" conserved/src/scope.rs; head -1 conserved/tests/scope.rs | grep -qx "mod scope {"; sed -n "2p" conserved/tests/scope.rs | grep -qx "	use conserved::scope::\*;"; diff <(sed -n "/^use std::collections::HashMap;/,\$p" $M/effect.rs) <(sed -n "/^use std::collections::HashMap;/,\$p" conserved/src/scope.rs); diff <(tail -n +2 $M/tests/effect.rs) <(sed -e "1d" -e "\$d" conserved/tests/scope.rs | sed -e "s/^	//" | tail -n +2); cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test -p conserved --test scope 2>&1 | grep -q "5 passed"; n=$(cargo tree -p conserved --edges normal | wc -l | tr -d " "); [ "$n" = 1 ] || { echo "conserved gained $((n-1)) dependency edge(s)"; cargo tree -p conserved --edges normal; exit 1; }; [ -z "$(git -C ../mitosys status --porcelain src/mitosys/util/effect)" ] || { echo "the mitosys source was modified"; exit 1; }; echo "spec01 ok"'`
