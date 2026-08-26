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

- [ ] `grep -c "SystemTime::now" src/drivers/linux/src/state.rs
      src/cli/src/lib.rs src/cli/src/state.rs` is `0` for all three files —
      the direct reads are gone.
- [ ] `grep -c "conserved::SystemClock\|SystemClock" src/drivers/linux/src/state.rs
      src/cli/src/lib.rs src/cli/src/state.rs` is at least 1 for each.
- [ ] The five non-test callers of `unix_now()`
      (`state.rs:140`, `drivers/linux/src/lib.rs:956`, `cli/lib.rs:632`,
      `cli/lib.rs:1206`, `cli/lib.rs:1229`) and `uptime_secs`'s own callers
      are **byte-unchanged**: `git diff --unified=0 -- src/drivers/linux/src/state.rs`
      touches only the `unix_now` function body (not `age_secs`), and
      `git diff --unified=0 -- src/cli/src/lib.rs` touches only the
      `unix_now` function body (not any of its three call sites).
- [ ] `grep -rn "Instant::now()" src/` (across the whole realm tree) returns
      the exact same set of file:line pairs before and after this spec —
      `net/src/dns.rs` (4), `drivers/linux/src/lib.rs` (4, non-test),
      `zfs/src/cli.rs` (2), `ssh/src/driver.rs` (1, test), plus the test
      files named in the grep at analysis time. None of these become
      `conserved::Instant`.
- [ ] A test asserting `unix_now()` (both crates) and `uptime_secs()` still
      return a plausible current-unix-seconds value (e.g. within a wide
      sanity window of the test's own wall-clock read) — a regression guard
      against the `as u64` clamp silently returning `0` for all normal-range
      dates.
- [ ] `cargo check -p realm-linux-driver -p realm-cli` succeeds.
- [ ] `cargo test -p realm-linux-driver -p realm-cli` passes.
- [ ] `cargo fmt --all --check` is silent.

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
