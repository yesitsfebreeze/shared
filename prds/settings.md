---
language: English
workers: 3
pipeline: 3
est-default: 4h
gantt-day: 8h
plane: auto
---

# shared — the family's memory and its one library

This board had no `settings.md` until 2026-08-23, so every knob read at its
default and `language` was never declared. Written when the board moved from
`.mi/prds` to the contract path.

Two things live in this tree, and only one of them is code:

- `conserved` — the domain-free crate (`ContentId`, `Clock`, `Scope`, `stats`),
  distributed to consumers as a **rev-pinned git dependency**, never a path
  dependency and never vendored.
- `learnings/` — prose true of more than one tree, and the only place a
  cross-tree decision is recorded.

## Done is adoption, not compilation

The tree's own words, from `p5-adoption/prd.md`: the extraction *"is not done
when the crate compiles — it is done when the duplicates are gone."* Every
remaining node is a consumer's adoption, and each is enforced by that
consumer's gate rather than by anything here.

The whole remaining spine is held by one event outside this tree: the git
remote at `https://github.com/inner-zirkle/shared` must exist and be pushed.
As of 2026-08-23 that repository **does not exist** in the `inner-zirkle`
organization — the org holds `kern`, `zirkle`, `mightty` and `realm` only. Four
PRDs across three trees name the push as their blocking event; none of them can
land until the repository is created, which is the user's call.

## Language

`learnings/` calls the learner tree `llm`. On disk it is `model/`. The
translation is the master board's rule, not a fix: nine of twelve learnings
still carry `binds: [mitosys, llm]`.
