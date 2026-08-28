---
type: memo
memo: distribution
subject: how conserved reaches its consumers — requirement settled (distributable to every Rust repo), mechanism decided (git dependency pinned by commit; mitosys's offline story is separate follow-up work)
status: decided
date: 2026-08-20
decided: 2026-08-21
---

# distribution — where the shared code lives

`learnings/shared-crate.md` §"Where it lives" names three options and
deliberately records no recommendation, because the deciding constraint is
not technical. This memo carries the decision when it is made; until its
status reads `decided`, no extraction lands (it is the fourth requirement of
`p0-foundation`).

## What is settled

The user stated the requirement 2026-08-20: the crate must be
**distributable to all other Rust repos** — the three trees here today and
any future one. That eliminates option 3 (a path dependency to a sibling
directory), which is host-only by construction: it does not exist for a
clone, and does not exist inside mitosys's dev container, which bind-mounts
the repo and nothing else.

## What is open — the two remaining mechanisms

**Option A — git dependency, pinned by commit.** This repo becomes a git
repository (p0 does that regardless); each consumer's `Cargo.toml` says
`conserved = { git = "...", rev = "<commit>" }`. Clean clone story, explicit
pin per consumer, drift impossible without a visible rev bump.
*Cost:* cargo fetches at build time. mitosys's container has no network, so
mitosys additionally needs `cargo vendor` output or a pre-populated registry
cache committed/mounted — the offline story must be designed, not assumed.

**Option B — vendored into each tree.** A copied `vendor/conserved/`
directory (or subtree) in each consumer, with the source commit hash
recorded, and a gate that fails when the recorded hash is behind the source.
Everything builds everywhere with zero network, container included.
*Cost:* the sync is manual and must be gated in every consumer, or the trees
silently compile against different content — the exact failure the workspace
dependency pin exists to prevent, reappearing one level up.

## The deciding question, put plainly

**Must `just check` keep passing inside mitosys's offline container without
new machinery?** Yes → Option B (vendoring) is the only mechanism that works
today. No, or the container may grow a vendored registry cache → Option A is
strictly cleaner and is what "distributable to any repo" most naturally
means; a hybrid (git dep + `cargo vendor` committed in mitosys only) puts
the cost where the constraint is.

## Decision (2026-08-21)

**Option A — git dependency, pinned by commit.** Each consumer's
`Cargo.toml` pins `conserved` by commit rev; drift requires a visible rev
bump.

**Who carries the offline cost:** mitosys, not this repo. mitosys's dev
container has no network, so its offline build story (`cargo vendor` output
or a pre-populated registry cache covering the pinned `conserved` rev) is
follow-up work scoped to mitosys, not designed here and not a blocker for
`p0`/`p1` in this repo. It must be designed before mitosys's adoption step
lands, and should be tracked as an explicit task rather than assumed away.

## Amendment

**Amended 2026-08-28 — the crate was renamed; this memo is not rewritten.**
The crate this memo calls `conserved` is now named `shared`, and the directory
holding it moved from `shared/conserved/` to `shared/shared/`
(`learnings/crate-name.md`, `binds: [mitosys, model, realm, shared]`). Every
`conserved` above — including the `subject:` line, the `Cargo.toml` snippet in
§"What is open", and both mentions in §"Decision (2026-08-21)" — is **the name
as it stood when the decision was made** and is left standing as the record;
read it as `shared` wherever it names the live crate. The decision itself is
unchanged: a git dependency pinned by commit rev, with mitosys carrying the
offline cost. Nothing else in this file moves, because a memo is a record and
the tree does not rewrite what was decided under the name it was decided under.
