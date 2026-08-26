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

- [ ] `model/Cargo.toml` gains `conserved = { git = "<pushed origin remote>",
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
- [ ] `rec_preimage` hashes `r.created.as_unix_nanos().to_le_bytes()` in
      place of the old unix-seconds bytes, and `REC_VERSION` (`mod.rs:44`)
      bumps from `1` to `2` — its own doc comment says to: *"Bump when the
      preimage layout changes; the id is a hash of the preimage, so a
      layout change is a new id space."*
- [ ] `rec_now()`'s replacement (or `rec_now()` itself, if kept for the
      other ~10 non-preimage call sites in `record/log.rs` and
      `record/store.rs` that use it for `as_of` defaulting and heat
      touches) reads through `conserved::SystemClock` exactly once for the
      id-birth path. A test swaps in `conserved::FixedClock`, builds the
      same `Record` content twice at the same fixed instant, and asserts
      `rec_id` returns the identical id both times — the reproducibility
      `rec_id`'s own doc comment claims ("recomputable by any peer from the
      content alone") and which a live clock read breaks.
- [ ] `impl Default for Record` no longer produces a value that can be
      stored under a fake, well-formed-looking id. Either the impl stops
      setting a usable `id` (and the two other construction sites —
      `record/log.rs:322`'s `decode_record`, `record/event.rs:1107`'s
      `to_record`, both of which currently spread `..Record::default()` or
      set every field explicitly — are checked against that), or the
      default `id` is made structurally impossible to mistake for a real
      one. A test names the invariant directly: a `Record` built via
      `Default` and passed to `StoreApi::put` without an explicit `rec_id`
      call must not end up stored under `[0u8; 32]`.
- [ ] The cascade this type change forces — `record/store.rs`'s
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
- [ ] The monotonic-clock inventory `learnings/clock.md` counted (65
      non-test `Instant::now()` sites, concentrated in `grade/probe.rs`,
      `grade/inproc.rs`, `loop/harness.rs`, `grade/measure.rs`,
      `node/refine.rs`) is untouched by this spec: none of them may become
      `conserved::Instant` or `conserved::Clock` — they are monotonic
      timers, not wall-clock reads, and out of this PRD's scope entirely.
- [ ] A comment beside the new `conserved` dependency (in `Cargo.toml` or
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
