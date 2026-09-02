---
type: learning
learning: a-shared-name-is-not-a-shared-function
subject: a similarity score is a candidate finder, never a licence to merge — three families in two trees share a name or a body and not a function
binds: [mitosys, model]
status: decided
date: 2026-08-29
code: mitosys src/mitosys/engine/util/util.rs:360; mitosys src/mitosys/engine/record/store.rs:1359; model src/record/event.rs:890; model src/daemon/mod.rs:1503
---

# a-shared-name-is-not-a-shared-function — three families, one rule

The duplication census behind `shared-crate.md` found three sets of functions
that score as duplicates and are not: one name with three behaviours, one
algorithm at two integer widths, and twenty copies of a scratch-name builder
of which one is correct. This document decides each of the three and states
the rule they earn. `mitosys/.pearde/prds/adopt-conserved` and
`model/.pearde/prds/adopt-conserved` read it before touching any site named below;
the code work is carried by two child PRDs, `mitosys/.pearde/prds/civil-from-days-has-one-width`
and `model/.pearde/prds/same-name-is-not-same-function`.

Every line number here is as of mitosys `251809fe` and model `279192e2`,
`HEAD` in each tree on 2026-08-29 — the probe extracts by symbol, so a moved
line changes nothing it measures.

## The rule

**A similarity score is a candidate finder, never a licence to merge.**

A merge of two candidates needs all four, checked, before one caller moves:

| condition | what it rules out |
|---|---|
| same name | family 2's `days_from_civil` and `civil_from_days` are inverses, not copies |
| same stated behaviour | family 1: one name, three contracts |
| same signature, domain and range | family 2: one algorithm, `i32` year against `i64` year |
| every call site read against the stated behaviour | family 3: a builder without the counter is a defect at one caller and harmless at the rest |

The census that found these scored `civil_from_days` a duplicate and it is
one — the divergence is in the signature, where a body matcher cannot look.

## Evidence

`sh .pearde/prds/a-shared-name-is-not-a-shared-function/probe/run.sh` from the
master root re-measures every site. It extracts the bodies from the trees at
run time and compiles them with bare `rustc` into a run-time directory — no
tree build, no lock taken in any member. `sh probe/census.sh` is the family 3
census. Both are read-only over the trees. The numbers below are the
2026-08-29 run.

## Family 1 — `model`, three functions named `one_line`

| site | behaviour | callers |
|---|---|---|
| `src/record/event.rs:890` | first line, clipped to `FEED_TEXT_MAX` (40, `event.rs:657`) with `…` | `event.rs:829`, `event.rs:872`, `event.rs:879` — the feed line |
| `src/chat/app.rs:164` | every whitespace run to one space, lines joined | `app.rs:132-149` — the pane |
| `src/improve/llm.rs:144` | first line, trimmed, 200 chars then `…` | `llm.rs:133`, `llm.rs:229`, `ollama.rs:283`, `ollama.rs:297` — a log head of an HTTP body |

One input, `"  hello   world  \n  second   line  "`, three outputs:

| site | output |
|---|---|
| `event.rs:890` | `"  hello   world  "` |
| `app.rs:164` | `"hello world second line"` |
| `llm.rs:144` | `"hello   world"` |

A 250-char line: 40, 250 and 201 chars. Probe verdict: `DIVERGE`.

**Decision: the copies stay, renamed.** Three consumers, three contracts, no
shared function.

| site | name | contract |
|---|---|---|
| `event.rs:890` | `feed_line` | first line, clipped to `FEED_TEXT_MAX` with `…` |
| `app.rs:164` | `pane_line` | every whitespace run to one space, lines joined |
| `llm.rs:144` | `log_head` | first line, trimmed, 200 chars then `…` |

The bodies do not change. A reader unifying by name would change what the
feed, the pane and the log print at once.

## Family 2 — `mitosys`, the calendar pair at two widths

The PRD's row `store.rs:1350` is `store.rs:1359` at `251809fe`, and its third
row is not a `civil_from_days` — it is the inverse. Five production bodies,
two test copies:

| site | fn | width |
|---|---|---|
| `src/mitosys/engine/util/util.rs:360` | `civil_from_days(i64) -> (i32, u32, u32)` | computes in `i64`, casts the year `as i32` at `util.rs:370` |
| `src/mitosys/engine/record/store.rs:1359` | `civil_from_days(i64) -> (i64, u32, u32)` | `i64` end to end |
| `src/mitosys/engine/util/util.rs:336` | `days_from_civil(i32, u32, u32) -> i64`, nested in `parse_rfc3339` (`util.rs:322`) | month via `if m > 2 { m - 3 } else { m + 9 }` |
| `src/mitosys/engine/channel/channel.rs:847` | `days_from_civil(i64, i64, i64) -> i64` | month via `(m + 9) % 12` |
| `src/mitosys/engine/record/tests/store.rs:331` | copy of store's | its doc comment claims a second copy "does not drift" |
| `src/mitosys/engine/channel/tests/channel.rs:79` | copy of channel's | — |

Probe rows:

| input | `util.rs:360` | `store.rs:1359` | verdict |
|---|---|---|---|
| `z = 1000000000000` | `(-1557058320, 12, 27)` | `(2737908976, 12, 27)` | `DIVERGE` — the `i32` year wraps silently |
| `z = -1000000000000` | `(1557062259, 1, 5)` | `(-2737905037, 1, 5)` | `DIVERGE` |
| round trip over 2,000,000 days, both pairs | — | — | 0 mismatches |

| input | `util.rs:336` | `channel.rs:847` | verdict |
|---|---|---|---|
| `2024-15-01` | `20150` | `19783` | `DIVERGE` — neither `parse_rfc3339` (`util.rs:322`) nor channel's parser (`channel.rs:795`) range-checks month or day |
| `2024-00-01` | `19692` | `19692` | agree |

Over the valid domain the pairs agree. Outside it, the two widths and the two
month formulas each give a different answer, and no caller today can tell.

**Decision: one pair, `i64` year, in `engine/util/util.rs`.**

| item | decision |
|---|---|
| signatures | `pub fn civil_from_days(z: i64) -> (i64, u32, u32)`, `pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64` |
| the cast | the `as i32` at `util.rs:370` goes; `date_string` and `datetime_string` format the `i64` |
| domain, in the doc comment | `m` in `1..=12`, `d` in `1..=31` |
| the parsers | `parse_rfc3339` and channel's parser reject a month or day outside the domain before calling — a stated behaviour change: `2024-15-01T00:00:00Z` parses today to two different instants and afterwards to none |
| overflow | `i64` throughout; `\|z\|` beyond `2^62` days is outside the domain and no caller can produce it (`u64` seconds / 86400 < 2.2e14) |
| deleted | `store.rs:1359`, `channel.rs:847`, `tests/store.rs:331`, `tests/channel.rs:79` — their callers use util's pair |
| where | tree-local. `shared/shared/src/clock.rs:81-82` rules that `civil_from_days` stays mitosys's: a calendar is not a clock, and it has no second consumer |

## Family 3 — `model`, the scratch-name builders

`sh probe/census.sh` lists every `env::temp_dir()` site: 47 in `model`, 5 in
`realm`. In `model`, 43 build a name of their own; 14 carry a per-process
counter (`fetch_add`), 29 do not. `daemon/mod.rs:1483` delegates;
`daemon/mod.rs:1636` is a sweep root, `grade/report.rs:591` a message,
`grade/tests/stale_record.rs:428` a fixed name — none of the four is a
scratch name.

The builder that has all three parts is `src/daemon/mod.rs:1503`
`scratch_registry_name` — counter `SCRATCH_REGISTRY_SEQ` at `daemon/mod.rs:1471`,
prefix `SCRATCH_PREFIX` at `daemon/mod.rs:1514`. Its own doc comment
(`daemon/mod.rs:1462-1516`) describes the defect the copies still have: macOS
resolves `SystemTime` to about a microsecond, so two calls in one tick get one
path. The loop harness's `src/loop/harness/tests.rs:11` `TEMP_DIR_SEQ` is the
other counter.

Of the 29 without a counter:

| site | shape | reading |
|---|---|---|
| `src/mcp/fold.rs:855` `snapshot_path()` (fn at `fold.rs:850`), called at `fold.rs:189` when a registry lock is held | nanos only | production, the defect |
| `src/gossip/seed_leech.rs:1111` | pid only | a per-process identity file keyed by pid on purpose — not a scratch path |
| `daemon/tests/counters.rs:124`, `grade/tests/baseline.rs:9`, `grade/tests/probe.rs:319`, `improve/tests/target.rs:8`, `utils/tests/deadline_gate.rs:333` | pid, no clock | five test builders |
| 22 test helpers | pid and clock, or clock only | collide on a fast clock |

Probe, 10,000 calls each:

| shape | result |
|---|---|
| `fold.rs:855`, nanos only | 9381 duplicate paths in 10,000 calls |
| `scratch_registry_name`, counter + pid + clock | 0 duplicate paths |
| the clock itself | 363 distinct values in 10,000 reads |

The duplicate count for the nanos-only shape moves by a few hundred between
runs (9391, 9246 and 9381 on 2026-08-29). The zero for the counter shape does
not.

**Decision: one builder. A scratch name carries pid, a per-process counter
and the clock, or it is not a scratch name.**

| item | decision |
|---|---|
| the builder | every name builder in `model` routes through one function; `scratch_registry_name` is the shape, its module is `model`'s board's call |
| `fold.rs:855` | routes through the builder — the production defect |
| `seed_leech.rs:1111` | stays as it is; its doc comment says it is a per-process identity file, not a scratch path |
| the five pid-only builders | each says in its doc comment that a fixed per-process path is the intent, or routes through the builder |

## Who reads it

| reader | before |
|---|---|
| `mitosys/.pearde/prds/adopt-conserved` | touching `engine/record/store.rs` or `engine/util/util.rs` |
| `model/.pearde/prds/adopt-conserved` | touching `src/record/event.rs` or any `temp_dir()` site |
| `mitosys/.pearde/prds/civil-from-days-has-one-width` | carries family 2 |
| `model/.pearde/prds/same-name-is-not-same-function` | carries families 1 and 3 |

## Re-measure

`sh .pearde/prds/a-shared-name-is-not-a-shared-function/probe/run.sh` from the master
root. Family 1 and both family 2 rows print `DIVERGE` until the children land;
afterwards `run.sh` extracts nothing for the deleted symbols and its `rustc`
step fails on them, which is the signal that the tree no longer has the
copies.
