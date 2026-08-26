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

- [ ] `boot`'s fallible resource-acquisition sequence (registry open,
      identity load/create, discovery listener bind, data-plane net
      listener bind) registers each successfully-acquired resource's
      teardown on a `conserved::Scope` as it happens, via `Scope::effect`.
      The scope is disposed (`Disposer::dispose`), not unwound, once `boot`
      returns `Ok`.
- [ ] A test forces a late step to fail (e.g. an already-bound or otherwise
      unbindable data-plane listen address, after the discovery listener
      has already bound successfully) and asserts the earlier step's
      registered inverse ran, in reverse order — via `conserved::Scope`'s
      `held()`/`failed()` or an observable side effect (the discovery
      listener's port becoming free again, or an explicit call counter in
      the registered `Undo`).
- [ ] `main.rs:126`'s comment no longer frames the rule as held "by hand":
      it names `conserved::Scope` as the type now carrying DOGMA 13 at this
      boot path, closing the citation `learnings/inventory.md` and
      `p1-scope/prd.md` (*"llm has no implementation — it cites DOGMA 13 in
      prose comments in `main.rs` and holds the rule by hand at each
      site"*) both named.
- [ ] `node::live::clear` (the `--reset-node` inverse-of-checkpoint site
      `Mode::ResetNode` reaches) is unchanged by this spec: it is a single
      registry write with an explicit before/after receipt already, not a
      multi-step acquisition, and the PRD's own text names `boot_live` —
      not `clear` — as "the real inverse-of-a-checkpoint work."

## Verify and Proof

```sh
cd ../model && cargo build -p llm && cargo test -p llm --lib daemon::
```
