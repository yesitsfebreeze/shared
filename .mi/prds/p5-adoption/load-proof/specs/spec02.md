# goal

`Scope` unwind-under-panic at scale: a scope holding N effects panicked out
of, with **all N inverses asserted to have run, in exact reverse registration
order**, at N = 100_000 — plus the shape check that separates a linear unwind
from a quadratic one. This is the case p1's ported `drop_unwinds` does not
cover: that test drops a scope holding **one** effect on the happy path.

est: 1.0h

## What is actually true today, measured before writing this

Run against `conserved/src/scope.rs` at `main`, on Apple M5 / rustc 1.94.0.
The implementer should not have to rediscover any of this:

- **Panicking out of a scope works and the order is right.** `Scope::drop`
  calls `close()`, and `close()` first takes `closed = true` and
  `mem::take`s both `order` and `live` *before* running a single inverse, so
  the unwind is a plain reverse iteration over a `Vec<u64>` with `HashMap`
  removals. At N = 100_000, all 100_000 inverses ran and the log was exactly
  `(0..N).rev()`.
- **It is linear, not quadratic, and it does not overflow the stack.**
  `close()` is a loop, not recursion. Measured cost per effect stayed flat
  across N = 10^4 … 4·10^5:

  | N | register (dev) | unwind (dev) | register (rel) | unwind (rel) |
  |---:|---:|---:|---:|---:|
  | 10_000 | 5.8 ms | 2.9 ms | 0.52 ms | 0.29 ms |
  | 100_000 | 53.9 ms | 30.7 ms | 4.14 ms | 3.29 ms |
  | 400_000 | 261 ms | 147 ms | 23.4 ms | 22.7 ms |

  Whole panic-out-of-100_000 test: **91 ms dev, 7.7 ms release.**
- **The stack overflow is real, but it is in a shape this spec does not
  test.** 100_000 *flat* effects on one scope are safe. 10_000 *nested*
  scopes — each scope held alive by an outer scope's inverse — recurse through
  `Drop` and **abort the process with `fatal runtime error: stack overflow`**
  (measured: depth 1_000 fine, depth 10_000 aborts). A stack-overflow test
  cannot be caught, so it would take the whole test binary down with it. This
  spec therefore tests nesting at a **safe** depth and records the limit in
  prose rather than asserting on the crash. Do not raise that depth.
- The panic-*during*-unwind cases are spec03's; they are worse than they look
  and are deliberately not mixed in here.

## What this assumes

- `conserved::scope::{Scope, Disposer, Undo, Closed}` as landed by p1.
- `conserved/tests/` test root, hard tabs width 2, `#![forbid(unsafe_code)]`.
- `conserved::Instant` is re-exported at the crate root by p3 — import
  `Scope` by name and spell `std::time::Instant`, never `use conserved::*`.

## Files

- `conserved/tests/load_scope.rs` — **new**. The only file this spec writes.

Touches nothing else, and in particular does not touch `conserved/src/` —
this spec proves the landed implementation, it does not change it.

## Ignore / feature decision

**Neither**, same as spec01 and for the same reason: measured 91 ms in dev for
the whole 100_000 case. N is a `const N: usize = 100_000;` at the top of the
module, plain, with no feature gate and no `#[ignore]`. A proof that has to be
opted into is a proof that will stop running.

## Shape

```rust
mod load_scope {
	use conserved::scope::Scope;
	use std::sync::{Arc, Mutex};
	use std::time::Instant;

	const N: usize = 100_000;
	// ...
}
```

`catch_unwind` over a closure that moves an `Arc<Mutex<Vec<usize>>>` is
`UnwindSafe` without help; where the closure borrows a `&Scope`, wrap it in
`std::panic::AssertUnwindSafe` — that is correct in a test and does not need a
comment defending it beyond one line. The panic message goes to the harness's
captured stderr, so no `set_hook` juggling is needed and none should be added:
a global hook set by one test corrupts the others in the same binary.

## Acceptance

- [x] `conserved/tests/load_scope.rs` exists and its **entire** contents are
      wrapped in `mod load_scope { … }`, so `cargo test -p conserved load`
      selects every test in it. Confirmed by running that filter and reading a
      non-zero `N passed`.
- [x] `panic_unwinds_all_n_in_reverse` — registers `N = 100_000` effects on one
      `Scope` inside `std::panic::catch_unwind`, each inverse pushing its own
      index onto a shared `Vec<usize>`, then panics. Asserts, in this order:
      the closure returned `Err` (the panic really happened, the test did not
      pass by never panicking); the log length is exactly `N`; and the log is
      **element-for-element equal** to `(0..N).rev().collect::<Vec<_>>()`.
      Comparing lengths alone is not enough and comparing only the first and
      last element is not enough — the whole vector, one `assert_eq!`.
- [x] That test's failure output does not dump 100_000 elements: on mismatch
      it reports the first index at which the log and the expected sequence
      diverge, plus both values there. (`assert_eq!` on two 100k vectors is
      unreadable; find the divergence, then assert.)
- [x] `panic_unwinds_all_n_in_reverse` never calls `Scope::close` — the unwind
      must come from `Drop` while the panic is in flight, which is the
      untested half of p1's second invariant. A comment says so and names
      `drop_unwinds` in `conserved/tests/scope.rs` as the happy-path test this
      one complements.
- [x] `panic_unwind_is_not_quadratic` — times the unwind of `N / 8 = 12_500`
      effects and of `N = 100_000` effects (drop, not `close()`), and asserts
      **both**:
      - the 100_000 unwind completes in under **5 s** wall clock (measured
        31 ms dev / 3.3 ms release, so ~150× headroom; a quadratic unwind at
        this N is minutes, so this bound cannot flake and cannot miss);
      - `t(100_000) < 25 * t(12_500)` — linear is 8×, measured 7.8× dev and
        ~11× release, quadratic is 64×. A comment states those four numbers so
        the bound is not mistaken for a guess.
- [x] `deep_nesting_unwinds_at_a_safe_depth` — 1_000 nested scopes, each held
      by the previous scope's inverse, dropped from the outside; asserts all
      1_000 ran. A comment records, in plain words, that the same shape
      **aborts the process with a stack overflow at depth 10_000** because the
      nested unwind recurses through `Drop`, that this is a real limit of
      `Scope` and not a thing this test proves absent, and that raising the
      constant will take the whole test binary down rather than fail a test.
- [x] `held_reports_all_n_before_the_panic` — after registering `N` effects,
      `s.held().len() == N` and `s.held()[0]`/`s.held()[N - 1]` are the first
      and last labels registered, proving `order` tracks registration order at
      this scale and not just at the 2-effect scale `held_reports_live_effects`
      covers.
- [x] `disposed_effects_are_not_replayed_by_the_panic` — registers `N`
      effects, disposes every third `Disposer` before the panic, then panics;
      asserts each inverse ran exactly once in total and that the panic-unwind
      log is exactly the undisposed indices in reverse. This is the
      interaction between `Disposer::dispose` and a panicking unwind, which no
      existing test reaches.
- [x] **Every one of these tests can fail.** For at least
      `panic_unwinds_all_n_in_reverse` and `panic_unwind_is_not_quadratic`,
      demonstrate it: reverse the expected order / set the wall-clock bound to
      1 ms, confirm a non-zero exit, put it back, and say in the
      implementation commit message that it was done.
- [x] No `#[ignore]`, no `cfg(feature = ...)`, no `N` that changes with the
      profile. One `N`, one value, always run.
- [x] `conserved/Cargo.toml` is byte-identical after this spec — no new
      dev-dependency; `std::time::Instant` and `std::panic::catch_unwind` are
      the whole harness.
- [x] Total added wall time for this file is under 3 s in dev on the
      implementer's machine; if it is not, the unwind is not linear and that
      is a finding, not a reason to lower `N`.
- [x] `cargo clippy -p conserved --all-targets -- -D warnings` is clean.

verify: `cargo test -p conserved --test load_scope && cargo test -p conserved --release --test load_scope && cargo test -p conserved load_scope 2>&1 | grep -qE '[1-9][0-9]* passed' && cargo clippy -p conserved --all-targets -- -D warnings && cargo fmt --all --check`

## Implemented 2026-08-21 — measured on the implementer's box

`conserved/tests/load_scope.rs`, five tests, no manifest change, no `#[ignore]`,
no feature gate, `const N: usize = 100_000` with one value in both profiles.

```
$ cargo test -p conserved --test load_scope
running 5 tests
test load_scope::deep_nesting_unwinds_at_a_safe_depth ... ok
test load_scope::panic_unwinds_all_n_in_reverse ... ok
test load_scope::held_reports_all_n_before_the_panic ... ok
test load_scope::disposed_effects_are_not_replayed_by_the_panic ... ok
test load_scope::panic_unwind_is_not_quadratic ... ok
test result: ok. 5 passed; 0 failed; ... finished in 0.37s
```

**0.37 s dev** for the whole file, against the box's 3 s budget. Release: 0.05 s.
Under the ticket filter `cargo test -p conserved load` the file reports
`5 passed`, in both profiles.

### The anti-quadratic numbers, in situ

The spec's idle figure was 7.8x dev. Measured *while the other four tests in the
file run in parallel on the same cores* — which is how the check will actually
run — ten consecutive runs gave:

```
dev:     ratio 15.84x, 10.56x, 10.36x, 8.93x, 10.01x, 8.96x
release: ratio 10.18x, 10.04x,  5.17x, 7.83x, 11.03x
t(12_500) 3.5-4.9 ms dev / 0.40-0.84 ms release
t(100_000) 33-60 ms dev / 4.1-4.7 ms release
```

The 25x bound holds with 1.6x margin at the worst observation, and quadratic is
64x, so the check still cannot miss. Both sides are best-of-3 (same argument as
`load_throughput.rs`: contention only slows a sample). The measured range is
recorded in the test's own doc comment so the bound is not mistaken for a guess.

**The gate can fail — both demonstrated, not asserted.**

1. Expected order reversed, `(0..N).rev()` -> `(0..N)`:

```
$ cargo test -p conserved --test load_scope panic_unwinds ; echo $?
unwind order is not exactly (0..100000).rev(): logs diverge at index 0: got 99999, want 0
test result: FAILED. 0 passed; 1 failed; ...
101
```

2. Wall-clock budget `5 s` -> `1 ms`:

```
$ cargo test -p conserved --test load_scope not_quadratic ; echo $?
unwinding 100000 effects took 27.215ms, over the budget
test result: FAILED. 0 passed; 1 failed; ...
101
```

Both reverted. Note that the failure output names the divergence index rather
than dumping 100_000 elements, which is the box above it.

`deep_nesting_unwinds_at_a_safe_depth` stays at `DEPTH = 1_000`. The depth-10_000
abort is recorded in prose in the test's doc comment and was **not** re-run: a
stack overflow takes the whole binary down and cannot be caught, so re-deriving
it costs a crashed test run and proves nothing new.
