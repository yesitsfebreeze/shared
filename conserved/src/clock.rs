//! Time as a parameter: [`Instant`], [`Clock`], [`SystemClock`],
//! [`FixedClock`].
//!
//! Law 2's verification — *serialize the record, fold from empty, compare* —
//! is impossible wherever a fold reads the wall clock. This module is the
//! compliant path: the timestamp is a value, the clock is a parameter, and
//! exactly one implementation of that parameter is permitted to ask the
//! operating system. `learnings/clock.md` counts ~65 non-test clock reads in
//! each tree against a shared law that forbids them.
//!
//! # The unit
//!
//! **[`Instant`] is nanoseconds since 1970-01-01T00:00:00Z, UTC, no leap
//! seconds.** The representable range is [`Instant::MIN`] =
//! 1677-09-21T00:12:43.145224192Z to [`Instant::MAX`] =
//! 2262-04-11T23:47:16.854775807Z.
//!
//! `learnings/clock.md` writes the type as `Instant(i64); // unix, whatever
//! precision the tree needs`. "Whatever the tree needs" is exactly the sentence
//! that lets two trees adopt one `i64` meaning two things, so it is resolved
//! here against what the call sites actually read. Most sites want seconds
//! (`../model`'s `rec_now()`, `unix_now()`, `epoch_seconds()`; mitosys's
//! `now_secs()`), but `../model`'s `event_now_millis()` documents
//! *milliseconds* as the event record's resolution and mitosys's
//! `membrane_nonce(SystemTime)` folds *nanoseconds* into a persisted membrane
//! id. Nanoseconds is the only `i64` resolution from which all three are
//! recoverable by exact integer division; nothing is recoverable upward. Its
//! one cost — the 1677..2262 range — is not a cost any site in either tree
//! pays.
//!
//! **Why the doc comment is not enough.**
//! `../model/src/node/transactional.rs:59` documents its `timestamp` field as
//! "unix epoch milliseconds" and line 72 computes `.as_secs()`. A unit stated
//! only in prose has already drifted in one of the two trees, so the unit is
//! pinned by `clock_instant_unit_is_unix_nanoseconds` in
//! `conserved/tests/clock_instant.rs`. Changing the resolution is a
//! wire-format change for both trees, not a refactor.
//!
//! # Why each item on the surface exists
//!
//! - **[`Instant::from_system_time`] / [`Instant::to_system_time`]** — mitosys
//!   already threads time as a parameter *spelled `SystemTime`*:
//!   `distill(&text, extra_kinds, llm, SystemTime::now())`
//!   (`engine/ingest/ingest_intake.rs:31`), `membrane_nonce(t: SystemTime)`
//!   (`engine/base/base_types.rs:685`), and ~25 more. Without the bridge the
//!   adoption node cannot convert those sites one at a time.
//! - **[`Instant::signed_nanos_since`]** — `../model/src/record/mod.rs:258`
//!   `rec_heat` computes `let dt = (now - r.heat_at) as f32;` and then branches
//!   on `dt <= 0.0`. It needs a **signed** difference; a `Duration`-returning
//!   `duration_since` cannot express the negative branch.
//! - **[`Instant::saturating_add`] / [`Instant::saturating_sub`]** — lease
//!   deadlines (`../model/src/daemon/leases.rs`, `expires_at`) and every
//!   `now + ttl`.
//! - **[`Instant::EPOCH`], [`Instant::MIN`], [`Instant::MAX`]** — `EPOCH`
//!   gives the literal `0` both trees use as a sentinel a name that is not a
//!   timestamp-shaped integer; `MIN`/`MAX` make the saturation behaviour
//!   testable without a clock.
//!
//! # What this module refuses
//!
//! A reader who adds one of these must delete the paragraph that refuses it
//! first.
//!
//! 1. **No `Default`.** `0` *is* a real instant — the epoch — and both trees
//!    already overload `0` as "unset": `until == 0` is an open record
//!    (`../model/src/record/mod.rs:174`), `heat_at == 0` is "never touched"
//!    (line 251), `LogThrottle`'s `last_secs == 0` is "never fired"
//!    (`../mitosys/src/mitosys/util/util.rs:222`). A `Default` would let the
//!    sentinel and the timestamp wear one spelling. `Instant::EPOCH` is the
//!    spelling for the epoch; the sentinel stays the consumer's `Option`.
//! 2. **This is not std's monotonic `Instant`.** std's `time::Instant` is an
//!    opaque monotonic reading, comparable only within one process and not
//!    serializable; this one is a wall-clock point, comparable and storable
//!    across processes and machines. The 27 monotonic `Instant::now()` sites
//!    in mitosys and the 20 in `../model` — profilers
//!    (`util/profile.rs:32`), deadlines (`api/agentic/cli.rs:467`,
//!    `grade/chat.rs:123`) — are the *other* kind and must not be converted.
//!    `conserved/src/` never imports std's monotonic `Instant`;
//!    `clock_src_never_uses_std_time_instant` enforces that by reading the
//!    source.
//! 3. **No date formatting.** mitosys's `now_rfc3339()`
//!    (`engine/record/store.rs:901`) and its `civil_from_days` stay mitosys's.
//!    A calendar is not a clock, and it has no second consumer.
//! 4. **No `chrono`, no dependency of any kind.** The bridge to the operating
//!    system is `std::time` and nothing else — `learnings/shared-crate.md` §2,
//!    and the reason `conserved-core` was condemned
//!    (`.mi/docs/memos/scaffold-reset.md`).
//! 5. **No monotonic/elapsed clock.** Measuring a duration inside one process
//!    is not a fold input and is not what law 2's verification needs.
//!
//! # Panics
//!
//! None. Every conversion here saturates, including a pre-epoch or far-future
//! `SystemTime`. The two trees currently spell that hazard three different
//! ways — `.unwrap_or(0)`, `.unwrap_or_default()`,
//! `.expect("system clock is after the epoch")` — and the third of those
//! aborts the process when a machine's clock is wrong. This type has one
//! answer and it is never a panic.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Nanoseconds in one second.
const NANOS_PER_SEC: i64 = 1_000_000_000;

/// Nanoseconds in one millisecond.
const NANOS_PER_MILLI: i64 = 1_000_000;

/// A point on the Unix timeline: nanoseconds since 1970-01-01T00:00:00Z, UTC,
/// no leap seconds.
///
/// The range is [`Instant::MIN`] (1677-09-21T00:12:43.145224192Z) to
/// [`Instant::MAX`] (2262-04-11T23:47:16.854775807Z).
///
/// Two refusals, repeated here from the module documentation because they are
/// the surface a caller meets:
///
/// - **No `Default`.** `0` is the epoch, a real instant, and both trees
///   already use `0` as an "unset" sentinel. Write [`Instant::EPOCH`] for the
///   epoch and `Option<Instant>` for "unset".
/// - **This is the wall clock.** std's `time::Instant` is monotonic,
///   process-local and unserializable; do not reach for one where this type is
///   wanted, or the other way round.
///
/// Every accessor **floors** rather than truncating, and every conversion
/// saturates rather than wrapping or panicking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Instant(i64);

impl Instant {
	/// 1970-01-01T00:00:00Z — zero nanoseconds.
	///
	/// This is the name for the literal `0`. It is *not* a sentinel: see the
	/// module's refusal 1.
	pub const EPOCH: Instant = Instant(0);

	/// The earliest representable instant: 1677-09-21T00:12:43.145224192Z.
	pub const MIN: Instant = Instant(i64::MIN);

	/// The latest representable instant: 2262-04-11T23:47:16.854775807Z.
	pub const MAX: Instant = Instant(i64::MAX);

	/// The instant `n` nanoseconds after the Unix epoch. Negative is before it.
	pub const fn from_unix_nanos(n: i64) -> Self {
		Self(n)
	}

	/// Nanoseconds since the Unix epoch. The exact representation; every other
	/// accessor is derived from this one and loses precision.
	pub const fn as_unix_nanos(self) -> i64 {
		self.0
	}

	/// The instant `ms` milliseconds after the Unix epoch, saturating at
	/// [`Instant::MIN`] / [`Instant::MAX`] rather than wrapping.
	pub fn from_unix_millis(ms: i64) -> Self {
		Self(ms.saturating_mul(NANOS_PER_MILLI))
	}

	/// Whole milliseconds since the Unix epoch, **floored** — not truncated.
	///
	/// `div_euclid`, not `/`: Rust's `/` truncates toward zero, so a naive
	/// implementation reports `-1` for an instant 1.5 ms before the epoch,
	/// where the floor is `-2`. Flooring is what makes the accessor monotone
	/// across the epoch, which is what a comparison or a bucket key needs.
	pub fn as_unix_millis(self) -> i64 {
		self.0.div_euclid(NANOS_PER_MILLI)
	}

	/// The instant `s` seconds after the Unix epoch, saturating at
	/// [`Instant::MIN`] / [`Instant::MAX`] rather than wrapping.
	pub fn from_unix_secs(s: i64) -> Self {
		Self(s.saturating_mul(NANOS_PER_SEC))
	}

	/// Whole seconds since the Unix epoch, **floored** — not truncated. See
	/// [`Instant::as_unix_millis`] for why.
	pub fn as_unix_secs(self) -> i64 {
		self.0.div_euclid(NANOS_PER_SEC)
	}

	/// Adopt a `SystemTime`, saturating at the ends of the range.
	///
	/// Never panics and never silently answers the epoch. A `SystemTime`
	/// before 1970 makes `duration_since(UNIX_EPOCH)` return `Err`, which is
	/// where both trees currently reach for `.unwrap_or_default()` (the epoch,
	/// silently wrong) or `.expect(..)` (an abort on a machine whose clock is
	/// wrong). Here the `Err` carries the distance *before* the epoch and is
	/// converted to a negative instant.
	pub fn from_system_time(t: SystemTime) -> Self {
		match t.duration_since(UNIX_EPOCH) {
			Ok(after) => Self(clamp_after_epoch(after.as_nanos())),
			Err(before) => Self(clamp_before_epoch(before.duration().as_nanos())),
		}
	}

	/// The same point spelled as a `SystemTime`, for the ~25 mitosys sites
	/// that already take one as a parameter.
	///
	/// Saturates at the platform's own range rather than panicking: see
	/// [`shift_from_epoch`].
	pub fn to_system_time(self) -> SystemTime {
		shift_from_epoch(self.0.unsigned_abs(), self.0 >= 0)
	}

	/// `self + d`, saturating at [`Instant::MAX`].
	pub fn saturating_add(self, d: Duration) -> Self {
		Self(self.0.saturating_add(nanos_of(d)))
	}

	/// `self - d`, saturating at [`Instant::MIN`].
	pub fn saturating_sub(self, d: Duration) -> Self {
		Self(self.0.saturating_sub(nanos_of(d)))
	}

	/// Nanoseconds from `earlier` to `self`, **signed**: negative when `self`
	/// is the earlier of the two, saturating at the ends rather than wrapping.
	///
	/// `../model`'s `rec_heat` (`src/record/mod.rs:258`) branches on the sign
	/// of exactly this quantity, which a `Duration`-returning `duration_since`
	/// cannot express.
	pub fn signed_nanos_since(self, earlier: Instant) -> i64 {
		self.0.saturating_sub(earlier.0)
	}
}

/// The largest nanosecond count this type reaches after the epoch.
const MAX_NANOS_AFTER: u128 = i64::MAX as u128;

/// The largest nanosecond count this type reaches before the epoch — the
/// magnitude of [`i64::MIN`], which is one larger than [`i64::MAX`].
const MAX_NANOS_BEFORE: u128 = 1u128 << 63;

/// `n` nanoseconds after the epoch, saturating at [`i64::MAX`].
///
/// Written as a comparison rather than `i64::try_from(..)` plus a fallback,
/// because every fallback spelling std offers contains the string `unwrap`,
/// and `clock_system_clock_is_the_only_reader`'s companion check greps this
/// file for exactly that word: no clock read in this crate may panic, and the
/// cheapest way to keep that legible is for the word never to appear.
const fn clamp_after_epoch(n: u128) -> i64 {
	if n > MAX_NANOS_AFTER {
		i64::MAX
	} else {
		// `n <= i64::MAX`, so the cast is exact.
		n as i64
	}
}

/// `n` nanoseconds *before* the epoch, as a negative instant, saturating at
/// [`i64::MIN`].
const fn clamp_before_epoch(n: u128) -> i64 {
	if n >= MAX_NANOS_BEFORE {
		i64::MIN
	} else {
		// `n < 2^63`, so the cast is exact and the negation cannot overflow.
		-(n as i64)
	}
}

/// A `Duration` as `i64` nanoseconds, saturating at [`i64::MAX`].
///
/// A `Duration` counts up to ~584 years of nanoseconds in a `u128`, which is
/// wider than this type's range; anything past it saturates rather than
/// wrapping.
fn nanos_of(d: Duration) -> i64 {
	clamp_after_epoch(d.as_nanos())
}

/// `UNIX_EPOCH` shifted `nanos` forward (`forward`) or backward, without
/// panicking.
///
/// `SystemTime`'s `Add`/`Sub` panic on overflow and its representable range is
/// the platform's, not this type's, so the shift is applied one set bit at a
/// time through `checked_add`/`checked_sub` and simply stops when the platform
/// runs out of range. The sum of the set bits is `nanos` exactly, so on every
/// platform either tree targets the result is exact; nowhere does it panic,
/// and nowhere does it collapse silently to the epoch.
fn shift_from_epoch(nanos: u64, forward: bool) -> SystemTime {
	let mut out = UNIX_EPOCH;
	let mut bit = 1u64 << 63;
	while bit != 0 {
		if nanos & bit != 0 {
			let step = Duration::from_nanos(bit);
			let next = if forward {
				out.checked_add(step)
			} else {
				out.checked_sub(step)
			};
			match next {
				Some(t) => out = t,
				None => return out,
			}
		}
		bit >>= 1;
	}
	out
}

// ---------------------------------------------------------------------------
// The clock: one trait, one reader of the operating system, one fixed source.
// ---------------------------------------------------------------------------

/// The one way to ask what time it is.
///
/// A fold that needs a timestamp takes one of these instead of reading the
/// wall clock, which is what makes *serialize the record, fold from empty,
/// compare* possible at all: substitute a [`FixedClock`] and the fold's output
/// is a function of its inputs again.
///
/// The trait carries **no `Send + Sync` supertrait**. Bounding it would forbid
/// a single-threaded test clock; a consumer that shares one across tasks
/// spells `Arc<dyn Clock + Send + Sync>`, which works because both concrete
/// implementations here are `Send + Sync`. `clock_trait_is_object_safe_and_shareable`
/// proves that spelling compiles, so the adoption node does not discover it.
pub trait Clock {
	/// The current instant, according to this clock.
	fn now(&self) -> Instant;
}

/// The **one** implementation permitted to read the operating system.
///
/// Everything else in either tree takes a [`Clock`]. Concentrating the read
/// here is what makes the count-of-clock-reads gate a countable thing at all.
///
/// `SystemClock` derives `Default` and [`Instant`] does not: a unit struct's
/// default is itself and carries no ambiguity, whereas a default *timestamp*
/// would be the epoch wearing the sentinel's clothes (see the module's
/// refusal 1).
///
/// # Panics
///
/// None. [`Clock::now`] goes through [`Instant::from_system_time`], which
/// saturates. The two trees currently spell that hazard three ways —
/// `.unwrap_or(0)` (`../mitosys/src/mitosys/util/util.rs:130`),
/// `.unwrap_or_default()` (line 122), and
/// `.expect("system clock is after the epoch")`
/// (`../model/src/daemon/leases.rs:367`, `../model/src/gossip/routing.rs:340`),
/// the last of which aborts a process because a machine's clock is wrong.
/// This type has one answer and it is never a panic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SystemClock;

impl Clock for SystemClock {
	fn now(&self) -> Instant {
		Instant::from_system_time(SystemTime::now())
	}
}

/// A clock that answers the same instant every time.
///
/// This is the type the ticket exists for: a fold parameterised over a
/// `FixedClock` is reproducible, so its output can be compared byte for byte
/// against a replay. `clock_fold_is_reproducible_under_a_fixed_clock` runs
/// that argument.
///
/// **No advancing clock.** `FixedClock` is `Copy`, so a test that wants a
/// later instant writes
/// `FixedClock::new(c.instant().saturating_add(d))`. A clock that advances
/// itself through interior mutability would be a second source of truth about
/// what time it is, and it has no call site in either tree today.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedClock(Instant);

impl FixedClock {
	/// A clock stopped at `at`.
	pub const fn new(at: Instant) -> Self {
		Self(at)
	}

	/// The instant this clock is stopped at.
	pub const fn instant(self) -> Instant {
		self.0
	}
}

impl Clock for FixedClock {
	fn now(&self) -> Instant {
		self.0
	}
}

/// Borrowed clocks are clocks.
///
/// The three forwarding impls below exist because `../model` holds its
/// substrate in `Arc` across tokio tasks (`src/daemon/`, `src/gossip/`) and
/// mitosys shares state across threads. Without them every consumer that
/// stores a clock behind a pointer re-derives this forwarding, which is two
/// implementations of one thing inside the crate whose admission test is
/// "one implementation is genuinely better than two".
impl<C: Clock + ?Sized> Clock for &C {
	fn now(&self) -> Instant {
		(**self).now()
	}
}

/// Boxed clocks are clocks — including `Box<dyn Clock>`, which is why
/// [`Clock`] is kept object-safe.
impl<C: Clock + ?Sized> Clock for Box<C> {
	fn now(&self) -> Instant {
		(**self).now()
	}
}

/// Shared clocks are clocks: the `Arc<dyn Clock + Send + Sync>` spelling
/// `../model`'s task-held substrate needs.
impl<C: Clock + ?Sized> Clock for std::sync::Arc<C> {
	fn now(&self) -> Instant {
		(**self).now()
	}
}

// ---------------------------------------------------------------------------
// serde — behind the optional `serde` feature p2 admitted, off by default.
// ---------------------------------------------------------------------------

/// An [`Instant`] serializes as its `i64` nanoseconds, **identically in every
/// format** — no `is_human_readable()` branch. The transparency is the point:
/// a field that is `i64` today can become an `Instant` without a wire change,
/// which is the only way `../model`'s `created`, `until`, `heat_at` and
/// `expires_at` can adopt the type at all.
///
/// This deliberately diverges from `ContentId`, which *does* branch. A
/// `[u8; 32]` in JSON is an array of 32 integers and `../model` already hands
/// ids out as hex by hand, so hex made an existing convention typed. There is
/// no comparable convention for timestamps: every field this type replaces is
/// already a JSON *number*, and a decimal string of the same digits is not
/// more readable — an `Instant`'s readability problem is that it is not a
/// date, and this module refuses a date formatter with a reason (refusal 3).
///
/// **The 2^53 note.** Nanoseconds since the epoch is ~1.8e18, past JSON's
/// exact-integer range, so a JavaScript client that reads the number as a
/// double loses precision. That hazard belongs to the JSON consumer, and the
/// escape hatch is at the boundary, not in the representation: a surface
/// handing a timestamp to a browser converts with
/// [`Instant::as_unix_millis`], which is exact well past the year 285,000.
/// One surface's problem does not move the whole family's encoding.
///
/// **Where transparency does not hold.**
/// `../model/src/node/transactional.rs:47-62` types `Commit.timestamp` as
/// `u64`, and postcard varint-encodes `u64` and `i64` differently (zigzag), so
/// substituting an `Instant` *there* changes the commit's blake3 content hash.
/// That is a deliberate decision for the adoption node to make; this module's
/// job is to have said it out loud first. Note also that `Commit` is the
/// **second** content hash in that tree fed by a live clock read (line 72) —
/// `learnings/clock.md` names only `rec_now()`.
///
/// **No serde for the clocks.** [`SystemClock`] and [`FixedClock`] get no
/// impls: a clock is not data, only the instant it produced is. Serializing a
/// clock would put "which clock made this" on a wire that only ever needed
/// "when".
#[cfg(feature = "serde")]
impl serde::Serialize for Instant {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_i64(self.0)
	}
}

/// The mirror of [`Serialize`](serde::Serialize): the wire integer is
/// **nanoseconds**, read straight through [`Instant::from_unix_nanos`].
///
/// Deferring to `i64`'s own `Deserialize` is what makes the rejections come
/// for free and stay consistent with the serializer: a JSON string, a float,
/// `null`, an array, and an integer past [`i64::MAX`] are all errors, in one
/// representation and only one.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Instant {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		<i64 as serde::Deserialize>::deserialize(deserializer).map(Instant::from_unix_nanos)
	}
}
