# goal

Make `conserved::scope`'s own second sentence true: run **every** inverse on
close even when one panics, give the scope a way to report the ones that
failed, and record the resulting eighth — and first *semantic* — deviation
from the mitosys source at its site.

est: 3.0h

One spec, not two. The characterisation tests in
`conserved/tests/load_unwind_panic.rs` go red the instant `close()` changes,
so the code change and the test change are one commit or the tree is red
between them.

## Everything below was run before it was written

A standalone prototype of the exact `close()` written here was built and run
against copies of all four existing test files (`scope.rs`, `load_scope.rs`,
`load_unwind_panic.rs`, and the new tests) on Apple M5, rustc 1.94.0. Every
number, every failure and every pass named below is a transcript, not a
prediction. Where a claim was *not* verified that way it says so.

---

## 1. The panic-payload contract — decided here

**Chosen: resume the first panic *reached*, record every failed label,
drop the other payloads.**

Concretely: the loop catches each inverse; the payload of the first inverse
that panics (first *reached*, which in a reverse unwind is the
later-registered) is kept and handed to `std::panic::resume_unwind` after the
loop finishes; every panicking inverse's **label** is appended to a new
`failed` list on `Inner` and read back through a new `Scope::failed()`.

Why the other payloads are not lost in the way that matters: **the panic hook
has already run for each of them.** A payload is produced *after* the hook
prints; every one of the N panic messages, with its file and line, is on
stderr before `close()` ever sees the `Err`. What is dropped is the ability to
`downcast` the 2nd..Nth payload, not the report of them. `failed()` covers the
identity half of that, by label, structurally.

Rejected, and why — judged against the call sites `Scope` actually has
(mitosys re-exports `util/effect` from 10+ crates: `api/plugin`,
`api/plugin/lua`, `api/surface`, `api/agentic`, `api/agentic/pool`,
`api/service`, `api/engine`, `engine/record`, `engine/layers`,
`engine/channel`; realm has three hand-rolled sites — `drivers/linux/overlay.rs`
`unmount_all`, `drivers/linux/zfs_volumes.rs:139`, `net/src/lib.rs:437` — that
would adopt it):

- **Collect them all and resume a composite payload.** Costs payload identity
  at every existing call site. Any consumer that catches around a close and
  downcasts to `&str`/`String` — which is the only thing anyone downcasts to,
  including this crate's own `payload_of` helper and every test in
  `load_unwind_panic.rs` — stops matching. It also forces a new public type
  into a crate whose whole argument is that it is minimal, for information the
  hook already printed. The strongest of the rejected options, and still not
  worth breaking every downcast in ten mitosys crates for.
- **Report through a return value instead of resuming.** Dead on the `Drop`
  path, which is not optional: `Drop::drop` calls `close()` and cannot return
  anything, so a plugin that forgets to close would get a *silently* failed
  unwind — strictly worse than today, where the panic at least escapes. It
  also changes `close()`'s signature under 10+ re-exporting crates, and it
  swallows a teardown panic that currently propagates. A `close_checked()`
  alongside was considered and rejected too: new API surface with no call
  site, and it still leaves `Drop` with nothing.

**`Disposer::dispose()` is unchanged.** A panic there propagates directly to
its caller and is not silently lost, so it does not need `failed()` and does
not write to it. `failed()` means, exactly: *inverses that panicked during this
scope's close.* Say that in its doc.

`failed()` is ordered **registration order**, the same as `held()` — the two
are counterparts and a reader diffs one against the other. The consequence,
which the tests assert: the resumed payload belongs to `failed()`'s **last**
entry, because the first inverse reached in reverse is the last-registered of
those that failed.

## 2. What `held()` must mean — decided here

**A failed inverse does not stay in `held()`; it moves to `failed()`. And
`held()` becomes honest at every instant of the unwind, not just at the end.**

The deeper defect is not only the abandoned tail. `close()` today
`mem::take`s **both** `order` and `live` before the first inverse runs, so
from that instant `held()` returns `[]` — on *every* close, panic or not. An
inverse that asks what is still outstanding is told "nothing" while 9_999
inverses are pending. That is the same lie the finding measured, just without a
panic to make it visible.

So `close()` stops detaching. It marks the scope closed, then repeatedly pops
one id from `g.order` under the lock, removes that entry from `g.live` under
the same lock, releases the lock, and runs the inverse. `held()` is unchanged
code and becomes true by construction: it reports exactly the effects whose
inverses have not yet run, in registration order, at any moment.

Three properties this turns on, each of which the implementer should not have
to rediscover:

- **Remove-before-run is required, not stylistic.** If the entry stayed in
  `live` while its inverse ran, an inverse that disposes *itself* would find
  it and run it a second time. Removing first makes exactly-once fall out.
  It also means an inverse never sees its own label in `held()`, which is
  correct — a running inverse is not owed, it is running.
- **The loop terminates.** `order` can only shrink: `effect()` returns
  `Err(Closed)` once `closed` is set, so nothing can push while the loop runs,
  and each iteration pops exactly one id.
- **Reentrancy still holds, for a slightly different reason.** The `Scope` doc
  currently credits it to "the live set is detached before any inverse runs".
  Rewrite that: the scope is marked closed before any inverse runs, and **no
  lock is ever held across an inverse** — not in `close()`, not in `dispose()`,
  not in `effect()`. Nothing iterates `live`; `close()` looks each id up by
  key. An inverse may therefore call `held()`, `failed()`, `close()`
  (idempotent no-op), `effect()` (`Err(Closed)`) or another effect's
  `dispose()` freely. Because no user code ever runs under the mutex, the
  mutex cannot be poisoned by an inverse — which matters now that `close()`
  re-acquires it after a caught panic.

One behaviour changes as a side effect, and it is an improvement: an inverse
that disposes a still-pending effect now runs that inverse **immediately**
rather than having it run later at its LIFO position (today `dispose()` during
a close is a silent no-op because `live` was taken out from under it). Still
exactly once, either way. Pinned by `dispose_during_unwind_runs_once` below.

## 3. The abort case stays honest — and is now *proved* unchanged

A panicking inverse **while a panic is already in flight** still aborts the
process with SIGABRT. `catch_unwind` around each inverse does not and cannot
change that: the runtime checks the thread's panic count at the *panic site*
and calls `abort` before any unwinding begins, so no catch downstack is ever
reached.

This is not asserted, it is measured: `double_panic_aborts_the_process` was
run **unchanged** against the prototype and **passed**, still on `signal() ==
Some(6)`, still finding `panic in a destructor during cleanup` on the child's
stderr. That test file is the proof, and this spec forbids touching that test.

The module doc must say all of this, plus the two limits that are documented
rather than fixed:

- the abort-during-abort case above, naming
  `conserved/tests/load_unwind_panic.rs::double_panic_aborts_the_process`;
- the **nested-scope depth limit** — `Drop -> close -> undo -> Drop -> …`
  recurses one stack frame chain per nesting level; depth 1_000 is proven safe
  by `load_scope::deep_nesting_unwinds_at_a_safe_depth`, and depth 10_000
  overflows the stack and aborts. Measured by p5's load proof; not fixed here.
- **`panic = "abort"`.** Under a profile that aborts on panic there is no
  unwinding at all, so the first panicking inverse ends the process and no
  later inverse runs. Neither tree sets it today (no `panic =` key in
  `Cargo.toml`, `conserved/Cargo.toml` or `../mitosys/Cargo.toml` — checked),
  but the guarantee is conditional on that and the doc should not pretend
  otherwise. **Not measured** — this one is from the language rule, unlike
  everything else in this spec.

## 4. `catch_unwind` and `UnwindSafe` — settled here, not left to discover

`catch_unwind` requires `F: UnwindSafe`, and `Undo = Box<dyn FnOnce() + Send>`
carries no such bound — `dyn FnOnce` is not `UnwindSafe` in general. The call
is therefore:

```rust
if let Err(payload) = catch_unwind(AssertUnwindSafe(undo)) { … }
```

`AssertUnwindSafe<F>` implements `FnOnce()` when `F: FnOnce()`, and
`Box<dyn FnOnce() + Send>: FnOnce()`, so `AssertUnwindSafe(undo)` is accepted
directly — no wrapper closure needed. **This compiles under
`#![forbid(unsafe_code)]`**: `AssertUnwindSafe` is a safe newtype and
`UnwindSafe` is a lint-shaped marker about logical state, not a memory-safety
bound. Verified by building the prototype with the same `forbid` attribute.

Why asserting it is sound here, in one paragraph the doc should carry:

- The closure is **owned by the scope and consumed once**. It is `mem::take`n
  out of its `Entry` (via `Option<Undo>`), moved into the catch, and dropped
  there. Nothing — not `Scope`, not `Disposer`, not any caller — can observe
  or invoke it again after the catch, so there is no "broken invariant read
  after a caught panic" for its captured state. That is the entire hazard
  `UnwindSafe` exists to flag.
- The only state `Scope` itself carries across the boundary is `Inner`, behind
  a `Mutex`, and the lock is not held across the call (§2). The scope's own
  invariant after a caught panic is exactly "this entry's inverse ran and
  panicked" — which is what `failed()` then records. Nothing is left half-
  updated.

## 5. The divergence — recorded in both trees' terms

### At its site: `conserved/src/scope.rs`'s `# Provenance` block

The block currently opens *"From the `use std::collections::HashMap;` line
below to the end of this file the copy is byte-for-byte identical to that
source"*. **That sentence stops being true with this spec** and must not be
deleted — extend it, in the shape `learnings/README.md` §"How a document is
corrected" rule 1 asks for: it *was* byte-for-byte identical through
`5313ca4`, and deviation 8 below is the first change to that body.

The block also says **"Six deviations, and no others"** while listing
**seven** — item 7 (the `mod scope { … }` test wrapper) was appended without
updating the count. Fix it to **"Eight deviations, and no others"** and add:

> 8. **The unwind no longer abandons the rest when one inverse panics.** The
>    first semantic deviation from the source; 1-7 are naming, pathing and
>    test-wrapping. mitosys's `util/effect` keeps the old behaviour until it
>    adopts — reconciling the two is `.mi/prds/p5-adoption/mitosys`. What
>    changed: `close()` catches each inverse, runs the whole reverse loop, and
>    resumes the first panic reached afterwards; `order` and `live` are no
>    longer detached from `Inner` before the loop, so `held()` is true during
>    the unwind and not only before it; `Inner` gains `failed` and `Scope`
>    gains `failed()`. Argued in `.mi/prds/p6-scope-unwind/`, measured in
>    `.mi/prds/p5-adoption/load-proof/finding.md`.

**Known consequence, recorded here because p1's folder is not this ticket's to
edit:** `.mi/prds/p1-scope/specs/spec01.md`'s `verify:` line diffs
`conserved/src/scope.rs` against the mitosys source and will now report a
difference. That is this ticket's whole point, not a regression. Do not edit
p1's spec, do not edit its acceptance ticks — say it in the implementation
note and let the board carry it.

### `learnings/shared-crate.md` §3

§3 says of `Scope`: *"Ports as-is. This is the clearest case in the whole
proposal — one side has a correct, dependency-free implementation and the
other has a comment."* "Correct" is now known to be false in one respect.
A rule-1 correction — an **addition**, deleting nothing — goes at the end of
§3, in the shape of the `**Landed as Scope / Disposer**` note already there.
Something close to:

> **Corrected 2026-08-21 — "as-is" did not survive contact.** p5's load proof
> measured that a panicking inverse abandoned every inverse still to come and
> that `held()` then reported `[]`
> (`.mi/prds/p5-adoption/load-proof/finding.md`). p6-scope-unwind fixed it in
> `conserved` — every inverse runs, `held()` is true during the unwind, and a
> new `failed()` names the inverses that panicked. The port was still the
> right move; "one side has a *correct* implementation" was the loose word,
> and the shared crate is where that got found.

Do **not** touch that document's frontmatter.

### The held `p5-adoption/mitosys` child — reported, not written

That child's `prd.md` is another ticket's file. The analyst does not edit it.
The requirement it needs is written out verbatim in
`## What p5-adoption/mitosys must gain` at the end of this spec, for the user
to paste.

## Files and dirs

Edited:

- `conserved/src/scope.rs` — `Inner` gains `failed: Vec<String>`; `close()`
  rewritten per §1-§2; `Scope::failed()` added; module doc gains a
  `# Panics during unwind` section (§3) and the `AssertUnwindSafe` paragraph
  (§4); the `Scope` struct doc's reentrancy sentence rewritten (§2); `held()`'s
  doc extended; the `# Provenance` block corrected and extended (§5).
- `conserved/tests/load_unwind_panic.rs` — §6.
- `learnings/shared-crate.md` — one additive paragraph at the end of §3 (§5).

Not touched, and each for a reason: `conserved/Cargo.toml` and the root
`Cargo.toml` (no dependency — `catch_unwind`, `resume_unwind` and
`AssertUnwindSafe` are all `std::panic`), `conserved/src/lib.rs`
(`pub mod scope;` and `#![forbid(unsafe_code)]` both already there),
`conserved/tests/scope.rs` (all five ported tests pass unchanged against the
prototype — verified), `conserved/tests/load_scope.rs` (all five pass
unchanged — verified, including the timing bound), `conserved/tests/*` for
clock / content_id / stats, `../mitosys` (**read-only**), and every other
ticket's folder under `.mi/prds/`.

## 6. The characterisation tests — old and new, side by side

`conserved/tests/load_unwind_panic.rs` keeps its `mod load_unwind_panic { … }`
wrapper (board convention; `cargo test -p conserved load` selects on the
module-qualified **function** name). Its module doc's numbered list — which
today reads *"silently abandons every inverse still to come, and `held()`
afterwards reports `[]`"* — is rewritten to describe the new contract, and
gains a sentence saying these are no longer characterisation tests of a defect
but the contract's proof. Branch 2 of that list (the abort) stays word for
word.

Every changed assertion below must appear in the file with the **old
expectation quoted in a comment directly above it**, so the diff is legible
without `git log -p`. That is a requirement of the ticket, not a nicety.

### `panicking_inverse_abandons_the_rest` → `panicking_inverse_does_not_abandon_the_rest`

Five effects, inverse 3 panics, `close()` inside `catch_unwind`.

| # | old | new |
|---|---|---|
| 1 | `assert!(outcome.is_err())` | **unchanged** — the panic still escapes `close()` |
| 2 | `assert_eq!(*log.lock().unwrap(), vec![4, 3]);` | `assert_eq!(*log.lock().unwrap(), vec![4, 3, 2, 1, 0]);` |
| 3 | `assert!(s.held().is_empty(), "held() reports {:?}, not the empty vec this pins", s.held());` — commented *"the scope under-reports: it says nothing is outstanding while three inverses are outstanding"* | `assert!(s.held().is_empty(), …);` **plus** `assert_eq!(s.failed(), vec!["e3".to_string()]);` — commented that `held()` is now empty **because every inverse ran**, and the one that panicked is named by `failed()`. Both halves are required: the `held()` assertion alone reads identical to the old one in the diff while meaning the opposite. |
| 4 | second `close()` runs nothing further | **unchanged** — still nothing, now because everything already ran |
| 5 | `keep_zero.dispose()` runs nothing, commented *"`live` was taken out of `Inner` before the loop, so `dispose()` finds nothing"* | assertion **unchanged**; comment replaced — effect 0's inverse already ran during the close, so `live` no longer holds it. `before` is now 5, not 2. |

The long comment above the test — the one that says this pins behaviour
contradicting the module doc and that changing it must be argued — is
**replaced**, not deleted. The replacement says: this was that argument's
outcome; the fix landed in p6-scope-unwind; the behaviour it used to pin is in
`git log -p` and in `.mi/prds/p5-adoption/load-proof/finding.md`, which stays
as the measured record of what was fixed and must not be edited.

### `abandonment_scales_with_the_panic_position` → `unwind_completes_regardless_of_the_panic_position`

10_000 effects, inverse 5_000 panics.

| old | new |
|---|---|
| `let want: Vec<usize> = (PANICS_AT..COUNT).rev().collect();` | `let want: Vec<usize> = (0..COUNT).rev().collect();` |
| `assert_eq!(ran.len(), COUNT - PANICS_AT, "exactly half should have run");` | `assert_eq!(ran.len(), COUNT, "every inverse should have run");` |
| `assert_eq!(ran, want, "the half that ran should be 9_999 down to 5_000");` | `assert_eq!(ran, want, "the unwind should be 9_999 down to 0");` |
| `assert_eq!(COUNT - ran.len(), PANICS_AT);` — *"exactly half did not"* | `assert_eq!(COUNT - ran.len(), 0, "nothing was abandoned");` plus `assert_eq!(s.failed(), vec![format!("e{PANICS_AT}")]);` and `assert!(s.held().is_empty());` |

`assert!(outcome.is_err())` is unchanged.

### `first_panicking_inverse_wins` — name kept, still true

Panics at 1 and 3.

| old | new |
|---|---|
| `assert_eq!(payload_of(&err), "inverse 3 panics", …)` | **unchanged.** This is the payload contract of §1 holding: the payload that escapes is the same one that escapes today. |
| `assert_eq!(ran, vec![4, 3]);` | `assert_eq!(ran, vec![4, 3, 2, 1, 0]);` |
| `assert!(!ran.contains(&1), "index 1's inverse should never have run");` | `assert!(ran.contains(&1), "index 1's inverse must have run");` — **the inversion the ticket exists for** |
| — | `assert_eq!(s.failed(), vec!["e1".to_string(), "e3".to_string()]);` — registration order, and the resumed payload is the **last** entry's |

The doc comment above it changes from *"The earlier one is in the abandoned
tail and never runs at all"* to: both run, both are named by `failed()`, and
only the payload of the first *reached* is resumed.

### `double_panic_aborts_the_process` — **byte-for-byte unchanged**

Including its doc comment and all four constants. Verified passing against the
prototype. This is §3's proof and the implementer must not touch it.

### New: `every_inverse_panicking_still_runs_them_all`

Five effects, **all five** inverses panic. Asserts `log == [4,3,2,1,0]`,
`payload_of(err) == "inverse 4 panics"` (the first reached),
`failed() == ["e0","e1","e2","e3","e4"]`, `held()` empty. This is the case the
old code could not even reach, and the one that proves catching does not chain
into an abort when no panic is in flight. Verified passing.

### New: `held_is_honest_during_the_unwind`

Three effects, `e1`'s inverse panics; each inverse records `(held(), failed())`
**at its own moment**, and the test asserts the sequence is exactly:

```
e2's inverse: held = ["e0", "e1"], failed = []
e1's inverse: held = ["e0"],       failed = []
e0's inverse: held = [],           failed = ["e1"]
```

Two mechanical facts the implementer would otherwise burn time on, both hit
and solved in the prototype:

- **An inverse cannot borrow the scope.** `Undo = Box<dyn FnOnce() + Send>` is
  implicitly `+ 'static`, so `move || … s.held() …` does not compile. Put the
  scope in an `Arc<Scope>` and capture a `Weak<Scope>`
  (`let w = Arc::downgrade(&s);` … `w.upgrade().expect(…)`). `Weak<Scope>` is
  `Send`, `Arc<Scope>` derefs for `s.effect(…)`, and the test calls
  `s.close()` explicitly so nothing depends on the drop.
- **`clippy::type_complexity` is `-D warnings` here.** A recorder typed
  `Arc<Mutex<Vec<(Vec<String>, Vec<String>)>>>` fails the clippy gate — hit in
  the prototype. Use two parallel `Arc<Mutex<Vec<Vec<String>>>>`, or a named
  struct, or a `type` alias.

### New: `dispose_during_unwind_runs_once`

Two effects; `e1`'s inverse disposes `e0`'s retained `Disposer` (passed in
through an `Arc<Mutex<Option<Disposer>>>`, since the closure is `'static`).
Asserts `log == [1, 0]` — `e0` ran exactly once, early, from the dispose —
and `held()` empty afterwards. Pins the §2 side effect so it is a decision on
record rather than something a later reader trips over. Verified passing.

## What the tests measured against the prototype

All four existing test files plus the new ones, dev and release:

- `conserved/tests/scope.rs` — 5 passed, **unchanged**.
- `conserved/tests/load_scope.rs` — 5 passed, **unchanged**, including
  `panic_unwind_is_not_quadratic`. Per-inverse `catch_unwind` plus a lock
  acquisition per iteration cost nothing measurable at 100_000:
  `unwind: 12500 -> 3.78 ms, 100000 -> 31.3 ms, ratio 8.29x` in dev and
  `12500 -> 408 µs, 100000 -> 3.84 ms, ratio 9.42x` in release — against p5's
  recorded pre-change baseline of 30.7 ms dev / 3.29 ms release. Bounds are
  25x and 5 s; neither is anywhere near.
- `load_unwind_panic.rs` **as it stands today** — 1 passed, 3 failed:
  `panicking_inverse_abandons_the_rest`,
  `abandonment_scales_with_the_panic_position` and
  `first_panicking_inverse_wins` all go red, exactly the three this spec
  rewrites, and `double_panic_aborts_the_process` passes untouched. That is
  the characterisation tests doing their job.
- `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean on
  the library.

## Acceptance

- [x] `conserved/src/scope.rs`: `close()` no longer detaches `order` or `live`
      from `Inner` before running inverses — no `mem::take` (or
      `std::mem::take`) appears on any **code** line of the file. `failed()`
      clones its list rather than taking it.
- [x] `close()` runs **every** live inverse in reverse registration order even
      when one or more panic, catching each with
      `catch_unwind(AssertUnwindSafe(undo))`, and calls
      `std::panic::resume_unwind` with the payload of the **first inverse
      reached** that panicked, after the loop has finished. No other payload is
      resumed and no payload is re-`panic!`ed (`resume_unwind` does not re-run
      the hook; `panic!` would print a second time).
- [x] `Inner` gains `failed: Vec<String>` and `Scope` gains
      `pub fn failed(&self) -> Vec<String>`, returning the labels of inverses
      that panicked during close, **in registration order**. `Disposer::dispose`
      is unchanged and does not write to it; its doc says so.
- [x] `held()` reports exactly the effects whose inverses have not yet run, at
      any moment including from inside a running inverse — proved by
      `held_is_honest_during_the_unwind` asserting the exact three-step
      sequence in §6.
- [x] The module doc gains a `# Panics during unwind` section stating: every
      inverse runs; which payload is resumed and that the rest are named by
      `failed()` after their messages were already printed by the panic hook;
      that a panicking inverse **while a panic is already in flight aborts the
      process (SIGABRT)**, that this is Rust's rule and that `catch_unwind`
      here does not change it, naming
      `conserved/tests/load_unwind_panic.rs::double_panic_aborts_the_process`;
      that the nested-scope depth limit (safe at 1_000, stack overflow at
      10_000) is documented and not fixed; and that under `panic = "abort"`
      there is no unwinding and the guarantee does not apply.
- [x] The doc carries the `AssertUnwindSafe` paragraph of §4 — that
      `Box<dyn FnOnce() + Send>` is not `UnwindSafe`, that asserting is sound
      because the closure is owned and consumed once and no lock is held
      across it, and that this needs no `unsafe`.
- [x] The `Scope` struct doc's reentrancy sentence no longer claims the live
      set is detached; it says the scope is marked closed first and no lock is
      held across an inverse.
- [x] The `# Provenance` block reads **"Eight deviations, and no others"**,
      lists eight numbered items, carries item 8 with the wording of §5
      (naming `p5-adoption/mitosys` as where the reconciliation happens), and
      **extends rather than deletes** the byte-for-byte sentence to say it held
      through the p1 port and that item 8 is the first change to that body.
- [x] `learnings/shared-crate.md` §3 gains the additive correction of §5 —
      it deletes no existing sentence, and the document's frontmatter is
      untouched.
- [x] `conserved/tests/load_unwind_panic.rs` reports exactly seven tests:
      `panicking_inverse_does_not_abandon_the_rest`,
      `unwind_completes_regardless_of_the_panic_position`,
      `first_panicking_inverse_wins`,
      `every_inverse_panicking_still_runs_them_all`,
      `held_is_honest_during_the_unwind`, `dispose_during_unwind_runs_once`,
      `double_panic_aborts_the_process`. All seven pass in dev **and**
      release. `cargo test -p conserved --test load_unwind_panic -- --list`
      contains no test named `panicking_inverse_abandons_the_rest` or
      `abandonment_scales_with_the_panic_position`.
- [x] Every assertion in the three rewritten tests that changed carries the
      **old expectation quoted in a comment immediately above it**, per the
      tables in §6. In particular `panicking_inverse_does_not_abandon_the_rest`
      asserts `failed() == ["e3"]` alongside the `held().is_empty()` line, so
      the diff cannot read as "unchanged".
- [x] `double_panic_aborts_the_process` and its four constants are unchanged,
      and it still passes on `signal() == Some(6)`.
- [x] `conserved/tests/scope.rs` and `conserved/tests/load_scope.rs` are
      **not edited** and still report 5 passed each.
- [x] `conserved/Cargo.toml` and the root `Cargo.toml` are unchanged.
      `cargo tree -p conserved --edges normal --depth 1` is exactly two lines
      (`conserved`, `blake3`) — no dependency and no dev-dependency added.
- [x] `#![forbid(unsafe_code)]` is still in `conserved/src/lib.rs` and the
      crate builds — no `unsafe` block was needed anywhere.
- [x] `cargo fmt --all --check` and
      `cargo clippy --workspace --all-targets -- -D warnings` are silent.
- [x] `git -C ../mitosys status --porcelain src/mitosys/util/effect` prints
      nothing — the source tree was read, never written.
- [x] The implementation note records: the measured `panic_unwind_is_not_quadratic`
      ratio and absolute timings on the implementer's box, before/after; and
      that `.mi/prds/p1-scope/specs/spec01.md`'s `verify:` diff now reports a
      difference **by design** and was deliberately not edited.

## What `p5-adoption/mitosys` must gain — for the user to add, not this ticket

That child's `prd.md` is another ticket's file. Proposed requirement, to be
added under its `## Requirements`:

> - [ ] **`conserved::scope` and `util/effect` no longer agree — reconcile
>       deliberately, do not diff-and-merge.** p6-scope-unwind made the first
>       *semantic* divergence from the byte-for-byte port (deviation 8 in
>       `conserved/src/scope.rs`'s `# Provenance`): on close, `conserved` runs
>       **every** inverse even when one panics, resumes the first panic
>       reached afterwards, keeps `held()` true *during* the unwind, and adds
>       `Scope::failed()` naming the inverses that panicked. mitosys's
>       `util/effect` still abandons the tail and still reports `held() == []`
>       with inverses owed — measured in
>       `.mi/prds/p5-adoption/load-proof/finding.md`. Adopting the crate
>       therefore **changes teardown behaviour** in all 10+ re-exporting
>       crates: a plugin whose inverse panics now has its remaining inverses
>       run rather than dropped. That is the point of adopting, and it must be
>       stated in the adoption commit rather than discovered. Check the tree
>       for any site that depends on the abandonment — `grep` teardown paths
>       for `catch_unwind` around a `close()` — and for anything that reads
>       `held()` during an unwind. `Scope::failed()` is new API with no mitosys
>       call site yet; adopt it where a teardown failure is currently swallowed.
>       The abort-during-abort case is **unchanged** in both trees.

Also worth telling that ticket: `.mi/prds/p1-scope/specs/spec01.md`'s
byte-for-byte `verify:` diff no longer holds, by design.

verify: `bash -c 'set -e; cd /Users/feb/dev/infra/shared; grep -q "Eight deviations, and no others" conserved/src/scope.rs; grep -q "^//! 8\. " conserved/src/scope.rs; grep -q "# Panics during unwind" conserved/src/scope.rs; grep -q "panic in a destructor during cleanup" conserved/src/scope.rs; grep -q "AssertUnwindSafe" conserved/src/scope.rs; grep -q "resume_unwind" conserved/src/scope.rs; grep -q "pub fn failed(&self)" conserved/src/scope.rs; grep -q "forbid(unsafe_code)" conserved/src/lib.rs; grep -q "p6-scope-unwind" learnings/shared-crate.md; if grep -vE "^[[:space:]]*//" conserved/src/scope.rs | grep -q "mem::take"; then echo "close() still detaches order/live before running any inverse"; exit 1; fi; L=$(cargo test -p conserved --test load_unwind_panic -- --list 2>/dev/null); for t in panicking_inverse_does_not_abandon_the_rest unwind_completes_regardless_of_the_panic_position first_panicking_inverse_wins every_inverse_panicking_still_runs_them_all held_is_honest_during_the_unwind dispose_during_unwind_runs_once double_panic_aborts_the_process; do printf "%s\n" "$L" | grep -q "load_unwind_panic::$t: test" || { echo "missing test: $t"; exit 1; }; done; for t in panicking_inverse_abandons_the_rest abandonment_scales_with_the_panic_position; do if printf "%s\n" "$L" | grep -q "load_unwind_panic::$t: test"; then echo "old characterisation test still present: $t"; exit 1; fi; done; cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test -p conserved --test scope 2>&1 | grep -q "5 passed"; cargo test -p conserved --test load_scope 2>&1 | grep -q "5 passed"; cargo test -p conserved --test load_unwind_panic 2>&1 | grep -q "7 passed"; cargo test -p conserved --release --test load_unwind_panic 2>&1 | grep -q "7 passed"; cargo test -p conserved scope; cargo test -p conserved load; n=$(cargo tree -p conserved --edges normal --depth 1 | wc -l | tr -d " "); [ "$n" = 2 ] || { echo "conserved gained a dependency edge"; cargo tree -p conserved --edges normal --depth 1; exit 1; }; [ -z "$(git -C ../mitosys status --porcelain src/mitosys/util/effect)" ] || { echo "the mitosys source was modified"; exit 1; }; echo "spec01 ok"'`
