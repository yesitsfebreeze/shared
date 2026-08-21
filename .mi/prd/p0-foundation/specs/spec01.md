# goal

Delete the condemned five-crate scaffold, so the tree holds exactly one crate
directory (`conserved/`) and no source that does not compile.

The reason is already recorded — `.mi/docs/memos/scaffold-reset.md`, status
`decided`, itemized. This spec is the act, not the argument. Nothing here is
salvaged: every type the scaffold sketched is specified better in the
learnings it ignored.

## Files and dirs

Deleted outright:

- `conserved-alloc/` (manifest only; `conserved-core = { path = ".." }`
  resolves to the workspace root)
- `conserved-net/` (same broken path dep; a "network sublayer" no consumer
  needs — fails admission criterion 1)
- `conserved-deriv/` (manifest names itself `conserved-derive`; duplicate)
- `conserved-derive/` (`Cargo.toml` is a prose sentence, not TOML; proc-macro
  source with no `proc-macro = true`)
- `conserved-core/` (reinvents `Instant` as an incrementing counter, the exact
  opposite of `learnings/clock.md`; pulls `chrono` against a spec that says the
  clock module has no dependencies; declares a `[[bin]]` whose `src/main.rs`
  does not exist)
- `conserved/src/lib.rs` (`blake3::hash_bytes`, `std::convert::Error`,
  `s.parse::<[u8; 32]>()`, `?` in non-`Result` fns — none of it exists)
- `conserved/doc/` (empty directory left by the same run)

Kept: `conserved/` as a directory — spec02 refills it.

Not touched: `learnings/`, `.mi/`, `AGENTS.md`, `README.md`, root `Cargo.toml`
(spec02 owns that one).

## Acceptance

- [ ] `conserved-alloc/`, `conserved-net/`, `conserved-deriv/`,
      `conserved-derive/`, `conserved-core/` do not exist on disk.
- [ ] `conserved/doc/` does not exist.
- [ ] The condemned source is gone, not moved: no file anywhere under
      `conserved/` contains `hash_bytes`, and no file in the repo is a copy of
      a deleted crate under a new name (`ls -d conserved*` lists exactly
      `conserved`).
- [ ] Nothing under `learnings/` or `.mi/docs/memos/` was edited by this spec —
      the memo is the record and the record only grows.

## est

0.25

verify: `sh -c 'set -e; for d in conserved-core conserved-alloc conserved-net conserved-deriv conserved-derive conserved/doc; do if [ -e "$d" ]; then echo "still present: $d"; exit 1; fi; done; if grep -rqF "hash_bytes" conserved; then echo "condemned source survives"; exit 1; fi; if [ "$(ls -d conserved* | tr -d "\n")" != "conserved" ]; then echo "unexpected crate dirs"; exit 1; fi; echo "scaffold condemned"'`
