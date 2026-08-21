# goal

Correct `learnings/clock.md`: its `code:` line and §"Honest scope" gain
`../model/src/node/transactional.rs:72`, the **second**
live-clock-into-a-content-hash the learning never named. An edit in place,
under spec01's rule 1. **The status does not move here** — that is spec04's,
and it is gated on p3 landing.

est: 0.5

## The defect in the learning

`learnings/clock.md:98-100` closes with:

> Two sites are known bugs today rather than candidates: `rec_now()` feeding
> `created` into the preimage, and any clock read reachable from
> `replay(as_of, kinds)`.

`rec_now()` is named, argued at lines 42-48, and is the learning's sharpest
case. The second one is not there at all:

```rust
// ../model/src/node/transactional.rs:72, inside Commit::new
let now = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_secs();
```

`Commit` (`../model/src/node/transactional.rs`) is postcard-encoded with
`signature: None` and blake3-hashed as its own content address
(`content_hash()`, line 104-109), and `parent_heads: Vec<[u8; 32]>` means that
id is **network-visible** — a peer receiving the commit re-derives the same id
or does not. It is the identical defect class to `rec_now()`, in the more
expensive place, and a document whose whole subject is "time is a parameter and
neither tree treats it as one" that names one of the two is undercounting the
exact thing it exists to count.

Three details the correction must carry, because each is a distinct hazard and
the learning is the only place they are collected:

1. The field is `timestamp: u64` — **unsigned**, so it cannot hold a pre-1970
   value at all, and `conserved::Instant` is `i64`. Substituting moves bytes as
   well as units: postcard varint-encodes `u64` and zigzag-encodes `i64`.
2. The read is `.as_secs()` while the field's own doc comment (line 59) says
   *"unix epoch milliseconds"*. The unit has already drifted inside one struct,
   in prose, which is the argument for `Instant` pinning its unit by test
   rather than by doc comment.
3. The `.unwrap()` is a third spelling of the pre-epoch hazard — a clock behind
   `UNIX_EPOCH` panics rather than returning a negative timestamp.

`.mi/prds/p5-adoption/llm/prd.md` already records this site, so the fact is not
lost if this ticket never runs; what is missing is the **learning** carrying it,
which is where `AGENTS.md`'s hard rule sends a reader before implementing.

## Mechanism — why this is an edit, not a superseding document

Per spec01. In short: `learnings/README.md` §"What is enforced" made the
supersede ceremony conditional on the folder being unversioned, and `ab154f7`
lifted that condition; `AGENTS.md`'s prohibition is on editing *to erase a
decision*, and adding a second code site reverses no decision. The corrected
sentence is not deleted — it is extended from "two sites" to name all of them —
and the verify below asserts the original `rec_now()` argument is still on disk.

**This spec does not depend on the `close` ticket running.** It is one file edit
that anyone — including the `llm` child, which needs the correction before it
starts — can make at any time. That is the operative reason the mechanism is an
edit.

## Files

- `learnings/clock.md` — the `code:` line in the frontmatter, and §"Honest
  scope". Nothing else in the document changes.

Not touched: the `status:` line (spec04), `README.md` (spec03/spec04), any
other learning, anything under `conserved/`, any sibling tree. `../model` is
**read** to check the citation and is never written — the user's 2026-08-21
scope answer holds.

## The edits

**Frontmatter `code:`** — currently:

```
code: mitosys src/mitosys/util/util.rs:107, llm src/record/mod.rs:239
```

gains the third site, in the same `<tree> <path>:<line>` shape the line already
uses, naming `llm` (the tree, whose directory is `../model`) so
`learnings/README.md` §"What a gate would check" item 3 can still resolve it.

**§"Honest scope"** — the closing paragraph moves from "Two sites are known
bugs today" to name all three, with `Commit::new` argued in two or three
sentences carrying details 1-3 above and the fact that `Commit`'s hash is a
network-visible id. The §"Why it happened in both" and §"The fix" sections are
untouched; the ~65-per-tree counts are untouched (the new site is one of the
65 already counted, not a 66th — say so explicitly, or the correction reads as
a count change it is not).

## Acceptance

- [x] The `code:` frontmatter line matches `transactional\.rs:72` and still
      names both original sites (`util.rs:107` and `record/mod.rs:239`).
- [x] §"Honest scope" names `transactional.rs:72` and says what it hashes into
      — `Commit`, `content_hash`, or both.
- [x] §"Honest scope" states all three of: the `u64`/`i64` sign mismatch, the
      doc-comment-says-milliseconds/code-says-seconds drift, and the
      `.unwrap()`.
- [x] §"Honest scope" states that the new site is **inside** the counted 65,
      not an addition to the count, so the table at lines 25-28 stays correct.
- [x] The `rec_now()` argument (`learnings/clock.md:42-48`) is still present
      verbatim — the erase-guard. Specifically the string
      `recomputable by any peer from the content alone` survives.
- [x] The `status:` line still reads `status: open`. Flipping it is spec04's  <!-- true when spec02 landed; spec04 then flipped it to decided, as this box anticipates -->
      job and is gated on commits that do not exist yet. *(This box is checked
      at the moment spec02 lands; spec04 legitimately unchecks it later, which
      is why the verify below does not assert it.)*
- [x] The cited line is real: line 72 of `../model/src/node/transactional.rs`  <!-- line 72 of ../model/src/node/transactional.rs holds SystemTime::now() — unchanged, no re-citation needed -->
      holds `SystemTime::now()`. If that file has moved under the citation, the
      citation is corrected to the line that holds the read — a stale `code:`
      line is the failure `learnings/README.md`'s would-be gate item 3 names.

verify: `bash -c 'set -e; cd /Users/feb/dev/infra/shared; C=learnings/clock.md; grep -qE "^code:.*transactional\.rs:72" $C || { echo "FAIL: code: line does not name the second site"; exit 1; }; grep -qE "^code:.*util\.rs:107" $C && grep -qE "^code:.*record/mod\.rs:239" $C || { echo "FAIL: an original code: site was dropped"; exit 1; }; H=$(awk "/^## Honest scope/,0" $C); echo "$H" | grep -q "transactional.rs:72" || { echo "FAIL: Honest scope does not name the second site"; exit 1; }; echo "$H" | grep -qE "Commit|content_hash" || { echo "FAIL: Honest scope does not say what the second site hashes into"; exit 1; }; echo "$H" | grep -q "u64" || { echo "FAIL: Honest scope does not record the u64/i64 mismatch"; exit 1; }; echo "$H" | grep -q "unwrap" || { echo "FAIL: Honest scope does not record the .unwrap() hazard"; exit 1; }; echo "$H" | grep -qiE "millisecond" || { echo "FAIL: Honest scope does not record the doc-comment unit drift"; exit 1; }; grep -q "recomputable by any peer from the content alone" $C || { echo "FAIL: the rec_now() argument was erased rather than extended"; exit 1; }; T=../model/src/node/transactional.rs; test -f $T || { echo "FAIL: cannot resolve the tree named in code:"; exit 1; }; L=$(grep -oE "transactional\.rs:[0-9]+" $C | head -1 | cut -d: -f2); sed -n "${L}p" $T | grep -q "SystemTime::now" || { echo "FAIL: the cited line does not hold the clock read"; exit 1; }; echo "spec02 ok"'`
