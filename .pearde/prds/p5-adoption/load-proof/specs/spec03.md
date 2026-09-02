# goal

The panic-**during**-unwind case, pinned by test rather than assumed: what
`Scope` actually does when one of its own inverses panics. Both branches —
a panicking inverse during an ordinary `close()`, which **silently abandons
every inverse still to come**, and a panicking inverse while a panic is
already in flight, which **aborts the process**. Neither is currently written
down anywhere in the crate.

est: 1.0h

## The finding — measured, not inferred

Run against `conserved/src/scope.rs` at `main` (Apple M5, rustc 1.94.0).
This is the part of this ticket worth more than the tests it produces.

**1. A panicking inverse abandons the rest of the unwind, silently.**

`Scope::close` takes the live set out of the mutex and then iterates:

```rust
for id in order.into_iter().rev() {
	if let Some(undo) = live.remove(&id).and_then(|e| e.undo) {
		undo();
	}
}
```

`live` is a local `HashMap<u64, Entry>`. When `undo()` panics, the loop is
abandoned and `live` is dropped during the unwind — and dropping a
`Box<dyn FnOnce()>` **does not call it**. Measured with five effects whose
inverse `3` panics:

- inverses that ran: `[4, 3]`. Effects `2`, `1`, `0` never ran and never will.
- `s.held()` afterwards returns `[]` — the scope reports nothing outstanding
  while three inverses are outstanding.
- a second `s.close()` is a no-op (`closed` was set before the loop began).
- the `Disposer` for an abandoned effect is inert: `live` was `mem::take`n out
  of `Inner`, so `dispose()` finds nothing and runs nothing.
- the scope's own `Drop`, running after the panic escapes `close()`, is safely
  a no-op — so the panic propagates normally and nothing aborts.

This directly contradicts the invariant `conserved/src/scope.rs`'s own module
doc states in its second sentence — *"Leaving something behind is not
expressible"* — and the one `learnings/shared-crate.md` cites as the reason
`Scope` was the first thing extracted. It is **not** unsound (no UB,
`#![forbid(unsafe_code)]` holds, no double free, no leak of memory — only of
effect), and it is inherited behaviour: p1 ported the file byte-for-byte from
mitosys `util/effect`, so mitosys has had it all along.

**2. A panicking inverse during an in-flight panic aborts the process.**

Scope dropped while unwinding a panic → `Drop::drop` → `close()` → an inverse
panics → panic in a destructor during cleanup → `abort`. Measured: child
process exits on **SIGABRT (6)** with
`panic in a destructor during cleanup … thread caused non-unwinding panic.
aborting.` This is Rust's rule, not a `Scope` bug, and no `catch_unwind` can
intercept it — which is exactly why it has to be tested out-of-process or not
at all.

## Fix or characterise — settled here, and why it is not a fork for the user

This spec **characterises**. It does not change `conserved/src/scope.rs`, and
the implementer must not:

- p1 landed `scope.rs` as a byte-for-byte port with exactly seven recorded
  deviations. Making `close()` run the remaining inverses (catching each
  inverse, resuming the first panic afterwards) would be an eighth, and a
  *semantic* one — the first divergence in behaviour from the mitosys tree
  that p5's `mitosys` child must later reconcile against. That is a change to
  the extracted contract, not a load proof.
- this ticket's mandate is to *exercise* the panic-during-unwind case; its
  frontmatter `verify` is a test command.
- `conserved/` is another agent's working tree.

A characterisation test is not a weaker check: it fails the moment the
behaviour changes in either direction — if someone fixes `close()` without
recording it, `panicking_inverse_abandons_the_rest` goes red and the fix has
to be argued for rather than slipped in. That is the correct gate for
behaviour nobody has yet decided to keep.

The decision *whether* to fix it belongs to a new ticket. This spec's last
acceptance box opens that ticket's evidence file; it does not open the ticket.

## Files

- `conserved/tests/load_unwind_panic.rs` — **new**.
- `.mi/prds/p5-adoption/load-proof/finding.md` — **new**, inside this ticket's
  own folder. The finding above, in prose, with the two measured transcripts,
  so it survives outside a test file's comments.

Touches nothing else. Not `conserved/src/`, not `conserved/Cargo.toml`, not
another ticket's folder, not `learnings/`.

## Ignore / feature decision

**Neither.** The in-process test is microseconds; the subprocess test spawns
one child that aborts immediately (~10 ms). Both run by default.

## The subprocess harness, in std

No dependency, no `#[ignore]`, no shell script. The test binary re-executes
itself:

```rust
#[test]
fn double_panic_aborts_the_process() {
	if std::env::var_os("CONSERVED_LOAD_DOUBLE_PANIC").is_some() {
		// child: drop a scope with a panicking inverse, mid-panic
		let _s = { /* scope holding one panicking inverse */ };
		panic!("outer panic in flight");
	}
	let out = std::process::Command::new(std::env::current_exe().unwrap())
		.args([
			"load_unwind_panic::double_panic_aborts_the_process",
			"--exact",
			"--nocapture", // without this the child's output dies with the abort
		])
		.env("CONSERVED_LOAD_DOUBLE_PANIC", "1")
		.output()
		.unwrap();
	// assert on out.status and out.stderr
}
```

Three details, each verified against a running child rather than assumed:

- The `--exact` filter is the **full module path**. `mod load_unwind_panic { … }`
  makes the reported name `load_unwind_panic::double_panic_…`; `--exact`
  matches that, not the bare function name. Misspell it and the child selects
  zero tests, **exits 0**, and the parent asserts on a success it invented —
  the bare-`echo` failure again, one level down.
- `--nocapture` on the child is **required**, not cosmetic. Without it the
  harness buffers the test's output and the abort discards the buffer:
  measured, the child's own `panic!` message never reaches the parent's
  `stderr` while the runtime's `panic in a destructor during cleanup` still
  does (that one is written by the runtime, not the harness). With
  `--nocapture` both appear.
- The vacuity guard that actually works is the child's **stdout**: a real run
  prints `running 1 test`, a mis-filtered run prints `running 0 tests`.
  Measured for both spellings.

| child invocation | `status.success()` | `signal()` | stdout has `running 1 test` | stderr has the destructor line |
|---|---|---|---|---|
| correct filter, `--nocapture` | `false` | `Some(6)` | yes | yes |
| misspelled filter | **`true`** | `None` | no | no |

## Acceptance

- [x] `conserved/tests/load_unwind_panic.rs` exists, entire contents wrapped
      in `mod load_unwind_panic { … }`, and `cargo test -p conserved load`
      selects it (non-zero `N passed` under that filter).
- [x] `panicking_inverse_abandons_the_rest` — five effects, inverse index 3
      panics, `close()` called inside `catch_unwind`. Asserts **all five** of
      the measured facts:
      - `catch_unwind` returned `Err` (the inverse's panic escaped `close`);
      - the inverses that ran are exactly `[4, 3]`, in that order;
      - `s.held()` is empty afterwards — the scope under-reports what is
        outstanding;
      - a second `s.close()` runs nothing further;
      - the `Disposer` retained for effect `0` runs nothing when disposed
        after the fact.
- [x] A comment directly above that test states, in one paragraph: this
      asserts current behaviour, that behaviour contradicts the module doc's
      *"Leaving something behind is not expressible"*, the behaviour is
      inherited byte-for-byte from mitosys `util/effect`, and the test exists
      so that changing it is a decision rather than an accident. It names
      `.mi/prds/p5-adoption/load-proof/finding.md`.
- [x] `abandonment_scales_with_the_panic_position` — 10_000 effects where the
      inverse at registration index 5_000 panics; asserts exactly 5_000
      inverses ran (indices 9_999 down to 5_000) and exactly 5_000 did not.
      This is what makes the size of the hole visible: half the scope.
- [x] `first_panicking_inverse_wins` — two inverses panic (indices 3 and 1);
      asserts the panic that escapes is the one from index 3, the later-
      registered of the two, i.e. the first reached in reverse order, and that
      index 1's inverse never ran at all.
- [x] `double_panic_aborts_the_process` — the subprocess test above. Asserts,
      on the child:
      - it did **not** exit successfully;
      - on unix, `ExitStatusExt::signal()` is `Some(6)` (SIGABRT) — not merely
        a non-zero code, since a compile error or a zero-test run is also
        non-zero-or-zero and must not be mistaken for the abort;
      - the child's captured stderr contains both `panic in a destructor
        during cleanup` and the outer panic's own message (both verified
        present with `--nocapture`, and the second verified absent without
        it — so the flag is load-bearing and the test proves it).
- [x] The child is invoked with `--nocapture` as well as `--exact`, and a
      comment says why (without it the abort discards the harness's buffered
      output and the child's own panic message never arrives — measured).
- [x] **The subprocess test cannot pass vacuously.** It asserts the child's
      **stdout** contains `running 1 test`, which a mis-filtered child does
      not print, in addition to the stderr assertions. Demonstrate the hole is
      closed: misspell the filter once, confirm every assertion fails and the
      test goes red rather than green, then put it back and say so in the
      implementation commit.
- [x] The parent branch of that test is guarded on an env var whose name
      contains `CONSERVED_`, and the child branch returns before touching the
      `Command` path — no risk of unbounded re-exec.
- [x] `#[cfg(unix)]` guards the signal assertion; on non-unix the test still
      asserts `!status.success()` and the stderr contents, so it does not
      silently vanish on another platform.
- [x] `.mi/prds/p5-adoption/load-proof/finding.md` exists and contains: the
      `close()` loop quoted, the `[4, 3]` transcript, the `held() == []`
      observation, the SIGABRT transcript, the sentence naming the invariant
      contradicted, and one paragraph stating that the fix is deliberately
      **not** taken here and why (the eighth-deviation argument above). It
      names no ticket that does not exist.
- [x] `conserved/Cargo.toml` byte-identical; no new dependency of any kind.
- [x] `cargo clippy -p conserved --all-targets -- -D warnings` is clean, and
      `cargo fmt --all --check` passes.

verify: `cargo test -p conserved --test load_unwind_panic && cargo test -p conserved --release --test load_unwind_panic && cargo test -p conserved load_unwind_panic 2>&1 | grep -qE '[1-9][0-9]* passed' && test -s .mi/prds/p5-adoption/load-proof/finding.md && grep -q 'panic in a destructor during cleanup' conserved/tests/load_unwind_panic.rs && cargo clippy -p conserved --all-targets -- -D warnings`

## Implemented 2026-08-21 — measured on the implementer's box

`conserved/tests/load_unwind_panic.rs` (four tests) and
`.mi/prds/p5-adoption/load-proof/finding.md`. No manifest change, nothing under
`conserved/src/` touched — the behaviour is **characterised, not fixed**, per
the user's explicit decision of 2026-08-21.

```
$ cargo test -p conserved --test load_unwind_panic
running 4 tests
test load_unwind_panic::first_panicking_inverse_wins ... ok
test load_unwind_panic::panicking_inverse_abandons_the_rest ... ok
test load_unwind_panic::abandonment_scales_with_the_panic_position ... ok
test load_unwind_panic::double_panic_aborts_the_process ... ok
test result: ok. 4 passed; 0 failed; ... finished in 0.05s
```

Release: `4 passed ... finished in 0.01s`. Under `cargo test -p conserved load`
the file reports `4 passed` in both profiles.

Every measured fact in the spec reproduced exactly: `[4, 3]`, `held() == []`,
the inert second `close()`, the inert retained `Disposer`, 5_000-of-10_000
abandonment, and `"inverse 3 panics"` as the escaping payload.

### The child, and the abort

Raw transcript, run by hand against the built test binary:

```
$ CONSERVED_LOAD_DOUBLE_PANIC=1 ./load_unwind_panic-4c5f3542b2e0ce37 \
    load_unwind_panic::double_panic_aborts_the_process --exact --nocapture ; echo $?

running 1 test
... panicked at conserved/tests/load_unwind_panic.rs:184:13: outer panic in flight
... panicked at conserved/tests/load_unwind_panic.rs:179:33: inverse panics during unwind
  20: conserved::scope::Scope::close           at conserved/src/scope.rs:198:5
  21: <conserved::scope::Scope as Drop>::drop  at conserved/src/scope.rs:218:8
... panicked at core/src/panicking.rs:233:5: panic in a destructor during cleanup
thread caused non-unwinding panic. aborting.
134
```

134 = 128 + 6, i.e. SIGABRT; `ExitStatusExt::signal()` reads `Some(6)`.

**`--nocapture` is proved load-bearing by the test itself**, not only by a
comment: `double_panic_aborts_the_process` spawns a **second** child *without*
the flag and asserts that the runtime's `panic in a destructor during cleanup`
still arrives while the child's own `outer panic in flight` does **not**. Both
directions pass, so the day the flag stops mattering this test goes red rather
than quietly relaxing.

**The vacuity hole is closed — demonstrated.** Filter changed from the
module-qualified `load_unwind_panic::double_panic_aborts_the_process` to the
bare `double_panic_aborts_the_process`:

```
$ cargo test -p conserved --test load_unwind_panic double_panic ; echo $?
the child selected no test — the filter is wrong, so nothing was proved.
stdout:
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out;   <- the CHILD, exiting 0
test result: FAILED. 0 passed; 1 failed; ...                                  <- the parent, caught it
101
```

The child exited 0, exactly as the spec predicted, and the `running 1 test`
stdout guard turned that into a red parent instead of an invented success.
Reverted.
