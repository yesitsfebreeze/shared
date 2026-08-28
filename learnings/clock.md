---
type: learning
learning: clock
subject: both trees read the wall clock ~65 times each in non-test code while their shared law says time enters as recorded data and is never consulted live — a symmetric violation nothing gates in either tree
binds: [mitosys, llm]
status: decided
date: 2026-08-18
decided: 2026-08-21
code: mitosys src/mitosys/util/util.rs:107, llm src/record/mod.rs:239, llm src/node/transactional.rs:72, shared shared/src/clock.rs
---

# Time is a parameter, and neither tree treats it as one

The law both trees run on is explicit. mitosys's law 1, under
**unrepresentable**:

> time is passed in as a parameter, not read

and, in the same law's *connects*:

> whatever cannot be re-derived — clock, model output, embedding, hash
> iteration order — enters as recorded data and is never consulted live.

Counted 2026-08-23, non-test code only — substrings `SystemTime::now` and
`Instant::now`, no parentheses; comment lines, `tests/` paths,
`tests.rs`/`*_tests.rs` files, and `#[cfg(test)]` blocks excluded:

| tree | `SystemTime::now()` / `Instant::now()` call sites |
|---|---|
| mitosys | **66** |
| llm | **69** |

Both trees also ship the read as a blessed utility:

```rust
// mitosys  util/util.rs:107
pub fn now_nanos() -> u128
pub fn now_ms()    -> u64
pub fn now_secs()  -> u64

// llm  record/mod.rs:239
pub fn rec_now() -> i64
```

`rec_now()` is the sharper case: it sits in the record module, beside
`rec_id`, and stamps `created` — the field that goes **into the preimage**
and therefore into the content id. The identity of a record depends on a
value read from the environment at the moment of writing, so the same
content hashed twice yields two ids, and the fold is not reproducible from
the content alone. That is the exact property `rec_id`'s own doc comment
claims: *"recomputable by any peer from the content alone."*

## Why it happened in both

Nothing gates it. This is law 3's own prediction — *the law on the lowest
rung is the one violated first, silently* — and it is on the lowest rung in
both trees at once. mitosys gates source layout, dependency ownership and
command namespaces; none of them looks at the clock. llm gates nothing.

It is also the cheapest possible violation to commit: `now()` is one call
and the alternative is threading a parameter through a signature that did
not have one.

## The fix, in the order it should be done

**1. Make it visible.** A gate — mitosys's `gates/` has the shape already —
that fails on `SystemTime::now()` / `Instant::now()` outside an allowlist.
Start with the allowlist holding all 65 sites, so the gate passes on day one
and *no new site can be added*. A ratchet, not a migration.

**2. Give time a type and one source.**

```rust
pub struct Instant(i64);            // unix, whatever precision the tree needs
pub trait Clock { fn now(&self) -> Instant; }
pub struct SystemClock;             // the only implementation that reads
pub struct FixedClock(Instant);     // tests, replay, refold-from-empty
```

The value of this is not testability, which is the usual argument and the
weaker one. It is that **law 2's verification becomes possible**: *serialize
the record, fold it with fresh state, compare.* A fold that reads the clock
cannot be compared against itself, so the tree's only proof that law 1 holds
is unavailable wherever a clock read hides inside the fold.

**3. Empty the allowlist from the leaves inward.** Record and fold paths
first — those are where a live read is a correctness bug rather than a style
one. Surfaces and logging last; a status bar reading the clock costs
nothing.

## Honest scope

Threading a clock through ~65 sites in each tree is not a small change, and
most of those sites are harmless — a log line, a heartbeat, a UI timestamp.
The claim here is not that all 130 are bugs. It is that **the tree cannot
currently tell which ones are**, because nothing distinguishes a clock read
inside a fold from one inside a status bar. Step 1 costs almost nothing and
buys exactly that distinction; steps 2 and 3 are then priced per site
instead of as one migration.

Three sites are known bugs today rather than candidates: `rec_now()` feeding
`created` into the preimage; `Commit::new` at llm's
`src/node/transactional.rs:72`; and any clock read reachable from
`replay(as_of, kinds)`.

`Commit::new` is the same defect as `rec_now()`, in the more expensive place.
It reads `SystemTime::now()` into `timestamp`, and `Commit::content_hash()`
(same file, lines 104-109) is blake3 over the postcard encoding of the whole
commit with `signature` set to `None` — `timestamp` included. That hash is the
commit's own id, and `parent_heads: Vec<[u8; 32]>` carries it across the
network, so a peer either re-derives the same id from the same content or the
DAG does not join up. Three hazards ride on that one read, and this is the only
place they are collected:

- The field is `timestamp: u64` — **unsigned**, so it cannot hold a pre-1970
  value at all, while `Instant` is `i64`. Substituting moves bytes as well as
  units: postcard varint-encodes a `u64` and zigzag-encodes an `i64`, so it is
  a format change, not only a type change.
- The read is `.as_secs()` while the field's own doc comment, one line above
  it, says *"unix epoch milliseconds"*. The unit has already drifted inside a
  single struct, between the code and the prose describing it — which is the
  argument for `Instant` pinning its unit by test rather than by doc comment.
- The `.unwrap()` is a third spelling of the pre-epoch hazard: a clock behind
  `UNIX_EPOCH` panics here rather than yielding a negative timestamp.

This site is **inside** the counts in the table above, not an addition to them:
it is one of llm's 66 reads, counted on 2026-08-18 and only now named. The
table is unchanged; what changes is how many of those reads are known bugs
rather than candidates.
## Landed

Step 2 of §"The fix" — *give time a type and one source* — is in
`shared/src/clock.rs`:

| commit | what it landed |
|---|---|
| `cb49f4a` | `Instant` — unix nanoseconds, decided and pinned by test |
| `b1fdcee` | `Clock`, `SystemClock`, `FixedClock` — time as a parameter |
| `c74bd90` | `Instant` under serde: transparently the `i64` it replaces |

**The implementation decided something this document left open.** The sketch
above writes `pub struct Instant(i64); // unix, whatever precision the tree
needs`. The landed type is **unix nanoseconds**, fixed, pinned by a test
rather than by a comment — because "whatever precision the tree needs" is two
precisions the day the second tree arrives, which is the same class of drift
this folder exists to catch. `i64` nanoseconds spans roughly ±292 years around
1970, which covers both trees' timestamps with room, and the sign is what lets
a pre-epoch value exist rather than panic.

**What is decided and what is merely possible.** The status above says the
argument is settled and the compliant path exists — not that the reads are
gone. On the day it flipped:

- **Steps 1 and 3 have not landed anywhere.** No ratchet exists in any tree,
  so a new `SystemTime::now()` still fails no check, and no allowlist has been
  emptied from the leaves inward. Both are consumer-tree work by design: p3's
  own third requirement keeps the ratchet out of this repo, and
  `p5-adoption`'s `ratchets` child carries it in the trees themselves.
- **No consumer tree reads `shared::Clock`.** mitosys still ships
  `now_nanos`/`now_ms`/`now_secs`; llm still ships `rec_now()`.
- **Both content-hash sites are still live bugs** in llm: `rec_now()` feeding
  `created` into the record preimage, and `Commit::new` at
  `src/node/transactional.rs:72` feeding `SystemTime::now()` into a
  network-visible commit id. The type that fixes them exists; nothing has been
  threaded through either yet.
- The counts in the table above are as of 2026-08-23.
