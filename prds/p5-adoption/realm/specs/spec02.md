---
complexity: 20          # 3 surgical call-through swaps, no signature/persisted-format change
footprint:
  - Cargo.toml
  - src/drivers/linux/Cargo.toml
  - src/drivers/linux/src/state.rs
  - src/cli/Cargo.toml
  - src/cli/src/lib.rs
  - src/cli/src/state.rs
---
<!-- footprint paths are relative to ../realm (this PRD's repo), matching the
     PRD's own frontmatter convention. -->

# spec02 — route realm's three non-test wall-clock reads through `conserved::Clock`

Replace the three places realm reads `SystemTime::now()` directly with a read
through `conserved::{Clock, SystemClock}`, changing no public signature and no
persisted field's type or unit — realm's `created_at`/`age_secs`/`uptime_secs`
stay `u64` unix seconds on disk, so there is no wire-format break of the kind
`llm` and `mitosys` have. This is the whole of the PRD's `Clock` requirement;
`spec01` is `Scope`, `spec03` is the `ContentId`/`stats` refusal record —
independent units, any order.

## The PRD's line numbers do not point at wall-clock reads — corrected here

The PRD names `drivers/linux/src/state.rs:140`, `cli/src/lib.rs:1025` and
`cli/src/state.rs:61` as the three reads. Verified against the tree:

- `state.rs:140` is real but is a **caller** of the actual read
  (`age_secs`'s `unix_now().saturating_sub(...)`), not the read itself. The
  read is at **`state.rs:202`**, inside `pub fn unix_now()`.
- `cli/src/lib.rs:1025` is inside `handle_destroy`'s ssh-driver-deprovision
  branch — no time-related code at all. It is unrelated to any clock read.
- `cli/src/state.rs:61` is a doc comment on the `ssh: Option<String>` field —
  also unrelated. The actual read is at **`cli/src/state.rs:91`**, inside
  `WorkspaceEntry::uptime_secs`.

The PRD's *count* (3 non-test reads) is correct; only the locations are
stale. Confirmed by grepping `../realm/src` for
`SystemTime::now\|Utc::now\|Local::now\|unix_now` and `Instant::now`: exactly
three non-test `SystemTime::now()` calls exist
(`drivers/linux/src/state.rs:202`, `cli/src/lib.rs:1331`,
`cli/src/state.rs:91`), and every `Instant::now()` call
(`net/src/dns.rs`, `drivers/linux/src/lib.rs`, `zfs/src/cli.rs`,
`ssh/src/driver.rs`, plus test files) is monotonic — none of those convert.
The PRD's own "must NOT be converted" examples are also mis-cited
(`zfs/src/cli.rs:437` is 2 lines off — the real deadline read is at line 439;
`drivers/linux/src/lib.rs:1038-1054` names a range with no timeout in it —
the real timeout logic is at lines 1285 and 1301). Not this spec's job to fix
those citations; noted so the next reader does not re-litigate by line number
alone — re-derive from the greps above.

## The three sites, and the minimal-footprint design

Two of the three reads already sit behind a private wrapper function that
every non-test call site goes through (`unix_now()`, defined once per crate).
Converting only the wrapper's body — not its signature, not its callers —
satisfies "route through `Clock`" with the smallest possible footprint: zero
changes at `state.rs:140`, `drivers/linux/src/lib.rs:956`, `cli/lib.rs:632`,
`cli/lib.rs:1206`, `cli/lib.rs:1229` (all unchanged, still `unix_now()`), and
zero changes to any test (the four test-side `unix_now()` calls in
`drivers/linux/tests/` are untouched).

1. **`src/drivers/linux/src/state.rs:201-206`**:
   ```rust
   pub fn unix_now() -> u64 {
       SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
   }
   ```
   becomes
   ```rust
   pub fn unix_now() -> u64 {
       conserved::SystemClock.now().as_unix_secs().max(0) as u64
   }
   ```
   `Instant::as_unix_secs` returns `i64` (floor division, can be negative
   pre-1970); `unix_now`'s contract is `u64`, so clamp at 0 the same way the
   old `.unwrap_or(0)` already treated "before the epoch" as "unset/zero" —
   do not let a negative `i64` wrap through `as u64`.
2. **`src/cli/src/lib.rs:1330-1333`** — the same transformation, same clamp,
   on `cli`'s private `unix_now()`.
3. **`src/cli/src/state.rs:88-95`** — `uptime_secs` has no wrapper; its own
   body's `SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)`
   is replaced the same way, inline.

## Files

- `Cargo.toml` (root) — add `conserved` to `[workspace.dependencies]` if
  `spec01` has not already landed it (same entry either way — do not add it
  twice; if the key exists, this spec touches nothing here). Same
  URL/rev caveat as `spec01`: do not hardcode a guess, re-check
  `git -C ../shared remote -v` and the parent PRD's `## Answers` at
  implementation time.
- `src/drivers/linux/Cargo.toml`, `src/cli/Cargo.toml` — add
  `conserved.workspace = true` to `[dependencies]` if not already present
  (`spec01` adds it to `src/drivers/linux/Cargo.toml` too — idempotent, do
  not duplicate the line).
- `src/drivers/linux/src/state.rs`, `src/cli/src/lib.rs`,
  `src/cli/src/state.rs` — the three read-site bodies, per above.

Not touched: any call site of `unix_now()`, any `Instant::now()` site, any
other file.

## Acceptance

- [x] `grep -c "SystemTime::now" src/drivers/linux/src/state.rs
      src/cli/src/lib.rs src/cli/src/state.rs` is `0` for all three files —
      the direct reads are gone.

      ```
      src/drivers/linux/src/state.rs:0
      src/cli/src/lib.rs:0
      src/cli/src/state.rs:0
      ```

- [x] `grep -c "conserved::SystemClock\|SystemClock" src/drivers/linux/src/state.rs
      src/cli/src/lib.rs src/cli/src/state.rs` is at least 1 for each.

      ```
      src/drivers/linux/src/state.rs:2
      src/cli/src/state.rs:1
      src/cli/src/lib.rs:2
      ```

- [x] The five non-test callers of `unix_now()`
      (`state.rs:140`, `drivers/linux/src/lib.rs:956`, `cli/lib.rs:632`,
      `cli/lib.rs:1206`, `cli/lib.rs:1229`) and `uptime_secs`'s own callers
      are **byte-unchanged**: `git diff --unified=0 -- src/drivers/linux/src/state.rs`
      touches only the `unix_now` function body (not `age_secs`), and
      `git diff --unified=0 -- src/cli/src/lib.rs` touches only the
      `unix_now` function body (not any of its three call sites).

      `git diff --unified=0` names four hunks in `state.rs` — the dropped
      `use std::time::{SystemTime, UNIX_EPOCH};`, the added
      `use conserved::Clock;`, the added doc comment at `@@ -200,0 +201,10 @@`
      and the body at `@@ -202,4 +212 @@ pub fn unix_now()`. No hunk touches
      `age_secs`. In `cli/src/lib.rs`, two hunks:
      `@@ -1329,0 +1330,5 @@ fn valid_workspace_name` (the doc comment) and
      `@@ -1331,4 +1336,2 @@ fn unix_now()` (the body). Lines 632, 1206 and
      1229 are in no hunk.

- [x] `grep -rn "Instant::now()" src/` (across the whole realm tree) returns
      the exact same set of file:line pairs before and after this spec —
      `net/src/dns.rs` (4), `drivers/linux/src/lib.rs` (4, non-test),
      `zfs/src/cli.rs` (2), `ssh/src/driver.rs` (1, test), plus the test
      files named in the grep at analysis time. None of these become
      `conserved::Instant`.

      ```
      src/net/tests/unit/dns.rs:175   src/net/src/dns.rs:80,94,105
      src/drivers/linux/tests/overlay.rs:224
      src/drivers/linux/tests/linux_container.rs:966
      src/drivers/linux/src/lib.rs:970,973,1084,1285,1301
      src/zfs/tests/zfs_integration.rs:166
      src/zfs/tests/unit/cli.rs:337,361,377,417,427,476
      src/zfs/src/cli.rs:439,449
      src/ssh/tests/unit/driver.rs:148
      ```

      Not one of those files appears in `git status --porcelain`, so the set
      is unchanged by construction, not by re-derivation. Two corrections to
      the analysis-time list, both bookkeeping: `net/src/dns.rs` holds 3, not
      4 (the 4th "dns" site is `net/tests/unit/dns.rs:175`), and
      `drivers/linux/src/lib.rs` holds 5 non-test sites, not 4.

- [x] A test asserting `unix_now()` (both crates) and `uptime_secs()` still
      return a plausible current-unix-seconds value (e.g. within a wide
      sanity window of the test's own wall-clock read) — a regression guard
      against the `as u64` clamp silently returning `0` for all normal-range
      dates.

      Three tests, each also asserting `!= 0` separately from the window, so
      a fired clamp is reported as the epoch rather than as "off by 1.7e9":

      ```
      test state::tests::unix_now_is_current_wall_clock_seconds ... ok   (realm-linux-driver)
      test tests::unix_now_is_current_wall_clock_seconds ... ok          (realm-cli)
      test state::tests::uptime_secs_counts_from_created_at ... ok       (realm-cli)
      ```

- [x] `cargo check -p realm-linux-driver -p realm-cli` succeeds.

      ```
          Checking realm-linux-driver v0.1.0 (/Users/feb/dev/infra/realm/src/drivers/linux)
          Checking realm-cli v0.1.0 (/Users/feb/dev/infra/realm/src/cli)
          Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.47s
      ```

- [x] `cargo test -p realm-linux-driver -p realm-cli` passes.

      ```
      realm-cli            lib   41 passed; 0 failed
      realm-cli            tests/cli.rs   9 passed; 0 failed
      realm-linux-driver   lib   85 passed; 0 failed
      doc-tests            2 passed; 0 failed
      ```

      Every other target in both crates reports `0 passed; 0 failed` — the
      integration suites are behind `linux_integration` / `zfs_integration` /
      `ssh_integration`, off by default.

- [ ] `cargo fmt --all --check` is silent.

      **Refuted, and not by this spec.** `cargo fmt --all --check` is red on
      `HEAD` before any file here was touched. Measured by extracting the
      four offending files at `git show HEAD:<path>` into a scratch directory
      with realm's own `rustfmt.toml` and running
      `rustfmt --check --edition 2021`:

      ```
      DIRTY dependency_tree.rs        (src/gates/tests/)
      DIRTY done_boxes_are_ticked.rs  (src/gates/tests/)
      DIRTY one_vocabulary.rs         (src/gates/tests/)
      DIRTY lib.rs                    (src/cli/tests/unit/)
      ```

      None of the four is in this spec's footprint and three are in no
      footprint of this PRD at all. Every file this spec *does* touch is
      `rustfmt --check` clean, run file by file with the repo's config:

      ```
      CLEAN src/drivers/linux/src/state.rs
      CLEAN src/cli/src/state.rs
      CLEAN src/cli/src/lib.rs          (its only diff is the #[path]-included pre-existing test file)
      CLEAN src/drivers/linux/tests/unit/state.rs
      CLEAN src/cli/tests/unit/state.rs
      ```

      Left unticked rather than tightened: the box as written cannot pass
      without editing files this spec must not edit.

## Verify and Proof

```sh
cd ../realm
cargo check -p realm-linux-driver -p realm-cli
cargo test -p realm-linux-driver -p realm-cli
cargo fmt --all --check
for f in src/drivers/linux/src/state.rs src/cli/src/lib.rs src/cli/src/state.rs; do
  grep -q "SystemTime::now" "$f" && { echo "FAIL: $f still reads SystemTime::now directly"; exit 1; }
done
echo "instant sites (must be unchanged from the analysis-time list):"
grep -rn "Instant::now()" src/
git status --porcelain -- Cargo.toml Cargo.lock src/drivers/linux/Cargo.toml \
  src/drivers/linux/src/state.rs src/cli/Cargo.toml src/cli/src/lib.rs src/cli/src/state.rs
git status --porcelain
```

## Footprint deviation — three files, each forced by an acceptance box

| file | why |
|---|---|
| `src/drivers/linux/tests/unit/state.rs` | the `unix_now` regression guard the fifth box demands. `src/gates/tests/source_layout.rs` forbids an inline `mod tests` in an implementation file, so the test cannot live in `state.rs` itself. |
| `src/cli/tests/unit/lib.rs` | the same guard for `cli`'s private `unix_now`, which only a `#[path]`-included module can see. |
| `src/cli/tests/unit/state.rs` | the `uptime_secs` guard. |

`src/gates/tests/dependency_tree.rs` is a fourth, shared with `spec01` and
recorded there — `conserved` is a new third-party name and that gate's own
failure message orders it updated in the same commit as the manifest change.
