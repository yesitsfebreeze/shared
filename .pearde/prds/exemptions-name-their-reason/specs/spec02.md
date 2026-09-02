---
complexity: 10
footprint:
  - prds/p5-adoption/ratchets/prd.md
  - prds/p5-adoption/realm/prd.md
  - prds/exemptions-name-their-reason/prd.md
---

# spec02 — seven boxes struck with measured reasons, `EXEMPT` still `&[]`

Job 2 of the PRD. spec01's gate is red on exactly two rows; this spec closes
them per box, in the form `shared/learnings/done-means-done.md` names, and
gives `EXEMPT` no entry. Land it in the same commit as spec01 if the red
window is unwanted.

Every reason below was **measured, not asserted**. The implementer re-runs each
measurement and quotes its own output beside the box it closes.

## What already stands

**Nothing in `prds/` has been touched.** The strikes are the implementer's
edit; what is done is the evidence-gathering, and the closure was proved
without landing it — see *The closure is proved* below.

`conserved/tests/done_boxes_are_ticked.rs` from spec01 is in the tree.

## The spelling — a correction to the PRD's own body

The PRD's § Job 2 says of the ratchets boxes: *"The box stays visible as
`- [ ]` with its reason, per `done-means-done`."* **That instruction is
`refuted` and must not be followed.** `../prds/memos/struck-box-spelling.md`
(user decision, 2026-08-27) settles the spelling after `done-means-done` was
written: *"`- [ ]` means open. `- [x]` means closed on evidence."* A box left
`- [ ]` with a note beneath it is still an open box to every counter in the
family, including the one spec01 lands — so following the PRD's sentence
literally would leave the gate red on the same two rows it is red on now. The
PRD's own § Acceptance boxes 4 and 5 say *"struck with a reason"*, which is
`- [~]`, and § Job 1's table already records `- [~]` as a closure. The body
sentence is the stale half of its own file.

**The edit, per box: change the marker `- [ ]` to `- [~]` and append the reason
as an indented paragraph beneath the box's existing text.** The wording of each
box is not rewritten and no number inside a box is edited to match a
measurement — a strike records that the bar was not cleared here, it does not
move the bar.

## The closure is proved

Measured 2026-08-28. All seven markers were rewritten `- [ ]` → `- [~]` with no
other edit, and the gate went from two rows red to green:

```
=== before ===
  prds/p5-adoption/ratchets/prd.md: 4 unticked box(es)
  prds/p5-adoption/realm/prd.md: 3 unticked box(es)

=== boxes rewritten (open -> struck) ===
ratchets struck: 4
realm    struck: 3

=== after ===
running 2 tests
test exemption_list_only_names_done_prds ... ok
test every_done_prd_has_no_unticked_box ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Both files were restored with `git checkout` — this was a measurement, not the
closure. `EXEMPT` was `&[]` throughout.

## `prds/p5-adoption/ratchets` — 4 boxes, lines 32, 38, 41, 44

The four boxes are directives to the *consumer* trees; their closure lives in
each consumer's own gate, which is strike-with-reason's stated condition. All
four commits in the PRD's own frontmatter resolve, and all four resolve to the
**same commit message**, `reproduced` 2026-08-28:

```
mitosys   e0e4cdd   2026-08-27  p5-adoption/ratchets — a wall-clock ratchet gate in each of the three trees
realm     01254aa   2026-08-27  p5-adoption/ratchets — a wall-clock ratchet gate in each of the three trees
model     b5ea9b2d  2026-08-27  p5-adoption/ratchets — a wall-clock ratchet gate in each of the three trees
shared    02d1889   2026-08-27  p5-adoption/ratchets — a wall-clock ratchet gate in each of the three trees
```

All three gate files are on disk, and all three are **green today**, run in
their own trees 2026-08-28:

| tree | gate file | lines | run |
|---|---|---|---|
| mitosys | `src/mitosys/gates/tests/write_path_reads_no_clock.rs` | 419 | `cargo test -p mitosys-gates --test write_path_reads_no_clock` — 7 passed, 0 failed |
| model | `gates/tests/clock_read_ratchet.rs` | 296 | `cargo test -p gates --test clock_read_ratchet` — 3 passed, 0 failed |
| realm | `src/gates/tests/clock_read_ratchet.rs` | 264 | `cargo test -p realm-gates --test clock_read_ratchet` — 3 passed, 0 failed |

**Box 2's reason as the PRD writes it is `refuted`, and this is the one place
the strike text must diverge from the PRD.** The PRD says the closure is *"the
same file's `CLOCK_READS` list, which excludes `Instant::now`"*. Measured, that
list **includes** it:

```
mitosys/src/mitosys/gates/tests/write_path_reads_no_clock.rs
:66  const CLOCK_READS: &[&str] = &["SystemTime::now", "Instant::now", "now_nanos", "now_ms"];
```

`CLOCK_READS` belongs to the older write-path check, which forbids *any* clock
read on the watched files and so is right to include the monotonic spelling.
The **whole-tree ratchet** the box asks for is further down the same file and
uses a different const:

```
:216 const WALL_CLOCK_READS: &[&str] = &["SystemTime::now", "now_nanos", "now_ms"];
:228 const RATCHET_CEILING: usize = 48;
:337 fn wall_clock_reads_may_only_decrease()
:375 fn monotonic_reads_are_never_counted()
```

The strike names `WALL_CLOCK_READS:216` and the self-check
`monotonic_reads_are_never_counted:375`, which fails if the two lists are ever
merged. realm and model each carry the same pair
(`WALL_CLOCK_READS = ["SystemTime::now"]`, `monotonic_reads_are_never_counted`),
so the distinction the box demands is enforced by construction in all three
trees, not merely observed.

**Box 4's counts are `refuted` by the landing commit itself** and the strike
should say so rather than repeating the PRD's table: `e0e4cdd`'s message
records *"realm holds 0, not 3; model holds 10, not 15. mitosys is 48, not
40."* The ceilings on disk agree — mitosys 48, model 10, realm 0.

| box (line) | what the strike names |
|---|---|
| 32 — *"Extend, do not reinvent, mitosys's gate"* | extended, not reinvented: the ratchet was added to the existing `write_path_reads_no_clock.rs` (+228 lines in `e0e4cdd`, one file changed), not to a new file |
| 38 — *"Count only the wall-clock kind"* | `WALL_CLOCK_READS:216`, which omits `Instant::now`, plus `monotonic_reads_are_never_counted:375` which fails if the lists merge. **Not** `CLOCK_READS:66` — see above |
| 41 — *"`../model` and `../realm` have no such gate"* | both built: `model/gates/tests/clock_read_ratchet.rs` and `realm/src/gates/tests/clock_read_ratchet.rs`, commits `b5ea9b2d` and `01254aa`, 3 passed each on 2026-08-28 |
| 44 — *"Landed when a new read fails a named check in each tree"* | the named check is `wall_clock_reads_may_only_decrease` in all three trees; `e0e4cdd`'s message records each ceiling proved red and green by hand with a `SystemTime::now` probe, and green under an `Instant::now` probe |

Each strike states, in one sentence, that the closure is held in another
repository's gate and that this board cannot re-run it as its own — which is
the reason the box is struck rather than ticked.

## `prds/p5-adoption/realm` — 3 boxes, lines 48, 54, 58

All three assert facts about `../realm`, a tree this board does not own.
Commits `realm c862aec` (2026-08-26, *"realm adopts conserved's Scope, Clock
and id vocabulary"*) and `realm c239677` (2026-08-27, *"16-fmt-gate"*) both
resolve.

**Box 1's line numbers are stale and the strike must not repeat them.** The box
names `overlay.rs:385`, `zfs_volumes.rs:139`, `net/src/lib.rs:437` — those were
the 2026-08-21 pre-adoption sites, and `c862aec` rewrote them. Measured in
realm 2026-08-28, `reproduced` as three files and three adoptions, `refuted` as
three line numbers:

```
src/drivers/linux/src/overlay.rs:80    use conserved::scope::{Scope, Undo};
src/drivers/linux/src/overlay.rs:422   let scope = Scope::new();
src/drivers/linux/src/overlay.rs:450   let scope = Scope::new();
src/drivers/linux/src/zfs_volumes.rs:47   use conserved::scope::{Scope, Undo};
src/drivers/linux/src/zfs_volumes.rs:237  let scope = Scope::new();
src/net/src/lib.rs:51    use conserved::scope::{Scope, Undo};
src/net/src/lib.rs:1122  let scope = Scope::new();
```

**Box 2's count is `refuted` in the same direction.** The box says *"`Clock`:
3 non-test wall-clock reads"*. All three named sites now read through
`conserved::SystemClock` — `drivers/linux/src/state.rs:212`,
`cli/src/lib.rs:1337`, `cli/src/state.rs:93` — and realm's ratchet holds
`RATCHET_CEILING = 0` with `WALL_CLOCK_READS = ["SystemTime::now"]`, green at 3
passed. Zero direct wall-clock reads remain, not three. The strike says the
count the box carries was the *before* count and names the ceiling that keeps
it at zero.

**Box 3's grep is `reproduced`**, re-run in realm 2026-08-28 across its `src/`
tree and root manifest, 75 `.rs`/`.toml` files:

```
$ grep -rnE 'blake3|sha2|Sha256' --include='*.rs' --include='*.toml' src/ Cargo.toml
EXIT=1
$ grep -rnE 'median|percentile' --include='*.rs' --include='*.toml' src/ Cargo.toml
EXIT=1
```

Both exit 1 — no match. The refusal recorded in the PRD's own body holds. The
strike quotes this output, not a pointer to the paragraph.

## The fourth form stays refused

`EXEMPT` is `&[]` and gets no entry. Neither PRD can fill the removal-condition
field: the boxes were not waiting on an observable event, only on evidence that
already existed being written down. Per
`shared/learnings/exemptions-name-their-reason.md` § The exemption contract, an
entry that cannot name all three fields means the `state` is wrong — and here
the `state` is right, which leaves the box, not the list, as the thing to
correct.

## Findings — outside this spec's scope, reported not fixed

1. **`prds/p5-adoption/realm/prd.md`'s frontmatter is malformed.** Between
   `blast-radius: mid` and `footprint:` sits a bare list item with no key:
   ```
   blast-radius: mid
     - "@realm/16-fmt-gate"
   footprint:
   ```
   The `needs:` key it belongs to is missing, so that dependency is invisible
   to any YAML reader. This spec does not fix it — **frontmatter is not the
   analyst's to edit**, and the file's `state` and body are what job 2 touches.
   It wants its own node.
2. **No deferral wearing a strike's clothes was found on this board.** All four
   `- [~]` in `prd.md` files and all seven in `spec*.md` were read:
   `p6-scope-unwind:59` strikes work that belongs to another ticket and names
   the mechanism that carries it; `p4-stats/specs/spec02.md:96,98` strike two
   boxes because the spec's own `verify:` line was unsatisfiable as written
   (libtest prints ` - should panic` into the string it greps for);
   `p5-adoption/close` and `p1-scope` and `load-proof` likewise carry measured
   reasons. Every one records a bar the code did not clear, none records work
   not yet done.

## Acceptance

- [x] `prds/p5-adoption/ratchets/prd.md`: the four boxes at lines 32, 38, 41
      and 44 read `- [~]`, each with a reason paragraph naming its gate file
      and the commit from the PRD's own frontmatter. The box wording above the
      reason is unedited

      Landed 2026-08-28. All four now read `- [~]` (source lines 32, 49, 76,
      91 after the reason paragraphs grew the file):
      ```
      $ grep -c '^- \[ \]' prds/p5-adoption/ratchets/prd.md
      0
      $ grep -c '^- \[~\]' prds/p5-adoption/ratchets/prd.md
      4
      ```
      Each reason names its gate file and a frontmatter commit — `e0e4cdd`
      (mitosys), `01254aa` (realm), `b5ea9b2d` (model). `git diff` shows the
      box wording itself is byte-identical: every `+` line is either the
      marker swap `- [ ]` -> `- [~]` on the box's first line or a new indented
      paragraph below it.
- [x] The box-38 reason names `WALL_CLOCK_READS` at
      `mitosys/src/mitosys/gates/tests/write_path_reads_no_clock.rs:216` and
      the self-check `monotonic_reads_are_never_counted`, **not** `CLOCK_READS`
      at `:66`. `grep -n 'CLOCK_READS' prds/p5-adoption/ratchets/prd.md` must
      show no bare `CLOCK_READS` presented as excluding `Instant::now`

      Measured in mitosys 2026-08-28 before writing the reason — the PRD's
      claim is `refuted`, `reproduced` as its opposite:
      ```
      :66  const CLOCK_READS: &[&str] = &["SystemTime::now", "Instant::now", "now_nanos", "now_ms"];
      :216 const WALL_CLOCK_READS: &[&str] = &["SystemTime::now", "now_nanos", "now_ms"];
      :228 const RATCHET_CEILING: usize = 48;
      :337 fn wall_clock_reads_may_only_decrease()
      :375 fn monotonic_reads_are_never_counted()
      ```
      `CLOCK_READS:66` **includes** `Instant::now`; `WALL_CLOCK_READS:216`
      omits it. The strike names `:216` and `:375`. Every `CLOCK_READS`
      occurrence in the board file is accounted for:
      ```
      $ grep -n 'CLOCK_READS' prds/p5-adoption/ratchets/prd.md
      34:      `WATCHED` file list and `CLOCK_READS = ["SystemTime::now", "Instant::now",
      54:      tree's own gate.** The const that holds it is `WALL_CLOCK_READS` at
      60:      (`gates/tests/clock_read_ratchet.rs:68`) each carry the same pair with
      61:      `WALL_CLOCK_READS = ["SystemTime::now"]`. Commits `e0e4cdd`, `01254aa`,
      65:      **Not `CLOCK_READS` at `:66`.** This PRD's own § Job 2 table in
      68:      that is `refuted` — line 66 reads
      69:      `const CLOCK_READS: &[&str] = &["SystemTime::now", "Instant::now",
      ```
      Line 34 is the box's own unedited wording, which lists `Instant::now`
      **inside** `CLOCK_READS` — correct as written. Lines 65-69 name
      `CLOCK_READS` only to say it is *not* the evidence. **No bare
      `CLOCK_READS` is presented as excluding `Instant::now`.**
- [x] Each of the three consumer gates is re-run in its own tree and its output
      quoted in the reason it closes: mitosys 7 passed, model 3 passed, realm
      3 passed, 0 failed each

      Implementer runs 2026-08-28, each `cd`'d into its own tree,
      `reproduced`:
      ```
      $ cd ../mitosys && cargo test -p mitosys-gates --test write_path_reads_no_clock
      running 7 tests
      test a_watch_list_that_scans_nothing_fails_rather_than_passes ... ok
      test monotonic_reads_are_never_counted ... ok
      test every_watched_file_still_exists ... ok
      test the_serde_skip_exemption_is_still_load_bearing ... ok
      test no_watched_file_reads_the_machine_clock ... ok
      test wall_clock_reads_may_only_decrease ... ok
      test a_walk_that_scans_nothing_fails_rather_than_passes ... ok
      test result: ok. 7 passed; 0 failed

      $ cd ../model && cargo test -p gates --test clock_read_ratchet
      test result: ok. 3 passed; 0 failed

      $ cd ../realm && cargo test -p realm-gates --test clock_read_ratchet
      test result: ok. 3 passed; 0 failed
      ```
      7 / 3 / 3, 0 failed each. Quoted into the ratchets strikes (boxes 32, 76,
      91) and the realm `Clock` strike.
- [x] `prds/p5-adoption/realm/prd.md`: the three boxes at lines 48, 54 and 58
      read `- [~]` with reason paragraphs. The box-48 reason gives the
      **current** `Scope::new()` sites, not the stale `385`/`139`/`437`, and
      says the numbers in the box are pre-adoption

      Landed 2026-08-28. The boxes are at source lines **49, 55, 59** — one
      lower than the spec's 48/54/58 because the board's `## Frontmatter
      repaired` pass added the missing `needs:` key above them. Same three
      boxes:
      ```
      $ grep -c '^- \[ \]' prds/p5-adoption/realm/prd.md
      0
      $ grep -c '^- \[~\]' prds/p5-adoption/realm/prd.md
      3
      ```
      Box 1's reason states the box's numbers are the 2026-08-21 pre-adoption
      sites and gives the current ones, measured in realm 2026-08-28
      (`reproduced` as three files, `refuted` as three line numbers):
      ```
      src/drivers/linux/src/overlay.rs:422    let scope = Scope::new();
      src/drivers/linux/src/overlay.rs:450    let scope = Scope::new();
      src/drivers/linux/src/zfs_volumes.rs:237  let scope = Scope::new();
      src/net/src/lib.rs:1122                 let scope = Scope::new();
      ```
      One measurement beyond the spec's: `overlay.rs` holds **two**
      `Scope::new()` sites, so the three files carry four call sites. Recorded
      in the strike; the box's "three real sites" wording is left unedited.
- [x] The box-54 reason states that realm's wall-clock count is **0**, not the
      3 the box names, and cites `RATCHET_CEILING = 0` in
      `realm/src/gates/tests/clock_read_ratchet.rs` with its green run quoted

      Landed. The reason names the box's 3 as the *before* count and 0 as the
      count today, `refuted` by `e0e4cdd`'s own message (*"realm holds 0, not
      3"*), and cites the ceiling measured in realm 2026-08-28:
      ```
      src/gates/tests/clock_read_ratchet.rs:41  const WALL_CLOCK_READS: &[&str] = &["SystemTime::now"];
      src/gates/tests/clock_read_ratchet.rs:55  const RATCHET_CEILING: usize = 0;
      ```
      with the green run quoted in the strike (3 passed, 0 failed). The three
      sites the box names now read through `conserved::SystemClock`
      (`drivers/linux/src/state.rs:212`, `cli/src/state.rs:93`,
      `cli/src/lib.rs:1337`) — zero direct wall-clock reads remain.
- [x] The box-58 reason carries the **re-run** grep output with both exit
      codes, quoted, not a pointer to the paragraph above it in that file

      Re-run by the implementer in realm 2026-08-28, `reproduced`, and pasted
      into the strike itself rather than referenced:
      ```
      $ cd ../realm
      $ grep -rnE 'blake3|sha2|Sha256' --include='*.rs' --include='*.toml' src/ Cargo.toml
      EXIT=1
      $ grep -rnE 'median|percentile' --include='*.rs' --include='*.toml' src/ Cargo.toml
      EXIT=1
      ```
      Both exit 1, no output. Population counted this run: 74 `.rs`/`.toml`
      under `src/` plus the root `Cargo.toml` = **75 files**, matching the
      analyst's figure.
- [x] `grep -c '^- \[ \]' prds/p5-adoption/ratchets/prd.md` returns 0 and the
      same on `prds/p5-adoption/realm/prd.md` returns 0. Neither file's
      frontmatter is touched — `git diff` on each shows no change above the
      first `---`-terminated block

      Implementer run 2026-08-28:
      ```
      $ grep -c '^- \[ \]' prds/p5-adoption/ratchets/prd.md
      0
      $ grep -c '^- \[ \]' prds/p5-adoption/realm/prd.md
      0
      ```
      Frontmatter: the edit script asserted the frontmatter block byte-equal
      before and after writing each file, and `git diff` confirms it. The
      ratchets diff's first changed line is the box at 32. **The realm diff
      carries one frontmatter line, `+needs:`, which is not this spec's** — it
      is the board's own `## Frontmatter repaired` pass of 2026-08-28,
      already in the working tree before this spec began (spec02 § Findings 1
      reported the bare `  - "@realm/16-fmt-gate"`; the board repaired it).
      This spec touched only the seven boxes and their reasons.
- [x] `EXEMPT` is still `&[]` —
      `grep -n 'const EXEMPT' conserved/tests/done_boxes_are_ticked.rs` returns
      `const EXEMPT: &[(&str, &str, &str)] = &[];`

      Implementer run 2026-08-28, after both jobs landed:
      ```
      $ grep -n 'const EXEMPT' conserved/tests/done_boxes_are_ticked.rs
      110:const EXEMPT: &[(&str, &str, &str)] = &[];
      ```
      No entry was added for either PRD. The fourth form stayed refused:
      neither could fill the removal-condition field, and the 3-tuple would
      not have compiled without it.
- [x] `cargo test -p conserved --test done_boxes_are_ticked` is **green** and
      quoted: 2 passed, 0 failed. No box was ticked to get there — all seven
      are strikes

      Implementer run 2026-08-28, `reproduced`:
      ```
      running 2 tests
      test exemption_list_only_names_done_prds ... ok
      test every_done_prd_has_no_unticked_box ... ok

      test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
      ```
      The two rows spec01 landed red are closed. **Zero boxes were ticked to
      get here**: `grep -c '^- \[~\]'` returns 4 on ratchets and 3 on realm,
      and neither file gained a `- [x]` — `git diff` adds no `[x]` line to
      either. `EXEMPT` was `&[]` throughout, so the green came from seven
      measured strikes and nothing else.
- [x] `cargo test --workspace` and `cargo fmt --check --all` and
      `cargo clippy --workspace --all-targets -- -D warnings` are all green,
      i.e. `just check` exits 0, and its exit code is stated

      The repo's own gate, run whole 2026-08-28. `justfile`'s `check` recipe is
      exactly those three commands in that order:
      ```
      $ just check
      ...
         Doc-tests conserved
      running 3 tests
      test conserved/src/stats.rs - stats::min_median_max (line 171) ... ok
      test conserved/src/stats.rs - stats::percentile (line 101) ... ok
      test conserved/src/stats.rs - stats::median (line 147) ... ok

      test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

      JUST_CHECK_EXIT=0
      ```
      **`JUST_CHECK_EXIT=0`.** Over the whole log: `test result: ok` on **15**
      targets and **0** lines matching `FAILED` or `^error`. The gate this PRD
      changed is green inside it:
      ```
      Running tests/done_boxes_are_ticked.rs
      running 2 tests
      test exemption_list_only_names_done_prds ... ok
      test every_done_prd_has_no_unticked_box ... ok
      ```
      The single red target spec01 landed on purpose is gone; fmt and clippy
      never went red at any point.
- [x] The PRD's own six `## Acceptance` boxes (lines 151–162) are `- [x]`,
      each citing the spec box that proves it and quoting the output. Box 1's
      tick records the one deviation: `unticked_boxes_in_acceptance` was
      **replaced by two functions** (`opens_an_unticked_box` and
      `unticked_boxes_in_file`) rather than renamed in place, and the test
      itself was renamed too — the PRD asked only for the function

      All six ticked 2026-08-28, each naming the spec box that proves it and
      carrying its own quoted output:
      ```
      $ grep -c '^- \[ \]' prds/exemptions-name-their-reason/prd.md
      0
      ```
      Box 1 cites `spec01` 1/5/6/7 and records the deviation in those words —
      two functions, not a rename, plus the test rename the PRD never asked
      for, argued from `spec01` § Findings 1. Box 2 cites `spec01` 2, box 3
      cites `spec01` 3 and `spec02` 8, box 4 cites `spec02` 1/2/3/7, box 5
      cites `spec02` 4/5/6/7, box 6 cites `spec02` 9/10.

      **The declared footprint named three paths; this file is a fourth**, as
      the PRD's own `## Built 2026-08-28` section records. It is `state:
      claimed`, so its own boxes were never in the gate's population — the
      ticks are the record, not the gate's condition.

## Verify and Proof

```sh
cd /Users/feb/dev/infra/shared
grep -c '^- \[ \]' prds/p5-adoption/ratchets/prd.md   # expect 0
grep -c '^- \[ \]' prds/p5-adoption/realm/prd.md      # expect 0
grep -n 'const EXEMPT' conserved/tests/done_boxes_are_ticked.rs
cargo test -p conserved --test done_boxes_are_ticked
just check; echo "JUST_CHECK_EXIT=$?"

# the reasons, re-measured in the trees that hold them
cd /Users/feb/dev/infra/mitosys && cargo test -p mitosys-gates --test write_path_reads_no_clock
cd /Users/feb/dev/infra/model   && cargo test -p gates --test clock_read_ratchet
cd /Users/feb/dev/infra/realm   && cargo test -p realm-gates --test clock_read_ratchet
cd /Users/feb/dev/infra/realm
grep -rnE 'blake3|sha2|Sha256'  --include='*.rs' --include='*.toml' src/ Cargo.toml; echo "EXIT=$?"
grep -rnE 'median|percentile'   --include='*.rs' --include='*.toml' src/ Cargo.toml; echo "EXIT=$?"
```
