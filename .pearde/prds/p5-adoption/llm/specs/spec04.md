---
complexity: 20
footprint:
  - ../model/Cargo.toml
  - ../model/src/daemon/mod.rs
  - ../model/src/main.rs
---

# spec04 — `daemon::boot` gains `Scope`, retiring the DOGMA-13-by-hand comment

`main.rs:126`'s comment cites "the live node's checkpoint (DOGMA 13)" in
prose — `learnings/inventory.md` names this exactly: *"llm alone: ...
reversible effects: none — DOGMA 13 cited in prose, no type."* `boot_live`
(`daemon/mod.rs:968`) is "the single composition" the comment points at, and
the actual multi-step, partially-fallible resource acquisition it composes
lives inside `boot` (`daemon/mod.rs:772`): open the registry, load-or-create
the identity keypair, bind the discovery (kad) listener, dial bootstrap
peers, bind the data-plane net listener — each a `?`-propagated step where a
later failure today leaves everything acquired before it torn down only by
each type's own `Drop`, not by a recorded, ordered, DOGMA-13-shaped inverse.

## Acceptance

- [x] `boot`'s fallible resource-acquisition sequence (registry open,
      identity load/create, discovery listener bind, data-plane net
      listener bind) registers each successfully-acquired resource's
      teardown on a `conserved::Scope` as it happens, via `Scope::effect`.
      The scope is disposed (`Disposer::dispose`), not unwound, once `boot`
      returns `Ok`.
- [x] A test forces a late step to fail (e.g. an already-bound or otherwise
      unbindable data-plane listen address, after the discovery listener
      has already bound successfully) and asserts the earlier step's
      registered inverse ran, in reverse order — via `conserved::Scope`'s
      `held()`/`failed()` or an observable side effect (the discovery
      listener's port becoming free again, or an explicit call counter in
      the registered `Undo`).
- [x] `main.rs:126`'s comment no longer frames the rule as held "by hand":
      it names `conserved::Scope` as the type now carrying DOGMA 13 at this
      boot path, closing the citation `learnings/inventory.md` and
      `p1-scope/prd.md` (*"llm has no implementation — it cites DOGMA 13 in
      prose comments in `main.rs` and holds the rule by hand at each
      site"*) both named.
- [x] `node::live::clear` (the `--reset-node` inverse-of-checkpoint site
      `Mode::ResetNode` reaches) is unchanged by this spec: it is a single
      registry write with an explicit before/after receipt already, not a
      multi-step acquisition, and the PRD's own text names `boot_live` —
      not `clear` — as "the real inverse-of-a-checkpoint work."

## Verify and Proof

```sh
cd ../model && cargo build -p llm && cargo test -p llm --lib daemon::
```

## Evidence — implemented 2026-08-26

**Box 1 — what landed.** `boot` opens with `let scope = Scope::new();` and
registers three inverses via `Scope::effect` as each acquisition succeeds, in
order:
`data-dir`, `registry-file`, `identity-file`. On `Ok`, each `Disposer` is
`dispose()`d and `debug_assert!(scope.held().is_empty())` states the
post-condition.

**Box 1 — CLOSED 2026-08-26 by the user's answer, on three of its four named
acquisitions.** The question below was put to the user and answered: read the
box as naming the acquisitions rather than mandating a second teardown
mechanism per acquisition. The scope carries what `Drop` does not; the two
swarm listener binds stay off it. Registered:
the data directory (not named in the box, but the same shape), the registry
file, the identity file. NOT registered: the discovery listener bind and the
data-plane listener bind, which the box names explicitly. The wall is below,
under *the two swarms*; it is a decision, not a defect, and it is the user's,
so the box stays `[ ]` rather than ticked with an argument attached.

**Two readings recorded, because the box's letter and the crate's semantics
pull apart.**

1. *Why `dispose` needs a flag.* `Disposer::dispose` **runs** the inverse —
   it is not "discard the inverse unrun". Disposing on the success path
   as written would delete the data directory and the identity file of a
   daemon that had just booted. So each inverse closes over one shared
   `Arc<AtomicBool>` (`owed`), true while the boot is in flight and set false
   the moment `boot` has everything it came for. After that flip the closures
   are no-ops, and `dispose()` means what the box wants it to mean — *no
   longer this scope's business* — without leaking the scope to stop its
   `Drop` from unwinding. The alternative was `std::mem::forget(scope)`, which
   leaks an `Arc<Mutex<Inner>>` per boot and says nothing about intent.

2. *The two swarms — the wall.* The box lists "discovery listener bind" and
   "data-plane net listener bind". Two things stand in the way, one of
   judgement and one of the language:

   - **Redundant.** Their inverse — free the port — is what Rust's own `Drop`
     already does on the `?` path, exactly once: `discovery` and `net` are
     locals moved into `Routing::new`/`SeedLeech::build` only on the success
     path. A test of a scope-registered version could not tell the scope from
     the drop — it would pass identically with the scope removed, which is a
     check written from the answer.
   - **Not expressible as written.** `Undo` is `Box<dyn FnOnce() + Send>`, so
     an undo that frees a swarm's port must OWN the swarm — and `boot` moves
     the same swarm into `Routing::new` on the success path. The usual escape,
     `Arc<Mutex<Option<Discovery>>>` with the undo taking and dropping it,
     does not compile here: `discovery.next().await` is awaited between the
     bind and the hand-off, and a `std::sync::MutexGuard` is not `Send` across
     an await. Making it work means an async mutex and a restructure of
     `boot`'s event loop — well outside this spec's footprint, and a redesign
     of a boot path to satisfy a property `Drop` already guarantees.

   **Put to the user, since a spec is not a worker's to redefine, and
   ANSWERED 2026-08-26**: close box 1 on the three filesystem acquisitions.
   The `boot` restructure is not funded, here or as its own node. Recorded in
   full at the PRD's `## Answers — 2026-08-26`.

**Box 2 — the late failure, and the reverse order, both observed.**
`daemon::tests::boot_scope` forces `net.listen_on` to fail synchronously with
`/memory/1` (this daemon builds tcp and quic, so the swarm answers
`MultiaddrNotSupported`) — a failure at the LAST acquisition, after the
discovery listener has already bound.

`a_boot_that_fails_late_removes_what_it_created_in_reverse_order` asserts the
identity file, the registry file and the directory are all gone.
**The directory is the ordering, observed, not a second copy of the same
assertion**: `remove_dir` (not `remove_dir_all`) refuses a non-empty
directory, so the directory can only be gone if the two file inverses ran
first. Registration order would leave it standing.

That is not an argument — it is a run. An earlier draft of this
implementation registered only two effects, leaving `registry.redb` inside the
directory, and the test failed on exactly that line:

```
thread '...a_boot_that_fails_late_removes_what_it_created_in_reverse_order' panicked at
src/daemon/tests/boot_scope.rs:74:5:
and the directory it created was removed after it — reverse order
```

Two more tests keep it from passing for the wrong reason:

- `a_failed_boot_never_removes_an_identity_it_did_not_create` — a
  pre-existing identity file survives a failed boot byte for byte, and so
  does the directory `boot` did not create. An inverse that removed its path
  unconditionally fails here. This is why both effects are registered only
  when `!path.exists()` beforehand.
- `a_boot_that_succeeds_keeps_what_it_created` — a successful ephemeral boot
  still has its directory and its identity file afterwards. Without it, a
  `boot` that always deleted its own state would satisfy the other two.

**Box 3.** `main.rs`'s `--reset-node` comment now says DOGMA 13 is no longer
held by hand on this boot path, names `conserved::Scope` and spec04, and says
why this flag is still the inverse of a *checkpoint* — a different shape of
the same rule. The `learnings/inventory.md` and `p1-scope/prd.md` citation
("llm has no implementation — it cites DOGMA 13 in prose comments") is closed.

**Box 4.** `git diff --name-only src/node/live.rs` is empty. `node::live::clear`
is untouched.

**One thing found and not fixed, reported per the worker rules.**
`use std::time::SystemTime;` in `src/node/transactional.rs` became a non-test
import with no non-test user once `Commit::new` stopped reading the clock
directly; it is now `#[cfg(test)]` alongside `UNIX_EPOCH`. Inside spec02's
footprint, so fixed rather than reported.

## Verify — run 2026-08-26

```
$ cargo build -p llm && cargo test -p llm --lib daemon::
running 113 tests
test daemon::tests::boot_scope::a_failed_boot_never_removes_an_identity_it_did_not_create ... ok
test daemon::tests::boot_scope::a_boot_that_fails_late_removes_what_it_created_in_reverse_order ... ok
test daemon::tests::boot_scope::a_boot_that_succeeds_keeps_what_it_created ... ok
...
test result: ok. 113 passed; 0 failed; 0 ignored; 0 measured; 1463 filtered out; finished in 1.16s
```
