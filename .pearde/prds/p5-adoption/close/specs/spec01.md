# goal

Decide and record **the correction mechanism** for `learnings/`: which
corrections are edits in place and which must be new superseding documents.
The decision is **correction-by-edit under git for additions; supersede-by-new-document
for reversals**, and the place it is written is `learnings/README.md`
§"What is enforced", whose second bullet is now factually false.

est: 0.5

## Why this spec exists and why it runs first

`AGENTS.md` §"The status ladder" says:

> No document here is ever edited to erase a decision — corrections are new
> documents that supersede the old one.

`learnings/clock.md` is incomplete: it names `rec_now()` as *the*
live-clock-into-a-content-hash and a second one exists
(`../model/src/node/transactional.rs:72`). Fixing that is either an edit or a
superseding document, and nothing on disk says which. spec02 and spec05 cannot
honestly run until this is settled, and the **`llm` child may need the
`clock.md` correction before this ticket ever runs** — so the mechanism this
spec writes down must be usable by anyone, at any time, without this node
having run. An edit is; authoring a superseding document with reciprocal
`supersedes:` / `superseded_by:` links is a ceremony that only a node like this
one performs.

## The decision, justified from `learnings/README.md`'s own words

`learnings/README.md` §"What is enforced", second bullet, states the *reason*
the folder needed the supersede ceremony and names the exact condition that
lifts it:

> It is not version controlled, so "the record only grows" has no subject
> here: a correction overwrites its predecessor instead of shadowing it.
> `git init` is one command if that changes.

**That condition has been met.** `p0-foundation`'s first requirement was
`git init`, quoting this very sentence as its reason; it landed in commit
`ab154f7` ("foundation: the reset workspace, the board, the learnings"), and
`git ls-files learnings/` lists all nine documents. A correction to a learning
now *shadows* its predecessor — `git log -p learnings/clock.md` is the
predecessor, permanently — which is precisely what the README said the
supersede mechanism was standing in for.

The two rules that remain, and their boundary:

1. **An addition or a factual correction is an edit in place.** It reverses no
   decision; git holds what it replaced. Naming a second code site in
   `clock.md` (spec02) and recording what p2 settled about serde in
   `content-addressing.md` (spec05) are both of this kind.
2. **A reversal of a decision is a new document** with `supersedes:` /
   `superseded_by:`. `AGENTS.md`'s prohibition is scoped by its own first
   clause — *"edited to erase a **decision**"* — and that is the case the
   ceremony exists for: the record of what a decision beat must survive the
   decision changing. Nothing in this ticket is a reversal.

The rule this spec writes must therefore be stated as a boundary, not as a
blanket permission, and it must carry the erase-guard: **an edit adds; it never
deletes the sentence it corrects.** spec03's and spec05's verify lines enforce
exactly that by asserting the superseded prose is still on disk.

## Files

- `learnings/README.md` — §"What is enforced" only. The second bullet is
  rewritten; the first bullet (the container/bind-mount limit) and the
  "**Nothing.**" opening are unchanged and still true.

Not touched by this spec: any other file in `learnings/`, `README.md`,
`AGENTS.md`, anything under `conserved/`, any sibling tree.

## What the bullet must say

Replacing the stale claim, and at minimum:

- The folder **is** version controlled as of `ab154f7` (`p0-foundation`
  requirement 1), so a correction shadows its predecessor instead of
  overwriting it, and `git log -p learnings/<doc>.md` is where the predecessor
  lives.
- Rule 1 above: additions and factual corrections are edits in place, and an
  edit adds rather than deleting the sentence it corrects.
- Rule 2 above: a reversal of a decision is a new document linked by
  `supersedes:` / `superseded_by:`, quoting `AGENTS.md`'s "edited to erase a
  decision" as the scope.
- That this is a **correction of this README by its own rule 1** — the sentence
  it replaces stays visible in git history, which is the rule demonstrating
  itself.

## Acceptance

- [x] `learnings/README.md` no longer contains the string
      `It is not version controlled`.
- [x] It names commit `ab154f7` as the commit that made the folder version
      controlled, and that commit resolves.
- [x] It contains the word `shadow` (the mechanism) and `supersede` (the
      reserved case), and states which kind of correction each applies to.
- [x] The first bullet of §"What is enforced" — the container bind-mount limit
      — is still present verbatim (`bind-mounts the repo and nothing else`),
      and the section still opens by stating that nothing is enforced. This
      spec corrects one false sentence; it does not rewrite the section.
- [x] §"What a gate would check, if one is ever written" is unchanged — this
      spec adds no gate, and the folder is still ungated.
- [x] No file outside `learnings/README.md` is modified:  <!-- true at spec01 landing; specs 02-05 modify the other files by design -->
      `git status --porcelain learnings/ README.md` names that file and nothing
      else.

verify: `bash -c 'set -e; cd /Users/feb/dev/infra/shared; R=learnings/README.md; if grep -q "It is not version controlled" $R; then echo "FAIL: README still claims the folder is not version controlled"; exit 1; fi; grep -q "ab154f7" $R || { echo "FAIL: README does not name the git-init commit"; exit 1; }; git cat-file -e ab154f7^{commit}; grep -qi "shadow" $R || { echo "FAIL: README does not say a correction now shadows its predecessor"; exit 1; }; grep -qi "supersede" $R || { echo "FAIL: README does not reserve the supersede ceremony for reversals"; exit 1; }; grep -q "bind-mounts the repo and nothing else" $R || { echo "FAIL: the container bullet was lost"; exit 1; }; grep -q "^## What a gate would check" $R || { echo "FAIL: the would-be-gate section was lost"; exit 1; }; test "$(git ls-files learnings/ | wc -l | tr -d " ")" -ge 9 || { echo "FAIL: learnings/ is not tracked"; exit 1; }; echo "spec01 ok"'`
