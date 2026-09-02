---
state: open
origin: derived
from: "@master/crate-is-named-shared"
priority: 15
blast-radius: low
repo: shared
footprint:
  - .pi/ontology/digest.md
---

# Every id in the ontology digest points at a store nothing reads

`.pi/ontology/digest.md` binds entity names to `kern:` ids and says so in its
own header — *"Pointers, not data: kern IDs … The substance lives in the memory
store."* Line 23:

```
conserved kern:629fc82759c9 — proposed shared crate: ContentId, Clock,
Scope/Handle, order stats, hex; partial | see: learnings/shared-crate.md
```

This surfaced as a question during the crate rename — rename the token here and
not in the store and the index desyncs. **It was measured, and the desync risk
is not live, because the binding is already dangling.**

## Measured — 2026-08-28

| probe | result |
|---|---|
| `kern get 629fc82759c9` | *"no thought with id"* | 
| four other ids from the same file | same |
| `kern health` on this directory | `thoughts: 0  reasons: 0` |
| which store `kern` v2.0.0 reads | `shared/.kern/data`, **not** the `.pi/kern/data` this file was written against |
| both stores tracked? | no — and `.pi/kern/data/` is gitignored (`.gitignore:8`) |
| the old `.pi` store | 1.7 MB, dated 2026-08-19, **no plaintext hit** for `conserved` or the id |

So the file is an index whose every entry resolves to nothing, and the store it
indexed is neither present nor recoverable from the tree.

## The fork

The rename node deliberately took (b) provisionally — all three `conserved`
hits are excluded from its grep and a section says *held pending a ruling, do
not rename*. Flipping one exclusion releases it.

- **(a) Rename the tokens with the rest of the live prose.** Safe now that
  nothing resolves, but it polishes an index in which every id is dead.
- **(b) Leave the tokens and fix the index.** Either rebuild it against the
  store `kern` actually reads, or record that it is historical. **Recommended**
  — the tokens are the least of what is wrong with the file.
- **(c) Delete `.pi/ontology/digest.md`.** Honest if nobody reads it, and it
  costs nothing to check first.

Whichever wins, the rename node's exclusion should be lifted or made permanent
in the same change, so the two do not disagree.

## Pointers

- `.pi/ontology/digest.md` — the file and its own header
- `.gitignore:8` — why neither store is in the tree
- `prds/rename-conserved-to-shared/prd.md` — the held exclusion
