# Ontology digest — /Users/feb/dev/infra

World model index. Pointers, not data: kern IDs, typed relations, search
hints. The substance lives in the memory store and in files — this file
only says where.

## Focus

Three independent Rust projects (llm, mitosys, realm) sharing doctrine and
converging on a shared crate (conserved) via the learnings/ admission rule.

## Entities

### Projects

llm kern:c63de11bb9aa — the learner: model, peer mesh, improving loop; one binary, edition 2024, no pin; candle/libp2p/redb | see: llm/AGENTS.md, llm/DOGMA.md
mitosys kern:d62a64b52c17 — the harness: record, orchestration, plugins, surfaces; 39 crates, four laws, edition 2021 pinned 1.94.0 | see: mitosys/AGENTS.md, mitosys/.mi/skills/process/laws.md
realm kern:93d2ff3b9e4f — container orchestration: ZFS, Linux drivers, net, SSH; 6 workspace members under src/ | see: realm/Cargo.toml

### Shared knowledge

learnings kern:aef0de645c76 — shared knowledge dir; admission rule: true of >1 project; 9 docs; status ladder open→partial→decided | see: learnings/README.md
conserved kern:629fc82759c9 — proposed shared crate: ContentId, Clock, Scope/Handle, order stats, hex; partial | see: learnings/shared-crate.md

### Decisions (learnings)

content-addressing kern:edd749d7aed8 — decided: blake3, [u8;32]; mitosys SHA-256→hex String vs llm blake3→[u8;32] | see: learnings/content-addressing.md
record-shape object_id:ontology/infra/record-shape — decided: llm ahead on bitemporal record; port direction reverses | see: learnings/record-shape.md
inventory object_id:ontology/infra/inventory — decided: capability matrix; mitosys ahead on structure, llm ahead on record/grading/hot-reload | see: learnings/inventory.md
storage kern:25bd5a659184 — partial: redb replaces LMDB/heed, sequenced after fold rewrite | see: learnings/storage.md
clock kern:f0288a424171 — open: ~65 wall-clock reads/tree against time-as-parameter law | see: learnings/clock.md
divergences kern:e14a269123f8 — open: four contradictions (test law, packaging, toolchain, deps) | see: learnings/divergences.md
two-halves object_id:ontology/infra/two-halves — open: mitosys=record-half, llm=learner-half; zero shared source | see: learnings/two-halves.md

### Cross-project patterns (scan findings)

blake3_hash duplicate kern:1fe9feff7805 — defined identically in llm utils/fs and node/hot_swap
ContentId newtype kern:b673b9754a37 — decision exists, neither tree implemented
hex encoding kern:52153200a2de — mitosys one canonical module, llm 8+ scattered
Scope/Handle kern:4fb6886df176 — reversible effects in mitosys, prose-only in llm
order statistics kern:ca3a2a5563c3 — mitosys unused percentile_sorted, llm hand-rolls upper median
LearnOrigin duplicate kern:f0e0b663cfc9 — two definitions in llm

### Crate scans (mitosys)

mitosys-util kern:bc6d97a47fc1 | mitosys-util-math kern:1bf27d888e2e | mitosys-util-effect kern:c1f4b935bb80 | mitosys-engine-base kern:191f3495dc59 | mitosys-gates kern:c7873b595cc5 | mitosys/engine-bootstrap kern:6a9c39266e7f

## Open questions

- clock: settle time-as-parameter violation (open)
- divergences: settle four contradictions before shared code (open)
- two-halves: three couplings (corpus, grading, reload seam) (open)
- conserved: extract Scope first (partial)
- storage: execute redb swap after fold rewrite (partial)
- content-addressing: ContentId newtype not yet implemented in either tree
