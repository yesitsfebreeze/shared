---
type: learning
learning: inventory
subject: what each tree has actually implemented, side by side — mitosys is ahead on structure and enforcement, llm is ahead on the record, grading and hot reload, and four capabilities exist on exactly one side
binds: [mitosys, llm]
status: decided
date: 2026-08-18
---

# The audit

Measured 2026-08-18 against both working trees. **Caveat on llm:** its tree
is live — 121 files uncommitted, and `src/mcp` (3.2k lines) appeared during
this audit. Numbers below are that day's state, not a released version.

Sizes are non-test lines where the split is meaningful, total lines
otherwise.

## Capability matrix

| capability | mitosys | llm |
|---|---|---|
| **append-only record** | `engine/record` 3.2k — `seq: i64` + RFC3339 **string** time, JSON events | `src/record` 5.3k — **blake3 content id**, bitemporal, heat decay |
| **pure fold replay** | `stream::project_messages`, `reconstruct_requests` | `log::replay(as_of, kinds)` — one entry point, plus `as_of(id,t)`, `recover(id)` |
| **log self-check** | — | `LogStats::balances()` |
| **durable KV** | `engine/store_core` 4.1k — LMDB via `heed` | `utils/fs` 1.5k — redb |
| **event broadcast** | `record/stream` `Emitter`/`Subscription` | `src/events` 1.5k `SystemEvent`, bounded drop-oldest |
| **reversible effects** | `util/effect` 262 — `Handle` + `Scope`, unwind in reverse | **none** — DOGMA 13 cited in prose, no type |
| **plugin host** | `api/plugin` 4.6k + Lua runtime + `.mitosys` manifests | **none** |
| **dylib boundary** | `api/surface/abi` — loads **once**, never swaps | `interface` + `src/reload` 1.2k — **hot swap at a tick**, layout fingerprint, last-good retained |
| **perf grading** | **none** (`percentile_sorted` exists with **zero callers**) | `src/grade` 8.2k + `src/improve` 5.8k — envelopes, pass·regression·fail, keep-or-rollback |
| **structural gates** | `gates` 2.1k — source layout, dependency tree, one-namespace-one-crate | **none** — `just check` is `cargo check` |
| **tool protocol** | `api/agentic` 13k — ACP **client**, spawns agents, answers their calls | `src/mcp` 3.2k — MCP **server** over stdio, five tools, each a pure fold |
| **vector math** | `util/math` 1.1k — `cosine`, `l2_normalize`, `OnlineSoftmax`, `QuantizedVec` | `utils/algebra` — `distance`, `norm`, `dot`, `add/sub/scale`; quantized bytes via `half`/`bytemuck` |
| **config** | `engine/config` 2.4k — strict TOML, unknown keys refused | `src/config` 1.6k — strict TOML, unknown keys refused |
| **file watching** | `util/watcher` 573 + `engine/ingest` (`notify`, `.miignore`) | `src/reload` (`notify`, dylib dir) |
| **content hash** | SHA-256 → 64-char hex `String` | blake3 → `[u8; 32]` |
| **peer mesh** | — | `src/gossip` 6.2k libp2p kad/gossipsub/quic |
| **model** | `engine/model` 384 — closure seam only, imports nothing | `src/node` 12.8k + `transformers` 3.6k + `learning` 3.1k, candle |
| **board reader** | `.mi/prd` + `plugins/board` Lua (`board.list`/`board.next`) | `.pi/prd` — same format, **no reader** |

## What only one side has

**mitosys alone:** the plugin host, reversible effects, structural gates,
the crate-per-responsibility split, the Lua surface, the genome/gene form.

**llm alone:** perf grading and the improving loop, the hot-swap loader, a
content-addressed bitemporal record, a peer mesh, an actual model.

The two lists barely intersect, which is the whole argument for
[[two-halves]] — and they are not symmetric in maturity. mitosys is ahead on
**structure and enforcement**; llm is ahead on **the record, measurement and
runtime swap**. The assumption that the harness leads and the learner
follows is wrong in exactly the places that matter most; see
[[record-shape]].

## Findings worth acting on

1. **`util::percentile_sorted` has zero callers** outside its own unit test.
   It is the exact primitive llm's `grade::measure::aggregate` needs, sitting
   unused in the tree that has no grading.
2. **mitosys has no benchmark or regression gate at all.** Conformance
   replay pins behaviour, nothing pins cost.
3. **llm has no `Scope`.** `main.rs` writes "the inverse of the live node's
   checkpoint (DOGMA 13)" in a comment; the rule is held by hand, per site.
4. **Both hand-roll order statistics differently.** mitosys has
   `percentile_sorted`; llm's `aggregate` returns min / **upper** median
   (index `n/2`) / max. Two definitions of median across a family that
   intends to share a grade envelope.
5. **Two content-hash algorithms.** See [[content-addressing]].
6. **~65 clock reads per tree**, in both, against a shared law that says
   time is a parameter. See [[clock]].
7. **Both wrap `notify`** for file watching, with independent debounce and
   ignore handling.
8. **The same board format, one reader.** llm's `.pi/prd` is byte-compatible
   with mitosys's `.mi/prd` — same frontmatter, same claim/release commit
   lock — and mitosys's `board` plugin could read it unchanged.
