---
complexity: 10
footprint:
  - ../mitosys/src/mitosys/util/util.rs
  - ../mitosys/src/mitosys/util/tests/unit/util.rs
---

# spec04 — delete `percentile_sorted`, a zero-caller function `conserved::stats` has no generic home for

Delete `util::percentile_sorted` (`util/util.rs:104-116`) and its test
`percentile_sorted_is_nearest_rank_with_edges_and_generic_types`
(`util/tests/unit/util.rs:26-40`). Measured this session: zero non-test call
sites (`grep -rn percentile_sorted src --include='*.rs'` returns only the
definition and its own test) — the deletion is dead-code removal, not a
replace-with-`conserved::stats::percentile` swap, because `conserved::stats`
is `&[f64]`-only (p4's decision) while `percentile_sorted` is generic over
`T: Copy` and one of its own test cases exercises `u128`, which has no home
in the `&[f64]`-only crate.

**Flag, do not silently resolve: the PRD's wording for this bullet is
internally inconsistent** and this spec follows the literal, larger
instruction ("percentile_sorted deletion also deletes ... util.rs:26") over
the smaller one that contradicts it. The PRD also says to "rewrite lines
31-34" of that same test to expect `Some(6.0)` instead of `Some(5.0)` under
p4's upper-median decision — but if the whole test function is deleted (as
the first clause requires, since its subject no longer exists), there is
nothing left at those lines to rewrite. This spec treats "becomes `Some(6.0)`
under p4's upper-median decision" as a rationale for why deleting the
`Some(5.0)` assertion is safe (it was already a different definition of
median than the one the family settled on, so nothing is lost by removing
it) rather than as an instruction to keep a rewritten assertion with no
function left to test. If that reading is wrong, it is a question for
whoever wrote the PRD bullet, not a fact this session could resolve by
reading more code.

## Acceptance

- [ ] `grep -rn percentile_sorted ../mitosys/src --include='*.rs'` returns no
      results.
- [ ] `util/util.rs` no longer defines `percentile_sorted`.
- [ ] `util/tests/unit/util.rs` no longer defines
      `percentile_sorted_is_nearest_rank_with_edges_and_generic_types` (all
      of it, including the `u128` vectors) — not a partially-edited
      survivor.
- [ ] The crate's other tests in that file (`hex_encode_...`,
      `hex_decode_...`, `content_hash_...`, `cmp_rank_...`, etc.) are
      untouched and still pass.
- [ ] `cargo test -p mitosys-util` passes inside the offline container
      (spec01's mechanism).

## Verify and Proof

```sh
cd ../mitosys
grep -rn percentile_sorted src --include='*.rs'
cargo test -p mitosys-util
just check
```
