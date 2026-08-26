---
complexity: 45          # the Drop/forget design decision and 3 call sites across 2 crates
footprint:
  - Cargo.toml
  - src/drivers/linux/Cargo.toml
  - src/drivers/linux/src/overlay.rs
  - src/drivers/linux/src/zfs_volumes.rs
  - src/net/Cargo.toml
  - src/net/src/lib.rs
---
<!-- footprint paths are relative to ../realm (this PRD's repo), matching the
     PRD's own frontmatter convention. -->

# spec01 — replace realm's three hand-rolled reverse-order undos with `conserved::Scope`

Adopt `conserved::scope::Scope` at realm's three real hand-rolled
LIFO-undo-on-failure sites, deleting the manual accumulator/rollback code each
one currently hand-rolls. This is the whole of the PRD's `Scope` requirement;
the `Clock` requirement is `spec02` and the `ContentId`/`stats` refusal record
is `spec03` — independent units, any order.

## The PRD's line numbers are stale — corrected here

The PRD cites `overlay.rs` line 385, `zfs_volumes.rs:139` and `net/src/lib.rs:437`.
Verified against the tree at analysis time, all three are off:

- `overlay.rs`'s `unmount_all` is at **line 395**, not 385 (`fn unmount_all` /
  the `/// Undo every mount [assemble] made, innermost first.` doc comment
  immediately above it).
- `zfs_volumes.rs:139` is inside `quota_bytes`'s doc comment (`u64::MAX quota`
  paragraph) — unrelated to undo. The real "Leave nothing behind: undo this
  call's own work" comment the PRD quotes is at **line 204**, inside
  `provision`.
- `net/src/lib.rs:437` is mid-way through IP-pool allocation in
  `create_workspace_net_inner`, not an undo site. The function's three actual
  hand-rolled rollback blocks are at **lines 461-467** (netns-creation
  failure: release the IP), **473-481** (plumbing failure: delete the iface,
  delete the netns, release the IP) and **504-511** (firewall-setup failure:
  delete the iface, delete the netns, drop the in-memory entry, release the
  IP) — one function, three nested LIFO undo points, which is the single site
  the requirement names.

Use the corrected locations below; do not re-derive from the PRD's numbers.

## The three sites

1. **`src/drivers/linux/src/overlay.rs`** — `assemble` (line 363) mounts the
   merged overlay, then optionally bind-mounts the `.git` mask. `unmount_all`
   (line 395) reverses that by hand: unmount the mask first (if present), then
   the merged view. Fixed two-step order, called both from `provision`'s
   error path (`let _ = unmount_all(&record);`) and from `rollback`/`destroy`
   elsewhere in the crate — check both callers before deleting `unmount_all`,
   since it is `pub(crate)`-visible beyond `provision`.
2. **`src/drivers/linux/src/zfs_volumes.rs`** — `provision` (line 192) loops
   over volumes, accumulating `done: Vec<ZfsVolumeRecord>`; on any
   `provision_one` error it calls `deprovision(&done)` and returns. Dynamic
   N-step LIFO undo — the cleanest fit of the three.
3. **`src/net/src/lib.rs`** — `create_workspace_net_inner` (line 404) builds,
   in order: IP allocation, netns creation, veth/plumbing, firewall setup.
   Each of the three error paths after IP allocation manually re-issues
   `ip_pool.release`, `run_ip(["netns","del",…])`, `run_ip(["link","delete",…])`
   for everything allocated so far — three copies of the same reverse-order
   logic, one per failure point.

## The Drop-semantics decision this spec must make

`conserved::Scope` (`conserved/src/scope.rs` in `../shared`) has **no
"commit" or "detach all" method**. `Scope::effect` returns a `Disposer` whose
`.dispose()` *runs* that one inverse immediately; `Scope`'s `Drop` always
calls `close()`, which unwinds every still-live effect. There is no API that
lets a caller keep everything registered and prevent the eventual `Drop` from
undoing it.

All three call sites need exactly that: undo everything registered *by this
call* if the call fails, keep everything if it succeeds, and the kept
resources must survive the function returning (state is persisted to disk
between separate CLI invocations — a `Scope`'s `Undo` closures cannot survive
a process exit regardless, so nothing here needs the `Scope` itself to
outlive the function).

**Decision: the `Scope` is local to each function, closed explicitly on the
error path, and `std::mem::forget`-ed on the success path.** Concretely:

```rust
let scope = conserved::scope::Scope::new();
// ... scope.effect("label", || { do_thing(); Box::new(move || undo_thing()) })? ...
match result {
    Ok(value) => {
        std::mem::forget(scope); // nothing left to undo; the Arc<Mutex<Inner>> leaks, once, harmlessly
        Ok(value)
    }
    Err(e) => {
        scope.close(); // explicit — do not rely on Drop for the error path either, for the same reason spec02 does not rely on implicit calls
        Err(e)
    }
}
```

This is a legitimate use of the existing API but it is not the API's own
happy path (mitosys's only production usage, `api/plugin/plugin.rs:469`,
stores the `Scope` as a long-lived field and lets its natural `Drop` do the
work — it never forgets). Flag this in the module-level comment at each of
the three call sites as "leaked deliberately — see `conserved::scope::Scope`,
no commit/detach-all primitive exists" so a later reader does not mistake it
for a bug. **Do not modify `conserved` to add such a primitive** — that
crate is `p1-scope`'s footprint, not this PRD's; if the awkwardness is worth
fixing, that is a finding for `p1-scope`, not a change made here.

## Files

- `Cargo.toml` (root) — add `conserved = { git = "<url>", rev = "<sha>" }` to
  `[workspace.dependencies]`. **Do not hardcode a URL from this spec.** The
  parent `p5-adoption/prd.md`'s `## Answers` names
  `https://github.com/inner-zirkle/shared`; `../shared`'s actual `origin`
  remote at analysis time was `https://github.com/yesitsfebreeze/shared.git`
  (`git -C ../shared remote -v`) — a mismatch, unresolved. Re-check
  `git -C ../shared remote -v` and the parent PRD's `## Answers` at
  implementation time and use whichever URL is then current; do not proceed
  on a guess.
- `src/drivers/linux/Cargo.toml`, `src/net/Cargo.toml` — add
  `conserved.workspace = true` to `[dependencies]`.
- `src/drivers/linux/src/overlay.rs` — `assemble`/`unmount_all` become one
  `Scope`-based flow per the decision above; the fixed two-step order (mask,
  then merged) is preserved by registering the merged mount first and the
  mask bind second, so `close()`'s LIFO unwind does mask-then-merged, same as
  today.
- `src/drivers/linux/src/zfs_volumes.rs` — `provision` registers each
  `provision_one` success on a `Scope` instead of pushing to `done: Vec<_>`;
  `deprovision` itself is **unchanged** (it is still the public teardown path
  called later, from persisted `ZfsVolumeRecord`s, by code outside
  `provision`).
- `src/net/src/lib.rs` — `create_workspace_net_inner`'s three duplicated
  rollback blocks collapse into one `Scope` that registers IP-allocation,
  netns-creation and plumbing as effects, in that order; firewall setup is
  the fourth (final) effect. Any error after registration calls
  `scope.close()` before returning; success forgets the scope per the
  decision above.

Not touched: any other file in `src/drivers/linux/`, `src/net/`, `src/cli/`,
`src/zfs/`, `src/ssh/`, `src/core/`, or anything under `../mitosys`,
`../model`, or `../shared/conserved`.

## Acceptance

- [ ] `conserved` appears exactly once in `Cargo.toml`'s
      `[workspace.dependencies]`, and `cargo tree -p realm-linux-driver -p realm-net -e normal | grep -c '^conserved'`
      is at least 2 (one edge per crate that now depends on it).
- [ ] `overlay.rs`: on a successful `provision`, the mask mount and the merged
      mount are still both mounted after the call returns — no unmount runs on
      the success path. A test (unit or the existing `linux_container`
      integration harness) asserts this by counting calls into the
      `unmount_detach`/mount seam, not by inspecting mount state alone. On a
      **failure injected between the two mounts**, the same test asserts
      unmount order is mask-then-none (only the merged mount ever landed) and
      on a failure **after both mounts**, mask-then-merged — the same order
      `unmount_all` produces today.
- [ ] `zfs_volumes.rs`: a test that fails `provision_one` on the Nth of M
      volumes (M ≥ 2) asserts `deprovision`'s underlying per-volume teardown
      call is invoked exactly N-1 times, in reverse order of provisioning —
      matching today's `deprovision(&done)` behavior — and that a **successful**
      M-volume `provision` invokes it zero times.
- [ ] `net/src/lib.rs`: three tests (or one parameterized test), one per
      injected failure point (netns creation, plumbing, firewall setup), each
      asserting the same sequence of `run_ip`/`ip_pool.release` calls as the
      current hand-rolled blocks produce for that failure point today
      (capture today's sequence before editing, as the fixture the new code
      is checked against). A fourth case, full success, asserts zero rollback
      calls.
- [ ] No file outside this spec's footprint is modified:
      `git -C ../realm status --porcelain` names only the six files/dirs
      listed above (plus `Cargo.lock`, which `cargo` may update for the new
      dependency).
- [ ] `cargo check -p realm-linux-driver -p realm-net` succeeds.
- [ ] `cargo test -p realm-linux-driver -p realm-net` passes, including the
      new tests above.
- [ ] `cargo fmt --all --check` is silent (realm's `rustfmt.toml`: hard tabs).

## Verify and Proof

```sh
cd ../realm
cargo check -p realm-linux-driver -p realm-net
cargo test -p realm-linux-driver -p realm-net
cargo fmt --all --check
git status --porcelain -- Cargo.toml Cargo.lock src/drivers/linux/Cargo.toml \
  src/drivers/linux/src/overlay.rs src/drivers/linux/src/zfs_volumes.rs \
  src/net/Cargo.toml src/net/src/lib.rs
# The line above should list every touched file; a `git status --porcelain`
# with no path filter should print nothing else.
git status --porcelain
```
