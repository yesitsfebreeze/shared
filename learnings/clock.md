---
type: learning
learning: clock
subject: both trees read the wall clock ~65 times each in non-test code while their shared law says time enters as recorded data and is never consulted live — a symmetric violation nothing gates in either tree
binds: [mitosys, llm]
status: open
date: 2026-08-18
code: mitosys src/mitosys/util/util.rs:107, llm src/record/mod.rs:239
---

# Time is a parameter, and neither tree treats it as one

The law both trees run on is explicit. mitosys's law 1, under
**unrepresentable**:

> time is passed in as a parameter, not read

and, in the same law's *connects*:

> whatever cannot be re-derived — clock, model output, embedding, hash
> iteration order — enters as recorded data and is never consulted live.

Counted 2026-08-18, non-test code only:

| tree | `SystemTime::now()` / `Instant::now()` call sites |
|---|---|
| mitosys | **65** |
| llm | **66** |

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

Two sites are known bugs today rather than candidates: `rec_now()` feeding
`created` into the preimage, and any clock read reachable from
`replay(as_of, kinds)`.
