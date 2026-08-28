mod load_scope {
	//! `Scope` unwind-under-panic at scale.
	//!
	//! p1's ported `drop_unwinds` (in `shared/tests/scope.rs`) drops a scope
	//! holding **one** effect on the happy path. That leaves the load half of
	//! the crate's second invariant — *"a scope unwinds on drop, including on
	//! panic"* — untested: N effects, panicked out of, all N inverses run in
	//! exact reverse registration order.
	//!
	//! # What was measured before this file was written
	//!
	//! Apple M5, `rustc 1.94.0`, against `shared/src/scope.rs` at `main`.
	//! `close()` sets `closed = true` and `mem::take`s both `order` and `live`
	//! *before* running a single inverse, so the unwind is a flat reverse
	//! iteration over a `Vec<u64>` with `HashMap` removals — a loop, not
	//! recursion. Cost per effect stays flat from 10^4 to 4*10^5:
	//!
	//! | N | register (dev) | unwind (dev) | register (rel) | unwind (rel) |
	//! |---:|---:|---:|---:|---:|
	//! | 10_000  | 5.8 ms | 2.9 ms  | 0.52 ms | 0.29 ms |
	//! | 100_000 | 53.9 ms | 30.7 ms | 4.14 ms | 3.29 ms |
	//! | 400_000 | 261 ms | 147 ms  | 23.4 ms | 22.7 ms |
	//!
	//! # Not ignored, not feature-gated
	//!
	//! The whole 100_000 panic case is ~91 ms in dev. A proof that has to be
	//! opted into is a proof that will stop running, so `N` is one plain
	//! constant with one value that does not change with the profile.
	//!
	//! The panic-*during*-unwind cases are deliberately not mixed in here; they
	//! are worse than they look and live in `shared/tests/load_unwind_panic.rs`.

	use shared::scope::{Scope, Undo};
	use std::panic::catch_unwind;
	use std::sync::atomic::{AtomicUsize, Ordering};
	use std::sync::{Arc, Mutex};
	use std::time::{Duration, Instant};

	const N: usize = 100_000;

	/// Reports the first index at which `got` and `want` differ, with both
	/// values there. `assert_eq!` on two 100k-element vectors produces output
	/// no human reads; the divergence point is the whole diagnostic.
	fn first_divergence(got: &[usize], want: &[usize]) -> Option<String> {
		for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
			if g != w {
				return Some(format!("logs diverge at index {i}: got {g}, want {w}"));
			}
		}
		if got.len() != want.len() {
			return Some(format!(
				"logs agree on the common prefix but lengths differ: got {}, want {}",
				got.len(),
				want.len()
			));
		}
		None
	}

	/// Registers `count` effects on `s`, each inverse pushing its own index onto
	/// `log`. Labels are `e{i}` so `held()` can be checked against them.
	fn register_logging(s: &Scope, count: usize, log: &Arc<Mutex<Vec<usize>>>) {
		for i in 0..count {
			let log = Arc::clone(log);
			s.effect(format!("e{i}"), move || {
				Box::new(move || log.lock().unwrap().push(i)) as Undo
			})
			.expect("scope is open during registration");
		}
	}

	/// The untested half of the second invariant: the unwind here comes from
	/// `Drop` while a panic is in flight. This test never calls `Scope::close`
	/// — the scope is simply dropped as the stack unwinds past it. It
	/// complements `drop_unwinds` in `shared/tests/scope.rs`, which covers
	/// the same mechanism on the happy path with a single effect.
	#[test]
	fn panic_unwinds_all_n_in_reverse() {
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::with_capacity(N)));

		let inner = Arc::clone(&log);
		let outcome = catch_unwind(move || {
			let s = Scope::new();
			register_logging(&s, N, &inner);
			panic!("unwinding out of a scope holding {N} effects");
		});

		// The panic really happened: without this, a test that never panicked
		// would pass by having nothing to unwind.
		assert!(outcome.is_err(), "the closure was expected to panic");

		let got = log.lock().unwrap().clone();
		let want: Vec<usize> = (0..N).rev().collect();
		assert_eq!(got.len(), N, "expected all {N} inverses to have run");
		if let Some(diff) = first_divergence(&got, &want) {
			panic!("unwind order is not exactly (0..{N}).rev(): {diff}");
		}
	}

	/// Times the unwind at two sizes 8x apart. Linear is 8x; measured 7.8x in
	/// dev and ~11x in release on an idle box; quadratic at this N would be
	/// 64x. The bound is 25x — above the measured ratio and far below
	/// quadratic.
	///
	/// The ratio observed *in situ*, with the four sibling tests in this file
	/// running in parallel on the same cores, is wider than the idle figure:
	/// ten consecutive runs gave 8.9x-15.8x in dev and 5.2x-11.0x in release.
	/// That is why the bound is 25 and not 12; it is still nowhere near 64.
	///
	/// The absolute bound is the same argument from the other side: the 100_000
	/// unwind measured 31 ms in dev and 3.3 ms in release, so 5 s is ~150x
	/// headroom, while a quadratic unwind at this N runs for minutes. Neither
	/// bound can flake and neither can miss.
	#[test]
	fn panic_unwind_is_not_quadratic() {
		/// Registers `count` counting effects and times only the drop.
		fn time_drop(count: usize) -> Duration {
			let ran = Arc::new(AtomicUsize::new(0));
			let s = Scope::new();
			for i in 0..count {
				let ran = Arc::clone(&ran);
				s.effect(format!("e{i}"), move || {
					Box::new(move || {
						ran.fetch_add(1, Ordering::Relaxed);
					}) as Undo
				})
				.expect("scope is open during registration");
			}
			let start = Instant::now();
			drop(s);
			let elapsed = start.elapsed();
			assert_eq!(ran.load(Ordering::Relaxed), count, "not every inverse ran");
			elapsed
		}

		/// Best of three, for the same reason `load_throughput.rs` takes the
		/// best sample: contention only ever slows a sample down, so the
		/// fastest is the least-perturbed measurement of the shape being
		/// asserted. Taking the best of both sides keeps the ratio honest.
		fn best_drop(count: usize) -> Duration {
			(0..3).map(|_| time_drop(count)).min().expect("3 samples")
		}

		let small = N / 8;
		let t_small = best_drop(small);
		let t_big = best_drop(N);

		println!(
			"unwind: {small} -> {t_small:?}, {N} -> {t_big:?}, ratio {:.2}x",
			t_big.as_secs_f64() / t_small.as_secs_f64().max(f64::MIN_POSITIVE)
		);

		assert!(
			t_big < Duration::from_secs(5),
			"unwinding {N} effects took {t_big:?}, over the 5 s budget"
		);
		assert!(
			t_big < 25 * t_small,
			"unwinding {N} took {t_big:?} against {t_small:?} for {small}: \
			 more than 25x for an 8x input, which is the shape of a quadratic unwind"
		);
	}

	/// Nested scopes are the one shape where the unwind *does* recurse: each
	/// inner scope is owned by the outer scope's inverse, so dropping the
	/// outermost drives `Drop -> close -> undo -> Drop -> ...` down the stack.
	///
	/// Depth 1_000 is safe and is what this test asserts. **Depth 10_000 aborts
	/// the process** with `fatal runtime error: stack overflow` — measured. That
	/// is a real limit of `Scope`, and this test does not prove it absent; it
	/// proves only that the mechanism is correct at a depth the stack can hold.
	/// A stack overflow cannot be caught, so raising this constant would take
	/// the whole test binary down rather than fail a single test. Do not raise
	/// it.
	#[test]
	fn deep_nesting_unwinds_at_a_safe_depth() {
		const DEPTH: usize = 1_000;

		let ran = Arc::new(AtomicUsize::new(0));
		// Built inside out: scope k+1 is moved into scope k's inverse, so the
		// outermost scope built last owns the entire chain.
		let mut nested: Option<Scope> = None;
		for i in 0..DEPTH {
			let outer = Scope::new();
			let held = nested.take();
			let ran = Arc::clone(&ran);
			outer
				.effect(format!("nest{i}"), move || {
					Box::new(move || {
						drop(held);
						ran.fetch_add(1, Ordering::Relaxed);
					}) as Undo
				})
				.expect("scope is open during registration");
			nested = Some(outer);
		}

		drop(nested);
		assert_eq!(
			ran.load(Ordering::Relaxed),
			DEPTH,
			"every one of {DEPTH} nested scopes should have unwound"
		);
	}

	/// `held_reports_live_effects` in `shared/tests/scope.rs` covers this at
	/// the 2-effect scale. This proves `order` still tracks registration order
	/// at 100_000 — and does so on a scope that is about to be panicked out of,
	/// which is when the report matters.
	#[test]
	fn held_reports_all_n_before_the_panic() {
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::with_capacity(N)));
		// The snapshot is carried out of the closure rather than asserted
		// inside it: an assertion inside `catch_unwind` would be swallowed as
		// just another panic and the test would pass on a real failure.
		let snapshot: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

		let inner_log = Arc::clone(&log);
		let inner_snap = Arc::clone(&snapshot);
		let outcome = catch_unwind(move || {
			let s = Scope::new();
			register_logging(&s, N, &inner_log);
			*inner_snap.lock().unwrap() = s.held();
			panic!("panicking out with {N} effects held");
		});
		assert!(outcome.is_err(), "the closure was expected to panic");

		let held = snapshot.lock().unwrap().clone();
		assert_eq!(held.len(), N, "held() should report every live effect");
		assert_eq!(held[0], "e0", "held() should be in registration order");
		assert_eq!(held[N - 1], format!("e{}", N - 1));
	}

	/// The interaction between `Disposer::dispose` and a panicking unwind: an
	/// effect disposed before the panic must not be replayed by it, and every
	/// inverse must still run exactly once overall.
	#[test]
	fn disposed_effects_are_not_replayed_by_the_panic() {
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::with_capacity(N)));
		// How much of `log` was written by the dispose phase, so the unwind
		// phase can be read off the tail.
		let split: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

		let inner_log = Arc::clone(&log);
		let inner_split = Arc::clone(&split);
		let outcome = catch_unwind(move || {
			let s = Scope::new();
			let mut disposers = Vec::with_capacity(N);
			for i in 0..N {
				let log = Arc::clone(&inner_log);
				let d = s
					.effect(format!("e{i}"), move || {
						Box::new(move || log.lock().unwrap().push(i)) as Undo
					})
					.expect("scope is open during registration");
				disposers.push(Some(d));
			}
			for (i, d) in disposers.iter_mut().enumerate() {
				if i % 3 == 0 {
					d.take().expect("each disposer is taken once").dispose();
				}
			}
			*inner_split.lock().unwrap() = inner_log.lock().unwrap().len();
			panic!("panicking out after disposing every third effect");
		});
		assert!(outcome.is_err(), "the closure was expected to panic");

		let full = log.lock().unwrap().clone();
		let split = *split.lock().unwrap();

		// Every inverse ran exactly once in total, disposed or unwound.
		let mut sorted = full.clone();
		sorted.sort_unstable();
		let all: Vec<usize> = (0..N).collect();
		if let Some(diff) = first_divergence(&sorted, &all) {
			panic!("some inverse ran twice or not at all: {diff}");
		}

		// The dispose phase ran the multiples of three, in ascending order.
		let disposed_want: Vec<usize> = (0..N).filter(|i| i % 3 == 0).collect();
		if let Some(diff) = first_divergence(&full[..split], &disposed_want) {
			panic!("dispose phase did not run exactly the disposed effects: {diff}");
		}

		// The panic-unwind is exactly the undisposed indices, in reverse.
		let unwind_want: Vec<usize> = (0..N).filter(|i| i % 3 != 0).rev().collect();
		if let Some(diff) = first_divergence(&full[split..], &unwind_want) {
			panic!("panic unwind replayed or skipped an effect: {diff}");
		}
	}
}
