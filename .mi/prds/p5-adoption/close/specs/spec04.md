# goal

`learnings/clock.md`: `open` → `decided`, **gated on p3 having landed** — and
`README.md`'s `clock.md` table row moved with it. The gate is not a note in
prose: the verify below derives the p3 landing commits from git and fails if
the status was flipped ahead of them.

est: 0.25

## What must exist first, precisely

At the time this spec was written, `conserved/src/clock.rs` was **on disk but
not committed** — `git ls-files conserved/src/` listed `content_id.rs`,
`lib.rs`, `scope.rs` and nothing else. It was committed part-way through this
analysis (`cb49f4a`, "p3/spec01: Instant — unix nanoseconds, decided and
pinned") while `.mi/prds/p3-clock/prd.md` still read `state: claimed`, with
p3's remaining specs unlanded. Both states are the reason this gate is written
as a check rather than a note: a learning that said `decided` in either of them
would have pointed at a file no clone has, or at half a ticket.

So the condition is stated as a property of the repository, not as a sha this
analyst could not know:

1. `git ls-files` lists a committed `conserved/src/clock*` — the module, in the
   record, not the working tree.
2. `cargo test -p conserved clock` passes.
3. Every commit `git log -- conserved/src/clock*` names is linked from
   `learnings/clock.md`.
4. `.mi/prds/p3-clock/prd.md` reads `state: done`.

Condition 3 is what makes this `AGENTS.md`'s Close step (*"update the learning
status to `decided`, **link to the commit**"*) rather than a status edit: the
commit list is read out of git at verify time, so a flip that links four of five
p3 commits fails and names the one it missed.

## Why `decided` and not `partial`, once p3 lands

`clock.md` is `open` — *"found and argued, not yet settled"*. What it argues
for is a three-step fix: (1) make it visible with a ratchet, (2) give time a
type and one source, (3) empty the allowlist from the leaves inward. p3 lands
step 2 in full — `Instant`, `Clock`, `SystemClock`, `FixedClock`, the unit
pinned by a test. Steps 1 and 3 are consumer-tree work and are **explicitly not
this repo's**: p3's own third requirement says *"the ratchet stays out … p5
carries it; this node only makes the compliant path exist"*, and p5's
`ratchets` child is `repo: consumers`.

So the honest reading of the ladder is `decided`: *"agreed; the record of what
it beat. **May still need extraction.**"* The argument is settled, the type
exists, and what remains — threading it through ~65 sites per tree — is
extraction, which `decided` carries by definition.

**The held children make this a claim that must be qualified, not withdrawn.**
The user's 2026-08-21 scope answer holds `ratchets`, `mitosys`, `llm` and
`realm`, so on the day this spec runs: no ratchet exists in any tree, no
consumer reads `conserved::Clock`, and both live-clock-into-a-content-hash
sites — `rec_now()` and `transactional.rs:72` — are still bugs in `../model`.
A `decided` that does not say so is a false record. The acceptance below
requires it said in the document, in the same paragraph as the flip.

## Files

- `learnings/clock.md` — frontmatter `status:` and `code:` (the `code:` line
  gains the landed module; spec02 already gave it the second `../model` site).
- `README.md` — the `clock.md` row of the table at lines 31-40 only.

Not touched: `learnings/shared-crate.md` and its `README.md` row (spec03),
anything under `conserved/`, any sibling tree.

## What must have landed first, in board terms

- **spec02**, so the flip is not stamping `decided` on a document that still
  undercounts its own subject. This is the ordering that matters most in this
  ticket: correcting the record and closing the record are two acts, and doing
  them in the wrong order publishes a settled-looking document that is wrong.
- **p3-clock**, committed, per the four conditions above.

## The edits

**`status: open` → `status: decided`**, and a `decided: <YYYY-MM-DD>` line if
the frontmatter convention `.mi/docs/memos/distribution.md` uses is adopted
here — optional, and the verify does not require it.

**`code:`** gains `shared conserved/src/clock.rs` (or whatever path condition 1
resolves), keeping both `../model` sites and the mitosys site.

**A short §"Landed"**, at the end, carrying: the p3 commits; that step 2 of
§"The fix" is done and steps 1 and 3 are not; that no consumer has adopted
`Clock`; that both content-hash sites are still live in `../model`; and that
the `Instant` unit landed as **unix nanoseconds**, which is narrower than the
document's own `// unix, whatever precision the tree needs` at line 71 — a
place where the implementation decided something the learning left open, and
therefore belongs back in the learning.

## Acceptance

- [ ] `conserved/src/clock*` is committed and `cargo test -p conserved clock`
      passes — checked, not assumed.
- [ ] `.mi/prds/p3-clock/prd.md` reads `state: done`.
- [ ] `learnings/clock.md` reads `status: decided`.
- [ ] Every commit touching `conserved/src/clock*` is linked in
      `learnings/clock.md` (short sha), derived from `git log` at verify time.
- [ ] Its `code:` line names the committed clock module **and** still names
      `transactional.rs:72` from spec02.
- [ ] The document states that the `Instant` unit landed as unix nanoseconds.
- [ ] The document states that steps 1 and 3 of §"The fix" have not landed and
      that no consumer tree reads `conserved::Clock` yet.
- [ ] §"The fix, in the order it should be done" and §"Honest scope" are still
      on disk — the erase-guard. Specifically `Make it visible` and
      `Empty the allowlist from the leaves inward` survive.
- [ ] `README.md`'s `clock.md` table row reads `decided`; the
      `shared-crate.md` row is untouched by this spec.
- [ ] **The counter-check**: if `conserved/src/clock*` is *not* committed, the
      verify fails with a message saying the status was flipped ahead of the
      code. This box exists so the gate itself is exercised — run the verify
      once with the flip applied and the module uncommitted (e.g. on a scratch
      branch) and confirm it fails for that reason, not another.

verify: `bash -c 'set -e; cd /Users/feb/dev/infra/shared; C=learnings/clock.md; P=.mi/prds/p3-clock/prd.md; DEC=0; grep -q "^status: decided" $C && DEC=1; LANDED=0; git ls-files conserved/src | grep -q "^conserved/src/clock" && grep -q "^state: done" $P && LANDED=1; if [ $DEC -eq 1 ] && [ $LANDED -eq 0 ]; then echo "FAIL: clock.md says decided but p3 has not landed (no committed conserved/src/clock*, or $P is not done) — the status was flipped ahead of the code"; exit 1; fi; if [ $LANDED -eq 0 ]; then echo "BLOCKED: p3 has not landed; clock.md is correctly still open and spec04 is not runnable yet"; exit 1; fi; if [ $DEC -eq 0 ]; then echo "FAIL: p3 landed but clock.md was not closed"; exit 1; fi; cargo test -p conserved clock 2>&1 | grep -qE "test result: ok\. [1-9]" || { echo "FAIL: cargo test -p conserved clock did not run and pass any test"; exit 1; }; for c in $(git log --format=%H -- "conserved/src/clock*" | cut -c1-7); do grep -q "$c" $C || { echo "FAIL: clock.md does not link p3 landing commit $c"; exit 1; }; done; grep -qE "^code:.*conserved/src/clock" $C || { echo "FAIL: code: does not name the landed module"; exit 1; }; grep -qE "^code:.*transactional\.rs:72" $C || { echo "FAIL: spec02 correction was lost"; exit 1; }; grep -qi "nanosecond" $C || { echo "FAIL: the landed Instant unit is not recorded"; exit 1; }; grep -q "Make it visible" $C || { echo "FAIL: the fix order was erased"; exit 1; }; grep -q "Empty the allowlist from the leaves inward" $C || { echo "FAIL: step 3 was erased"; exit 1; }; grep -E "^\| .clock\.md." README.md | grep -q "decided" || { echo "FAIL: README clock row still stale"; exit 1; }; echo "spec04 ok"'`
