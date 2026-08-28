//! Reversible effects — a scope that unwinds what was registered on it.
//!
//! Every side effect registers through a [`Scope`]: the registration runs,
//! hands back the thing that undoes it, and the scope tracks that inverse
//! until the caller disposes it or the scope closes and unwinds everything in
//! reverse. Leaving something behind is not expressible. The semantics are
//! Cordis's (the framework under DeepSeek Harness), matching the Go reference
//! (`internal/effect`).
//!
//! Two invariants, stated by the source crate's README and preserved here:
//!
//! - **Unwind order is the reverse of registration order.** A later effect may
//!   depend on an earlier one, so undoing them in registration order would tear
//!   down the ground under something still standing.
//! - **A scope unwinds on drop, including on panic.** An effect that only
//!   reverses on the happy path is not reversible.
//!
//! # Panics during unwind
//!
//! An inverse may itself panic. [`Scope::close`] catches each one, **runs the
//! whole reverse loop anyway**, and resumes the panic afterwards, so one bad
//! inverse can no longer abandon the ones registered before it. Concretely:
//!
//! - **Every live inverse runs**, in reverse registration order, however many
//!   of them panic.
//! - **One payload is resumed**: the first inverse *reached* that panicked,
//!   which in a reverse unwind is the later-registered of those that failed.
//!   It is handed to [`std::panic::resume_unwind`] once the loop has finished,
//!   so it escapes `close()` exactly as it did before, and the panic hook is
//!   not run a second time.
//! - **The other payloads are named, not lost.** Every panicking inverse's
//!   label is recorded and read back through [`Scope::failed`], in
//!   registration order — so the resumed payload is `failed()`'s **last**
//!   entry. The payloads themselves are dropped, but each of their messages,
//!   with file and line, was already written to stderr by the panic hook
//!   before `close()` ever saw the `Err`: what is dropped is the ability to
//!   `downcast` the 2nd..Nth payload, not the report of them.
//! - **[`Scope::held`] stays true throughout.** The live set is no longer
//!   detached before the loop, so an inverse that asks what is still
//!   outstanding is told the truth at that instant rather than `[]`.
//!
//! Three limits are **documented here rather than fixed**:
//!
//! 1. **A panicking inverse while a panic is already in flight aborts the
//!    process** (SIGABRT). That is Rust's rule for a panic in a destructor
//!    during cleanup, not a `Scope` bug: the runtime checks the thread's panic
//!    count at the *panic site* and calls `abort` before any unwinding begins,
//!    so the `catch_unwind` in `close()` is never reached and cannot change
//!    it. Pinned out of process by
//!    `shared/tests/load_unwind_panic.rs::double_panic_aborts_the_process`,
//!    which asserts on `signal() == Some(6)` and on the runtime's
//!    `panic in a destructor during cleanup` line on the child's stderr.
//! 2. **Nested scopes have a depth limit.** `Drop -> close -> undo -> Drop ->
//!    …` recurses one stack frame chain per nesting level. Depth 1_000 is
//!    proven safe by
//!    `shared/tests/load_scope.rs::deep_nesting_unwinds_at_a_safe_depth`;
//!    depth 10_000 overflows the stack and aborts. A stack overflow cannot be
//!    caught, so nothing here helps.
//! 3. **Under `panic = "abort"` there is no unwinding at all**, so the first
//!    panicking inverse ends the process and no later inverse runs. Neither
//!    tree sets that profile today, but every guarantee above is conditional
//!    on unwinding being enabled.
//!
//! ## Why `AssertUnwindSafe` here needs no `unsafe`
//!
//! [`std::panic::catch_unwind`] requires `F: UnwindSafe`, and [`Undo`] —
//! `Box<dyn FnOnce() + Send>` — carries no such bound; `dyn FnOnce` is not
//! `UnwindSafe` in general. The call is therefore
//! `catch_unwind(AssertUnwindSafe(undo))`. `AssertUnwindSafe<F>` implements
//! `FnOnce()` when `F: FnOnce()`, so the inverse is passed straight in and no
//! wrapper closure is needed. `AssertUnwindSafe` is a safe newtype and
//! `UnwindSafe` is a lint-shaped marker about logical state rather than a
//! memory-safety bound, so this compiles under the crate's
//! `#![forbid(unsafe_code)]` unchanged.
//!
//! Asserting it is sound here for two reasons:
//!
//! - The closure is **owned by the scope and consumed once**. It is taken out
//!   of its entry, moved into the catch, and dropped there; nothing — not
//!   `Scope`, not [`Disposer`], not any caller — can observe or invoke it
//!   again afterwards, so there is no broken invariant left to be read after a
//!   caught panic. That is the entire hazard `UnwindSafe` exists to flag.
//! - The only state `Scope` itself carries across the boundary is its `Inner`,
//!   behind a `Mutex`, and **no lock is ever held across an inverse**. The
//!   scope's invariant after a caught panic is exactly "this entry's inverse
//!   ran and panicked", which is what `failed()` then records: nothing is left
//!   half-updated, and the mutex cannot be poisoned by an inverse — which
//!   matters now that `close()` re-acquires it after a caught panic.
//!
//! # Provenance
//!
//! Moved, not rewritten, from mitosys: `src/mitosys/util/effect/effect.rs`,
//! crate `mitosys-util-effect`, at commit
//! `c96fdb9134eff9a7a575f28fd3bf9d358e880e81` (the file itself last changed in
//! `c667b779b3e5d87cad302864d5799cb02f09ae4f`). From the
//! `use std::collections::HashMap;` line below to the end of this file the
//! copy is byte-for-byte identical to that source; only this doc comment
//! differs. **That sentence held exactly as written from the p1 port at
//! `5313ca4` until p6-scope-unwind, and is kept rather than deleted because it
//! is what the port was**; deviation 8 below is the first and so far only
//! change to that body, and every line beneath this comment that item 8 does
//! not name is still the source's. There, the module sat under `lib.rs`'s
//! `Layer: L0 · May import: nothing`, having been split out of
//! `mitosys::effect` in `src/core/src/` — the one file there that referenced
//! nothing else in the crate.
//!
//! Eight deviations, and no others:
//!
//! 1. **Module `effect` → `scope`.** `effect` is the name mitosys's plugin
//!    contract gave it; `shared` has no plugin contract. The ticket's
//!    acceptance line and `learnings/shared-crate.md` §3 both say `scope`.
//! 2. **`effect/effect.rs` → `shared/src/scope.rs`.** mitosys's convention
//!    is "a crate is its directory" (`[lib] path = "lib.rs"`, no `src/`);
//!    `shared` resolved that divergence dimension in favour of the cargo
//!    default (`AGENTS.md` §divergences, p0-foundation).
//! 3. **Test import path.** `shared/tests/scope.rs`'s import line reads
//!    `use shared::scope::*;` where the source read
//!    `use mitosys_util_effect::effect::*;`. Forced by 1 and 2. Nothing else
//!    in the test file changed — not a name, not an assertion, not a blank
//!    line. (It is the file's first line in the source and its second here,
//!    because of deviation 7.)
//! 4. **Doc-comment framing.** The source doc opened "the plugin contract's
//!    foundation" and `lib.rs` carried the layer line and the `src/core/src/`
//!    split history. `shared` is domain-free, so that framing moved *into*
//!    this Provenance block rather than being deleted or left standing as if
//!    `shared` had plugins — deleted provenance reads as provenance that
//!    never existed. Kept above, because they are semantics and not domain:
//!    the Cordis / Go `internal/effect` attribution and the two README
//!    invariants.
//! 5. **`Disposer`, not `Handle`.** `learnings/shared-crate.md` §3 and the
//!    source crate's README ABI block both write `Handle`; the actual type in
//!    the source is `Disposer`, and the crate boundary does not force a
//!    rename. The source wins — the learning's wording is loose. This is not a
//!    spec violation.
//! 6. **No compatibility shim, no crate README copied.** `mitosys::effect` is
//!    a `pub use` shim for call sites inside mitosys; `shared` has no call
//!    sites to keep resolving, so there is nothing to shim. mitosys keeps its
//!    own copy untouched; reconciling the two is `p5-adoption`. The source
//!    crate's README is about a mitosys crate and does not move — its two
//!    invariants survive above, per deviation 4.
//! 7. **`shared/tests/scope.rs` is wrapped in `mod scope { … }`.** Added
//!    after the port, at the board's request, and a wrapper rather than a
//!    rewrite: strip the first line, the last line and one leading tab from
//!    every line and the file is again byte-for-byte the source's from its
//!    line 2. The reason is that this ticket's gate is
//!    `cargo test -p shared scope`, and cargo's filter matches **test
//!    function names**, not file or target names. Unwrapped, four of the five
//!    ported tests do not contain the substring `scope` and the gate reported
//!    `1 passed; 4 filtered out` while exiting 0 — a gate that can pass having
//!    run almost nothing. Wrapped, the tests report as `scope::close_unwinds_lifo`
//!    and friends, and the filter selects all five. p3-clock and p4-stats
//!    adopted the same convention independently.
//! 8. **The unwind no longer abandons the rest when one inverse panics.** The
//!    first semantic deviation from the source; 1-7 are naming, pathing and
//!    test-wrapping. mitosys's `util/effect` keeps the old behaviour until it
//!    adopts — reconciling the two is `.mi/prds/p5-adoption/mitosys`. What
//!    changed: `close()` catches each inverse, runs the whole reverse loop, and
//!    resumes the first panic reached afterwards; `order` and `live` are no
//!    longer detached from `Inner` before the loop, so `held()` is true during
//!    the unwind and not only before it; `Inner` gains `failed` and `Scope`
//!    gains `failed()`. Argued in `.mi/prds/p6-scope-unwind/`, measured in
//!    `.mi/prds/p5-adoption/load-proof/finding.md`.
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// The inverse of one effect.
pub type Undo = Box<dyn FnOnce() + Send>;

/// Registering on a scope that has already begun unwinding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Closed {
	pub label: String,
}

impl fmt::Display for Closed {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "effect {:?}: scope is closed", self.label)
	}
}

impl std::error::Error for Closed {}

struct Entry {
	label: String,
	undo: Option<Undo>,
}

#[derive(Default)]
struct Inner {
	closed: bool,
	seq: u64,
	order: Vec<u64>,
	live: HashMap<u64, Entry>,
	/// Labels of the inverses that panicked during close, in the order they
	/// were reached — which is *reverse* registration order, because that is
	/// the order the loop pops them in. [`Scope::failed`] reverses on read so
	/// that callers see registration order, the same as [`Scope::held`], and
	/// so that appending stays O(1) at every failure.
	failed: Vec<String>,
}

/// Owns a set of effects and unwinds them on close: LIFO, idempotent, and
/// reentrancy-safe — the scope is marked closed before any inverse runs, and
/// **no lock is ever held across an inverse**, so an inverse that disposes
/// other effects, or asks what is still held, sees a scope already closed and
/// a consistent live set rather than a list being iterated under it.
#[derive(Default)]
pub struct Scope {
	inner: Arc<Mutex<Inner>>,
}

/// Handle to one registered effect. Disposing runs the inverse once, at most,
/// whether called here or by the scope's close — whichever comes first.
pub struct Disposer {
	inner: Arc<Mutex<Inner>>,
	id: u64,
}

impl Disposer {
	/// Runs the inverse once, at most. A panic in it propagates straight to
	/// this caller and is not silently lost, so it is **not** recorded in
	/// [`Scope::failed`], which means exactly "inverses that panicked during
	/// this scope's close".
	pub fn dispose(self) {
		let undo = {
			let mut g = self.inner.lock().unwrap();
			g.live.remove(&self.id).and_then(|e| e.undo)
		};
		if let Some(undo) = undo {
			undo();
		}
	}
}

impl Scope {
	pub fn new() -> Self {
		Scope::default()
	}

	/// Runs `register` immediately and tracks the inverse it returns under
	/// `label`. `register` runs outside the scope's lock, so an effect may
	/// register nested effects. If the scope closes while `register` is
	/// running, the fresh inverse is run immediately and [`Closed`] is
	/// returned: a half-installed effect must not survive its scope.
	pub fn effect(
		&self,
		label: impl Into<String>,
		register: impl FnOnce() -> Undo,
	) -> Result<Disposer, Closed> {
		let label = label.into();
		if self.inner.lock().unwrap().closed {
			return Err(Closed { label });
		}

		let undo = register();

		let mut g = self.inner.lock().unwrap();
		if g.closed {
			drop(g);
			undo();
			return Err(Closed { label });
		}
		let id = g.seq;
		g.seq += 1;
		g.order.push(id);
		g.live.insert(
			id,
			Entry {
				label,
				undo: Some(undo),
			},
		);
		drop(g);

		Ok(Disposer {
			inner: Arc::clone(&self.inner),
			id,
		})
	}

	/// Unwinds every live effect in reverse registration order. Idempotent;
	/// a second close returns immediately.
	///
	/// **Every inverse runs, even when one panics.** Each is caught, the loop
	/// continues, and the payload of the first inverse *reached* that panicked
	/// is resumed once the loop is done; the labels of all of them are readable
	/// through [`Scope::failed`]. See the module's `# Panics during unwind`,
	/// including the abort-during-abort case that no catch can change.
	///
	/// The live set is **not** detached first. Each iteration pops one id from
	/// `order` and removes its entry from `live` under the same lock, then
	/// releases the lock and runs the inverse. Three things follow, and all
	/// three are load-bearing:
	///
	/// - [`Scope::held`] is true at every instant of the unwind, not only
	///   before it, because it reads the same `order`/`live` the loop is
	///   draining.
	/// - Removing before running is what makes exactly-once fall out: an
	///   inverse that disposes *itself* finds nothing left to run. It also
	///   means a running inverse never sees its own label in `held()`, which is
	///   correct — it is running, not owed.
	/// - The loop terminates because `order` can only shrink: `closed` is set
	///   before it starts, so [`Scope::effect`] returns [`Closed`] and nothing
	///   can push while it runs.
	///
	/// One consequence worth naming: an inverse that disposes a still-pending
	/// effect now runs that effect's inverse **immediately**, rather than the
	/// dispose being a silent no-op as it was when `live` had been taken out
	/// from under it. Still exactly once, either way.
	pub fn close(&self) {
		{
			let mut g = self.inner.lock().unwrap();
			if g.closed {
				return;
			}
			g.closed = true;
		}

		let mut resumed: Option<Box<dyn std::any::Any + Send>> = None;
		loop {
			// The lock is taken to pop one entry and released before the
			// inverse runs, never held across it.
			let (label, undo) = {
				let mut g = self.inner.lock().unwrap();
				let Some(id) = g.order.pop() else {
					break;
				};
				match g.live.remove(&id) {
					Some(Entry {
						label,
						undo: Some(undo),
					}) => (label, undo),
					_ => continue,
				}
			};
			if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(undo)) {
				self.inner.lock().unwrap().failed.push(label);
				if resumed.is_none() {
					resumed = Some(payload);
				}
			}
		}

		if let Some(payload) = resumed {
			std::panic::resume_unwind(payload);
		}
	}

	/// Names the effects currently live, in registration order — what a close
	/// would unwind now. A snapshot, for introspection.
	///
	/// Honest during a close as well as outside one: an effect leaves this list
	/// exactly when its inverse is about to run, so from inside an inverse this
	/// reports precisely what is still owed. An inverse that panicked is not
	/// held — its `FnOnce` was consumed and nothing can ever run it again — it
	/// moves to [`Scope::failed`].
	pub fn held(&self) -> Vec<String> {
		let g = self.inner.lock().unwrap();
		g.order
			.iter()
			.filter_map(|id| g.live.get(id).map(|e| e.label.clone()))
			.collect()
	}

	/// Names the inverses that panicked during this scope's close, in
	/// registration order — the counterpart to [`Scope::held`], which a reader
	/// diffs against it. A snapshot, for introspection; the list is cloned, not
	/// taken, so it survives being read.
	///
	/// The last entry is the one whose payload `close()` resumed, because the
	/// first inverse reached in a reverse unwind is the last-registered of
	/// those that failed. [`Disposer::dispose`] never writes here: a panic
	/// there propagates to its own caller and is not lost.
	pub fn failed(&self) -> Vec<String> {
		let g = self.inner.lock().unwrap();
		g.failed.iter().rev().cloned().collect()
	}
}

impl Drop for Scope {
	/// A dropped scope unwinds: a plugin that forgets to close still cannot
	/// leak, which is the property the whole contract exists for.
	fn drop(&mut self) {
		self.close();
	}
}
