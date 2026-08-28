---
complexity: 6
footprint:
  - prds/p5-adoption/prd.md
---

# spec02 — close the five boxes from the ledger: three ticked, two struck

`prd.md` carries five `- [ ]` boxes under `## Requirements`, and the board's
own gate (`shared/tests/done_boxes_are_ticked.rs`, whole-file, `- [~]` read
as a closure) turns red the moment this node is `state: done` with any of
them open. This spec closes each one against the `## Ledger — 2026-08-28`
that `spec01` wrote, in the two forms `learnings/done-means-done.md` allows:
a tick with the verify output quoted, or a strike with the reason and the
owner. Nothing else in the file changes, and frontmatter is not touched.

## What already stands

`spec01`'s ledger holds the evidence; the probe at `probe/ledger.sh` re-runs
it. The verdict per box, from the analyst's run on 2026-08-28:

| box | verdict | why |
|---|---|---|
| mitosys | **strike** | five of six clauses hold; "clock reads route through `Clock`" does not — `engine/util/util.rs:206-222` still reads `SystemTime::now`, 41 reads under ceiling 48. Owner: `mitosys/prds/adopt-conserved` (`state: open`) |
| llm | **strike** | `Scope`, `rec_now()` → `Clock`, `min_median_max`, and the edition note all hold; "the ~65 wall-clock reads route through `Clock`" cannot — they were monotonic `Instant::now` (llm child box 3), and the real wall-clock count went 15 → ceiling 10. Owner of the remainder: model's ratchet and `model/prds/adopt-conserved` (`state: open`) |
| realm | **tick** | 4 `Scope::new()` sites; `ContentId`/`stats` refused on two greps that exit 1; the adoption forced no crate change (`795f1df` + `ad1b3b4` touch `prds/` only); `cd ../realm && just check` exit 0 |
| ratchets | **tick** | `wall_clock_reads_may_only_decrease` green in all three trees — mitosys 7/0, model 3/0, realm 3/0 — ceilings 48 / 10 / 0 |
| load proof | **tick** | `cargo test -p shared load -- --include-ignored` — 5 + 1 + 7 passed, 0 failed |

## What is left

Edit the five boxes in place, in `## Requirements`:

- A **tick** rewrites the marker to `[x]` and appends, indented under the box,
  the verify output quoted from the implementer's own run of
  `probe/ledger.sh` (the `ok` lines for that box, with their counts and
  `N passed; 0 failed`). The box text is unchanged.
- A **strike** rewrites the line to the form the brief fixes —
  the marker `[~]`, the original text wrapped in `~~…~~`, then ` — ` and the
  reason: which clauses hold (with file:line), which one does not (with
  file:line and the count), and the owner PRD by path and state. Keep the
  original wording inside the strikethrough verbatim; a strike records the
  bar as written, it does not move it.

Run the probe yourself before ticking; quote its output, not the analyst's.
If the probe prints a `FAIL` line, stop — the box it belongs to is not
closable and the spec is blocked, not done.

## Acceptance

- [x] the realm box is `[x]` and the text under it quotes `just check` exit 0 with the vendor-check `content match` line, the four `Scope::new()` sites, and the two greps that exit 1
      — quoted from the implementer's probe run 2026-08-28: `Scope adopted at 4 sites: ...`, `... both exit 1`, `cd ../realm && just check — exit 0 (vendor-check rev agreement + content match, ...)`
- [x] the ratchets box is `[x]` and the text under it quotes `wall_clock_reads_may_only_decrease ... ok` from each of the three trees with their `N passed; 0 failed` summaries and the three `RATCHET_CEILING` values
      — mitosys `ok. 7 passed; 0 failed`, model `ok. 3 passed; 0 failed`, realm `ok. 3 passed; 0 failed`; `RATCHET_CEILING` 48 / 10 / 0 at `:228` / `:82` / `:55`
- [x] the load-proof box is `[x]` and the text under it quotes the three `load_*` binaries' `passed; 0 failed` lines
      — `load_scope` 5 passed; 0 failed, `load_throughput` 1 passed; 0 failed, `load_unwind_panic` 7 passed; 0 failed
- [x] the mitosys box is `[~]`, its original text inside `~~…~~`, the reason naming `engine/util/util.rs:206-222` and `mitosys/prds/adopt-conserved`
      — `grep -n 'engine/util/util.rs:206-222' prd.md` → line 43 (the box) and the ledger row; `grep -q 'mitosys/prds/adopt-conserved'` exit 0
- [x] the llm box is `[~]`, its original text inside `~~…~~`, the reason naming the llm child's box 3 (monotonic `Instant::now`), the 15 → 10 wall-clock count, and `model/prds/adopt-conserved`
      — the box reads "monotonic `Instant::now` (74 in non-test source today), which the llm child's box 3 ... forbids converting"; "15 → ceiling 10"; `grep -q 'model/prds/adopt-conserved'` exit 0
- [x] `prd.md` holds no open box in any spelling the gate counts (`-`, `*`, `+`, `1.`, `1)` followed by an empty bracket pair) — under `## Requirements`, `## Acceptance`, or any other heading
      — `! grep -qE '^[[:space:]]*([-*+]|[0-9]+[.)]) \[[[:space:]]*\]' prd.md` exit 0; ticked 3, struck 2
- [x] `cargo test -p shared --test done_boxes_are_ticked` passes — 3 passed; 0 failed
      — `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (`every_done_prd_has_no_unticked_box`, `the_matcher_reads_every_spelling_of_one_rendered_box`, `exemption_list_only_names_done_prds` all `ok`); second probe run after the edits: `EXIT=0`, 0 `FAIL`, 6 `GAP`, identical to the first

## Verify and Proof

```sh
sh prds/p5-adoption/probe/ledger.sh
! grep -qE '^[[:space:]]*([-*+]|[0-9]+[.)]) \[[[:space:]]*\]' prds/p5-adoption/prd.md
test "$(grep -cE '^[[:space:]]*- \[x\] \*\*' prds/p5-adoption/prd.md)" -eq 3
test "$(grep -cE '^[[:space:]]*- \[~\] ~~' prds/p5-adoption/prd.md)" -eq 2
grep -q 'mitosys/prds/adopt-conserved' prds/p5-adoption/prd.md
grep -q 'model/prds/adopt-conserved' prds/p5-adoption/prd.md
cargo test -p shared --test done_boxes_are_ticked
```
