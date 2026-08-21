# goal

`git init` and one initial commit, so the board is claimable and "the record
only grows" has a subject. The board protocol claims by commit; until this
lands, no node on this board can be claimed by anyone.

Runs after spec01–spec03, so the initial commit records the reset workspace
and the settled distribution decision — not the condemned scaffold and not a
document calling a decided question open.

## Files and dirs

- `.git/` — created by `git init` at the repo root
  (`/Users/feb/dev/infra/shared`). Note there is no parent repository: this is
  a new root, not a submodule.
- `.gitignore` — new, at the repo root:

  ```
  # Build output
  /target

  # macOS
  .DS_Store

  # Local index/cache, regenerable, binary
  /.pi/kern/data/
  ```

- The initial commit, containing: `AGENTS.md`, `README.md`, `Cargo.toml`,
  `rustfmt.toml`, `.editorconfig`, `.gitignore`, `conserved/`, `learnings/`,
  `.mi/` (the board, the memos, the gantt plan), `.pi/ontology/`.

If a `user.name` / `user.email` is not configured, set them per-repository
rather than globally.

## Acceptance

- [x] `git rev-parse --git-dir` succeeds from the repo root and resolves to
      this repository's own `.git`, not a parent's.
- [x] `git log --oneline` shows at least one commit with a message that says
      what it is (the foundation: reset workspace, board, learnings).
- [x] `git status --porcelain` is empty — every file is either committed or
      deliberately ignored. No "I'll commit that later".
- [x] The commit carries the board and the learnings, not just the code:
      `git ls-files` includes `.mi/prd/prd.md`,
      `.mi/prd/p0-foundation/prd.md`, `.mi/docs/memos/distribution.md`,
      `.mi/docs/memos/scaffold-reset.md`, `learnings/shared-crate.md`,
      `conserved/Cargo.toml`, `conserved/src/lib.rs`,
      `conserved/tests/smoke.rs`.
- [x] Nothing regenerable or binary is tracked: `git ls-files` matches no
      `target/`, no `.DS_Store`, no `*.mdb`.
- [x] The deleted scaffold is absent from the tracked tree:
      `git ls-files` matches no `conserved-core`, `conserved-alloc`,
      `conserved-net`, `conserved-deriv`, `conserved-derive`.

## est

0.5

verify: `sh -c 'set -e; git rev-parse --git-dir >/dev/null; test -n "$(git log --oneline)"; if [ -n "$(git status --porcelain)" ]; then echo "tree dirty"; git status --short; exit 1; fi; for f in .mi/prd/prd.md .mi/prd/p0-foundation/prd.md .mi/docs/memos/distribution.md .mi/docs/memos/scaffold-reset.md learnings/shared-crate.md conserved/Cargo.toml conserved/src/lib.rs conserved/tests/smoke.rs; do git ls-files --error-unmatch "$f" >/dev/null; done; if git ls-files | grep -qE "(^|/)target/|\.DS_Store|\.mdb$|^conserved-(core|alloc|net|deriv|derive)/"; then echo "tracked what must not be tracked"; exit 1; fi; echo "repository initialised and committed"'`
