//! `conserved` — the domain-free primitives shared by the Rust trees.
//!
//! This crate was **deliberately empty at p0**. The foundation ticket's job was
//! a repository that can hold the crate, not the crate's contents: nothing is
//! extracted until a candidate passes the admission test in
//! `learnings/shared-crate.md`. The first inhabitant is [`scope`], moved from
//! mitosys's `util/effect` in p1 and carrying no dependencies; `ContentId`
//! arrives in p2 — with `blake3` behind it, and only if the gate lets it.
//!
//! Modules stay independent: `scope` depends on nothing, and nothing this
//! crate later admits may be dragged in behind it.
//!
//! # The four divergences, resolved here
//!
//! `AGENTS.md` §divergences records four dimensions on which the consuming
//! trees disagree. The shared crate cannot inherit a contradiction, so it
//! resolves each one explicitly:
//!
//! - **Test law** — integration tests live at `conserved/tests/` (the mitosys
//!   shape). No tests beside the module they cover.
//! - **Toolchain** — edition 2021, `rust-version = "1.94.0"`: the crate
//!   compiles for its strictest consumer.
//! - **Packaging** — one crate, no pre-split. The split happens if and when
//!   the dependency gate objects, not before.
//! - **Dependencies** — pinned once in the workspace manifest's
//!   `[workspace.dependencies]` and inherited by members.
//!
//! How the crate reaches its consumers is settled too: a git dependency
//! pinned by commit rev (`.mi/docs/memos/distribution.md`).

#![forbid(unsafe_code)]

mod content_id;
pub mod scope;
pub use content_id::{ContentId, ContentIdParseError};
