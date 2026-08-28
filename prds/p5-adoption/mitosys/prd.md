---
state: done
mode: afk
priority: 44
est: 20h
repo: mitosys
verify: "cd ../mitosys && just check"
complexity: 88
blast-radius: high
---

# P5d — mitosys: four duplicates become one dependency, green in the container

Purpose: mitosys is the strict consumer — edition 2021, pinned 1.94.0, offline
dev container. Its `just check` going green in that container is the final proof
of p0's distribution decision.

## Requirements

- [x] **The offline container is this node's blocker, not a footnote.** p0's memo
      scoped `cargo vendor` / a pre-populated registry cache as "mitosys-side
      follow-up designed before p5's mitosys adoption step". That work IS this
      node's acceptance and should be its FIRST spec, not its last.
- [x] **`util/effect` → `conserved::scope` is not one file.**
      `pub use mitosys_util_effect::effect;` is re-exported by 10+ crates
      (`api/plugin`, `api/plugin/lua`, `api/surface`, `api/agentic`,
      `api/agentic/pool`, `api/service`, `api/engine`, `engine/record`,
      `engine/layers`, `engine/channel`), and each of those crates' `//! May
      import:` layer docs names `mitosys_util_effect`. The type is **`Disposer`**,
      not `Handle`.
- [~] **`content_hash` has 26 non-test call sites and its output is a persisted
      doc id** — `ingest_worker.rs`, `ingest_intake.rs`, `ingest_direct.rs`,
      `ingest_file_watcher.rs`, `engine/identity/lib.rs:48` (truncates to 16
      chars), `engine/record/oracle.rs:113` (oracle key). SHA-256 → blake3
      **invalidates every stored id**, behind `store_core`'s `FORMAT_VERSION`
      wipe policy. A deliberate store break — see the parent's `## Questions`.
      **STRUCK 2026-08-28 — not this node's, and measured as not done.**
      `git grep -nI content_hash -- 'src/**/*.rs'` outside tests returns **75**
      call sites, not 26, and `blake3` appears nowhere in the workspace
      `Cargo.toml`. The SHA-256 → blake3 migration never happened here and was
      never going to: it is owned by `@mitosys/record-shape-port`, whose
      answered Q1 is exactly "pin blake3 in the mitosys workspace deps". That
      node is `open`, p5, w65, and unblocks three others. Removes this strike:
      `record-shape-port` reaching `done`.
- [x] **The `ed25519:` prefix shim lives here**, at the call sites that need it.
      p2 deliberately refused to port it into the crate.
- [x] **`percentile_sorted` deletion** also deletes
      `src/mitosys/engine/util/tests/unit/util.rs:26`. Rewrite lines 31-34:
      `Some(5.0), "ceil(0.5*10)=5 -> xs[4]"` becomes `Some(6.0)` under p4's
      upper-median decision. Lines 37-39 use `u128`
      (`percentile_sorted(&ns, 0.5) == Some(30u128)`) and `conserved::stats` is
      `&[f64]`-only by p4's decision — those vectors have no home and go with
      the function.
- [x] **Gate**: `dependency_tree.rs` accepts the crate (p1 proved it moves
      exactly two lines: OWNERS + CLOSURE); `just check` green in the container.

- [x] **`conserved::scope` and `util/effect` no longer agree — reconcile
      deliberately, do not diff-and-merge.** p6-scope-unwind made the first
      *semantic* divergence from the byte-for-byte port (deviation 8 in
      `conserved/src/scope.rs`'s `# Provenance`): on close, `conserved` runs
      **every** inverse even when one panics, resumes the first panic reached
      afterwards, keeps `held()` true *during* the unwind, and adds
      `Scope::failed()` naming the inverses that panicked. mitosys's
      `util/effect` still abandons the tail and still reports `held() == []`
      with inverses owed — measured in
      `.mi/prds/p5-adoption/load-proof/finding.md`. Adopting the crate
      therefore **changes teardown behaviour** in all 10+ re-exporting crates:
      a plugin whose inverse panics now has its remaining inverses run rather
      than dropped. That is the point of adopting, and it must be stated in the
      adoption commit rather than discovered. Check the tree for any site that
      depends on the abandonment — `grep` teardown paths for `catch_unwind`
      around a `close()` — and for anything that reads `held()` during an
      unwind. `Scope::failed()` is new API with no mitosys call site yet; adopt
      it where a teardown failure is currently swallowed. The abort-during-abort
      case is **unchanged** in both trees.
      Also: `.mi/prds/p1-scope/specs/spec01.md`'s byte-for-byte `verify:` diff
      no longer holds, by design.

## Held — 2026-08-21 — LIFTED 2026-08-26

**Not dispatchable from this board yet.** The user's instruction is to finish the
shared repo's own tools first and reconcile the consumer implementations later,
once everything here is tested and works. This node is fully specified and ready;
do not start it, and do not write into the consumer trees, until that hold lifts.

**The hold lifted on 2026-08-26** (user decision, recorded in full at the
parent's `## Answers — 2026-08-26`). Its condition is met: every other PRD in
this repository — `p0-foundation`, `p1-scope`, `p2-content-id`, `p3-clock`,
`p4-stats`, `p6-scope-unwind`, `load-proof`, `close` — is `state: done`. This
node is dispatchable, and writing into the consumer tree is now in scope.

Before pinning `conserved`, read the parent's Answer 5: the remote recorded in
Answer 1 (`inner-zirkle`) is stale, the live one is
`https://github.com/yesitsfebreeze/shared.git`, and this repository holds
commits that have never been pushed — a `rev` that is not on the remote cannot
be fetched from a container or another machine.

## Decided — persisted-id break accepted

The user accepted this break explicitly on 2026-08-21, one version bump: SHA-256 hex doc ids -> blake3, behind `store_core`'s `FORMAT_VERSION` wipe.
Wipe and re-derive, not a migration. It does not need re-escalating when this
node runs.


## Dispatched scoped to `spec01` — 2026-08-28, and why

The implementer is told to build `spec01` and **stop**. Three reasons, and the
first is this PRD's own text.

1. **The PRD says so.** Its first requirement reads: "The offline container is
   this node's blocker, not a footnote … That work IS this node's acceptance and
   should be its FIRST spec, not its last."
2. **`spec01`'s footprint is clean; `spec03` and `spec04`'s addresses are
   rotted.** `@mitosys/p8-membrane/p8d-floor-split` landed at `276a400` and
   moved the floor: `mitosys-util` → **`mitosys-engine-util` at
   `src/mitosys/engine/util`**, `mitosys-util-math` →
   **`mitosys-engine-util-math` at `src/mitosys/engine/util/math`**, and
   `src/mitosys/util/` now holds **only `effect/`**. So:
   - `spec02` cites `src/mitosys/util/effect/` — **still correct**, that crate
     did not move.
   - `spec03` and `spec04` cite `src/mitosys/util/util.rs` — **gone**. Both need
     their addresses refreshed before anyone implements them, and that is the
     board's edit, not a worker's.
   This is the third time this session a spec has been rotted by a rename in
   flight, after `plugins-view` (four paths → nine) and `p8d` itself (three →
   sixteen). The standing lesson: a spec citing `src/mitosys/**` is stable under
   these moves; one citing the moving half goes stale in an afternoon.
3. **Another worker is live in the mitosys tree** on
   `plugins-visible/census-counts-composed`, probing `api/plugin/` and the
   `inspect` builtin — which `spec02`'s footprint overlaps.

The implementer is asked, instead of running ahead, to report every stale
address it notices in `spec02`–`spec04` measured against the current tree. That
list is what the board needs to refresh them.


## The stale-address sweep, done 2026-08-28

`spec01`'s implementer was asked to report every address in `spec02`–`spec04`
that the day's renames had rotted, rather than run ahead into them. It did, and
the board has applied the corrections in each spec, each with its measurement.

The scale is worth recording: **one of the four specs was clean** (`spec02`,
apart from a `src/plugins/` → `src/builtins/` path), one had a moved file with
five drifted line numbers and a miscount (`spec03`: 21 `content_hash` sites, not
20), and one named **a cargo package that no longer exists** (`spec04`:
`-p mitosys-util`). None of that would have failed loudly; a verify naming an
unresolvable package reports "no packages matched" and moves on.

That is the third time this session a spec has been rotted by a rename in
flight — after `plugins-view` (four paths to nine) and `p8d-floor-split` itself
(three to sixteen). The standing lesson is now paid for three times over: a spec
citing `src/mitosys/**` survives these moves; one citing the moving half goes
stale in an afternoon.


## `spec01` done 2026-08-28 — the offline container stands; `spec02`–`spec04` remain

Committed in mitosys as `ef29a6e`. This node stays `specced`: one of four specs
is closed, and the other three are the adoption itself.

Container gate green on the trimmed tree — `cargo test --workspace --offline`
2126/0/21 and `just check` 2127/0/21, **exactly the `276a400` host baseline**,
reproduced inside the container against a 356MB vendor directory. Cold
`cargo build --workspace --offline` finished in 57.37s. `just vendor` proven
idempotent *and* refilling: deleting `vendor/serde-1.0.229` makes
`cargo metadata --offline` exit 101, and the recipe restores it.

### The trim, on the user's decision

**886MB → 356MB**; tar+gzip proxy 115.4MB → 55.7MiB. `.git` went 223MB → 370MB
rather than the ~700MB the untrimmed tree would have cost.

`cargo vendor` **cannot** filter — moving `vendor/windows-0.61.3` aside makes
`cargo metadata --offline` exit 101 (`required by package tao v0.36.0`), because
cargo resolves across every platform before deciding what to compile.
`cargo-vendor-filterer` answers it with **stubs**: real `Cargo.toml`, empty
`src/lib.rs`, matching `.cargo-checksum.json`. Resolution stays truthful, no
source ships, and a build for an excluded platform fails loudly on an empty
crate rather than mysteriously. 123 of the 585 entries are stubs. Putting a
platform back is one `--platform=<triple>` line, documented in `DOCKER.md`.

`aws-lc-sys` (68MB) **stays and is not a platform artefact**: `reqwest →
hyper-rustls → rustls → aws-lc-rs → aws-lc-sys`, and the container's cold build
compiles it. It leaves when the model client stops needing TLS.

### `.gitattributes`, added by the board before staging, and load-bearing

`core.autocrlf` is `input` and **221 vendored files contain CRLF**. Without
`vendor/** -text`, git strips the CR on commit, a fresh clone checks out
different bytes, and **every one of those `.cargo-checksum.json` entries
fails** — so the offline build would have worked only on the machine that
vendored it. Measured on `vendor/indoc-2.0.7/tests/test_formatdoc.rs`:

```
on disk, and in .cargo-checksum.json  538586016cf601451b582604908f7ab594ce642bbbd7a458ba2dff1d300f297d
CRLF stripped, as git would store it  5be5d5d8b400adef91a7fc49f21c0219062c161728ea66210f47940c12326a26
```

The staged blob now hashes to the former. A vendor tree that verifies only
where it was made is worse than none: it fails at **checksum** time, long after
the fetch it was supposed to replace.

### What this changes for `shared-remote-is-private`

Measured, not assumed, in a throwaway crate pinning `conserved` by rev:
`cargo vendor` **does** vendor git dependencies — `conserved` landed at 204K —
and emits a `[source."git+<url>?rev=<rev>"] … replace-with =
"vendored-sources"` stanza for `.cargo/config.toml`. `cargo build --offline`
against it finished in 3.83s **with no git access at all**.

So `spec02`'s adoption collapses the credential requirement to **one machine,
once per rev bump**: whoever runs `just vendor`. Container and CI need neither
the credential nor `~/.cargo/git/db/shared-*`.

**The hazard, which must not be lost:** `cargo vendor` resolves a rev from the
**local** git db. A rev that exists only on this machine vendors happily and
freezes into the tree, and the pin looks healthy everywhere while naming a
commit no remote has. Vendoring **hides** the never-pushed-commit problem rather
than fixing it. Whoever bumps `conserved` must confirm the rev is on the remote
first; nothing in the mechanism will tell them.

Also arriving with `conserved`: `blake3`, `arrayvec` and `constant_time_eq` are
not in mitosys's `Cargo.lock` today, so `dependency_tree.rs`'s OWNERS/CLOSURE
and `deny.toml` both gain entries.

### Two defects found and filed, neither this spec's

- **`typos-gate-cannot-run`** — `_typos.toml` is invalid TOML (duplicate
  `extend-words` under `[default]`), `typos-cli` exits 78, and the gate has
  never run while `quality.yml` claims it green. The `vendor/*` exclude added
  here is at line 12 inside `[files]`, a different table, and is **clean and
  independent** of the duplicate — it starts working the moment the file
  parses.
- **`harness-last-words-race`** — `a_harness_that_dies_mid_turn`'s stderr
  reader loses to its exit path: 4/20 with `init: true`, 6/20 without, **0/30
  in isolation**. Intra-binary contention, costing roughly one `just check` in
  four.

And one corrected in place: the `acp_mockagent.rs` doc comment blaming "the
platform's shell and on timing, 2 runs in 3 in the Linux dev container". It was
neither. **PID 1 was `sleep infinity`, which never `wait()`s**, so a SIGKILLed
orphan stayed a zombie and the test's `kill -0` liveness probe reported it alive
forever; macOS passed because launchd reaps. `init: true` puts `docker-init` at
PID 1 and both tests pass in 0.08s. A reproduction called flaky for two months
was deterministic all along, on a cause nobody had looked for.

Also noted: `cargo machete` is already red on `src/` alone at `276a400` — 11
crates, 14 unused deps, which looks like `p8d-floor-split` fallout — and once
`vendor/` is committed, `quality.yml`'s bare `cargo machete` will walk it and
needs a path argument.


## Done 2026-08-28 — `spec02`–`spec04`, and the address does not move

Container `just check` — this node's own `verify:` — **EXIT=0, 2139/0/21**,
with `conserved` compiled from `vendor/` under `CARGO_NET_OFFLINE=true` and an
**empty `/usr/local/cargo/git/`**: no credential, no git access. Host
`just check` 2140/0/21. Board re-ran vectors 10/0, `dependency_tree` 8/0,
`fmt --check` exit 0. Landed in mitosys as `2d04000d`.

### `vectors::address` is pinned to SHA-256, on the user's decision

It is now byte-for-byte what `content_hash` computed before it became blake3,
so **no address in any existing store moved**. `content_hash` stays blake3 and
its call-site count drops **21 → 20** — and the one that left is exactly the one
that was never an identity.

**The rule, in one line, written at both hash sites and in the `vectors` module
doc: an id may move behind a format bump, because the fold re-mints it from the
event's text; an address may not move at all, because nothing re-mints a file
name.**

An `assert_ne!` now sits beside the equality in `vectors/tests/vectors.rs`.
Rewriting the old assertion to the new spelling would have left **nothing**
guarding the split, and the next person to "unify the two hashes" would
re-couple the address and strand every journal. Now that fails a test.

### Re-verified by replaying the original fixture — a user's first run

The pre-adoption journal and its SHA-256 blobs, restored from git into the
working tree only (`git show >`, never `git checkout --`, so the index stayed
clean), replayed through the new code:

1. **The fold succeeds.** No `Corrupt`, no refusal. Confirmed independently
   outside the tree: SHA-256 over the hex of every `to_bits` in Python
   reproduces `c2b2950f…768d0`, the name the file already had.
2. **Ids are re-minted consistently** — and this **refutes a claim the worker
   had itself written into the docs earlier**, that an id is "carried in the
   event, never re-derived on replay". Measured: the fold recomputes it from the
   event's *text*, `blake3(text)` where the recording held `sha256(text)`, both
   checked against oracles outside the tree. Corrected everywhere it appeared,
   and the real argument is stronger than the one it replaced.
3. **One thing still breaks, and it is silent.** Filed, not worked around.

### The remaining gap, filed as `@mitosys/recall-references-dangle-silently`

An id stored as a **literal cross-entity reference** is matched, not re-minted,
so after a hash change it matches nothing. Two entities came back
`heat 1.0 / access 1` where the recording had `1.96 / 2`, and the delivered
ranking moved — **no error, no log, no refusal**.

A user's first run now: the pack wipes on `FORMAT_VERSION` 16, **the fold
succeeds**, the graph rebuilds whole under blake3 ids, and accumulated recall
heat and access counts are silently dropped. **Materially smaller than "comes up
empty", and worse in kind** — a workspace that refuses to boot tells you
something is wrong; this one ranks results differently and says nothing.
`Ingest::replaces` is the same shape and the fixture does not exercise it, so
two is a lower bound rather than a census.

### The fixture's blob names did change, and that is not the rule moving

`embed_text`, the fake embedder, derives its vector **from** `content_hash(text)`
— so blake3 gave it different floats, and different floats have a different
SHA-256 address, correctly. A real embedder returns model output. The caveat is
written into the fixture's module doc so nobody reads the diff as a
counter-example.

### One more defect, filed as `@mitosys/config-tests-race-on-the-environment`

The first container `just check` after the pin went red on
`load_of_a_foreign_root_pins_data_dir_to_that_root` — and the mechanism was
found rather than shrugged at. `Config::load` reads `MITOSYS_MEMORY_DIR`; a
**sibling test** `set_var`s and `remove_var`s that same variable with no guard,
in parallel threads of one binary. Measured 1 failure, then 66/66 five times
running. The fix pattern exists one crate over (`static ENV: Mutex<()>`), and
the gate that inventories process-global **statics** does not reach the process
**environment** — which is why nothing caught it.

## Measured — 2026-08-28, closing the boxes this node left open

The gate `done_boxes_are_ticked::every_done_prd_has_no_unticked_box` was red on
`main` because this PRD is `state: done` carrying seven `- [ ]`. Each was
measured against the mitosys tree rather than assumed, and six of the seven had
in fact landed — the boxes were never ticked, which is a bookkeeping failure and
not an implementation one. The seventh is genuinely not done and is struck above
with its real owner named.

| # | requirement | verdict |
|---|---|---|
| 1 | the offline container is the blocker | **done** — `vendor/conserved-0.1.0` is 20 tracked files, `.cargo/config.toml` carries the source replacement, and `cargo metadata --offline --locked` exits 0 on a cold `CARGO_HOME` with no `HOME` and no credential helper |
| 2 | `util/effect` → `conserved::scope` | **done** — `effect.rs` is 79 lines ending `pub use conserved::scope::{Closed, Disposer, Scope, Undo};`. The 13 crates still naming `mitosys_util_effect` reach the crate through that re-export; the shim is the landing, not a gap |
| 3 | `content_hash` → blake3 | **STRUCK** — 75 non-test call sites, no `blake3` in the workspace. Owned by `@mitosys/record-shape-port` |
| 4 | the `ed25519:` shim lives here | **done** — `src/mitosys/engine/util/util.rs:113`, "the one place an `ed25519:` prefix is tolerated", with the refusal in the crate documented beside it |
| 5 | `percentile_sorted` deleted | **done** — the function is gone; the only surviving mention is `engine/util/README.md:43` recording that it *was* |
| 6 | `dependency_tree.rs` accepts the crate | **done** — `src/mitosys/gates/tests/dependency_tree.rs:122` carries `"conserved"` in the accept list, with the reasoning at :135-142 |
| 7 | `conserved::scope` and `util/effect` reconciled | **done** — and not by diff-and-merge: `util/effect` no longer holds an implementation to diverge. It is the re-export from row 2, so the two agree by construction. The teardown-behaviour change this box warned about is the behaviour the tree now has |

Row 2 and row 7 are one fact seen twice, which is why both close together.
