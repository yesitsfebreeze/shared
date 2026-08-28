---
type: learning
learning: toolchain
subject: what this workspace is missing, measured — 81 GB of target dirs rebuilding 295 shared crates four times, three of four trees with no CI, and 1,197 dependencies behind no supply-chain gate
binds: [mitosys, model, realm, shared]
status: proposed
date: 2026-08-25
---

# The measurement

Taken 2026-08-25 against the four trees. Everything below is a number from
this machine, not a vendor claim.

| axis | measured |
|---|---|
| Rust source | 3,166 `.rs` across four workspaces |
| prose | 2,075 `.md` — larger than most of the code |
| scripts | 60 `.sh`, 34 `.py`, all ungated |
| dependencies | mitosys 637, model 560, realm 114, shared 72 |
| **overlap** | **295 crates resolved in both mitosys and model** |
| **target dirs** | **80.8 GB** — mitosys 67 G, model 13 G, realm 552 M, shared 254 M |
| toolchain | `1.94.0` pinned identically in all three `rust-toolchain.toml` |
| CI | was **1 of 4** — only `realm`. Now 4 of 4 (see below) |

Two of those lines are the whole story. The compiler is pinned to the same
version everywhere, and 295 crates are compiled from identical source in two
separate 60-GB-class target directories. That is a cache that does not exist
yet. And `just check` — the gate that defines whether these trees are green —
runs in three of four trees only when a human remembers to type it.

# Speed

**1. `sccache` — the largest single win available here.** A shared compilation
cache in front of all four workspaces. The precondition that usually kills
sccache is toolchain drift between projects; this workspace does not have it,
because all three `rust-toolchain.toml` pin `1.94.0`. The 295 overlapping
crates are compiled twice today and would be compiled once.

```
brew install sccache
printf '[build]\nrustc-wrapper = "sccache"\n' >> ~/.cargo/config.toml
sccache --show-stats     # watch the hit rate climb across trees
```

**2. Reclaim the 67 GB.** `mitosys/target` was last written 2026-08-19 and
holds 67 GB — the bulk is artifacts from toolchain and dependency generations
that will never be linked again. `cargo clean` is the blunt instrument;
scheduling it is the real fix, because this regrows.

**3. `bacon` in the other three trees.** Already installed, already wired into
`model/justfile` as `just watch`. `mitosys`, `realm` and `shared` have no
watch recipe at all, so the loop there is still manual `just check`. This
costs nothing — the tool is on the machine.

**4. Do *not* adopt `cargo-nextest` workspace-wide.** It is installed and
unused, which looks like free speed. Measured on `shared` (81 tests, warm):

```
cargo test          543 ms
cargo nextest run   952 ms
```

Nextest runs a process per test; on 81 sub-millisecond tests that overhead
dominates and it loses by 1.8×. Its wins are elsewhere — `--max-fail`, retry
of flaky tests, per-test isolation, JUnit output for CI. Adopt it in
`mitosys` (the 637-dep tree with the slow suite) and leave `shared` on
`cargo test`. This is the one place where the popular answer is wrong for this
workspace.

# Correctness

**1. CI in the three trees that lacked it — DONE 2026-08-25.** `mitosys`,
`model` and `shared` now each carry `.github/workflows/ci.yml`, mirroring that
tree's own gate command byte-for-byte (the principle `realm`'s workflow already
states). `shared` had no justfile at all, so one was added and CI mirrors it.

Writing them surfaced what the absence had been hiding. Every one of the three
gates was RED on `main`, and none of the three failures could have survived a
day of CI:

- `shared` — `done_boxes_are_ticked` failed: `shared-classify` marked
  `state: done` with three unticked acceptance boxes. Closed with evidence.
- `mitosys` — `mitosys-gates`' own `done_boxes_are_ticked` test did not
  **compile** under `-D warnings` (`clippy::manual_ignore_case_cmp`), so the
  board gate had been silently absent from every local `just check` since it
  landed.
- `shared` again — the identical lint in its own copy of that file, which only
  appeared once the new justfile ran clippy. `shared` had never run clippy.
- `model` — two files unformatted, plus a live `one_vocabulary` violation
  (`Error` declared in both `src/utils/fs/mod.rs` and `src/version/ledger.rs`).
  The formatting is fixed; the duplicate is a naming decision, still open.

A gate that only runs when remembered is a convention, not a ratchet. Three of
four trees had quietly proved it.

**2. There is a red gate right now.** `shared`'s own crate test fails on
this working tree:

```
every_done_prd_has_a_ticked_acceptance
  prds/done-means-done/shared-classify/prd.md: 3 unticked box(es)
```

A PRD marked `state: done` carrying three unticked acceptance boxes — exactly
what `done-means-done.md` forbids. The rule caught it; nothing was watching.
That is the argument for item 1, already proven on this machine.

**3. `cargo-deny` — 1,197 dependency resolutions, no gate.** License
compatibility, duplicate versions, and RustSec advisories in one config.
For a workspace this dependency-heavy with zero supply-chain checking, this
is the widest unguarded surface.

**4. `cargo-audit`** if `deny` is too much ceremony — CVE checking alone.

**5. `cargo-machete` — unused dependencies.** At 637 declared deps in mitosys,
some fraction is dead weight paid for on every cold build. Cheap to run once
and see.

**6. `typos` — 2,075 markdown files with no spell gate.** In a workspace where
the prose *is* the specification and PRDs are parsed by tests, a typo in a
frontmatter key is a silent behaviour change, not a cosmetic flaw.

**7. `cargo-mutants` — tests the tests.** It mutates code and reports which
mutations no test catches. This family's culture is ratchets and evidence;
mutation testing is the ratchet applied to the test suite itself, and it fits
here better than it fits most codebases.

**8. `gitleaks`** across the six git repos — these moved to a personal account
on 2026-08-23, so history was rewritten recently enough to be worth one scan.

# Productiveness

- **`ruff`** — 34 Python files (`prds/`, `pearde/`) with no linter, no
  formatter, no `pyproject.toml`. `uv` is already installed and unused. The
  board tooling is the least-gated code in the workspace.
- **`cargo-insta`** — snapshot testing. The `shared` crate's tests assert on
  document shape; insta is built for exactly that and makes the diffs readable.
- **`watchexec`** — the `bacon` equivalent for the Python and shell side.
- **`cargo-expand`** — for macro-generated code when a derive misbehaves.
- **`difftastic`** — structural diffs. `delta` is installed already and is the
  better default; difftastic wins on refactors that only move code.

# On two tools that look stale and are not

`hyperfine` (116 days) and `tokei` (110 days) both read as "slow" by the
scout's activity heuristic. Both are finished tools solving bounded problems,
not abandoned ones. The heuristic is right to flag them and wrong about what
the flag means — worth remembering before trusting that column elsewhere.

# The install, in payoff order

```
brew install sccache typos-cli gitleaks
cargo install cargo-deny cargo-machete cargo-mutants
cargo install cargo-insta          # if snapshot tests are wanted
uv tool install ruff
```

Everything above is MIT or Apache-2.0 and was verified active within the last
two weeks on 2026-08-25.
