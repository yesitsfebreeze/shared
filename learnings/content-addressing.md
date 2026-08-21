---
type: learning
learning: content-addressing
subject: mitosys hashes content with SHA-256 into a hex String and llm with blake3 into [u8;32]; blake3 wins, and the representation matters more than the algorithm
binds: [mitosys, llm]
status: decided
date: 2026-08-18
code: mitosys src/mitosys/util/util.rs:9, llm src/utils/algebra/mod.rs:24, llm src/record/mod.rs:226
---

# One content hash: blake3, `[u8; 32]`

Both trees content-address. They disagree on the algorithm **and** on the
representation, and the second disagreement is the expensive one.

```rust
// mitosys  util/util.rs:9
pub fn content_hash(s: &str) -> String        // SHA-256 -> 64 lowercase hex chars

// llm  utils/algebra/mod.rs:24
pub fn content_id(v: &Vector) -> Id           // Id = [u8; 32], blake3
// llm  record/mod.rs:226
pub fn rec_id(r: &Record) -> [u8; 32]         // blake3 of a fixed-order preimage
// llm  utils/fs/mod.rs:202
pub fn compute_version_hash(..) -> [u8; 32]   // blake3(postcard(VersionRecord))
```

llm already states its own convention: *"The repo's one hashing convention
(the registry's `content_id` and `compute_version_hash` use blake3 too), so
a record's id is recomputable by any peer from the content alone."*

mitosys has no equivalent sentence, and the difference is not cosmetic: if
the two trees ever share a record — which [[record-shape]] argues they
should — an id computed on one side does not exist on the other.

## Decision: blake3, and ids are bytes

**blake3 over SHA-256.** Faster on the sizes both trees hash, keyed and
derive-key modes available if provenance ever needs them, and — decisively —
llm's peers already recompute ids from it across a network. Changing llm
means changing a wire format that has other nodes on the far end; changing
mitosys means changing a local function with a stated wipe-on-format-bump
policy behind it (`store_core`: `FORMAT_VERSION` gates every open, mismatched
stores are rejected and wiped, "no migration code exists to rot").

The cost falls on the side that can absorb it. That asymmetry decides it.

**`[u8; 32]` over hex `String`.** This is the larger of the two changes and
the one worth arguing:

- A hex `String` is 64 bytes plus an allocation for 32 bytes of entropy, in
  a type that permits values that are not ids — `"hello"` is a `String`.
- `[u8; 32]` is `Copy`, allocation-free, and comparable in one instruction.
- Law 3's first rung is *types make the illegal value inexpressible*. A
  newtype over `[u8; 32]` reaches that rung; `String` cannot.

Hex is a **rendering**, and belongs where rendering belongs: `Display`, and
a `FromStr` that rejects anything that is not 64 hex digits. mitosys already
has the encoder — `util::hex::encode`/`decode`, and `decode` even tolerates
an `ed25519:` prefix — so this is moving an existing function behind a type,
not writing one.

## The shape

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentId([u8; 32]);

impl ContentId {
    pub fn of(bytes: &[u8]) -> Self;          // blake3
    pub fn as_bytes(&self) -> &[u8; 32];
}
impl fmt::Display for ContentId;              // lowercase hex, 64 chars
impl FromStr for ContentId;                   // rejects anything else
```

Serde: bytes on a binary wire, hex string in JSON, so a record stays
readable by eye where it is already JSON.

## What this beat

- **Keep both, convert at the boundary.** A conversion function is a place
  for the two to disagree about padding, case, or prefix handling, and it
  has to exist at every crossing. One representation has no boundary.
- **SHA-256 everywhere.** Its argument is ubiquity — hardware acceleration
  and every language having it. Neither tree needs either: nothing outside
  this family verifies these hashes.
- **A hex `String` newtype.** Keeps existing call sites and still closes the
  "any string is an id" hole. Rejected: it keeps the allocation and the 2×
  size for no remaining benefit once `Display` exists.

## Where it lands

`ContentId` is item one of [[shared-crate]] — it is the smallest thing both
trees genuinely need, it imports one crate, and nothing else in the proposal
can be typed correctly without it.
