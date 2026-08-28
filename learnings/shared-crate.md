---
type: learning
learning: shared-crate
subject: the concrete proposal — one small crate both trees depend on, holding ContentId, Clock, Scope and order statistics, with a stated admission test and everything it deliberately excludes
binds: [mitosys, llm]
status: decided
date: 2026-08-18
code: mitosys src/mitosys/util/, llm src/utils/, shared conserved/src/scope.rs, shared conserved/src/content_id.rs
---

# `shared` — the crate both trees depend on

**Renamed 2026-08-28. It was `conserved`; it is now `shared`.** The original
name came from biology, and the sentence that carried it is worth keeping as a
record of what the admission test below is *for*: a conserved region is the part
of a genome that is identical across species because it cannot afford to drift.
That is exactly the admission test. What it was not is the name of the
repository the crate ships from, which is `shared`, and one thing with two names
cost every consumer a `conserved` in its manifest resolving from an address that
said `shared`. The name now matches the address. The etymology is history, not a
claim about the word `shared` — there is no such thing as a "shared region" in
genomics, and nothing below rests on the word. See [[crate-name]] for the
decision, what the rename cost, and what it deliberately did not rename.

## The admission test

A thing belongs in this crate when **all four** hold:

1. **Both trees need it**, today, not speculatively.
2. **It is domain-free** — no agent, no model, no peer, no surface.
3. **It passes mitosys's `dependency_tree.rs` gate** — that is, it is
   dependency-light enough to sit under a core crate. This is a mechanical
   test, not a judgment call, which is the point of using it.
4. **One implementation is genuinely better than two** — because the two
   have already drifted, or because drifting would be silent.

Item 4 is the one that excludes most candidates. Two independent cosine
implementations that disagree in the tenth decimal are an annoyance; two
content-hash implementations that disagree at all are a family that cannot
share a record.

## What goes in

### 1. `ContentId` — blake3, `[u8; 32]`

The full argument is [[content-addressing]]. In brief: mitosys hashes
SHA-256 into a hex `String`, llm hashes blake3 into `[u8; 32]`, and an id
computed on one side does not exist on the other.

```rust
pub struct ContentId([u8; 32]);
impl ContentId {
    pub fn of(bytes: &[u8]) -> Self;
    pub fn as_bytes(&self) -> &[u8; 32];
}
impl Display for ContentId;   // 64 lowercase hex
impl FromStr  for ContentId;  // rejects everything else
```

Drift is silent without this, which is criterion 4 in its strongest form.
**Dependency: `blake3`.**

### 2. `Clock` — time as a parameter

The full argument is [[clock]]. Both trees read the wall clock ~65 times
each in non-test code, against a shared law that forbids it, and llm's
`rec_now()` feeds a live read into a content-hash preimage.

```rust
pub struct Instant(i64);
pub trait  Clock { fn now(&self) -> Instant; }
pub struct SystemClock;      // the ONE implementation permitted to read
pub struct FixedClock(Instant);
```

The point is not testability. It is that law 2's verification — *serialize
the record, fold from empty, compare* — is impossible wherever a fold reads
the clock. **No dependencies.**

### 3. `Scope` / `Handle` — reversible effects

mitosys has it (`util/effect`, 262 lines, imports nothing, unwinds in
reverse on drop). llm does not: it cites DOGMA 13 in prose comments in
`main.rs` and holds the rule by hand at each site.

Ports as-is. This is the clearest case in the whole proposal — one side has
a correct, dependency-free implementation and the other has a comment.
**No dependencies.**

**Landed as `Scope` / `Disposer`.** `Handle` above is the proposal's word; the
type that exists in `conserved::scope` is `Disposer`, and that is the name to
reach for at a call site.

**Corrected 2026-08-21 — "as-is" did not survive contact.** p5's load proof
measured that a panicking inverse abandoned every inverse still to come and
that `held()` then reported `[]`
(`.mi/prds/p5-adoption/load-proof/finding.md`). p6-scope-unwind fixed it in
`conserved` — every inverse runs, `held()` is true during the unwind, and a
new `failed()` names the inverses that panicked; it is recorded at its site as
deviation 8, the first *semantic* divergence from the port. The port was still
the right move; "one side has a *correct* implementation" was the loose word,
and the shared crate is where that got found.

### 4. Order statistics — one definition of "median"

mitosys has `percentile_sorted` **with zero callers**. llm's
`grade::measure::aggregate` computes min / upper-median (index `n/2`) / max
by hand, and is the thing that would call it.

```rust
pub fn percentile(sorted: &[f64], p: f64) -> Option<f64>;
pub fn median(sorted: &[f64])            -> Option<f64>;  // ONE definition, stated
pub fn min_median_max(sorted: &[f64])    -> Option<(f64, f64, f64)>;
```

Two definitions of median across a family that intends to share a grade
envelope is a regression that reads as a real one. **No dependencies.**

### 5. `hex` — encode and decode

mitosys already has it, tolerating an `ed25519:` prefix on decode. It moves
in behind `ContentId`'s `Display`/`FromStr` rather than staying a public
utility with two callers spelling it differently.

## Size and shape

Roughly 600–800 lines including tests. One crate to start, with one
dependency (`blake3`) reachable only through `ContentId` — still true of the
crate as built by default, since p2 added `serde` as an **optional** feature
with `default = []` rather than as a second dependency ([[content-addressing]]
carries that argument).

**If mitosys's dependency gate objects** to a core crate pulling `blake3`
transitively for `Scope`, split at the obvious line: `conserved` (Clock,
Scope, stats, hex — no dependencies) and `conserved-id` (ContentId —
blake3). Do not pre-split; let the gate decide, which is what the gate is
for.

## What stays out, and why

| candidate | why not yet |
|---|---|
| **vector math / quantization** | mitosys's `util/math` is richer (`OnlineSoftmax`, `QuantizedVec`, `quantized_cosine_distance`); llm's `utils/algebra` is thinner but tied to candle tensor layout. Real overlap, but criterion 4 is weak — two cosines that agree are fine. Revisit when llm needs quantized distance. |
| **the event spine** | Both have one, but mitosys's is `std` (`Arc`/`Mutex`/`Condvar`) and llm's is `tokio::sync::broadcast`. Sharing means picking a runtime for both trees — a much larger decision than this crate should carry. |
| **the record** | [[record-shape]] argues mitosys should adopt llm's shape, which is a port, not a shared crate. Sharing the *record* means sharing the fold, and neither tree's fold is domain-free yet. |
| **the file watcher** | Both wrap `notify`, with independent debounce and ignore rules. Genuine duplication, but `notify` is a heavy dependency to push under a core crate; revisit after criteria 1–3 are proven on something small. |
| **the reload seam** | llm's `interface` crate is the right shape and mitosys's `abi.rs` needs it, but it is a *seam*, not a utility: its whole contract is what may cross a dylib boundary. It deserves its own crate, on its own schedule, once the swap work actually starts. |
| **the grade envelope** | `Baseline`/`Target`/`Grade`/`normalized_ms`/`pass_window` is the highest-value thing llm has that mitosys lacks — but it is a *tool*, and it belongs in the port described by [[two-halves]] step 4, not in a floor crate. |

## Where it lives — the one constraint, now resolved

The learnings folder is prose, and a plain sibling directory costs it
nothing but a gate. **Code is different.** A `path = "../conserved"`
dependency from either tree:

- does not exist inside mitosys's dev container, which bind-mounts the repo
  and nothing else — so `just check` would fail there, not merely skip a
  doc check;
- does not exist for any clone of either repository;
- is not pinned, so the two trees can silently compile against different
  content of the same crate — which is the failure the workspace
  dependency pin exists to prevent, reappearing one level up.

This is a real decision and it is not the same decision as the one already
made for prose. Three options, in the order they cost:

1. **Vendored into each tree** (subtree or a copied directory with a
   recorded source hash). Everything builds everywhere; the sync is manual
   and must be gated.
2. **A git dependency** to a small `conserved` repository, pinned by commit
   in each tree's `Cargo.toml`. Builds in the container (cargo fetches over
   the network at build time — which mitosys's container does not have,
   so this needs vendoring or a pre-populated registry cache to work
   offline).
3. **A path dependency to a sibling directory.** Simplest, and host-only:
   accepts that neither tree builds in the container or on a fresh clone.

**Resolved: option 2 — a git dependency, pinned by commit rev.** The
constraint that decides it was never a technical question, and the user
settled it on 2026-08-20: the crate must be distributable to *every* Rust
repo. That eliminates option 3, which is host-only by construction, and it
prices option 1's manual sync against a pin that cannot silently drift. The
argument in full lives in `.mi/docs/memos/distribution.md`
(`status: decided`, `decided: 2026-08-21`), and `p0-foundation`'s answers
close it: do not re-escalate this.

It was **proven once before anything was extracted**, which is exactly what
§"First move" asked for. In `9fff8ea`, mitosys resolved, locked, compiled and
ran a test against `conserved` through a rev-pinned git dependency; the rev is
recorded in `.mi/prds/p1-scope/proof.md`. The provisional half of that proof
is the URL — it pinned `file:///Users/feb/dev/infra/shared`, and the board's
answer of 2026-08-21 names `https://github.com/inner-zirkle/shared` as the
real one, added as `origin` and not yet pushed.

**Superseded 2026-08-23 — the address moved.** Every repository in the family
left the `inner-zirkle` organisation for the personal account, so the URL every
consumer pins today is `https://github.com/yesitsfebreeze/shared.git`. What did
*not* move is the visibility: it is still **private** (`gh repo view
yesitsfebreeze/shared` → `"visibility":"PRIVATE"`, checked 2026-08-28), so the
cost this section priced against option 2 is still owed. Read the
`inner-zirkle` URL in the paragraph directly above — and every other
occurrence of that organisation anywhere in this document — as a historical
record of what was decided on 2026-08-21, never as an address to copy. The
copyable pin is in §"Landed".

**Corrected 2026-08-28 — the visibility moved too, later the same day.** The
sentence above is accurate as of the morning it was written and is left standing
as the record; it is no longer true. `yesitsfebreeze/shared` is **public**, at
the user's decision, verified anonymously —
`https://api.github.com/repos/yesitsfebreeze/shared` returns HTTP 200 with
`"private": false, "visibility": "public"`. What that changes is
*authentication* and nothing else: the vendored copies in all three consumers
stay, because a public remote is still a network round trip and an offline build
— mitosys's dev container has no network at build time — still cannot make it.
Read every "PRIVATE" below, and in each consumer's `.cargo/config.toml`, as the
reason vendoring was built rather than a reason it is still needed. See
[[crate-name]].

The cost option 2 carries is the one this section named against it: mitosys's
dev container has no network at build time, so it needs vendoring or a
pre-populated registry cache. That was scoped as mitosys-side follow-up in the
`mitosys` child of `p5-adoption` — a cost of the chosen option, not a
reopening of the choice.

**Superseded 2026-08-28 — that scope was too narrow, and it cost two trees a
build.** Pricing this against mitosys's container alone asked the wrong
question. The container was never the sharp edge; the **CI runner** is, and
this section never considered one. `model` and `realm` have no Dockerfile at
all — what they have is `.github/workflows/ci.yml`, and `actions/checkout`
hands a job a token scoped to its *own* repository and nothing in `shared`.
So a private remote costs **every** consumer a resolution failure on every
machine that lacks the user's credential, not mitosys alone, and the cost is
owed by whoever builds — not by whoever publishes.

All three consumers pay it the same way, and it is cheap:

- **mitosys** — a committed `vendor/` plus a source replacement in
  `.cargo/config.toml`. Already landed; `cargo metadata --offline --locked`
  exits 0 on a cold `CARGO_HOME` with no `HOME` and no credential helper.
- **model** and **realm** — the same mechanism, landed 2026-08-28 at
  **192K** each: `vendor/conserved-0.1.0` and a `.cargo/config.toml` that
  replaces **only** the `git+…?rev=…` source. `[source.crates-io]` is
  deliberately left on the network, because crates.io is public and reachable
  from every runner; vendoring the whole graph would cost each repo hundreds
  of megabytes to solve a problem 192K wide. `Cargo.lock` is untouched in
  both — it still names the git URL and the rev, so the pin is still the pin
  and drift still requires a visible rev bump. Only where the bytes come from
  moves. Each tree gates the copy with
  `scripts/conserved-vendor-check.sh`, wired in front of `just verify`
  (`model`) and `just check` (`realm`).

The trap worth writing down, because it is silent: `cargo vendor` resolves a
rev out of the **local** git db, so a commit that never left the vendoring
machine copies in looking healthy while naming something no clone can fetch.
Push before vendoring; the gate above asserts
`git -C ../shared branch -r --contains <rev>` prints something.

## First move

`Scope` alone, into whichever home is chosen. It imports nothing, one tree
already has it working and the other measurably lacks it, and it cannot fail
in an interesting way — so what it actually tests is the *mechanism*: does
the dependency resolve, in the container, on a clone, under both gates.
Learn that on 262 lines, not on the record.
## Landed

Where the decision above turned into a crate. Every sha is on `main`.

| commit | what it landed |
|---|---|
| `ab154f7` | `git init`; the reset workspace; the board; `.mi/docs/memos/distribution.md` |
| `0d4bc10` | the fresh-clone gate (`scripts/fresh-clone-check.sh`) — p0's acceptance |
| `eb55b49` | p0's acceptance ticks |
| `5313ca4` | `Scope`/`Disposer` ported into `conserved::scope`, zero dependencies |
| `0b2f964`, `85dad04` | the `mod scope` test wrapper and its deviation record |
| `9fff8ea` | **the distribution mechanism proven once**, from mitosys |
| `f7b454a` | the proof re-pinned at the final port rev |
| `c122240`, `ec58a75`, `8e12122`, `c275645` | `ContentId`: blake3 in, 64 lowercase hex out; proptest round-trips; the optional `serde` feature; the deviation record |

The two that carry the argument are `9fff8ea` (the constraint above, made
real) and `5313ca4` (§"First move", done). The rest are the record.

Against §"What goes in", item by item:

1. **`ContentId`** — in the crate, `conserved/src/content_id.rs`.
2. **`Clock`** — in the crate, `conserved/src/clock.rs`, landed after this
   section was first drafted. [[clock]] carries its own record of what the
   implementation decided.
3. **`Scope` / `Disposer`** — in the crate, `conserved/src/scope.rs`.
4. **Order statistics** — in the crate as `conserved::stats`, one definition
   of median (the upper one), `conserved/src/stats.rs`.
5. **`hex`** — in, folded into `ContentId`'s `Display`/`FromStr` exactly as
   §5 said it would be, rather than as a public utility.

## What is still outstanding

`decided` covers the proposal — the admission test, what goes in, where the
code lives, how it reaches a consumer. The extraction is complete; adoption
is not. The admission audit of 2026-08-23, every admitted item:

| admitted item | in `conserved` | mitosys still carries | model still carries |
|---|---|---|---|
| `ContentId` | `conserved/src/content_id.rs` | `util/util.rs` `content_hash` + `digest` (SHA-256) | `utils/algebra` `content_id`, `utils/fs` `blake3_hash`/`compute_version_hash`, `record` `rec_id` — blake3 already, local copies |
| `Clock` | `conserved/src/clock.rs` | `util/util.rs` `now_nanos`/`now_ms`/`now_secs` + direct wall reads | `record` `rec_now()`, `src/node/transactional.rs:72` |
| `Scope`/`Disposer` | `conserved/src/scope.rs` | `util/effect` (semantics diverged — deviation 8) | prose site `src/main.rs:126` |
| order statistics | `conserved/src/stats.rs` | `util/util.rs` `percentile_sorted`, zero callers | `src/grade/measure.rs:203` `aggregate` |
| `hex` | behind `ContentId`'s `Display`/`FromStr` | `util/util.rs` `pub mod hex` | — |

One exemption — the only admitted item a tree keeps a copy of:

- mitosys `util::hex` stays for its **non-id** callers: `decode` tolerates
  the `ed25519:` prefix for key strings, p2 refused that prefix into the
  crate, and the shim lives at mitosys call sites
  (`.mi/prds/p5-adoption/mitosys/prd.md`). `ContentId::from_str` replaces
  only the id renderings.

One migration — named, not an exemption:

- mitosys re-keys its SHA-256 hex ids to blake3 `ContentId`. The user
  accepted both persisted-id breaks on 2026-08-21
  (`.mi/prds/p5-adoption/prd.md` §Answers, item 3): wipe and re-derive
  behind `store_core`'s `FORMAT_VERSION` bump, one version bump each tree.
- The re-key is lossless through the export escape hatch:
  `id == content_hash(text)` (`mitosys/src/mitosys/engine/graph/merge.rs:96`),
  so every id re-derives from exported content. `apply_import`
  (`commands_export.rs`) unions by stored id and never recomputes — the
  mitosys child re-keys at import or bumps `EXPORT_VERSION` and names the
  fate of v1 exports.
- When the swap completes, `sha2` leaves `mitosys-util` — one hash per
  tree, the argument `mitosys/src/mitosys/engine/record/Cargo.toml`
  already makes.

Where adoption lives now:

- One child PRD per consumer tree: `mitosys/prds/adopt-conserved` and
  `model/prds/adopt-conserved`. They carry the requirements written in
  `.mi/prds/p5-adoption/{mitosys,llm}/prd.md`; this board stays the
  cross-tree ledger and dispatches nothing into the trees — the user's
  answer 2 of 2026-08-21 stands.
- Both trees pin the same rev:
  `conserved = { git = "https://github.com/yesitsfebreeze/shared.git", rev = <sha> }`.
  The rev is pushed and on `origin/main`; the repository is still private, so
  the pin is unreachable to anything without the user's credential —
  publishing is the user's call (answer 1 of 2026-08-21); the children name
  that event and do not push.

  **Reachability, not existence, is what actually bites.** A private remote
  costs each consumer a resolution failure on every machine that lacks the
  credential, and the CI runner is the one that matters, because
  `actions/checkout` hands a job a token scoped to its *own* repository and
  nothing in `shared`. mitosys pays this with `vendor/` and a source
  replacement in `.cargo/config.toml`; model and realm did not, and their CI
  has been red on `main` since adoption — `failed to authenticate when
  downloading repository`, all four realm jobs, run 33122525527, 2026-08-27.
  The fix is the same mechanism at a fraction of the size: `conserved` alone
  is **192K** vendored, and replacing only the *git* source leaves
  `[source.crates-io]` on the network, where it is public and reachable.
- realm adoption is `p5-adoption`'s `realm` child, outside the
  master-board `conserved-crate` PRD — its done condition names mitosys
  and model only.
