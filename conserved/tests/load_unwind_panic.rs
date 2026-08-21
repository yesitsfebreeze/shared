mod load_unwind_panic {
	//! The panic-**during**-unwind case: what `Scope` does when one of its own
	//! inverses panics. Both branches are pinned here by test rather than
	//! assumed, because neither was written down anywhere in the crate.
	//!
	//! 1. A panicking inverse during an ordinary `close()` **silently abandons
	//!    every inverse still to come**, and `held()` afterwards reports `[]`.
	//! 2. A panicking inverse while a panic is already in flight **aborts the
	//!    process** (SIGABRT). That is Rust's rule for a panic in a destructor
	//!    during cleanup, not a `Scope` bug, and no `catch_unwind` can
	//!    intercept it — which is why it is tested out of process.
	//!
	//! These are **characterisation** tests. They assert what the code does
	//! today, not what it ought to do. The full write-up, with transcripts, is
	//! `.mi/prds/p5-adoption/load-proof/finding.md`.
	//!
	//! Measured on Apple M5, `rustc 1.94.0`, against `conserved/src/scope.rs`
	//! at `main`.

	use conserved::scope::{Disposer, Scope, Undo};
	use std::panic::{catch_unwind, AssertUnwindSafe};
	use std::sync::{Arc, Mutex};

	/// Registers `count` effects on `s` whose inverses push their own index onto
	/// `log`, and which panic afterwards if their index is in `panics_at`.
	/// Returns the `Disposer` for every effect, in registration order.
	fn register(
		s: &Scope,
		count: usize,
		log: &Arc<Mutex<Vec<usize>>>,
		panics_at: &'static [usize],
	) -> Vec<Disposer> {
		(0..count)
			.map(|i| {
				let log = Arc::clone(log);
				s.effect(format!("e{i}"), move || {
					Box::new(move || {
						log.lock().unwrap().push(i);
						if panics_at.contains(&i) {
							panic!("inverse {i} panics");
						}
					}) as Undo
				})
				.expect("scope is open during registration")
			})
			.collect()
	}

	/// Extracts a panic payload as a string, for asserting on *which* panic
	/// escaped.
	fn payload_of(err: &Box<dyn std::any::Any + Send>) -> String {
		if let Some(s) = err.downcast_ref::<&str>() {
			(*s).to_string()
		} else if let Some(s) = err.downcast_ref::<String>() {
			s.clone()
		} else {
			String::from("<non-string panic payload>")
		}
	}

	// This test asserts CURRENT behaviour, deliberately, and it is not the
	// behaviour the crate advertises: `conserved/src/scope.rs`'s module doc
	// says in its second sentence that "Leaving something behind is not
	// expressible", and here three of five inverses are left behind with the
	// scope reporting nothing outstanding. It is not unsound — no UB, no double
	// free, no memory leak; only an effect leak — and it is inherited, since p1
	// ported `scope.rs` byte-for-byte from mitosys's `util/effect`, so mitosys
	// has behaved this way all along. Fixing it would be a semantic divergence
	// from that port and is a separate decision, not a load proof. This test
	// exists so that changing the behaviour in either direction goes red and
	// has to be argued for rather than slipped in. The full finding, with
	// transcripts, is `.mi/prds/p5-adoption/load-proof/finding.md`.
	#[test]
	fn panicking_inverse_abandons_the_rest() {
		let s = Scope::new();
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
		let mut disposers = register(&s, 5, &log, &[3]);
		let keep_zero = disposers.remove(0);

		// `AssertUnwindSafe` because the closure borrows `&s`, which must
		// outlive the catch so the post-panic state can be inspected.
		let outcome = catch_unwind(AssertUnwindSafe(|| s.close()));

		// 1. The inverse's panic escaped `close()`.
		assert!(
			outcome.is_err(),
			"the panicking inverse should escape close()"
		);
		// 2. Exactly two inverses ran: 4, then 3 — which panicked. Effects 2, 1
		//    and 0 never ran and never will.
		assert_eq!(*log.lock().unwrap(), vec![4, 3]);
		// 3. The scope under-reports: it says nothing is outstanding while
		//    three inverses are outstanding.
		assert!(
			s.held().is_empty(),
			"held() reports {:?}, not the empty vec this pins",
			s.held()
		);
		// 4. A second close runs nothing further — `closed` was set before the
		//    loop began.
		let before = log.lock().unwrap().len();
		s.close();
		assert_eq!(
			log.lock().unwrap().len(),
			before,
			"a second close ran something"
		);
		// 5. The retained `Disposer` for effect 0 is inert: `live` was taken
		//    out of `Inner` before the loop, so `dispose()` finds nothing.
		keep_zero.dispose();
		assert_eq!(
			log.lock().unwrap().len(),
			before,
			"disposing an abandoned effect ran its inverse"
		);
	}

	/// Makes the size of the hole visible: the abandoned tail is everything
	/// registered before the panicking inverse, so a panic halfway through the
	/// registration order loses half the scope.
	#[test]
	fn abandonment_scales_with_the_panic_position() {
		const COUNT: usize = 10_000;
		const PANICS_AT: usize = 5_000;

		let s = Scope::new();
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
		let _disposers = register(&s, COUNT, &log, &[PANICS_AT]);

		let outcome = catch_unwind(AssertUnwindSafe(|| s.close()));
		assert!(
			outcome.is_err(),
			"the panicking inverse should escape close()"
		);

		let ran = log.lock().unwrap().clone();
		let want: Vec<usize> = (PANICS_AT..COUNT).rev().collect();
		assert_eq!(ran.len(), COUNT - PANICS_AT, "exactly half should have run");
		assert_eq!(ran, want, "the half that ran should be 9_999 down to 5_000");
		// And exactly half did not: 0..5_000 are abandoned.
		assert_eq!(COUNT - ran.len(), PANICS_AT);
	}

	/// With two panicking inverses, the one that escapes is the first *reached*
	/// — which, in a reverse unwind, is the later-registered of the two. The
	/// earlier one is in the abandoned tail and never runs at all.
	#[test]
	fn first_panicking_inverse_wins() {
		let s = Scope::new();
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
		let _disposers = register(&s, 5, &log, &[1, 3]);

		let outcome = catch_unwind(AssertUnwindSafe(|| s.close()));
		let err = outcome.expect_err("a panicking inverse should escape close()");
		assert_eq!(
			payload_of(&err),
			"inverse 3 panics",
			"the escaping panic should be index 3's, the first reached in reverse"
		);

		let ran = log.lock().unwrap().clone();
		assert_eq!(ran, vec![4, 3]);
		assert!(!ran.contains(&1), "index 1's inverse should never have run");
	}

	/// The second branch: an inverse that panics while a panic is already in
	/// flight. `Drop::drop` -> `close()` -> `undo()` panics -> panic in a
	/// destructor during cleanup -> `abort`. `catch_unwind` cannot intercept an
	/// abort, so this is tested by re-executing the test binary as a child and
	/// asserting on how the child died.
	#[test]
	fn double_panic_aborts_the_process() {
		// Child branch. It diverges (panics, then aborts), so it can never
		// reach the `Command` below — there is no re-exec loop to bound.
		if std::env::var_os(CHILD_ENV).is_some() {
			let _s = {
				let s = Scope::new();
				s.effect("boom", || {
					Box::new(|| panic!("inverse panics during unwind")) as Undo
				})
				.expect("scope is open during registration");
				s
			};
			panic!("{OUTER_PANIC}");
		}

		let with_flag = run_child(&["--exact", "--nocapture"]);
		let stdout = String::from_utf8_lossy(&with_flag.stdout).into_owned();
		let stderr = String::from_utf8_lossy(&with_flag.stderr).into_owned();

		// Vacuity guard, and the important one: a child given a filter that
		// selects nothing prints `running 0 tests` and **exits 0**, and every
		// assertion below about "it did not succeed" would then be asserting on
		// a success the parent invented. `--exact` matches the *module-qualified*
		// test name, which is why the constant below carries `load_unwind_panic::`.
		assert!(
			stdout.contains("running 1 test"),
			"the child selected no test — the filter is wrong, so nothing was proved.\n\
			 stdout:\n{stdout}"
		);

		assert!(
			!with_flag.status.success(),
			"the child exited successfully; it was expected to abort.\nstdout:\n{stdout}"
		);

		#[cfg(unix)]
		{
			use std::os::unix::process::ExitStatusExt;
			// Not merely "non-zero": a compile error or a mis-filtered run is
			// also non-zero-or-zero and must not be mistaken for the abort.
			assert_eq!(
				with_flag.status.signal(),
				Some(6),
				"expected SIGABRT (6), got {:?}.\nstderr:\n{stderr}",
				with_flag.status
			);
		}

		assert!(
			stderr.contains("panic in a destructor during cleanup"),
			"the runtime's double-panic message is missing.\nstderr:\n{stderr}"
		);
		assert!(
			stderr.contains(OUTER_PANIC),
			"the child's own panic message is missing.\nstderr:\n{stderr}"
		);

		// `--nocapture` on the child is load-bearing, not cosmetic, and this
		// proves it: without the flag the harness buffers the test's own output
		// and the abort discards the buffer, so the outer panic's message never
		// reaches the parent — while the runtime's destructor line, written
		// directly rather than through the harness, still does.
		let without_flag = run_child(&["--exact"]);
		let quiet_stderr = String::from_utf8_lossy(&without_flag.stderr).into_owned();
		assert!(
			quiet_stderr.contains("panic in a destructor during cleanup"),
			"the runtime line should survive even without --nocapture.\nstderr:\n{quiet_stderr}"
		);
		assert!(
			!quiet_stderr.contains(OUTER_PANIC),
			"without --nocapture the child's own panic message should be lost with \
			 the discarded buffer; it was not, so the flag is no longer load-bearing \
			 and this test's reasoning needs revisiting.\nstderr:\n{quiet_stderr}"
		);
	}

	/// Guards the child branch of `double_panic_aborts_the_process`.
	const CHILD_ENV: &str = "CONSERVED_LOAD_DOUBLE_PANIC";
	/// The child's own panic message, asserted on from the parent.
	const OUTER_PANIC: &str = "outer panic in flight";
	/// **Module-qualified**, because `mod load_unwind_panic { … }` makes that
	/// the name the harness reports and `--exact` matches. Misspell it and the
	/// child selects zero tests and exits 0.
	const CHILD_FILTER: &str = "load_unwind_panic::double_panic_aborts_the_process";

	fn run_child(extra: &[&str]) -> std::process::Output {
		let exe = std::env::current_exe().expect("the test binary knows its own path");
		std::process::Command::new(exe)
			.arg(CHILD_FILTER)
			.args(extra)
			.env(CHILD_ENV, "1")
			.output()
			.expect("the child test binary runs")
	}
}
