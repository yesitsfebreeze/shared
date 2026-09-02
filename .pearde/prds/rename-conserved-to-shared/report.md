# rename-conserved-to-shared — implementer report, round 0902

Verdict: DONE

Boxes **28/28**. All three `## Verify and Proof` blocks run clean under
`sh -e` — `EXIT=0` each. Two structural defects found and fixed in this PRD's
own documents: the verify blocks were not `sh -e`-correct, and the census
excludes still named the pre-`27db1b7` board path. See §Verify blocks and
§The census.

The rev the family pins:

dfc98fba70039863797f7185d860ef392becb21f

Published on `origin/main` **2026-09-02 09:27:44 +0200**. `mitosys`, `model`
and `realm` can be dispatched.

## What this round closed

Three boxes were open at the start, all in spec03 and its mirror in `prd.md`.
Two are now closed, one is annotated and left open on purpose.

| box | where | now |
|---|---|---|
| **Pushed** | `prd.md` §Acceptance, spec03 | `[x]` |
| §Landed `- rev:` and `- pushed to origin/main:`, no `TBD` | spec03 | `[x]` |
| the gate passes for the right reason, at `state: done` | spec03 | `[x]` — proved on a fixture worktree |

No code changed. The rename, the gate file and the prose corrections all landed
in the earlier round's commits; this round measured them against `origin/main`
and filled the one field that could not be written before the push existed.

## Pushed

```
$ git branch -r --contains dfc98fba70039863797f7185d860ef392becb21f
  origin/HEAD -> origin/main
  origin/main

$ git branch -r --contains HEAD
  origin/HEAD -> origin/main
  origin/main

$ git rev-list --count origin/main..HEAD
0
```

`HEAD` is `27db1b7`, and nothing local is unpushed.

### How many commits the push published — 7, not the 11 predicted

spec03 predicted "10 plus this PRD's own", measured 2026-08-28. The real figure
is **7**, and the difference is not a miscount.

```
$ git rev-list --count 24997ea..978dbf6
7
$ git log --oneline 24997ea..978dbf6
978dbf6 a-shared-name-is-not-a-shared-function — Three families of same-shaped functions …
0060b2c p5-adoption — adoption: every consumer, one implementation
81805cf p5-adoption — specced at 14, its verify can fail now, and claimed by its implementer
7bc774d p5-adoption — claimed by its analyst
673cc25 rename-conserved-to-shared — the boxes the commit closed, and the rev §Landed names
dfc98fb rename-conserved-to-shared — `conserved` becomes `shared`: directory, package, and the gate …
0567f75 rename-conserved-to-shared — claimed by its implementer
```

`24997ea` was itself pushed at **2026-08-28 13:57:19**, between spec03's
measurement and the rename commit at 21:49. Ten commits of backlog had already
drained by the time the rename existed. Three of the seven are this PRD's:
`0567f75`, `dfc98fb`, `673cc25`.

The push happened in the board's earlier round on this machine, not in this one.
Reconstructed from `git reflog show --date=iso origin/main`, which is the only
record of when a ref moved:

| `origin/main` moved to | when |
|---|---|
| `24997ea` | 2026-08-28 13:57:19 +0200 |
| `978dbf6` | **2026-09-02 09:27:44 +0200** — first push containing `dfc98fb` |
| `27db1b7` | 2026-09-02 09:32:37 +0200 |

## The gate — a deadlock, proved on a fixture

`shared/tests/landed_rev_is_published.rs` fires only at `state: done`. That
makes the box circular on the live tree: **`pearde collect` will not write
`done` while any box is open, and the box cannot be true until the state *is*
`done`.** Neither side moves first. The first collect refused on exactly this.

The proof therefore runs on a copy, and the live gate is re-run by the collect
after it writes `done`.

### The fixture

`git worktree add <scratch> HEAD` at `27db1b7`, this round's uncommitted
`prd.md` and `spec03.md` copied in, `state: done` written **only in the copy**.

The live `prd.md` frontmatter was never edited — it still reads
`state: claimed`, and `shasum` of it is
`312accff2588df8b23719c6915156e1e7c02db05` before and after. The worktree was
removed with `git worktree remove --force`; `git worktree list` is back to the
single entry.

### The pass — armed, no early return

```
$ sed -n '2p' .pearde/prds/rename-conserved-to-shared/prd.md
state: done
$ cargo test -p shared --test landed_rev_is_published
test parser::a_filled_rev_is_read_out_of_its_backticks ... ok
test parser::state_is_read_out_of_the_frontmatter ... ok
test parser::a_later_section_is_not_read_as_landed ... ok
test parser::the_placeholder_is_not_mistaken_for_a_sha ... ok
test landed_rev_is_a_published_rename_commit ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All four assertions ran — 40-hex, `cat-file -e`, `branch -r --contains`
non-empty, `name = "shared"` at that rev. The manifest half needs no arming and
passes on the live tree too: `git show
dfc98fba70039863797f7185d860ef392becb21f:shared/Cargo.toml | grep '^name'` →
`name = "shared"`.

On the live tree at `state: claimed` the same command also prints
`5 passed; 0 failed` — but by **returning early**. Green, and worthless as
evidence. That distinction is the whole box.

### The three failure paths, re-run in the same fixture

spec03's original fixtures ran on 2026-08-28 against `prds/…`. Commit `27db1b7`
moved the board and the gate's `PRD` const with it, so those proved nothing
about the path in force today — a gate reading a file that no longer exists
returns early on `read_to_string` and looks exactly as green as one that passed.
Re-run at `state: done`:

| fixture | §Landed rev | line | panic |
|---|---|---|---|
| placeholder | `TBD` | `:99` | *"…names rev `TBD`, which is not a 40-character sha"* |
| pre-rename commit | `70d7e15c…` — the rev `mitosys` pins today | `:127` | *"…has no `shared/Cargo.toml`. That rev predates the rename, so a consumer pinning it gets the crate under its old name"* |
| unpushed commit | `e317b390…` — made in the fixture, `git branch -r --contains` empty | `:115` | *"…is on NO remote branch of this repository — it exists only on this machine, so nothing that clones the repo can ever fetch it"* |

Every panic names `.pearde/prds/rename-conserved-to-shared/prd.md`, so the gate
reads the migrated path and fires on it. Each fixture was restored before the
next, and none of it touched the live tree.

**What the collect observes:** after it writes `state: done`, `cargo test -p
shared --test landed_rev_is_published` reports `5 passed; 0 failed` on the live
tree — the same run as the fixture's, against the same rev, already published
and already renamed.

## Verify blocks — made correct under `sh -e`

The collect runs a `## Verify and Proof` block as **one script that stops at the
first non-zero command**. Three lines were wrong under that rule.

| spec | line | why it failed | fix |
|---|---|---|---|
| 01 | `git grep -nI conserved -- Cargo.toml Cargo.lock shared/Cargo.toml` | zero matches is the **pass**, and a bare `grep` exits 1 on zero matches | `! git grep -qI … && echo "manifests + lock: 0 conserved"` |
| 01 | `( cd "$W" && cargo metadata …; echo "exit=$?" )` | under `set -e` the subshell dies before the `echo`, so the exit could never be reported | capture into `md`, `echo`, then assert `[ "$md" -eq 0 ]` |
| 03 | `REV=$(sed -n … prds/rename-conserved-to-shared/prd.md)` | `27db1b7` moved the board; `sed` on a missing file exits 2 and kills the block at the assignment | path is `.pearde/prds/…` |

The general rule now stated at the top of each block: **a grep whose wanted
result is zero matches gets the explicit negated form, never a bare grep.**
spec03 gained one more of those for the `TBD` check.

### Proof — each block extracted and run standalone

```
$ for n in 01 02 03; do sh -e $SCR/v$n.sh > $SCR/out$n.log 2>&1; echo "spec$n EXIT=$?"; done
spec01 EXIT=0  |  spec01 verify: OK
spec02 EXIT=0  |  spec02 verify: OK
spec03 EXIT=0  |  spec03 verify: OK
```

Key assertions from the three logs:

```
out01: in-crate survivors: 7
out01: manifests + lock: 0 conserved
out01: metadata exit=0
out01: passed=90 failed=0
out02: survivors: 18
out02: crate-name.md tokens: 57
out02: 826a62076bdfb1d8e19f0a39bba2cee4a0e173cb  -   (admission test, base)
out02: 826a62076bdfb1d8e19f0a39bba2cee4a0e173cb  -   (admission test, worktree)
out02: passed=90 failed=0
out03: unpushed: 0
out03: rev = dfc98fba70039863797f7185d860ef392becb21f
out03: name = "shared"
out03: no TBD in the PRD
out03: passed=90 failed=0
```

Each block ends in an explicit `echo "specNN verify: OK"`, so a truncated run is
distinguishable from a passing one.

## The census — 7 is right, and the real number is 18

**The code is not wrong.** The two extra in-crate lines are
`shared/tests/landed_rev_is_published.rs:13` and `:29`, and both are the string
`.pearde/prds/rename-conserved-to-shared/prd.md` — **this PRD's own directory
name**, which the gate must name in order to read the file. `conserved` appears
there as a substring of `rename-conserved-to-shared`, not as the crate name, and
a done PRD's directory name is an address (`prd.md` §7). Renaming it would break
the gate and the master board's graph edges. So: 7 is correct, the survivor
table was stale, and the table is what I fixed.

Two entries were added to `prd.md`'s `| survivor | why |` table, and the §Verify
enumeration is now a counted table totalling 18:

| added | why |
|---|---|
| `landed_rev_is_published.rs:13`, `:29` | this PRD's own address; spec03 added the file after the table was written |
| `learnings/a-shared-name-is-not-a-shared-function.md:17`, `:18`, `:176`, `:177` | `mitosys/.pearde/prds/adopt-conserved` and `model/.pearde/prds/adopt-conserved` — PRD addresses, the class already granted to `shared-crate.md:335-336`. The file landed in `978dbf6` from a **different** PRD |

12 + 2 + 4 = **18**, and `git grep … | wc -l` prints 18.

### The bigger defect underneath it: the exclude named a folder that no longer exists

Every census in this PRD excluded `':(exclude)prds'`. Commit `27db1b7` moved the
board to `.pearde/`. The exclusion therefore excluded nothing, and the same
command returns **861** lines — the entire board falling back into the census.
It never *failed*, because a grep with matches exits 0; it silently stopped
measuring anything. Corrected to `':(exclude).pearde'` in `prd.md` §Verify,
spec02's block, and spec02's boxes 6 and 9.

This is the third instance of the same `prds/` → `.pearde/` drift this round —
after spec03's acceptance command and spec03's verify block. Recorded as a
defect below.

## Verify — spec03 §Verify and Proof, run whole

| check | result |
|---|---|
| `cargo test -p shared --test landed_rev_is_published` | `5 passed; 0 failed` |
| `cargo test --workspace --no-fail-fast` | **`passed=90 failed=0`** |
| `cargo fmt --check --all` | exit 0, no output |
| `cargo clippy --workspace --all-targets -- -D warnings` | `Finished dev profile`, no warnings |
| `git branch -r --contains $REV` | `origin/HEAD -> origin/main`, `origin/main` |
| `git show "${REV}":shared/Cargo.toml \| grep '^name'` | `name = "shared"` |
| `git rev-list --count origin/main..HEAD` | `0` |
| `grep -n 'TBD' …/prd.md` | nothing — no `TBD` survives anywhere in the file |

**90/0** is spec01's 85 plus the gate's 5, exactly as spec03 predicted. The 84/1
baseline in `prd.md` §Verify predates the box-gate fix and is superseded.

### Nothing of the record moved

```
$ git status --porcelain -- .pearde/ .mi/gantt/ .pi/ prds/
 M .pearde/.state/history.jsonl
 M .pearde/.state/plan.json
 M .pearde/prds/p5-adoption/prd.md
 M .pearde/prds/rename-conserved-to-shared/prd.md
 M .pearde/prds/rename-conserved-to-shared/specs/spec03.md
?? .pearde/.gitignore
?? prds/
```

`.mi/gantt/` and `.pi/` list nothing. Two of those rows are mine — `prd.md` and
`specs/spec03.md`, both inside this PRD's own folder. The other five are other
sessions' state and another PRD's body; untouched by this round.

```
$ git -C .. status --porcelain mitosys/prds model/prds realm/prds \
      mitosys/.pearde model/.pearde realm/.pearde
(empty)
```

The three consumer PRDs are untouched. Their rev bump is their own node's.

## Changes made this round

Two files, both inside `.pearde/prds/rename-conserved-to-shared/`.

1. **`prd.md` §Landed** — `- pushed to origin/main:` was `TBD`. Now names the
   date, the push that carried it, and the three commits of this PRD's that went
   with it.
2. **`prd.md` §Acceptance** and **`specs/spec03.md` §Acceptance** — all three
   remaining boxes ticked, each with its output quoted.
3. **All three `## Verify and Proof` blocks rewritten to be `sh -e`-correct**,
   with the `prds/` → `.pearde/` paths corrected and an explicit terminal
   `echo`. Assertions unchanged — only their shell form.
4. **`prd.md` §Verify and its survivor table reconciled** to 18, with the
   exclude corrected to `.pearde`, and the two new survivor classes named.
5. **`spec01` box 5 and `spec02` boxes 1, 6, 8, 9, 10 amended** to the numbers
   and paths that are true today, each with the measurement quoted.

Nothing outside the PRD folder was written. No commit and no push on the live
tree, and no `git add` in it. The fixture worktree lived under the scratchpad,
carried its own index, and is gone.

## Defects outside this PRD's scope

Three, recorded rather than fixed.

### 1. `prds/` survives as an untracked stub

`27db1b7` moved the board to `.pearde/`, but `/Users/feb/dev/infra/shared/prds/`
still exists, holding two files under `prds/.state/`. It shows as `?? prds/` and
belongs to another session's state. A stale `prds/` next to a live `.pearde/`
is the exact shape that makes a path-typo bug invisible — the gate's own const
would have been one. Not this PRD's to delete.

### 2. `.pearde/.state/round.md` is stale

Every edit to a PRD in this tree fires a hook: *"A PRD moved and
`.pearde/.state/round.md` has not been rewritten since."* That file is the
orchestrator's, not an implementer's footprint, so this round did not write it.
It is still owed.

### 3. The `prds/` → `.pearde/` drift is systemic, and it fails silently

Commit `27db1b7` rewrote the paths it knew about. Three separate survivors
turned up inside this one PRD: spec03's acceptance command, spec03's verify
block, and the `':(exclude)prds'` pathspec in `prd.md` and spec02.

The exclusion case is the dangerous shape. **A stale path in a `grep` exclude
does not error — it silently stops excluding**, and the check goes on passing
while measuring 861 lines instead of 18. A stale path in a `sed` at least exits
non-zero. Any board check in any of the four trees that names `prds/` in a
pathspec is now measuring the whole board.

Worth a sweep no single PRD owns: `git grep -nI "'(exclude)prds'\|prds/" -- '*.md'`
on each member. Out of scope here — this round fixed only what is inside
`.pearde/prds/rename-conserved-to-shared/`.

## Held, unchanged

`.pi/ontology/digest.md`'s three `conserved` hits stay as written, per `prd.md`
§"`.pi/ontology/digest.md` is held pending a ruling".
`@shared/the-ontology-digest-indexes-a-dead-store` is the node that owns them
and is `open`, `after rename-conserved-to-shared` — it unblocks with this one.
