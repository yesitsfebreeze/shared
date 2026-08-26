# Finding — `Scope` and the panicking inverse

Measured 2026-08-21 against `conserved/src/scope.rs` at `main` (`a3d8bcc`),
Apple M5, `rustc 1.94.0 (4a4ef493e 2026-03-02)`. Pinned by
`conserved/tests/load_unwind_panic.rs`. Nothing here is inferred; every
transcript below was produced by a running process.

There are two branches, and they fail differently.

---

## 1. A panicking inverse abandons the rest of the unwind, silently

`Scope::close` detaches the live set from the mutex and then iterates it:

```rust
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
```

`live` is a **local** `HashMap<u64, Entry>`. When `undo()` panics, the loop is
abandoned and `live` is dropped as the stack unwinds — and dropping a
`Box<dyn FnOnce()>` **does not call it**. Every inverse still in the map is
discarded, unrun, with no diagnostic of any kind.

### Transcript — five effects, inverse `3` panics

```
inverses that ran : [4, 3]
s.held()          : []
second s.close()  : ran nothing
Disposer for 0    : dispose() ran nothing
panic escaping    : "inverse 3 panics"
process           : exits normally, panic propagates, nothing aborts
```

Read that line by line:

- **`[4, 3]`.** Effect `4`'s inverse ran; effect `3`'s ran and panicked.
  Effects `2`, `1` and `0` never ran and never will.
- **`held()` returns `[]`.** The scope reports nothing outstanding while three
  inverses are outstanding. This is the part that makes the leak invisible:
  there is no query that reveals it.
- **A second `close()` is a no-op**, because `closed` was set before the loop
  began.
- **The `Disposer` for an abandoned effect is inert.** `live` was `mem::take`n
  out of `Inner`, so `dispose()` looks in the (now empty) shared map, finds
  nothing, and runs nothing. The caller who kept a handle has no recourse
  either.
- **The scope's own `Drop`** runs after the panic escapes `close()` and is
  safely a no-op, so the panic propagates normally.

### The size of the hole scales with the panic's position

10_000 effects with the inverse at registration index 5_000 panicking: exactly
5_000 inverses run (indices `9_999` down to `5_000`) and exactly 5_000 are
abandoned. A panic halfway through the unwind loses half the scope.
(`abandonment_scales_with_the_panic_position`.)

With two panicking inverses, the one that escapes is the **first reached** in
reverse order — the later-registered. With panics at indices 1 and 3, index 3's
panic escapes and index 1's inverse never runs at all.
(`first_panicking_inverse_wins`.)

### The invariant this contradicts

`conserved/src/scope.rs`'s own module doc, second sentence:

> Leaving something behind is not expressible.

It is expressible. `learnings/shared-crate.md` cites that same property as the
reason `Scope` was the first thing extracted.

It is **not unsound**: no UB, `#![forbid(unsafe_code)]` holds, no double free,
no memory leak — what leaks is *effect*, not allocation. And it is **inherited**:
p1 ported `scope.rs` byte-for-byte from mitosys's `src/mitosys/util/effect/effect.rs`,
so mitosys has behaved this way for the whole life of that file.

---

## 2. A panicking inverse during an in-flight panic aborts the process

A scope dropped while a panic unwinds runs `Drop::drop` → `close()` → an
inverse panics → panic in a destructor during cleanup → `abort`. This is
Rust's rule, not a `Scope` bug, and **no `catch_unwind` can intercept it** —
which is why it is tested out of process or not at all.

### Transcript — child process, `--exact --nocapture`

```
$ CONSERVED_LOAD_DOUBLE_PANIC=1 ./load_unwind_panic \
    load_unwind_panic::double_panic_aborts_the_process --exact --nocapture

running 1 test

thread '…::double_panic_aborts_the_process' panicked at
conserved/tests/load_unwind_panic.rs:184:13:
outer panic in flight

thread '…::double_panic_aborts_the_process' panicked at
conserved/tests/load_unwind_panic.rs:179:33:
inverse panics during unwind
stack backtrace:
  …
  20: conserved::scope::Scope::close        at conserved/src/scope.rs:198:5
  21: <conserved::scope::Scope as Drop>::drop  at conserved/src/scope.rs:218:8
  …

thread '…::double_panic_aborts_the_process' panicked at
core/src/panicking.rs:233:5:
panic in a destructor during cleanup
thread caused non-unwinding panic. aborting.

exit=134            # 128 + 6, i.e. SIGABRT
```

### Two harness facts, each verified against a running child

| child invocation | `status.success()` | `signal()` | stdout has `running 1 test` | stderr has the destructor line | stderr has the outer message |
|---|---|---|---|---|---|
| module-qualified filter, `--nocapture` | `false` | `Some(6)` | yes | yes | yes |
| module-qualified filter, no `--nocapture` | `false` | `Some(6)` | yes | yes | **no** |
| bare function name as filter | **`true`** | `None` | **no** | no | no |

- **`--exact` must be given the module-qualified name.** `mod load_unwind_panic { … }`
  makes the harness report the test as `load_unwind_panic::double_panic_…`.
  Given the bare function name, the child selects zero tests, prints
  `running 0 tests`, and **exits 0** — a parent asserting only "the child did
  not succeed" would then be asserting on a success it invented. The guard that
  closes this is asserting the child's **stdout** contains `running 1 test`.
- **`--nocapture` is load-bearing.** Without it the harness buffers the test's
  own output and the abort discards the buffer, so the child's `outer panic in
  flight` never reaches the parent — while the runtime's `panic in a destructor
  during cleanup`, written directly rather than through the harness, still does.
  The test asserts both directions, so the flag cannot quietly stop mattering.

---

## Why this ticket characterises rather than fixes

Deliberate, and decided by the user on 2026-08-21 rather than by an analyst.

`p1` landed `scope.rs` as a byte-for-byte port with exactly seven recorded
deviations, all of them naming, pathing or test-wrapping. Making `close()` run
the remaining inverses — catching each one and resuming the first panic
afterwards — would be an eighth deviation and the first **semantic** one: the
first behavioural divergence from the mitosys tree that `p5-adoption`'s
`mitosys` child must later reconcile. That is a change to the extracted
contract, not a load proof, and this ticket's mandate is to *exercise* the
panic-during-unwind case.

A characterisation test is not the weaker choice. `panicking_inverse_abandons_the_rest`
fails the moment the behaviour changes **in either direction**: someone who
fixes `close()` without recording the decision turns this test red and has to
argue for the change rather than slip it in. That is the correct gate for
behaviour nobody has yet decided to keep.

The decision *whether* to fix belongs to **`.mi/prds/p6-scope-unwind/prd.md`**,
which is open and blocked on this ticket. This file is that ticket's evidence.
