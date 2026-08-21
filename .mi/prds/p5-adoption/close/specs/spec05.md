# goal

`learnings/content-addressing.md`: **no status change** — it is already
`decided`, which is the top of the ladder — but its one-line serde sentence is
now under-specified against what p2 landed, so it gains what p2 settled, plus
the `code:` line naming the implementation. An edit in place, under spec01's
rule 1.

est: 0.25

## The decision this spec records: no status change, and why

The ticket asks whether p2 settling the serde question moves this document's
status. It does not, for two reasons:

1. **`decided` is terminal.** `AGENTS.md`'s ladder is `open` → `partial` →
   `decided`; there is no rung above it, and the document has been `decided`
   since 2026-08-18. p2 did not settle a question this learning left open — it
   *implemented* what the learning already decided (blake3, `[u8; 32]`, hex as
   a rendering). The ladder's `decided` explicitly *"may still need
   extraction"*, and extraction is what happened.
2. **The serde question p2 settled was not this document's open question.** It
   was a contradiction *between* documents, recorded in `.mi/gantt/plan.md`
   §"Lost work": this learning's line 77 asks for serde, while p2's ticket and
   `shared-crate.md` §"Size and shape" both say "the one dependency
   (`blake3`)". p2 resolved it by making `serde` an **optional feature with
   `default = []`**, so both sentences stay literally true of the crate as
   built by default. Nothing was decided differently; a false either/or was
   removed.

What *is* wrong is that a reader of this learning — which `AGENTS.md`'s hard
rule sends them to before implementing — cannot tell that `ContentId` will not
serialize unless they enable a feature. That is a trap with a real cost: a
consumer writing `conserved = { git = ..., rev = ... }` and putting a
`ContentId` inside a `#[derive(Serialize)]` struct gets a compile error whose
cause is documented nowhere in `learnings/`. Recording the feature is the
correction; the status is untouched.

## Mechanism

Per spec01: an addition that reverses no decision is an edit in place. Line 77
is **extended**, not replaced — the verify below asserts the original sentence
is still on disk, which is the erase-guard.

## Files

- `learnings/content-addressing.md` — the `code:` frontmatter line, and
  §"The shape" (the serde sentence at line 77). The `status:` line does not
  change and neither does §"What this beat".

Not touched: `README.md` — its `content-addressing.md` row already reads
`decided` and stays that way, so this is the one learning in this ticket whose
table row does not move. Nothing under `conserved/`; no sibling tree.

## The edits

**`code:`** — currently three sites in the two consumer trees. It gains
`shared conserved/src/content_id.rs`, the one implementation, keeping the three
duplicate sites (they are still there until p5's held children run).

**§"The shape"**, after line 77's *"Serde: bytes on a binary wire, hex string
in JSON…"* — three or four sentences recording what p2 decided, each of which
is a thing a consumer needs and cannot infer:

- serde is an **optional feature, `default = []`**; consumers write
  `features = ["serde"]`. Without it the type has no `Serialize`/`Deserialize`.
- The default dependency contract is unchanged:
  `cargo tree -p conserved --edges normal` still shows exactly one edge,
  `blake3`. This is why the feature is not a second dependency in the sense
  `shared-crate.md` §"Size and shape" means.
- **"Bytes on a binary wire" means the 32 raw bytes as a fixed-size tuple, not
  `serialize_bytes`.** Under serde `[u8; 32]` is a fixed tuple and postcard
  writes it with no length prefix; `serialize_bytes` would write a varint
  length first — 33 bytes, not 32. Taking the original sentence literally would
  have moved `../model`'s redb keys and its peer wire. This is the single most
  load-bearing addition: it is the difference between the substitution being
  byte-compatible and being a silent format change, and it is currently
  recorded only in a spec file the learnings do not point at.
- The split is on `is_human_readable()`, not on a format name.

Commit `8e12122` ("p2/spec03: serde for ContentId, behind an optional feature")
is the record; link it.

## Acceptance

- [x] `status:` still reads `decided` — this spec does not move it.
- [x] `code:` names `conserved/src/content_id.rs`, which is committed, and
      still names all three original sites (`util.rs:9`,
      `algebra/mod.rs:24`, `record/mod.rs:226`).
- [x] The document records that serde is an optional feature with
      `default = []` and names how a consumer enables it.
- [x] It states that the binary encoding is the 32 raw bytes as a fixed-size
      tuple and **not** `serialize_bytes`, and says what taking the sentence
      literally would have cost.
- [x] It links `8e12122`, and that sha resolves.
- [x] **Erase-guard**: the original sentence
      `Serde: bytes on a binary wire` is still present, and §"What this beat"
      is still present with all three rejected alternatives.
- [x] `cargo test -p conserved --features serde` passes — the document now
      describes a feature, so the feature is exercised.
- [x] `README.md` is not modified by this spec.

verify: `bash -c 'set -e; cd /Users/feb/dev/infra/shared; A=learnings/content-addressing.md; grep -q "^status: decided" $A || { echo "FAIL: status moved or was lost"; exit 1; }; grep -qE "^code:.*conserved/src/content_id\.rs" $A || { echo "FAIL: code: does not name the landed implementation"; exit 1; }; git ls-files --error-unmatch conserved/src/content_id.rs >/dev/null || { echo "FAIL: code: names a file that is not committed"; exit 1; }; for t in "util.rs:9" "algebra/mod.rs:24" "record/mod.rs:226"; do grep -qE "^code:.*$t" $A || { echo "FAIL: original code: site $t was dropped"; exit 1; }; done; grep -q "default = \[\]" $A || { echo "FAIL: the learning does not record that serde is off by default"; exit 1; }; grep -qE "features = \[.serde.\]|--features serde" $A || { echo "FAIL: the learning does not say how a consumer enables serde"; exit 1; }; grep -q "serialize_bytes" $A || { echo "FAIL: the fixed-tuple-not-serialize_bytes decision is not recorded"; exit 1; }; grep -q "8e12122" $A || { echo "FAIL: the landing commit is not linked"; exit 1; }; git cat-file -e 8e12122^{commit}; grep -q "Serde: bytes on a binary wire" $A || { echo "FAIL: the original sentence was erased rather than extended"; exit 1; }; grep -q "^## What this beat" $A || { echo "FAIL: the record of what the decision beat was erased"; exit 1; }; cargo test -p conserved --features serde >/dev/null || { echo "FAIL: the serde feature the document now describes does not pass its tests"; exit 1; }; grep -E "^\| .content-addressing\.md." README.md | grep -q "decided" || { echo "FAIL: README content-addressing row is not decided"; exit 1; }; echo "spec05 ok"'`
