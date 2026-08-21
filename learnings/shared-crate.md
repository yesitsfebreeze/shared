---
type: learning
learning: shared-crate
subject: the concrete proposal — one small crate both trees depend on, holding ContentId, Clock, Scope and order statistics, with a stated admission test and everything it deliberately excludes
binds: [mitosys, llm]
status: partial
date: 2026-08-18
code: mitosys src/mitosys/util/, llm src/utils/
---

# `conserved` — the crate both trees depend on

Named for the biology the family already runs on: a conserved region is the
part of a genome that is identical across species because it cannot afford
to drift. That is exactly the admission test below. (If the name reads as
too cute, `shared-floor` says the same thing and nothing else changes.)

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
dependency (`blake3`) reachable only through `ContentId`.

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

## Where it lives — the one unresolved constraint

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

No recommendation is recorded here because the constraint that decides it —
whether container and clone builds must keep working — is not a technical
question. It is worth deciding **before** the first line is extracted, since
each option implies a different repository layout.

## First move

`Scope` alone, into whichever home is chosen. It imports nothing, one tree
already has it working and the other measurably lacks it, and it cannot fail
in an interesting way — so what it actually tests is the *mechanism*: does
the dependency resolve, in the container, on a clone, under both gates.
Learn that on 262 lines, not on the record.
