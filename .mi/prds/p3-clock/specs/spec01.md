# goal

`Instant` — the timestamp type, with its epoch and its resolution **decided,
written in the doc comment, and pinned by a test**, so mitosys and the llm tree
cannot adopt the same `i64` meaning two different things.

est: 1.5h

## What p0 leaves behind (assumed, not created here)

- Package `conserved` at `conserved/`, `edition = "2021"`,
  `rust-version = "1.94.0"`, `#![forbid(unsafe_code)]` already on
  `conserved/src/lib.rs`, which is doc-comment-and-attributes only.
- Root `Cargo.toml` is `[workspace]`-only, `resolver = "2"`, one member,
  `[workspace.dependencies]` empty.
- Tests live at `conserved/tests/` (the mitosys shape). No `src/**/tests/`, no
  `#[cfg(test)]` module beside the code.
- `conserved/Cargo.toml`'s `[dependencies]` table is empty.
- `rustfmt.toml` sets `hard_tabs = true`, `tab_spaces = 2` — write to it.

**This spec adds no dependency.** It does not touch `conserved/Cargo.toml` at
all. If it appears to need one, stop: the module's contract is `std::time` and
nothing else.

If p0 landed a different package name or test root, stop and re-spec — the
ticket's own `verify` (`cargo test -p conserved clock`) names the package.

## The decision this spec makes: epoch and resolution

**`Instant(i64)` is nanoseconds since 1970-01-01T00:00:00Z, UTC, no leap
seconds.** Representable range `Instant::MIN` = `1677-09-21T00:12:43.145224192Z`
to `Instant::MAX` = `2262-04-11T23:47:16.854775807Z`.

`learnings/clock.md` writes the type as `Instant(i64); // unix, whatever
precision the tree needs`. "Whatever the tree needs" is exactly the sentence
that lets two trees adopt it meaning two things, so it is resolved here against
what the call sites actually read:

| tree | site | resolution it needs |
|---|---|---|
| llm | `src/record/mod.rs:239` `rec_now()` → `Record.created: i64` "unix seconds" (**and into `rec_preimage`**) | seconds |
| llm | `src/daemon/leases.rs:364` `unix_now()`, `src/node/live.rs:779` / `src/loop/checkpoint.rs:182` `epoch_seconds()`, `src/chat/mod.rs:585` `now_secs()`, `src/gossip/routing.rs:337` `unix_secs()` | seconds |
| llm | `src/record/event.rs:1140` `event_now_millis()` — doc: "the event record's timestamp resolution" | **milliseconds** |
| llm | `src/node/transactional.rs:72` `Commit.timestamp` | seconds (doc says "milliseconds" — see below) |
| mitosys | `util/util.rs:137` `now_secs()`, `engine/record/store.rs:902` `now_rfc3339()` | seconds |
| mitosys | `util/util.rs:128` `now_ms()` (8 live callers) | milliseconds |
| mitosys | `engine/base/base_types.rs:685` `membrane_nonce(SystemTime) -> u128` — nanoseconds folded into a **persisted membrane id** | **nanoseconds** |
| mitosys | `engine/transport/memory_rpc.rs:62` `jittered()` — `subsec_nanos()` | sub-millisecond |

**Rejected: seconds.** It is what the dangerous site (`rec_now()` → the
content-hash preimage) uses and what most sites use, which is why it is the
obvious candidate. It is wrong because a second-resolution `Instant` cannot
express `event_now_millis` (the llm tree's *documented* event-record
resolution) or `membrane_nonce` — so those sites would have to keep a raw
`SystemTime::now()` read, and this ticket's whole point is that the compliant
path must exist for every site, not most of them.

**Also rejected: milliseconds.** It covers the event log, but adopting it at
`membrane_nonce` would change every persisted membrane id (nanos truncated to
millis is a different nonce), and it cannot serve `jittered()`. Nanoseconds is
the only i64 resolution from which seconds, milliseconds and nanoseconds are
all recoverable by exact integer division; nothing is recoverable upward. Its
one cost — the 1677..2262 range — is not a cost any site in either tree pays.

**Why the doc comment is not enough, in one sentence the module must carry:**
`../model/src/node/transactional.rs:59` documents `timestamp` as "unix epoch
milliseconds" and line 72 computes `.as_secs()`. A unit stated only in prose
has already drifted in one of the two trees; hence the pinning test below.

## Files

- `conserved/src/clock.rs` — **new**. `Instant` and nothing else in this spec
  (`Clock`/`SystemClock`/`FixedClock` are spec02, in the same file).
- `conserved/src/lib.rs` — add `pub mod clock;` and
  `pub use clock::Instant;`. **Nothing else in this file changes.**
- `conserved/tests/clock_instant.rs` — **new**.

Touches nothing else. Does **not** touch `conserved/Cargo.toml`, `../mitosys`,
`../model`, `../realm`, `learnings/`, or any other ticket's directory.

## The surface

```rust
/// A point on the Unix timeline: nanoseconds since 1970-01-01T00:00:00Z,
/// UTC, no leap seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Instant(i64);

impl Instant {
	pub const EPOCH: Instant;   // 0
	pub const MIN: Instant;     // i64::MIN ns — 1677-09-21T00:12:43.145224192Z
	pub const MAX: Instant;     // i64::MAX ns — 2262-04-11T23:47:16.854775807Z

	pub const fn from_unix_nanos(n: i64) -> Self;
	pub const fn as_unix_nanos(self) -> i64;

	pub fn from_unix_millis(ms: i64) -> Self;   // saturating
	pub fn as_unix_millis(self) -> i64;         // FLOOR, not truncate
	pub fn from_unix_secs(s: i64) -> Self;      // saturating
	pub fn as_unix_secs(self) -> i64;           // FLOOR, not truncate

	pub fn from_system_time(t: SystemTime) -> Self;  // saturating, never panics
	pub fn to_system_time(self) -> SystemTime;

	pub fn saturating_add(self, d: Duration) -> Self;
	pub fn saturating_sub(self, d: Duration) -> Self;
	pub fn signed_nanos_since(self, earlier: Instant) -> i64;  // saturating
}
```

Every item above is anchored to a call site, and the module doc comment says
which:

- **`from_system_time` / `to_system_time`** — mitosys already threads time as a
  parameter *spelled `std::time::SystemTime`*: `distill(&text, extra_kinds,
  llm, SystemTime::now())` (`engine/ingest/ingest_intake.rs:31`),
  `entity_detail_by_id(&g, id, SystemTime::now())`
  (`engine/commands/commands_graph_ops.rs:139`), `membrane_nonce(t:
  SystemTime)` (`engine/base/base_types.rs:685`), and ~25 more. Without the
  bridge, p5 cannot convert those sites one at a time.
- **`signed_nanos_since`** — `../model/src/record/mod.rs:258` `rec_heat` does
  `let dt = (now - r.heat_at) as f32;` and then branches on `dt <= 0.0`. It
  needs a **signed** difference; a `Duration`-returning `duration_since` cannot
  express the negative branch.
- **`saturating_add` / `saturating_sub`** — lease deadlines
  (`../model/src/daemon/leases.rs`, `expires_at`) and every `now + ttl`.
- **`EPOCH`, `MIN`, `MAX`** — `EPOCH` gives the literal `0` both trees use as a
  sentinel a name that is not a timestamp-shaped integer; `MIN`/`MAX` make the
  saturation behaviour testable without a clock.

### The refusals, recorded in the module doc comment

A reader who adds one of these must first delete the paragraph that refuses it.

1. **No `Default`.** `[u8; 32]`-style zero-value reasoning applies: `0` *is* a
   real instant (the epoch), and both trees already overload `0` as "unset" —
   `until == 0` is an open record (`../model/src/record/mod.rs:174`),
   `heat_at == 0` is "never touched" (line 251), `LogThrottle`'s `last_secs ==
   0` is "never fired" (`../mitosys/src/mitosys/util/util.rs:222`). A
   `Default` would let the sentinel and the timestamp wear one spelling.
   `Instant::EPOCH` is the spelling for the epoch; the sentinel stays the
   consumer's `Option`.
2. **This is not `std::time::Instant`.** std's is an opaque monotonic reading,
   comparable only within one process and not serializable; this one is a
   wall-clock point, comparable and storable across processes and machines.
   The 27 `Instant::now()` sites in mitosys and the 20 in `../model` are the
   monotonic kind — profilers (`util/profile.rs:32`), deadlines
   (`api/agentic/cli.rs:467`, `grade/chat.rs:123`) — and p5 must **not**
   convert them. `conserved/src/` never imports `std::time::Instant`; the test
   below enforces that.
3. **No date formatting.** mitosys's `now_rfc3339()`
   (`engine/record/store.rs:901`) and its `civil_from_days` stay mitosys's. A
   calendar is not a clock, and it has no second consumer.
4. **No `chrono`, no dependency of any kind.** `learnings/shared-crate.md` §2
   and the reason `conserved-core` was condemned
   (`.mi/docs/memos/scaffold-reset.md`).
5. **No monotonic/elapsed clock.** Measuring a duration inside one process is
   not a fold input and is not what law 2's verification needs.

## Acceptance

Every test function below lives in `conserved/tests/clock_instant.rs` and is
named with a `clock_` prefix. This is not cosmetic: `cargo test -p conserved
clock` filters on **test names**, not file names (verified: a test named
`plain_name` in `tests/clockfile.rs` is filtered *out* by that command), so a
test without the prefix is invisible to the ticket's own gate.

- [ ] `conserved/src/clock.rs` exists; `Instant` is a newtype over `i64` whose
      field is **private**, deriving `Clone, Copy, Debug, PartialEq, Eq, Hash,
      PartialOrd, Ord`.
- [ ] `conserved/src/lib.rs` gained exactly `pub mod clock;` and
      `pub use clock::Instant;` — no other line of that file changed by this spec. (p1 already added `pub mod scope;` there; append, do not rewrite.)
- [ ] `conserved/Cargo.toml` is byte-identical to its p0 content: `[dependencies]`
      still empty, and `cargo tree -p conserved --edges normal` still resolves
      zero external packages.
- [ ] `clock_instant_unit_is_unix_nanoseconds` — the unit-pinning test the
      ticket's acceptance names. Asserts **all** of:
      `Instant::EPOCH.as_unix_nanos() == 0`;
      `Instant::from_unix_secs(1).as_unix_nanos() == 1_000_000_000`;
      `Instant::from_unix_millis(1).as_unix_nanos() == 1_000_000`;
      `Instant::from_unix_secs(1_787_270_400).as_unix_millis() == 1_787_270_400_000`
      (2026-08-21T00:00:00Z);
      `Instant::from_unix_nanos(1_787_270_400_123_456_789).as_unix_secs() == 1_787_270_400`.
      Its comment states that changing the resolution is a wire-format change
      for both trees, not a refactor.
- [ ] `clock_instant_epoch_is_unix_not_boot_and_not_a_counter` — asserts
      `Instant::EPOCH.to_system_time() == std::time::UNIX_EPOCH` and
      `Instant::from_system_time(std::time::UNIX_EPOCH).as_unix_nanos() == 0`.
      This is the test that fails if someone reintroduces the condemned
      scaffold's tick-counter reading of `Instant`
      (`.mi/docs/memos/scaffold-reset.md`).
- [ ] `clock_instant_range_bounds` — `Instant::MIN.as_unix_nanos() ==
      i64::MIN`, `Instant::MAX.as_unix_nanos() == i64::MAX`, and
      `Instant::MIN < Instant::EPOCH && Instant::EPOCH < Instant::MAX`. The
      comment names the two calendar dates.
- [ ] `clock_instant_as_unix_secs_floors_before_the_epoch` — the truncation
      trap, as its own named test:
      `Instant::from_unix_nanos(-1_500_000_000).as_unix_secs() == -2` (Rust's
      `/` truncates toward zero and would give `-1`; the accessor must use
      `div_euclid`), `Instant::from_unix_nanos(-1).as_unix_secs() == -1`,
      `Instant::from_unix_nanos(-1_500_000_000).as_unix_millis() == -1_500`,
      and `Instant::from_unix_nanos(1_999_999_999).as_unix_secs() == 1`.
- [ ] `clock_instant_conversions_saturate_rather_than_wrap` —
      `Instant::from_unix_secs(i64::MAX) == Instant::MAX`,
      `Instant::from_unix_secs(i64::MIN) == Instant::MIN`,
      `Instant::from_unix_millis(i64::MAX) == Instant::MAX`,
      `Instant::MAX.saturating_add(Duration::from_secs(1)) == Instant::MAX`,
      `Instant::MIN.saturating_sub(Duration::from_secs(1)) == Instant::MIN`.
      No test in this file may panic on overflow in debug mode.
- [ ] `clock_instant_signed_nanos_since_is_signed` —
      `later.signed_nanos_since(earlier) > 0`,
      `earlier.signed_nanos_since(later) < 0`,
      `t.signed_nanos_since(t) == 0`, and
      `Instant::MIN.signed_nanos_since(Instant::MAX) == i64::MIN` (saturating,
      not a wrap and not a panic).
- [ ] `clock_instant_system_time_round_trip` — for
      `Instant::EPOCH`, one instant in 2026, one pre-1970 instant, and
      `Instant::MAX`: `Instant::from_system_time(i.to_system_time()) == i`.
      The pre-1970 case is its own assertion, because
      `SystemTime::duration_since(UNIX_EPOCH)` returns `Err` there and a naive
      `.unwrap_or_default()` silently yields the epoch.
- [ ] `clock_instant_ordering_is_chronological` — a `Vec<Instant>` built from
      out-of-order unix seconds sorts into ascending chronological order, and
      pre-epoch instants sort before `Instant::EPOCH`. (Derived `Ord` on `i64`
      gives this; the test is what stops a later `Ord` impl on an unsigned
      representation from silently reversing it.)
- [ ] `clock_instant_has_no_default` — reads `conserved/src/clock.rs` as text
      and asserts it contains neither `derive(` … `Default` on `Instant` nor
      `impl Default for Instant`, and that the refusal paragraph is present
      (the file contains the string `Instant::EPOCH` inside a `//!`- or
      `///`-prefixed line). The comment says why: the epoch and "unset" must
      not share a spelling.
- [ ] `clock_src_never_uses_std_time_instant` — reads every `*.rs` under
      `conserved/src/` and asserts none contains `std::time::Instant` or
      `use std::time::Instant`. Its comment names the collision: `conserved::
      Instant` is wall-clock, `std::time::Instant` is monotonic, and the ~47
      monotonic sites across the two trees must not be converted.
- [ ] `cargo test -p conserved clock` reports at least 10 tests passing and
      **0 failed** — i.e. every test this spec adds is reachable through the
      ticket's own frontmatter gate, not merely through `--test clock_instant`.
- [ ] The module doc comment on `conserved/src/clock.rs` states the epoch, the
      resolution, the representable range, and all five refusals above.
- [ ] No `unsafe` (the crate already `#![forbid(unsafe_code)]`), no `unwrap`,
      no `expect`, and no `panic!` on any path reachable from a public method —
      including `from_system_time` on a pre-epoch or far-future `SystemTime`.
      The two trees currently spell this hazard three different ways
      (`.unwrap_or(0)`, `.unwrap_or_default()`,
      `.expect("system clock is after the epoch")`); this type has one answer
      and it is never a panic.
- [ ] `cargo fmt --all --check` passes (hard tabs, width 2) and
      `cargo clippy -p conserved --all-targets -- -D warnings` is clean.

verify: `cargo fmt --all --check && cargo clippy -p conserved --all-targets -- -D warnings && cargo test -p conserved --test clock_instant && cargo test -p conserved clock && cargo test -p conserved clock 2>&1 | grep -qE "[1-9][0-9]* passed"`
