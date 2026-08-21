---
state: specced
mode: afk
priority: 46
est: 2h
repo: shared
verify: "grep -q '^status: decided' learnings/shared-crate.md && grep -q '^status: decided' learnings/clock.md"
---

# P5f — close: flip the learnings this board settled

Purpose: `AGENTS.md`'s "Close" step. No node on this board flipped the
learnings it settled; the gantt audit already records this under "Lost work".
Per `AGENTS.md:149` this links the landing commits, so it runs **last**.

## Requirements

- [ ] **`learnings/shared-crate.md`** — `partial` → `decided`, linking the
      landing commits.
- [ ] **`learnings/clock.md`** — `open` → `decided`, and its `code:` line plus
      §"Honest scope" gain `../model/src/node/transactional.rs:72`, the second
      live-clock-into-a-content-hash the learning never named.
- [ ] **`README.md:39-40`** renders both statuses and goes stale with them.
- [ ] **Decide the correction mechanism and say which was chosen.**
      `learnings/README.md`'s own rule is that corrections are new documents, not
      edits that erase. So `clock.md`'s incompleteness is either an edit here or
      a superseding learning — and if the `llm` node needs the correction
      earlier than this node runs, it cannot wait for this node.
