# goal

`Clock`, `SystemClock` and `FixedClock` — one trait, exactly one implementation
permitted to read the operating system, and a fixed source that makes a
deterministic fold *expressible*, which is the whole reason this ticket exists.

est: 1.25h

## What this assumes

- **spec01 has landed**: `Instant` with its unit pinned (unix nanoseconds),
  `from_unix_nanos`/`as_unix_nanos`, `from_system_time`/`to_system_time`,
  `saturating_add`, `signed_nanos_since`, all public from `conserved`, in
  `conserved/src/clock.rs`.
- p0's layout: package `conserved`, tests at `conserved/tests/`, `[dependencies]`
  empty, `#![forbid(unsafe_code)]` on `lib.rs`.

**This spec adds no dependency either.** `SystemClock` reads `std::time`, not
`chrono` — that is the line `conserved-core` crossed and was condemned for
(`.mi/docs/memos/scaffold-reset.md`).

## Files

- `conserved/src/clock.rs` — extended (same file spec01 created). `Clock`,
  `SystemClock`, `FixedClock`, and the blanket impls.
- `conserved/src/lib.rs` — the existing `pub use clock::Instant;` becomes
  `pub use clock::{Clock, FixedClock, Instant, SystemClock};`. No other line of
  that file changes.
- `conserved/tests/clock_source.rs` — **new**.

Touches nothing else. Does **not** touch `conserved/Cargo.toml`, `../mitosys`,
`../model`, `../realm`, `learnings/`, or any other ticket's directory.

## The surface

```rust
/// The one way to ask what time it is.
pub trait Clock {
	fn now(&self) -> Instant;
}

/// The ONE implementation permitted to read the operating system.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SystemClock;

/// A clock that answers the same instant every time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedClock(Instant);

impl FixedClock {
	pub const fn new(at: Instant) -> Self;
	pub const fn instant(self) -> Instant;
}

impl<C: Clock + ?Sized> Clock for &C {}
impl<C: Clock + ?Sized> Clock for Box<C> {}
impl<C: Clock + ?Sized> Clock for std::sync::Arc<C> {}
```

Each choice is anchored to a call site, and the doc comments say which:

- **`SystemClock` derives `Default`, `Instant` does not.** A unit struct's
  default is itself and carries no ambiguity; a default *timestamp* would (see
  spec01's refusal 1).
- **The blanket impls over `&C`, `Box<C>`, `Arc<C>`** — `../model` holds its
  substrate in `Arc` across tokio tasks (`src/daemon/`, `src/gossip/`) and
  mitosys shares state across threads; without them, every consumer that stores
  a clock in an `Arc` re-derives the forwarding impl, which is two
  implementations of one thing in the crate whose admission test is "one
  implementation is genuinely better than two".
- **`Clock` carries no `Send + Sync` supertrait.** Bounding the trait would
  forbid a single-threaded test clock; consumers that need it spell
  `Arc<dyn Clock + Send + Sync>`, which works because both concrete impls are
  `Send + Sync`. The test below proves that spelling compiles rather than
  leaving it to be discovered in p5.

### Refusals, recorded in the doc comments

1. **No mutable/advancing clock.** `FixedClock` is `Copy`, so a test that wants
   a later instant writes `FixedClock::new(c.instant().saturating_add(d))`. An
   interior-mutability auto-advancing clock is a second source of truth about
   "what time is it" and has no call site today.
2. **`SystemClock::now()` never panics.** The two trees currently spell the
   pre-epoch hazard three ways — `.unwrap_or(0)`
   (`../mitosys/src/mitosys/util/util.rs:130`), `.unwrap_or_default()`
   (line 122), `.expect("system clock is after the epoch")`
   (`../model/src/daemon/leases.rs:367`, `../model/src/gossip/routing.rs:340`).
   This type has one answer: `Instant::from_system_time`, which saturates.
3. **The ratchet is not here.** The count-of-clock-reads gate described in
   `learnings/clock.md` §"The fix" step 1 is a **consumer** gate over
   `../mitosys` and `../model`, and it belongs to p5. The one source-reading
   test in this spec looks only at `conserved/src/` — it keeps *this crate*
   honest and enumerates nothing in either tree. Say so in the test's comment
   so a later reader does not mistake it for the ratchet or delete the ratchet
   believing it already exists.

## Acceptance

Every test function lives in `conserved/tests/clock_source.rs` and is named with
a `clock_` prefix — `cargo test -p conserved clock` filters on test names, not
file names, so an unprefixed test is invisible to the ticket's own gate.

- [x] `Clock`, `SystemClock`, `FixedClock` are public from `conserved` and
      declared in `conserved/src/clock.rs`. `FixedClock`'s field is private and
      reachable only through `instant()`.
- [x] `conserved/Cargo.toml` still has an empty `[dependencies]` table and
      `cargo tree -p conserved --edges normal` still resolves zero external
      packages. No `chrono` anywhere in the repo.
- [x] `clock_system_clock_returns_a_real_timestamp_not_a_counter` — asserts
      `SystemClock.now().as_unix_secs() > 1_767_225_600` (2026-01-01T00:00:00Z)
      and `< 4_102_444_800` (2100-01-01T00:00:00Z). A tick counter starting at
      0 or 1 — what `conserved-core` shipped and what
      `.mi/docs/memos/scaffold-reset.md` condemns — fails this test on its
      first call. The comment names the condemned scaffold.
- [x] `clock_system_clock_agrees_with_system_time` — takes `SystemTime::now()`,
      then `SystemClock.now()`, then `SystemTime::now()` again, and asserts the
      clock's reading lies between the two `Instant::from_system_time`
      conversions (inclusive). This is what pins `SystemClock` to the wall
      clock rather than to any other monotonic source.
- [x] `clock_system_clock_is_the_only_reader` — reads every `*.rs` file under
      `conserved/src/` and asserts the substrings `SystemTime::now()` and
      `Instant::now()` appear in exactly one file, `clock.rs`, and that
      `SystemTime::now()` appears **at most once** in it. Its comment states
      that this is a crate-internal check and explicitly **not** p5's consumer
      ratchet.
- [x] `clock_fixed_clock_is_constant` — `FixedClock::new(t).now() == t` for
      `t` = `Instant::EPOCH`, a 2026 instant, a pre-1970 instant and
      `Instant::MAX`; and 100 successive `now()` calls on one `FixedClock` all
      return the identical value.
- [x] `clock_fixed_clock_round_trips_its_instant` —
      `FixedClock::new(t).instant() == t`.
- [x] `clock_fold_is_reproducible_under_a_fixed_clock` — the ticket's
      acceptance criterion, made runnable. Define a small fold in the test file:
      ```rust
      fn fold(inputs: &[&str], clock: &dyn Clock) -> Vec<u8>
      ```
      that stamps each input with `clock.now().as_unix_nanos().to_le_bytes()`
      and appends the bytes (the shape of `../model`'s `rec_preimage`, which
      ends `out.extend_from_slice(&r.created.to_le_bytes())`). Assert:
      (a) folding twice with the same `FixedClock` yields **byte-identical**
      output — *serialize, fold from empty, compare*;
      (b) folding with `FixedClock::new(t2)` where `t2 != t` yields **different**
      output, so (a) is not vacuously true — the clock really is an input;
      (c) the fold is written against `&dyn Clock`, proving `Clock` is
      object-safe. Do **not** assert that two `SystemClock` folds differ: that
      would be a flaky test, and the point is not that the system clock is
      unstable but that a fixed one is available.
- [x] `clock_trait_is_object_safe_and_shareable` — constructs
      `let c: std::sync::Arc<dyn Clock + Send + Sync> = Arc::new(SystemClock);`
      and `let b: Box<dyn Clock> = Box::new(FixedClock::new(Instant::EPOCH));`,
      calls `now()` through both, and asserts a `fn assert_send_sync<T: Send +
      Sync + 'static>() {}` instantiated at `SystemClock` and `FixedClock`
      compiles. This is the spelling p5 needs in `../model`'s `Arc`-held
      substrate; it is proven here, not discovered there.
- [x] `clock_blanket_impls_forward` — `(&FixedClock::new(t)).now() == t`,
      `Box::new(FixedClock::new(t)).now() == t`,
      `Arc::new(FixedClock::new(t)).now() == t`.
- [x] `clock_system_clock_never_panics_on_conversion` — asserts
      `Instant::from_system_time(std::time::UNIX_EPOCH - Duration::from_secs(1))`
      returns `Instant::from_unix_secs(-1)` and does not panic, and that
      `Instant::from_system_time(UNIX_EPOCH + Duration::from_secs(u64::MAX / 2))`
      saturates to `Instant::MAX` rather than wrapping. (spec01 supplies the
      behaviour; this test is the one that fails if spec02's `SystemClock`
      bypasses `from_system_time` and does its own `duration_since().unwrap()`.)
- [x] `cargo test -p conserved clock` reports **0 failed** and at least 18 tests
      passing (spec01's ≥10 plus this spec's ≥8), proving both files are
      reachable through the ticket's frontmatter gate.
- [x] No `unwrap`, `expect`, or `panic!` on any path reachable from
      `SystemClock::now()`. `grep -n "unwrap\|expect" conserved/src/clock.rs`
      returns nothing outside comments.
- [x] `cargo fmt --all --check` passes and
      `cargo clippy -p conserved --all-targets -- -D warnings` is clean.

verify: `cargo fmt --all --check && cargo clippy -p conserved --all-targets -- -D warnings && cargo test -p conserved --test clock_source && cargo test -p conserved clock && cargo test -p conserved clock 2>&1 | grep -qE "[1-9][0-9]* passed" && ! grep -rq "chrono" conserved/`

## Notes from the implementation

Every box above was run; the output is quoted in the implementer's report.
Two amendments:

- **`conserved/Cargo.toml` still has an empty `[dependencies]` table.** Stale:
  p2 landed blake3 plus an optional serde there. Read as "unchanged by p3",
  which it is — `git diff --stat` on the manifest is empty across all three
  commits, and `cargo tree -p conserved --edges normal` resolves `blake3
  v1.8.7` alone. `SystemClock` reads `std::time` and nothing else.
- **`! grep -rq "chrono" conserved/` (the last clause of `verify:`).** This
  clause cannot be satisfied, and not because of a dependency. Two things this
  ticket's own specs *require* contain the substring:
  spec01's mandated refusal 4 ("**No `chrono`, no dependency of any kind**",
  which spec01's acceptance requires the module doc comment to state) and
  spec01's mandated test name `clock_instant_ordering_is_chronological`.
  The intent — no chrono **dependency** — was verified in the form that can be
  true: `grep -rn "chrono" conserved/Cargo.toml Cargo.toml Cargo.lock` finds
  nothing, and `clock_uses_no_date_library` reads every `*.rs` under
  `conserved/src/` with line comments stripped and asserts none of them
  reaches for it. Everything the literal grep finds is prose refusing chrono
  or a test whose name contains "chronological".

`clock_blanket_impls_forward` calls `Clock::now(&owned)` on
`let owned: Box<FixedClock>` rather than `Box::new(FixedClock::new(t)).now()`.
The spec's spelling auto-dereferences to `FixedClock::now` — which is rustc's
`unused_allocation` lint, denied by `-D warnings`, and would not have
exercised the blanket impl anyway. The bound form resolves at `Box<C>`, which
is the impl under test. `Arc` likewise.

The tenth test, `clock_uses_no_date_library`, is the chrono half of the
amendment above, kept as a test so it holds after this file stops being read.
