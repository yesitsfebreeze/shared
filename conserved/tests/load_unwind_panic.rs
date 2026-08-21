mod load_unwind_panic {
	//! The panic-**during**-unwind case: what `Scope` does when one of its own
	//! inverses panics. Both branches are pinned here by test rather than
	//! assumed, because neither was written down anywhere in the crate.
	//!
	//! 1. A panicking inverse during an ordinary `close()` **does not abandon
	//!    the inverses still to come**: every one of them runs, the payload of
	//!    the first inverse *reached* that panicked is resumed afterwards, the
	//!    labels of all of them are readable through `failed()`, and `held()`
	//!    tells the truth at every instant of the unwind rather than reporting
	//!    `[]` from the first inverse onward.
	//! 2. A panicking inverse while a panic is already in flight **aborts the
	//!    process** (SIGABRT). That is Rust's rule for a panic in a destructor
	//!    during cleanup, not a `Scope` bug, and no `catch_unwind` can
	//!    intercept it — which is why it is tested out of process.
	//!
	//! These **were** characterisation tests of a defect: branch 1 used to read
	//! "silently abandons every inverse still to come, and `held()` afterwards
	//! reports `[]`", and said so deliberately. p6-scope-unwind argued that
	//! behaviour and fixed it, so they are now the contract's proof rather than
	//! a record of what the code happened to do. Branch 2 is unchanged in every
	//! word, and so is the test that proves it. The full write-up of what was
	//! measured, with transcripts, is
	//! `.mi/prds/p5-adoption/load-proof/finding.md`, which stays as the record
	//! of the defect and must not be edited.
	//!
	//! Measured on Apple M5, `rustc 1.94.0`, against `conserved/src/scope.rs`
	//! at `main`.

	use conserved::scope::{Disposer, Scope, Undo};
	use std::panic::{catch_unwind, AssertUnwindSafe};
	use std::sync::{Arc, Mutex, Weak};

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

	// This test used to assert CURRENT behaviour, deliberately, and that
	// behaviour was not what the crate advertised: `conserved/src/scope.rs`'s
	// module doc says in its second sentence that "Leaving something behind is
	// not expressible", and three of five inverses were left behind with the
	// scope reporting nothing outstanding. The old comment here said that
	// fixing it would be a semantic divergence from p1's byte-for-byte port of
	// mitosys's `util/effect`, that it was a separate decision, and that this
	// test existed so the change had to be argued for rather than slipped in.
	//
	// This is that argument's outcome. p6-scope-unwind made the case, and the
	// fix landed: deviation 8 in `scope.rs`'s `# Provenance`, the first
	// semantic one. So the assertions below now pin the contract instead of the
	// defect. The behaviour they used to pin is in `git log -p` on this file
	// and, measured, in `.mi/prds/p5-adoption/load-proof/finding.md` — which
	// stays as the record of what was fixed and must not be edited.
	#[test]
	fn panicking_inverse_does_not_abandon_the_rest() {
		let s = Scope::new();
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
		let mut disposers = register(&s, 5, &log, &[3]);
		let keep_zero = disposers.remove(0);

		// `AssertUnwindSafe` because the closure borrows `&s`, which must
		// outlive the catch so the post-panic state can be inspected.
		let outcome = catch_unwind(AssertUnwindSafe(|| s.close()));

		// 1. The inverse's panic escaped `close()`. Unchanged: the payload
		//    still propagates, it is just resumed after the loop rather than
		//    ending it.
		assert!(
			outcome.is_err(),
			"the panicking inverse should escape close()"
		);
		// 2. Every inverse ran, 4 down to 0, including the three that used to
		//    be abandoned behind the panic at 3.
		//    OLD: assert_eq!(*log.lock().unwrap(), vec![4, 3]);
		assert_eq!(*log.lock().unwrap(), vec![4, 3, 2, 1, 0]);
		// 3. `held()` is empty — and this line means the OPPOSITE of what it
		//    used to. OLD: it was empty because `close()` had detached `live`
		//    and `order` before the first inverse ran, so the scope reported
		//    nothing outstanding while three inverses were outstanding. NOW: it
		//    is empty because every inverse actually ran. The `failed()`
		//    assertion below is what distinguishes the two, and is why both
		//    halves are here: on its own the `held()` line reads identical in
		//    the diff while asserting the reverse.
		assert!(
			s.held().is_empty(),
			"held() reports {:?}, not the empty vec this pins",
			s.held()
		);
		// 3b. The one inverse that panicked is named, by label, in
		//     registration order. There was no `failed()` before this ticket.
		//     OLD: (nothing — the API did not exist)
		assert_eq!(
			s.failed(),
			vec!["e3".to_string()],
			"the panicking inverse should be named by failed()"
		);
		// 4. A second close runs nothing further — `closed` was set before the
		//    loop began. Unchanged, but for a different reason: nothing is left
		//    to run because everything already ran, not because it was dropped.
		let before = log.lock().unwrap().len();
		s.close();
		assert_eq!(
			log.lock().unwrap().len(),
			before,
			"a second close ran something"
		);
		// 5. The retained `Disposer` for effect 0 is inert. Assertion
		//    unchanged; the reason is not. OLD: "`live` was taken out of
		//    `Inner` before the loop, so `dispose()` finds nothing" — the
		//    inverse was still owed. NOW: effect 0's inverse already ran during
		//    the close, which removed it from `live`, so there is genuinely
		//    nothing left. `before` is 5 here, where it used to be 2.
		assert_eq!(before, 5, "all five inverses should have run by now");
		keep_zero.dispose();
		assert_eq!(
			log.lock().unwrap().len(),
			before,
			"disposing an abandoned effect ran its inverse"
		);
	}

	/// The panic's position no longer decides how much of the scope survives.
	/// This test used to make the size of the hole visible — a panic halfway
	/// through the registration order lost half the scope; now it proves the
	/// hole is gone at the same scale.
	#[test]
	fn unwind_completes_regardless_of_the_panic_position() {
		const COUNT: usize = 10_000;
		const PANICS_AT: usize = 5_000;

		let s = Scope::new();
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
		let _disposers = register(&s, COUNT, &log, &[PANICS_AT]);

		let outcome = catch_unwind(AssertUnwindSafe(|| s.close()));
		// Unchanged — the panic still escapes.
		assert!(
			outcome.is_err(),
			"the panicking inverse should escape close()"
		);

		let ran = log.lock().unwrap().clone();
		// OLD: let want: Vec<usize> = (PANICS_AT..COUNT).rev().collect();
		let want: Vec<usize> = (0..COUNT).rev().collect();
		// OLD: assert_eq!(ran.len(), COUNT - PANICS_AT, "exactly half should have run");
		assert_eq!(ran.len(), COUNT, "every inverse should have run");
		// OLD: assert_eq!(ran, want, "the half that ran should be 9_999 down to 5_000");
		assert_eq!(ran, want, "the unwind should be 9_999 down to 0");
		// OLD: assert_eq!(COUNT - ran.len(), PANICS_AT);  // "exactly half did not"
		assert_eq!(COUNT - ran.len(), 0, "nothing was abandoned");
		// OLD: (nothing — `failed()` did not exist, and `held()` was not asserted
		//       here because it lied.)
		assert_eq!(s.failed(), vec![format!("e{PANICS_AT}")]);
		assert!(s.held().is_empty(), "held() reports {:?}", s.held());
	}

	/// With two panicking inverses, the one that escapes is the first *reached*
	/// — which, in a reverse unwind, is the later-registered of the two. Both
	/// now run, both are named by `failed()` in registration order, and only
	/// the payload of the first reached is resumed; the earlier one used to sit
	/// in the abandoned tail and never run at all.
	#[test]
	fn first_panicking_inverse_wins() {
		let s = Scope::new();
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
		let _disposers = register(&s, 5, &log, &[1, 3]);

		let outcome = catch_unwind(AssertUnwindSafe(|| s.close()));
		let err = outcome.expect_err("a panicking inverse should escape close()");
		// Unchanged, and deliberately so: the payload that escapes is the same
		// one that escaped before the fix. That is the payload contract — the
		// other panics are reported by label through `failed()`, after the
		// panic hook has already printed each of their messages to stderr.
		assert_eq!(
			payload_of(&err),
			"inverse 3 panics",
			"the escaping panic should be index 3's, the first reached in reverse"
		);

		let ran = log.lock().unwrap().clone();
		// OLD: assert_eq!(ran, vec![4, 3]);
		assert_eq!(ran, vec![4, 3, 2, 1, 0]);
		// The inversion this ticket exists for.
		// OLD: assert!(!ran.contains(&1), "index 1's inverse should never have run");
		assert!(ran.contains(&1), "index 1's inverse must have run");
		// OLD: (nothing — the API did not exist.) Registration order, so the
		//      resumed payload belongs to the LAST entry: "inverse 3 panics".
		assert_eq!(s.failed(), vec!["e1".to_string(), "e3".to_string()]);
	}

	/// The case the old code could not even reach: *every* inverse panics.
	/// All five still run, the first reached is the one resumed, and all five
	/// are named. It also proves that catching a panic per inverse does not
	/// chain into an abort when no panic is already in flight — each catch
	/// completes before the next inverse's panic begins.
	#[test]
	fn every_inverse_panicking_still_runs_them_all() {
		let s = Scope::new();
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
		let _disposers = register(&s, 5, &log, &[0, 1, 2, 3, 4]);

		let outcome = catch_unwind(AssertUnwindSafe(|| s.close()));
		let err = outcome.expect_err("a panicking inverse should escape close()");
		assert_eq!(
			payload_of(&err),
			"inverse 4 panics",
			"the resumed payload should be the first inverse reached"
		);

		assert_eq!(*log.lock().unwrap(), vec![4, 3, 2, 1, 0]);
		assert_eq!(
			s.failed(),
			vec![
				"e0".to_string(),
				"e1".to_string(),
				"e2".to_string(),
				"e3".to_string(),
				"e4".to_string(),
			],
			"failed() is registration order, so the resumed payload is its last entry"
		);
		assert!(s.held().is_empty(), "held() reports {:?}", s.held());
	}

	/// `held()` is true at every *instant* of the unwind, not only at the end.
	/// Each inverse records what the scope reported at its own moment.
	///
	/// Two mechanical notes. An inverse cannot borrow the scope — `Undo` is
	/// `Box<dyn FnOnce() + Send>` and so implicitly `+ 'static` — so the scope
	/// goes in an `Arc` and each inverse captures a `Weak`. And the two
	/// recorders are separate `Vec`s rather than one `Vec<(_, _)>`, because
	/// `clippy::type_complexity` rejects the tuple form under `-D warnings`.
	#[test]
	fn held_is_honest_during_the_unwind() {
		type Snapshots = Arc<Mutex<Vec<Vec<String>>>>;

		let s = Arc::new(Scope::new());
		let held_at: Snapshots = Arc::new(Mutex::new(Vec::new()));
		let failed_at: Snapshots = Arc::new(Mutex::new(Vec::new()));

		for i in 0..3 {
			let w: Weak<Scope> = Arc::downgrade(&s);
			let held_at = Arc::clone(&held_at);
			let failed_at = Arc::clone(&failed_at);
			s.effect(format!("e{i}"), move || {
				Box::new(move || {
					let scope = w.upgrade().expect("the scope outlives its own close()");
					held_at.lock().unwrap().push(scope.held());
					failed_at.lock().unwrap().push(scope.failed());
					if i == 1 {
						panic!("inverse 1 panics");
					}
				}) as Undo
			})
			.expect("scope is open during registration");
		}

		let outcome = catch_unwind(AssertUnwindSafe(|| s.close()));
		assert!(
			outcome.is_err(),
			"the panicking inverse should escape close()"
		);

		// e2 sees e0 and e1 still owed; e1 sees only e0; e0 sees nothing owed
		// and e1 already named as failed. Before the fix every one of these
		// would have been `[]`, because `close()` detached `order` and `live`
		// before the first inverse ran — on every close, panic or not.
		assert_eq!(
			*held_at.lock().unwrap(),
			vec![
				vec!["e0".to_string(), "e1".to_string()],
				vec!["e0".to_string()],
				Vec::<String>::new(),
			]
		);
		assert_eq!(
			*failed_at.lock().unwrap(),
			vec![
				Vec::<String>::new(),
				Vec::<String>::new(),
				vec!["e1".to_string()],
			]
		);
		assert!(s.held().is_empty(), "held() reports {:?}", s.held());
		assert_eq!(s.failed(), vec!["e1".to_string()]);
	}

	/// The one behaviour that changed as a side effect of not detaching `live`,
	/// pinned so it is a decision on record rather than something a later
	/// reader trips over: an inverse that disposes a still-pending effect runs
	/// that effect's inverse **immediately**, rather than at its LIFO position.
	/// Before the fix the dispose was a silent no-op, because `live` had been
	/// taken out from under it. Exactly once, either way.
	#[test]
	fn dispose_during_unwind_runs_once() {
		let s = Scope::new();
		let log: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));

		let zero = {
			let log = Arc::clone(&log);
			s.effect("e0", move || {
				Box::new(move || log.lock().unwrap().push(0)) as Undo
			})
			.expect("scope is open during registration")
		};

		// The inverse is `'static`, so `e0`'s `Disposer` is handed in through a
		// shared cell rather than captured by reference.
		let parked: Arc<Mutex<Option<Disposer>>> = Arc::new(Mutex::new(Some(zero)));
		{
			let log = Arc::clone(&log);
			let parked = Arc::clone(&parked);
			s.effect("e1", move || {
				Box::new(move || {
					log.lock().unwrap().push(1);
					if let Some(d) = parked.lock().unwrap().take() {
						d.dispose();
					}
				}) as Undo
			})
			.expect("scope is open during registration");
		}

		s.close();

		assert_eq!(
			*log.lock().unwrap(),
			vec![1, 0],
			"e0's inverse should run exactly once, early, from inside e1's dispose"
		);
		assert!(s.held().is_empty(), "held() reports {:?}", s.held());
		assert!(s.failed().is_empty(), "nothing panicked");
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
