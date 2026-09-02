---
type: learning
learning: realm-transport
subject: the kind: realm transport lives in the mitosys repo and a two-tree integration test is the proof — option (b) stands, the membrane revisit trigger has not fired, realm's permanently-blocked boxes stay open by decision
binds: [mitosys, realm]
status: decided
date: 2026-08-24
code: realm src/cli/src/lib.rs, realm src/drivers/linux/src/lib.rs, realm src/ssh/src/driver.rs, mitosys src/mitosys/api/plugin, mitosys src/mitosys/api/plugin/world/wit/mitosys-plugin.wit
---

# realm-transport — where the carrier meets the protocol, and the gate between them

A board exists because the work does not fit on either tree alone.
`realm/.pearde/prds/prd.md` records that the `kind: realm` transport lives in the
mitosys repo and that no ACP framing has crossed the carrier realm already
proves (`realm/.pearde/prds/prd.md:54`, `realm/.pearde/prds/prd.md:70`). The carrier is real
and tested; the protocol is real and tested. The seam between them is not on
either tree's board, which is what this document closes.

## Decision

The `kind: realm` transport lives in the **mitosys repo** (option (b), chosen
on 2026-08-21 and kept on 2026-08-24) and is a membrane-loadable plugin that
drives realm's carrier surface. Realm stays the substrate, the transport
plugin surfaces the substrate inside mitosys, and the carrier boundary realm
must keep stable is its **CLI subcommand shape** (`realm create`, `realm ps`,
`realm exec`, `realm destroy`, `realm snapshot`, `realm rollback`, `realm
info`, `realm logs`, `realm metrics`) plus the **live bidirectional stdio
channel** to a process that is pid 1 of the container
(`attach_hands_over_live_pipes_into_the_container` proves this on
`realm/.pearde/prds/prd.md:53`) and the **SSH-equivalent** `ssh_exec_args` channel
(`realm/.pearde/prds/prd.md:78`). The plugin wraps that surface; the surface is the
contract realm owes this seam.

## What proves it

A **two-tree integration test** drives ACP framing across the carrier realm
already proves. The test has two halves:

1. **On realm** — a green `cargo test --workspace` plus the ZFS-gated suites
   (`zfs_integration`, `zfs_volume`, `zfs_quota`, `zfs_rollback`) and the
   SSH-gated suites (`ssh_integration`, `ssh_exec`) on the Lima guest
   `realm-zfs` and on the CI arms (`ubuntu-latest` and `macos-latest`). The
   realm half is the carrier; it stands on its existing gate and is not
   changed by this PRD.
2. **On mitosys** — a new test under the child PRD filed at
   `mitosys/.pearde/prds/p9-realm-transport/prd.md` (spec04) that loads the
   `kind: realm` transport plugin, dispatches an agent, and asserts the
   workspace is provisioned, the agent runs, and the workspace is destroyed
   on settle. The test runs under the child PRD's gate (whatever that PRD
   defines) and crosses the seam with the real `realm` binary on the host.

The proof is the bridge: realm's gate proves the carrier works on the
substrate, mitosys's gate proves the protocol drives it, and the integration
test is the only thing in either tree that proves the seam.

## Gate ownership, plainly

- **realm's gate** is `cargo test --workspace` on `ubuntu-latest` and
  `macos-latest` (the CI arm, gated by `.github/workflows/ci.yml`), plus the
  ZFS-gated suites that only run on the Lima guest `realm-zfs` (a real pool
  and sshd are required, and the guest is where they live), plus the root
  arm under `just test-root` (a `just` recipe the PRD itself names). The
  gate stays where it is; this PRD adds nothing to it.
- **mitosys's gate** is the new child PRD's gate, which is whatever the
  child PRD at `mitosys/.pearde/prds/p9-realm-transport/prd.md` defines — most
  likely `just all` (Rust + Lua, the project's standard gate) plus a
  feature-gated test that depends on a `realm` binary on `PATH`. The
  integration test is what gives the child PRD something dispatchable.

The seam is gated by **both** gates, each in its own tree, and the
integration test is the only thing that crosses them.

## The carrier boundary realm must keep stable

The plugin depends on the surface, and the surface is the contract. The
named boundary:

- **CLI subcommands**: `realm create` (with `--image <rootfs-dir>`,
  `--zfs <dataset@snap>`, `--driver <uri>`, `--cpus N`, `--memory MiB`,
  `--disk MiB`), `realm ps`, `realm exec <ws> -- <cmd>`, `realm destroy`,
  `realm snapshot <ws> --snap <name>`, `realm rollback <ws> <snap>`,
  `realm info`, `realm logs`, `realm metrics --json`. The names and the
  flag shapes are the contract; the JSON output of `realm metrics --json`
  is the only piece whose schema is load-bearing.
- **Container transport**: a live bidirectional stdio channel to a process
  that is pid 1 of the container, asserted by
  `attach_hands_over_live_pipes_into_the_container`
  (`realm/src/drivers/linux/tests/linux_container.rs`). The channel is the
  contract; the test name is the proof.
- **SSH transport**: the same shape over `ssh_exec_args` —
  `realm/.pearde/prds/prd.md:78` — gated by
  `exec_over_ssh_streams_output_and_propagates_the_exit_code` on the
  `ssh` driver. The remote end is the contract; the test name is the proof.

A change to any of these is a breaking change to the plugin. The plugin
moves on the carrier; it does not reshape it.

## Alternatives considered

Each fork `.pearde/prds/realm-transport/prd.md` `## The forks` names has a verdict.
The standing option (b) is included as the chosen one, not omitted for being
the standing answer — a learning that names only what was picked is a claim,
and the board reads both halves.

### Where the transport lives

- **(a) `kind: realm` in the mitosys repo, plus a broad cross-repo
  contract.** The transport lives where option (b) puts it, but the
  contract reaches across the seam into realm's source: realm promises an
  internal shape, the plugin depends on it, and the integration test
  asserts a `kind: realm` workspace against a `realm` crate built in
  place. **Not chosen.** The cross-repo contract turns realm's public
  surface (CLI + agent tool contract) into a partial surface, which is the
  same drift `.pearde/prds/realm-transport/prd.md` `## Out of scope` excludes —
  *"no hard dependency on mitosys"* in realm's tree is symmetric with
  *"no public surface claim outside the CLI"* in the cross-repo case.
  Cost in the tree that loses: realm loses the right to reshape its
  internals without breaking the plugin.
- **(b) `kind: realm` in the mitosys repo, plugin wraps the CLI +
  agent tool contract surface (the standing answer).** The transport
  lives in the mitosys repo and consumes realm's carrier surface, which
  is the surface realm already keeps stable for itself. **Chosen.** The
  surface is the contract realm owes its own callers; the plugin rides it
  for free. Cost in the tree that loses: none — both trees keep their
  boundaries intact.
- **(c) The plugin moves out of the mitosys repo and pins the WIT
  world.** Realm ships its own plugin in its own repo, importing
  `api/plugin/world` as a versioned package. **Not chosen today.** The
  trigger is named in `shared/learnings/plugin-core.md` §The revisit
  trigger (a second repo pinning the WIT world's version), but the
  trigger has not fired (see `## Membrane revisit finding` below) — the
  `api/plugin/world` version is still one tree's bump, and the plugin
  still ships through mitosys's plugin host. Cost in the tree that loses:
  the family loses the admission test's criterion 1 (only mitosys has
  plugins) — the very criterion `plugin-core.md` settled 2026-08-23 —
  and pays the cross-repo rev-pin cost `plugin-core.md` §The sharp edge
  stays one repo's already prices.
- **(d) Rewrite the boxes as a compatibility-surface claim realm can
  meet.** The two PERMANENTLY BLOCKED boxes become a documented claim
  about the carrier surface, not a transport implementation. **Not
  chosen.** This is the scope-narrowing path the realm board's
  `## Answers` 2026-08-21 entry called the user's to make, and the user
  chose to keep them as written. Cost in the tree that loses: the
  intent is lost from realm's board, which is the cost option (b)'s
  rejector named in `realm/.pearde/prds/prd.md:187-199`.

### What proves it

- **A two-tree integration test that drives ACP framing across the
  carrier realm already proves.** **Chosen.** The test runs on both
  trees under both trees' gates, gated by a named job on each — realm's
  CI arm plus the Lima guest's ZFS/SSH suites, mitosys's `just all`
  plus the new test under the child PRD. The seam is gated by both, the
  integration test is the only thing that crosses them. Cost in the tree
  that loses: the cost is paid on the gate, not in source — neither
  tree gives up code.
- **A single-tree contract test that mocks the carrier.** **Not
  chosen.** A test that mocks the carrier proves the plugin's
  integration with the WIT world, not the plugin's integration with
  realm. A green mock is a green claim about a contract that does not
  exist on a host without `realm`; the test passes on a CI box with no
  ZFS and no sshd and proves nothing about the seam. Cost in the tree
  that loses: the test gives a false positive on the board.
- **No test, just documented behavior.** **Not chosen.** A learning
  that names a test but does not run it is a wish — the failure mode
  `shared/learnings/divergences.md` §The general lesson names
  ("a shared rule with nothing running it decays"). Cost in the tree
  that loses: the seam becomes a paragraph rather than a gate.

### Whether it reopens membrane's home

This is a finding, not a decision; see `## Membrane revisit finding` below.

## Membrane revisit finding

The revisit trigger in `shared/learnings/plugin-core.md` §The revisit trigger
reads:

> Reopen as a new master-board PRD when either event occurs: (1) a second
> tree wants to **host** plugins, or (2) a second repo needs to pin
> `api/plugin/world`'s package version.

The current state, read off the recorded facts:

- `realm/.pearde/prds/prd.md:27-34` plans realm *as* a mitosys plugin and forbids
  a hard dependency on mitosys; the `kind: realm` transport stays in the
  mitosys repo (option (b)).
- `shared/learnings/plugin-core.md` itself names the very event as an
  example trigger — *"e.g. realm's plugin moving out of the mitosys
  repo."* Whether the trigger has **fired** is the open question.
- `api/plugin/world` is one file, `package mitosys:plugin@0.1.0`, and
  today one repo bumps it (`shared/learnings/plugin-core.md` §The sharp
  edge stays one repo's).
- `.pearde/prds/membrane-home` settled 2026-08-23 on the same facts: criterion 1
  fails (only mitosys has plugins), criteria 2 and 3 pass, criterion 4 is
  unanswerable while there is one implementation.

**Verdict: the trigger has not fired as of today (2026-08-24).** A *plan*
is not a *move* — `realm/.pearde/prds/prd.md:27` records the plan, the move is a
commit that ships realm's plugin from a different repository, and that
commit has not landed. The `api/plugin/world` version is still one tree's
bump, and `membrane-home` settled 2026-08-23 on the same shape. The
finding is consistent with `.pearde/prds/membrane-home` `state: done` and adds
nothing to it.

**The condition that flips this finding.** The day realm's plugin moves
out of the mitosys repo, criterion (2) of the trigger fires (a second
repo needs to pin `api/plugin/world`'s package version), the verdict
here flips, and a new master-board PRD lands at `.pearde/prds/membrane-reopen/`
with `state: open` and `from: realm-transport` recorded on the body. The
mechanism is `shared/learnings/plugin-core.md` §The revisit trigger, the
filing is `.pearde/prds/membrane-reopen/prd.md` (does not exist today — its
absence is part of the verdict; if it existed the verdict would be (b)).
No `.pearde/prds/membrane-reopen/` exists, so the verdict is (a).

## Disposition of realm's blocked boxes

The two boxes in `realm/.pearde/prds/prd.md` `## Requirements` and the matching
acceptance line are PERMANENTLY BLOCKED by the user's 2026-08-21 answer
(option (b) in `realm/.pearde/prds/prd.md` `## Escalation — SETTLED`). The realm
board keeps them unstruck and never reaches `state: done` while realm
ships standalone — **this is the recorded decision, not a gap**, and the
master board's `## Acceptance` line *"Realm's permanently-blocked boxes
either become closable, or the learning records that they stay open by
decision and why"* is closed by this section recording the second: **they
stay open by decision.**

The answer the user gave (`realm/.pearde/prds/prd.md:133-143`, 2026-08-21) is
quoted by reference: *"keep both boxes, mark them PERMANENTLY BLOCKED. Do
not strike them, do not create a mitosys ticket, do not write mitosys
code."* The user explicitly accepted the consequence: the realm root PRD
never reaches `state: done` while realm ships standalone. The
contradiction with `## Out of scope` (*"No mitosys transport in this repo
— the `kind: realm` transport lives in the mitosys repo."*) is knowingly
retained rather than edited away — the boxes record an intent whose
home is another repository, and the exclusion records why it cannot be
built here.

The cost, spelled out so a later session does not re-derive it: the realm
board's `state` will not transition to `done` while the two boxes are
unstruck, and that is the user's recorded decision. The board carries the
record; the cost is the price of the contradiction's transparency.

**What would make them closable.** A single commit in the mitosys repo
that lands the `kind: realm` transport — the plugin the child PRD at
`mitosys/.pearde/prds/p9-realm-transport/prd.md` (spec04) defines — and a
two-tree integration test that proves the seam. With that commit, the two
boxes become a requirement (the transport exists) and a verified
acceptance (the test is green on both trees), and the realm board's
escalation entry can be struck. The boxes are not closable in realm's
tree; they are closable in the union of the two trees, and the union
is what the child PRD plus the integration test build.

## What this binds

- **mitosys** owes: the `kind: realm` transport plugin (the child PRD
  at `mitosys/.pearde/prds/p9-realm-transport/`), an integration test that
  drives it against a real `realm` binary, and a gate that proves the
  test on the host that runs CI.
- **realm** owes: the carrier boundary named in `## The carrier boundary
  realm must keep stable` — the CLI subcommand shape, the live
  bidirectional stdio channel, and the SSH-equivalent — and the carrier
  tests that already gate it (`realm/src/drivers/linux/tests/`,
  `realm/src/ssh/tests/`, `realm/src/cli/tests/`).

The seam is gated by both trees; the contract is the surface both trees
already keep stable.
