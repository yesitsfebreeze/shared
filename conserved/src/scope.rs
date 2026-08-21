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
//! # Provenance
//!
//! Moved, not rewritten, from mitosys: `src/mitosys/util/effect/effect.rs`,
//! crate `mitosys-util-effect`, at commit
//! `c96fdb9134eff9a7a575f28fd3bf9d358e880e81` (the file itself last changed in
//! `c667b779b3e5d87cad302864d5799cb02f09ae4f`). From the
//! `use std::collections::HashMap;` line below to the end of this file the
//! copy is byte-for-byte identical to that source; only this doc comment
//! differs. There, the module sat under `lib.rs`'s
//! `Layer: L0 · May import: nothing`, having been split out of
//! `mitosys::effect` in `src/core/src/` — the one file there that referenced
//! nothing else in the crate.
//!
//! Six deviations, and no others:
//!
//! 1. **Module `effect` → `scope`.** `effect` is the name mitosys's plugin
//!    contract gave it; `conserved` has no plugin contract. The ticket's
//!    acceptance line and `learnings/shared-crate.md` §3 both say `scope`.
//! 2. **`effect/effect.rs` → `conserved/src/scope.rs`.** mitosys's convention
//!    is "a crate is its directory" (`[lib] path = "lib.rs"`, no `src/`);
//!    `conserved` resolved that divergence dimension in favour of the cargo
//!    default (`AGENTS.md` §divergences, p0-foundation).
//! 3. **Test import path.** `conserved/tests/scope.rs` line 1 reads
//!    `use conserved::scope::*;` where the source read
//!    `use mitosys_util_effect::effect::*;`. Forced by 1 and 2. Nothing else
//!    in the test file changed — not a name, not an assertion, not a blank
//!    line.
//! 4. **Doc-comment framing.** The source doc opened "the plugin contract's
//!    foundation" and `lib.rs` carried the layer line and the `src/core/src/`
//!    split history. `conserved` is domain-free, so that framing moved *into*
//!    this Provenance block rather than being deleted or left standing as if
//!    `conserved` had plugins — deleted provenance reads as provenance that
//!    never existed. Kept above, because they are semantics and not domain:
//!    the Cordis / Go `internal/effect` attribution and the two README
//!    invariants.
//! 5. **`Disposer`, not `Handle`.** `learnings/shared-crate.md` §3 and the
//!    source crate's README ABI block both write `Handle`; the actual type in
//!    the source is `Disposer`, and the crate boundary does not force a
//!    rename. The source wins — the learning's wording is loose. This is not a
//!    spec violation.
//! 6. **No compatibility shim, no crate README copied.** `mitosys::effect` is
//!    a `pub use` shim for call sites inside mitosys; `conserved` has no call
//!    sites to keep resolving, so there is nothing to shim. mitosys keeps its
//!    own copy untouched; reconciling the two is `p5-adoption`. The source
//!    crate's README is about a mitosys crate and does not move — its two
//!    invariants survive above, per deviation 4.
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
}

/// Owns a set of effects and unwinds them on close: LIFO, idempotent, and
/// reentrancy-safe — the live set is detached before any inverse runs, so an
/// inverse that disposes other effects sees a scope already closed rather
/// than a list being iterated under it.
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
	pub fn close(&self) {
		let (order, mut live) = {
			let mut g = self.inner.lock().unwrap();
			if g.closed {
				return;
			}
			g.closed = true;
			(std::mem::take(&mut g.order), std::mem::take(&mut g.live))
		};
		for id in order.into_iter().rev() {
			if let Some(undo) = live.remove(&id).and_then(|e| e.undo) {
				undo();
			}
		}
	}

	/// Names the effects currently live, in registration order — what a close
	/// would unwind now. A snapshot, for introspection.
	pub fn held(&self) -> Vec<String> {
		let g = self.inner.lock().unwrap();
		g.order
			.iter()
			.filter_map(|id| g.live.get(id).map(|e| e.label.clone()))
			.collect()
	}
}

impl Drop for Scope {
	/// A dropped scope unwinds: a plugin that forgets to close still cannot
	/// leak, which is the property the whole contract exists for.
	fn drop(&mut self) {
		self.close();
	}
}
