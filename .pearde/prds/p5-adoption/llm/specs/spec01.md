---
complexity: 30
footprint:
  - ../model/Cargo.toml
  - ../model/Cargo.lock
  - ../model/src/record/mod.rs
  - ../model/src/record/store.rs
  - ../model/src/record/log.rs
  - ../model/src/record/event.rs
  - ../model/src/mcp/fold.rs
---

# spec01 — the content-hash preimage: `Record.created` becomes `Instant`, and the Default-id hole closes

`Record.created` (`../model/src/record/mod.rs:178`, currently `i64` unix
seconds) becomes `conserved::Instant` (unix nanoseconds), and `rec_now()`
(`mod.rs:239`) reads the wall clock through `conserved::SystemClock` instead
of calling `SystemTime::now()` directly. This is the accepted persisted-id
break from the PRD's `## Decided` section — wipe and re-derive, not a
migration — and it is the first time `../model` depends on `conserved` at
all, so the dependency itself lands here.

`impl Default for Record` (`mod.rs:180`) sets `id: [0u8; 32]`, which
`conserved::content_id.rs`'s own module doc names directly: *"a zero
`ContentId` is not the hash of anything... p5 carries the model-side fix;
this crate does not supply the hole."* This spec is that fix.

## Acceptance

- [x] `model/Cargo.toml` gains `conserved = { git = "<pushed origin remote>",
      rev = "<a commit rev present on that PUSHED remote>" }` in
      `[dependencies]` (alphabetically between `candle-nn` and `crossterm`).
      Verify the URL and rev against the live repo at run time, not from a
      prior recorded answer: the parent `p5-adoption/prd.md`'s `## Answers`
      §1 names `https://github.com/inner-zirkle/shared`, which is stale
      (repos moved off `inner-zirkle` to the personal account 2026-08-23).
      As of this analysis `shared/.git`'s `origin` reads
      `https://github.com/yesitsfebreeze/shared.git`, and local `main`
      (`ea014a0797b4...`) was AHEAD of pushed `origin/main`
      (`9a342e1e849d...`) — confirm with `git ls-remote
      https://github.com/yesitsfebreeze/shared.git main` that the rev you
      pin is actually fetchable before wiring it in, or the dependency
      resolves locally and fails for anyone else.
- [x] `rec_preimage` hashes `r.created.as_unix_nanos().to_le_bytes()` in
      place of the old unix-seconds bytes, and `REC_VERSION` (`mod.rs:44`)
      bumps from `1` to `2` — its own doc comment says to: *"Bump when the
      preimage layout changes; the id is a hash of the preimage, so a
      layout change is a new id space."*
- [x] `rec_now()`'s replacement (or `rec_now()` itself, if kept for the
      other ~10 non-preimage call sites in `record/log.rs` and
      `record/store.rs` that use it for `as_of` defaulting and heat
      touches) reads through `conserved::SystemClock` exactly once for the
      id-birth path. A test swaps in `conserved::FixedClock`, builds the
      same `Record` content twice at the same fixed instant, and asserts
      `rec_id` returns the identical id both times — the reproducibility
      `rec_id`'s own doc comment claims ("recomputable by any peer from the
      content alone") and which a live clock read breaks.
- [x] `impl Default for Record` no longer produces a value that can be
      stored under a fake, well-formed-looking id. Either the impl stops
      setting a usable `id` (and the two other construction sites —
      `record/log.rs:322`'s `decode_record`, `record/event.rs:1107`'s
      `to_record`, both of which currently spread `..Record::default()` or
      set every field explicitly — are checked against that), or the
      default `id` is made structurally impossible to mistake for a real
      one. A test names the invariant directly: a `Record` built via
      `Default` and passed to `StoreApi::put` without an explicit `rec_id`
      call must not end up stored under `[0u8; 32]`.
- [x] The cascade this type change forces — `record/store.rs`'s
      `rec_visible`/`rec_live_at` comparisons against `as_of: i64`,
      `supersede`'s `next.created = now`, `sort_by_key(|r| r.created)`;
      `record/log.rs`'s `StoredRecord.created` wire field and
      `event_key(record.created, seq)`; `mcp/fold.rs`'s
      `"created_epoch_secs": record.created` JSON export — compiles and
      keeps its EXTERNAL behavior (the `as_of`/`at`/`now` parameters this
      spec does not touch stay `i64` unix seconds; the JSON field keeps
      emitting whole seconds via `.as_unix_secs()`). This spec does not
      widen `as_of`, `at`, `now`, `heat_at`, or `until` to `Instant` —
      those stay out of scope; only `created` and what a compiler forces
      around it change.
- [x] The monotonic-clock inventory `learnings/clock.md` counted (65
      non-test `Instant::now()` sites, concentrated in `grade/probe.rs`,
      `grade/inproc.rs`, `loop/harness.rs`, `grade/measure.rs`,
      `node/refine.rs`) is untouched by this spec: none of them may become
      `conserved::Instant` or `conserved::Clock` — they are monotonic
      timers, not wall-clock reads, and out of this PRD's scope entirely.
- [x] A comment beside the new `conserved` dependency (in `Cargo.toml` or
      the first `use conserved::...`) records that this edition-2024 tree
      (`../model/Cargo.toml:15`) now consumes an edition-2021 crate
      (`conserved/Cargo.toml`'s `edition.workspace = true`, `2021` in
      `shared/Cargo.toml:7`) — legal per-crate in Cargo, recorded per the
      PRD's own requirement, not fixed.

## Verify and Proof

```sh
cd ../model && cargo build -p llm \
  && cargo test -p llm --lib record:: \
  && ! rg "conserved::(Instant|Clock)" src/grade src/loop src/node/refine.rs
```

## Evidence — implemented 2026-08-26

Ticked against runs, not reading. Verdicts are `reproduced` unless stated.

**Box 1 — the pinned rev is fetchable.** `git ls-remote
https://github.com/yesitsfebreeze/shared.git main` →
`9a342e1e849dd5775cbadfe6b32e275a076e5f09`. Local `shared` `main` is
`b10b6bf…`, **8 ahead** of that — the spec's warning is still live, so the rev
pinned is the pushed one, not local `HEAD`. `git diff origin/main main --
conserved/` is one added test file and no change to `conserved/src/`, so the
pushed rev's crate source is byte-identical to local. `cargo fetch` resolved
it off the network:

```
    Updating git repository `https://github.com/yesitsfebreeze/shared.git`
From https://github.com/yesitsfebreeze/shared
 * [new ref]         9a342e1e849dd5775cbadfe6b32e275a076e5f09 -> refs/commit/9a342e1e…
      Adding conserved v0.1.0 (https://github.com/yesitsfebreeze/shared.git?rev=9a342e1e…)
```

**Reading recorded — where the dependency is spelled.** The box says
"`[dependencies]` (alphabetically between `candle-nn` and `crossterm`)". This
tree inherits every dependency from `[workspace.dependencies]`
(`async-trait = { workspace = true }` and 25 more), so the git+rev spec sits in
`[workspace.dependencies]` and `conserved = { workspace = true }` in
`[dependencies]` — both in the named alphabetical slot. Spelling it inline in
`[dependencies]` would have been the one dependency in the crate that does not
follow the tree's own convention.

**Box 2.** `REC_VERSION: u8 = 2`, and `rec_preimage` ends
`out.extend_from_slice(&r.created.as_unix_nanos().to_le_bytes())`.

**Box 3.** `rec_now()` is now `SystemClock.now().as_unix_secs()` — kept, as the
box permits, for the ~10 non-preimage `as_of`/heat sites. The id-birth path is
the new `rec_born_at<C: Clock>(clock) -> Instant`, and the three callers that
mint a record's `created` (`EventLog::append`, `EventLog::supersede`) each take
**one** clock read and derive their second count from it, so `created` and the
`now` that closes a belief can no longer straddle a second boundary. The test
the box names is
`record::tests::identity::the_same_content_born_at_the_same_instant_gets_the_same_id`
— and it is not vacuous: its second half asserts a **one-nanosecond-later**
`FixedClock` gives a *different* id, so the first half is testing the clock and
not testing `assert_eq!(x, x)`.

**Box 4 — and what it did NOT close.** `REC_ID_UNSET` names the sentinel
`conserved::content_id`'s module doc points at, and
`record::tests::identity::a_default_record_is_never_stored_under_the_unset_id`
runs the box's own sentence: `Record::default()` → `put` → the row answers
under its content hash and `get(REC_ID_UNSET)` is `None`. The two other
construction sites were checked: `event.rs`'s `to_record` spreads
`..Record::default()` and now documents `REC_ID_UNSET` by name; `log.rs`'s
`decode_record` sets `id` from the stored row, which is a read boundary and
never the unset value for a row this build wrote.

**Verdict on the hole itself: `refuted`** (fixture: `MemStore`, the tree's one
`StoreApi` impl). The box is written as though a default `Record` could reach
storage under `[0u8; 32]`. It could not: `MemStore::put` already opened with
`if r.id == [0u8; 32] { r.id = rec_id(r); }` before this node, and
`grep -rn 'impl StoreApi' src/` finds exactly one impl. So the new test passes
on the tree as it stood — it does **not** discriminate before from after, and
claiming it fixed a live defect would be false.

What this node actually changed is worth having and is smaller than the box
implies: the invariant had **no test at all**, and the sentinel was an
anonymous `[0u8; 32]` literal repeated at the guard and at the `Default`, with
nothing tying either to `conserved`'s stated refusal. It now has a name, a
doc comment pointing back at `content_id.rs`'s own paragraph, and a check that
fails if someone deletes the guard. Recorded this way rather than ticked as a
repair.

**Box 5 — the cascade, and one decision inside it.** `store.rs`'s
`rec_visible` and `mod.rs`'s `rec_live_at` compare `created.as_unix_secs()`
against the `i64` second they were always given (`as_unix_secs` **floors**, so
a record born mid-second is still born by that second — the old field's exact
answer). `supersede` writes `Instant::from_unix_secs(now)`. `sort_by_key(|r|
r.created)` needed no change: `Instant` is `Ord`.

`event_key(created, seq)` is fed `created.as_unix_secs()`, **not** nanos. Its
own doc is the reason — *"two events in the same second still order by the
sequence that separated them"* — and the key space is queried by
`key_ceiling(at)` built from an `as_of` in seconds. Feeding nanoseconds into a
key whose windows are built from seconds would have made every range query
empty. Recorded because it is a choice the box did not spell out.

`StoredRecord.created` is `Instant`, whose serde impl is transparent over its
`i64`; the postcard shape is unchanged and the UNIT is what moved, which is
exactly what the `REC_VERSION` bump exists to refuse on read.
`mcp/fold.rs`'s `"created_epoch_secs"` keeps emitting whole seconds via
`.as_unix_secs()`. `as_of`, `at`, `now`, `heat_at` and `until` were NOT widened.

**Box 6.** `grep -rn 'conserved::(Instant|Clock)' src/grade src/loop
src/node/refine.rs` → no matches. `git diff --name-only` names no file under
`src/grade`, `src/loop`, or `src/node/refine.rs`. 68 non-`record` `Instant::now()`
sites still stand, untouched.

**Box 7.** The comment sits beside the dependency in
`[workspace.dependencies]`, naming this tree's edition 2024 against
`conserved`'s 2021, and recording it rather than fixing it.

**Footprint correction.** The specs' `footprint:` omits
`../model/src/record/tests/`, which this tree's test law puts beside the module
under test — six test files had to change with the type. Committed with the
rest and named here.

## Verify — run 2026-08-26

```
$ cargo build -p llm
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.77s

$ cargo test -p llm --lib record::
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 1519 filtered out; finished in 1.25s

$ grep -rn 'conserved::(Instant|Clock)' src/grade src/loop src/node/refine.rs
none — the monotonic sites are untouched
```
