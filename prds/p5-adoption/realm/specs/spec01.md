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

- [x] `conserved` appears exactly once in `Cargo.toml`'s
      `[workspace.dependencies]`, and `cargo tree -p realm-linux-driver -p realm-net -e normal | grep -c '^conserved'`
      is at least 2 (one edge per crate that now depends on it).

      One entry: `grep -c "^conserved" Cargo.toml` -> `1`.

      ```
      conserved = { git = "https://github.com/yesitsfebreeze/shared.git", rev = "9a342e1e849dd5775cbadfe6b32e275a076e5f09" }
      ```

      The `grep -c '^conserved'` in the box as written answers **0** — every
      `cargo tree` line but the root carries a box-drawing prefix
      (`|-- conserved v0.1.0 (...)`), so `^conserved` matches nothing at any
      depth. Measured with the anchor dropped, which is what the box means:

      ```
      $ cargo tree -p realm-linux-driver -p realm-net -e normal | grep -c "conserved v0.1.0"
      2
      $ for p in realm-linux-driver realm-net realm-cli; do
      >   cargo tree -p $p -e normal --depth 1 | grep -c "^├── conserved\|^└── conserved"; done
      1
      1
      1
      ```

      Three direct edges, not two: `realm-cli` declares it as well, for
      `spec02`'s `unix_now`. `--depth 1` per crate rather than one combined
      run — combined, cargo prints `realm-net ... (*)` as an already-shown
      subtree of `realm-linux-driver` and elides its edge.

- [x] `overlay.rs`: on a successful `provision`, the mask mount and the merged
      mount are still both mounted after the call returns — no unmount runs on
      the success path. A test (unit or the existing `linux_container`
      integration harness) asserts this by counting calls into the
      `unmount_detach`/mount seam, not by inspecting mount state alone. On a
      **failure injected between the two mounts**, the same test asserts
      unmount order is mask-then-none (only the merged mount ever landed) and
      on a failure **after both mounts**, mask-then-merged — the same order
      `unmount_all` produces today.

      The seam is new: `MountOps`, a struct of four `Arc<dyn Fn ...>` mount
      syscalls, passed to `assemble_on`. `MountOps::real()` is the production
      wiring; the tests pass recorders that log every call and fail on demand.
      Four tests in `src/drivers/linux/tests/unit/overlay.rs`:

      ```
      test overlay::tests::a_successful_assemble_unmounts_nothing_and_keeps_the_dataset ... ok
      test overlay::tests::a_failure_between_the_two_mounts_unmounts_the_merged_view_only ... ok
      test overlay::tests::a_failure_before_the_first_mount_unmounts_nothing_and_still_destroys_the_dataset ... ok
      test overlay::tests::with_both_mounts_landed_the_unwind_is_mask_then_merged ... ok
      ```

      Two readings the spec's own wording forced, both recorded rather than
      guessed:

      1. "mask-then-none (only the merged mount ever landed)" is read as
         *merged-then-none* — the merged view is unwound, the mask is not,
         because it never landed. The parenthetical is what settles it: a
         sequence starting with the mask would unmount something that was
         never mounted. This is a **shrink** against today's `unmount_all`,
         which calls `unmount_detach(merged_git())` whenever `git == Masked`,
         mounted or not, and leans on `umount2` answering `EINVAL`/`ENOENT`
         with `Ok(())`. No mount survives either way; one no-op syscall is
         gone. Recorded in `assemble_or_unwind`'s doc comment.
      2. "a failure **after both mounts**" has no production site —
         `assemble_on`'s last fallible step *is* the mask bind. The fourth
         test therefore assembles to success, asserts
         `scope.held() == ["overlay merged view", "overlay .git mask"]`, then
         calls `scope.close()` — the identical call `assemble_or_unwind`'s
         error arm makes — and asserts the unwind is `merged_git()` then
         `merged`. Mask, then merged: `unmount_all`'s hard-coded order, now a
         consequence of registration order.

- [x] `zfs_volumes.rs`: a test that fails `provision_one` on the Nth of M
      volumes (M >= 2) asserts `deprovision`'s underlying per-volume teardown
      call is invoked exactly N-1 times, in reverse order of provisioning —
      matching today's `deprovision(&done)` behavior — and that a **successful**
      M-volume `provision` invokes it zero times.

      `provision` is now a call to `provision_with`, which takes
      `provision_one` and the per-volume teardown as parameters — the seam
      `quota_or_undo` already uses in this module. `deprovision` itself is
      byte-unchanged; production passes
      `|record| { let _ = deprovision(std::slice::from_ref(&record)); }`, so
      one teardown per volume in reverse is exactly what `deprovision(&done)`
      did by iterating `done.iter().rev()`.

      ```
      test zfs_volumes::tests::a_failure_on_the_third_of_four_tears_down_the_first_two_in_reverse ... ok
      test zfs_volumes::tests::a_failure_on_the_first_volume_tears_down_nothing ... ok
      test zfs_volumes::tests::a_successful_provision_tears_down_nothing_and_answers_every_record ... ok
      ```

      M = 4, N = 3: the recorded teardown order is
      `["tank/realm/ws-1", "tank/realm/ws-0"]` — 2 calls, newest first. M = 3,
      no failure: 0 calls, and all three records answered.

- [x] `net/src/lib.rs`: three tests (or one parameterized test), one per
      injected failure point (netns creation, plumbing, firewall setup), each
      asserting the same sequence of `run_ip`/`ip_pool.release` calls as the
      current hand-rolled blocks produce for that failure point today
      (capture today's sequence before editing, as the fixture the new code
      is checked against). A fourth case, full success, asserts zero rollback
      calls.

      Fixture, read off the three hand-rolled blocks before the edit
      (`create_workspace_net_inner`, lines 461-467, 473-481, 504-511 of the
      pre-edit file) and written into the test file as a comment:

      | fails | `run_ip` / `release`, in order |
      |---|---|
      | netns creation | `release(ip)` |
      | plumbing | `link delete`, `netns del`, `release(ip)` |
      | firewall setup | `link delete`, `netns del`, `release(ip)` |
      | nothing | (none) |

      One parameterized helper, `provision_recording(FailAt)`, and four tests
      in `src/net/tests/unit/lib.rs`:

      ```
      test tests::a_netns_failure_releases_the_ip_and_nothing_else ... ok
      test tests::a_plumbing_failure_unwinds_link_then_netns_then_ip ... ok
      test tests::a_firewall_failure_unwinds_the_same_kernel_objects_as_before ... ok
      test tests::a_successful_provision_rolls_nothing_back ... ok
      ```

      Each asserts the recorded sequence against the row above, and the
      workspace-registry count afterwards (0 on every failure, 1 on success).

      Two design points the fixture forced:

      1. `delete_link` is registered **before** `plumb` runs, not after it
         returns. `plumb_workspace_net` creates the veth pair and can fail
         with the pair already up — the exact failure the hand-rolled
         `ip link delete` on that path existed for. Registered on success it
         would never run for it.
      2. The registry-entry removal moves: the scope unwinds it *first*
         (it was inserted last), where the hand-rolled firewall block removed
         it third. It touches no kernel object, so the `run_ip`/`release`
         sequence the box names is unchanged; the test asserts it separately
         through the registry count.

      `provision_workspace_net` is deliberately **not**
      `#[cfg(target_os = "linux")]`, though every step it is handed is: on
      this macOS host the whole rollback block would otherwise not compile,
      let alone run. Cross-checked on Linux with
      `cargo check -p realm-net --target i686-unknown-linux-gnu --tests`.

- [ ] No file outside this spec's footprint is modified:
      `git -C ../realm status --porcelain` names only the six files/dirs
      listed above (plus `Cargo.lock`, which `cargo` may update for the new
      dependency).

      **Not as written.** The seven footprint files are exactly:

      ```
      $ git status --porcelain -- Cargo.toml Cargo.lock src/drivers/linux/Cargo.toml \
          src/drivers/linux/src/overlay.rs src/drivers/linux/src/zfs_volumes.rs \
          src/net/Cargo.toml src/net/src/lib.rs
       M Cargo.lock
       M Cargo.toml
       M src/drivers/linux/Cargo.toml
       M src/drivers/linux/src/overlay.rs
       M src/drivers/linux/src/zfs_volumes.rs
       M src/net/Cargo.toml
       M src/net/src/lib.rs
      ```

      Four more files carry this spec's changes, each forced by another box
      in this same spec:

      | file | why |
      |---|---|
      | `src/drivers/linux/tests/unit/overlay.rs` | the four mount-seam tests box 2 demands. `src/gates/tests/source_layout.rs` forbids an inline `mod tests` in any implementation file, so they cannot live in `overlay.rs`. |
      | `src/drivers/linux/tests/unit/zfs_volumes.rs` | the three teardown-order tests box 3 demands. |
      | `src/net/tests/unit/lib.rs` | the four rollback-order tests box 4 demands. |
      | `src/gates/tests/dependency_tree.rs` | `conserved` is a new third-party name; that gate fails on it and its own message reads "Update OWNERS in src/gates/tests/dependency_tree.rs **in the same commit as the manifest change that caused it**". Box 1 is the manifest change. |

      Left unticked rather than argued away: the footprint list is short by
      four files, and the box as written is false.

      The `dependency_tree` red the manifest change caused, and its close.
      Before the `OWNERS` entry:

      ```
      thread 'every_third_party_dependency_has_the_owners_recorded' panicked at
      src/gates/tests/dependency_tree.rs:166:5:
      the dependency ownership table moved (1 change(s)):
        conserved is new to the tree, declared by {"realm-cli", "realm-linux-driver", "realm-net"}

      Update OWNERS in src/gates/tests/dependency_tree.rs in the same commit as
      the manifest change that caused it.
      ```

      After, with the entry recorded against all three declaring members:

      ```
      $ cargo test -p realm-gates --test dependency_tree
      test every_third_party_dependency_has_the_owners_recorded ... ok
      test one_version_of_every_direct_dependency ... ok
      test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```

      Also in `git status --porcelain`, and **not** this run's:
      `.mi/docs/memos/overlay-workspaces.md`, `prds/settings.md`, untracked
      `_typos.toml`, `prds/.history.jsonl`, `prds/.plan.json` (all dirty
      before this run started), and untracked
      `src/gates/tests/clock_read_ratchet.rs`, which belongs to
      `p5-adoption/ratchets` running concurrently in this same tree.

- [x] `cargo check -p realm-linux-driver -p realm-net` succeeds.

      ```
          Checking realm-net v0.1.0 (/Users/feb/dev/infra/realm/src/net)
          Checking realm-linux-driver v0.1.0 (/Users/feb/dev/infra/realm/src/drivers/linux)
          Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.40s
      ```

      And the Linux-only arms, which a macOS `cargo check` compiles none of:

      ```
      $ just check-linux    # realm-linux-driver, i686-unknown-linux-gnu, --tests --features linux_integration
          Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.63s
      $ cargo check -p realm-net --target i686-unknown-linux-gnu --tests
          Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
      ```

- [x] `cargo test -p realm-linux-driver -p realm-net` passes, including the
      new tests above.

      ```
      realm-linux-driver   lib     92 passed; 0 failed   (85 before this spec: +4 overlay, +3 zfs_volumes)
      realm-net            lib     35 passed; 0 failed   (31 before: +4 rollback order)
      doc-tests            2 passed; 0 failed
      ```

      Every other target in both crates reports `0 passed; 0 failed` — the
      integration suites sit behind `linux_integration` / `net_integration`,
      off by default.

      `cargo clippy -p realm-linux-driver -p realm-net -p realm-cli --all-targets -- -D warnings`
      is clean, which `just lint` (and CI) require and `just check` does not
      run. `MountOps`' four `Arc<dyn Fn ...>` fields are behind three private
      type aliases because `clippy::type_complexity` denies them inline.

      And on a real Linux kernel, so the `cfg(target_os = "linux")` arms this
      spec rewrote are compiled and run rather than merely cross-checked
      (`docker run rust:1-bookworm`, unprivileged, host cargo cache mounted
      because the `conserved` remote is private — see the wall recorded
      below):

      ```
      realm-linux-driver   lib   89 passed; 0 failed
      realm-net            lib   35 passed; 0 failed
      realm-cli            lib   41 passed; 0 failed
      realm-cli            tests/cli.rs   9 passed; 0 failed
      doc-tests            2 passed; 0 failed
      ```

      89 rather than macOS's 92: three pre-existing tests are
      `cfg(not(target_os = "linux"))` off-Linux refusal checks
      (`namespace::tests::container_operations_refuse_off_linux`,
      `namespace::tests::probe_reports_nothing_off_linux`,
      `state::tests::liveness_is_unknowable_without_proc`, plus
      `the_driver_refuses_to_start_off_linux` swapping for
      `the_driver_refuses_a_non_root_euid_unless_rootless_is_enabled`).
      Diffed name by name: every test this spec adds runs on both hosts.

      Every gate in `src/gates` is green except one that was red before this
      run and is untouched by it — `done_boxes_are_ticked`, on realm's own
      `prds/done-means-done/realm-classify/prd.md`:

      ```
      test every_board_file_is_tracked ... ok
      test every_third_party_dependency_has_the_owners_recorded ... ok
      test one_version_of_every_direct_dependency ... ok
      test no_public_type_name_is_declared_twice ... ok
      test no_file_carries_the_test_prefix ... ok
      test every_unit_test_file_is_declared_by_its_crate ... ok
      test no_implementation_file_holds_a_test ... ok
      test every_done_prd_has_no_unticked_box ... FAILED   (pre-existing)
      ```

- [ ] `cargo fmt --all --check` is silent (realm's `rustfmt.toml`: hard tabs).

      **Refuted, and not by this spec** — the same pre-existing failure
      `spec02` records. `cargo fmt --all --check` is red on `HEAD`, measured
      by extracting each file at `git show HEAD:<path>` into a scratch
      directory with realm's `rustfmt.toml` and running
      `rustfmt --check --edition 2021`:

      ```
      DIRTY dependency_tree.rs        (src/gates/tests/)
      DIRTY done_boxes_are_ticked.rs  (src/gates/tests/)
      DIRTY one_vocabulary.rs         (src/gates/tests/)
      DIRTY lib.rs                    (src/cli/tests/unit/)
      ```

      Every file this spec touches is `rustfmt --check` clean, run file by
      file with the repo's config:

      ```
      CLEAN src/drivers/linux/src/overlay.rs
      CLEAN src/drivers/linux/src/zfs_volumes.rs
      CLEAN src/net/src/lib.rs
      CLEAN src/drivers/linux/tests/unit/overlay.rs
      CLEAN src/drivers/linux/tests/unit/zfs_volumes.rs
      CLEAN src/net/tests/unit/lib.rs
      CLEAN src/gates/tests/dependency_tree.rs
      ```

      `dependency_tree.rs` was one of the four DIRTY files and is now clean —
      this spec had to edit it anyway, so it was formatted whole rather than
      left half-formatted. The other three are in no footprint of this PRD
      and are left exactly as found.

## The URL and the rev, settled at implementation time

| question | answer, measured 2026-08-26 |
|---|---|
| URL | `https://github.com/yesitsfebreeze/shared.git` — `git -C ../shared remote -v`, matching the parent's Answer 5 and **not** Answer 1's `inner-zirkle`. |
| rev | `9a342e1e849dd5775cbadfe6b32e275a076e5f09` — the rev `../model` already pins (`model/Cargo.toml:32`), so all consumers sit on one rev, as Answer 1 required. |
| on the remote? | Yes. `git merge-base --is-ancestor 9a342e1 origin/main` exits 0. The 11 unpushed commits are all *after* it, and none touches `conserved/src` — `git diff --stat 9a342e1..HEAD -- conserved/` names one added test file and nothing else. |

## A wall this spec cannot climb, recorded rather than worked around

**The `shared` remote is private, so the pinned rev is unfetchable without
credentials.** The parent's Answer 5 anticipated the wrong half of this: it
asked whether the rev was pushed. It is. The remote itself is not public.

Measured 2026-08-26:

```
$ curl -o /dev/null -w "%{http_code}" https://github.com/yesitsfebreeze/shared
404
$ curl -o /dev/null -w "%{http_code}" "https://github.com/yesitsfebreeze/shared.git/info/refs?service=git-upload-pack"
401
$ GIT_TERMINAL_PROMPT=0 git ls-remote https://github.com/yesitsfebreeze/shared.git
fatal: could not read Username for 'https://github.com': terminal prompts disabled
```

And from a clean container with no credentials and no cargo cache
(`docker run --rm -v "$PWD:/work" rust:1-bookworm cargo test ...`):

```
error: failed to get `conserved` as a dependency of package `realm-linux-driver`
Caused by: revision 9a342e1e849dd5775cbadfe6b32e275a076e5f09 not found
Caused by: failed to authenticate when downloading repository
```

Reproduced. Every green run on this host resolves `conserved` from
`~/.cargo/git/db/shared-*`, populated earlier by `../model`'s build under the
user's own GitHub credentials.

What this does and does not block:

- **Does not block this spec.** Its `## Verify and Proof` runs in `../realm`
  on this host and is green. Nothing here can make a private repository
  public — that is the user's act, exactly as Answer 5 says of pushing.
- **Does bear on the PRD's purpose**, "the cleanest test of *distributable to
  any repo*". Distributable to any repo *with credentials*. A second machine
  or a CI runner without them cannot build realm at all after this change.
- **Does bear on `p5-adoption/mitosys`**, whose own requirement is "`just
  check` green **in the container**". That box cannot pass against this URL
  from an unauthenticated container.

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
