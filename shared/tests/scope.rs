mod scope {
	use shared::scope::*;
	use std::sync::atomic::{AtomicUsize, Ordering};
	use std::sync::{Arc, Mutex};

	#[test]
	fn close_unwinds_lifo() {
		let order = Arc::new(Mutex::new(Vec::new()));
		let s = Scope::new();
		for name in ["a", "b", "c"] {
			let order = Arc::clone(&order);
			s.effect(name, move || {
				Box::new(move || order.lock().unwrap().push(name))
			})
			.unwrap();
		}
		s.close();
		assert_eq!(*order.lock().unwrap(), vec!["c", "b", "a"]);
	}

	#[test]
	fn dispose_runs_once_and_deregisters() {
		let runs = Arc::new(AtomicUsize::new(0));
		let s = Scope::new();
		let d = {
			let runs = Arc::clone(&runs);
			s.effect("counted", move || {
				Box::new(move || {
					runs.fetch_add(1, Ordering::SeqCst);
				})
			})
			.unwrap()
		};
		d.dispose();
		s.close();
		assert_eq!(runs.load(Ordering::SeqCst), 1);
	}

	#[test]
	fn closed_scope_refuses_and_unwinds_late_effects() {
		let s = Scope::new();
		s.close();
		assert!(s.effect("late", || Box::new(|| {})).is_err());

		// The in-flight case: the scope closes between the registration
		// running and its inverse being tracked.
		let s2 = Scope::new();
		let undone = Arc::new(AtomicUsize::new(0));
		let res = {
			let undone = Arc::clone(&undone);
			let s2ref = &s2;
			s2.effect("racing", move || {
				s2ref.close();
				Box::new(move || {
					undone.fetch_add(1, Ordering::SeqCst);
				})
			})
		};
		assert!(res.is_err());
		assert_eq!(
			undone.load(Ordering::SeqCst),
			1,
			"a half-installed effect survived its scope"
		);
	}

	#[test]
	fn drop_unwinds() {
		let runs = Arc::new(AtomicUsize::new(0));
		{
			let s = Scope::new();
			let runs = Arc::clone(&runs);
			s.effect("held", move || {
				Box::new(move || {
					runs.fetch_add(1, Ordering::SeqCst);
				})
			})
			.unwrap();
		}
		assert_eq!(runs.load(Ordering::SeqCst), 1);
	}

	#[test]
	fn held_reports_live_effects() {
		let s = Scope::new();
		let da = s.effect("a", || Box::new(|| {})).unwrap();
		s.effect("b", || Box::new(|| {})).unwrap();
		da.dispose();
		assert_eq!(s.held(), vec!["b"]);
	}
}
