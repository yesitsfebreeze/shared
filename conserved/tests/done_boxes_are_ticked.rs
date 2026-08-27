//! Every `state: done` PRD on this tree's board carries a ticked acceptance,
//! enforced.
//!
//! The family rule, from `shared/learnings/done-means-done.md`: a `done` PRD
//! requires evidence per box, and a box that cannot close is struck with a
//! reason rather than ticked. The walk fails any `state: done` PRD whose
//! own `## Acceptance` section contains an `- [ ]` box.
//!
//! # What this file does NOT count
//!
//! - Specs: each spec lives in its own file under `specs/specNN.md`, named
//!   with the `specNN.md` spelling, so the walk's `name == "prd.md"` filter
//!   never reaches them.
//! - Boxes outside `## Acceptance`: the `## Requirements` section above it
//!   is intentionally read-write work-in-progress; the rule is about the
//!   acceptance gate, not the work log.
//!
//! # The exemption list
//!
//! One `shared/*` PRD the master board's scan named on 2026-08-23 is
//! first-iteration exempt by relative path, with the reason recorded for
//! each. The exemption is shrink-only: a child PRD's completion
//! (`shared/prds/done-means-done/shared-classify/prd.md`) is what removes
//! the entry, because that child PRD records the orchestrator's per-row
//! action.

use std::fs;
use std::path::{Path, PathBuf};

/// The repository root: the directory holding the `[workspace]` manifest.
fn root() -> PathBuf {
	let mut dir = Path::new(env!("CARGO_MANIFEST_DIR"));
	loop {
		if fs::read_to_string(dir.join("Cargo.toml"))
			.unwrap_or_default()
			.contains("[workspace]")
		{
			return dir.to_path_buf();
		}
		dir = dir.parent().expect("the repository root");
	}
}

/// Every `prd.md` under `<root>/prds/`, as paths relative to the root.
fn board_files(root: &Path) -> Vec<String> {
	let mut out = Vec::new();
	walk(&root.join("prds"), &mut out);
	out.sort();
	out
		.iter()
		.map(|p| {
			p.strip_prefix(root)
				.unwrap_or(p)
				.to_string_lossy()
				.into_owned()
		})
		.collect()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
	let Ok(read) = fs::read_dir(dir) else {
		return;
	};
	for entry in read.filter_map(Result::ok) {
		let path = entry.path();
		let name = entry.file_name().to_string_lossy().into_owned();
		if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
			if !name.starts_with('.') {
				walk(&path, out);
			}
		} else if name == "prd.md" {
			out.push(path);
		}
	}
}

/// The one `shared/*` PRD the master board's scan named on 2026-08-23 with
/// unticked acceptance boxes while carrying `state: done`. The reason
/// column is recorded so a future shrink can find the work that removes
/// the entry.
const EXEMPT: &[(&str, &str)] = &[];

/// Reads the first frontmatter block and returns the value for `key:` if
/// present. A `state:` line is read byte-for-byte — no YAML, no
/// `serde_yaml`, just the shape the board's own files use.
fn frontmatter_state(text: &str) -> Option<String> {
	let mut lines = text.lines();
	if lines.next()?.trim() != "---" {
		return None;
	}
	for line in lines {
		if line.trim() == "---" {
			return None;
		}
		if let Some(rest) = line.strip_prefix("state:") {
			return Some(rest.trim().to_string());
		}
	}
	None
}

/// Counts `- [ ]` boxes inside the PRD's own `## Acceptance` section. Stops
/// at the next `## ` heading. A `- [x]` (or `- [~]`) is a closure and is
/// not counted; this is the literal-byte walk `shared/learnings/
/// done-means-done.md` records as the rule.
fn unticked_boxes_in_acceptance(text: &str) -> usize {
	let mut in_acceptance = false;
	let mut count = 0usize;
	for line in text.lines() {
		if line.starts_with("## ") {
			in_acceptance = line
				.trim_start_matches('#')
				.trim()
				.eq_ignore_ascii_case("acceptance");
			continue;
		}
		if !in_acceptance {
			continue;
		}
		if line.trim_start().starts_with("- [ ]") {
			count += 1;
		}
	}
	count
}

/// Every `state: done` PRD on this tree's board carries a ticked acceptance.
///
/// A gate that reads nothing must fail: a future PRDs/ that has been moved
/// or deleted would otherwise turn this check silently off (law 3: a rule
/// that cannot run is a wish — so the rule runs).
#[test]
fn every_done_prd_has_a_ticked_acceptance() {
	let root = root();
	let files = board_files(&root);
	assert!(
		!files.is_empty(),
		"found no prd.md under {}/prds — a gate that reads nothing must fail, \
		 because a moved board would otherwise turn this check silently off",
		root.display()
	);

	let exempt_paths: Vec<&str> = EXEMPT.iter().map(|(p, _)| *p).collect();

	let mut bad: Vec<(String, usize)> = Vec::new();
	for rel in &files {
		let Ok(text) = fs::read_to_string(root.join(rel)) else {
			continue;
		};
		let Some(state) = frontmatter_state(&text) else {
			continue;
		};
		if state != "done" {
			continue;
		}
		if exempt_paths.contains(&rel.as_str()) {
			continue;
		}
		let n = unticked_boxes_in_acceptance(&text);
		if n > 0 {
			bad.push((rel.clone(), n));
		}
	}

	if !bad.is_empty() {
		let detail = bad
			.iter()
			.map(|(p, n)| format!("  {p}: {n} unticked box(es)"))
			.collect::<Vec<_>>()
			.join("\n");
		panic!(
			"{} `state: done` PRD(s) carry unticked acceptance boxes — the \
			 family rule is that a tick requires evidence and an unclosable \
			 box is struck with a reason rather than ticked (see \
			 `shared/learnings/done-means-done.md`):\n{detail}",
			bad.len()
		);
	}
}

/// The exemption list is shrink-only. An entry added today that names a
/// PRD which is no longer `state: done` (i.e. the lane has already closed
/// it) is a defect: it hides a real tick from the gate.
#[test]
fn exemption_list_only_names_done_prds() {
	let root = root();
	let mut bad: Vec<String> = Vec::new();
	for (rel, _reason) in EXEMPT {
		let path = root.join(rel);
		let Ok(text) = fs::read_to_string(&path) else {
			bad.push(format!(
				"  {rel}: file missing — exemption names a path that does not exist"
			));
			continue;
		};
		let state = frontmatter_state(&text).unwrap_or_default();
		if state != "done" {
			bad.push(format!(
				"  {rel}: state is `{state}`, not `done` — the lane's child PRD \
				 has closed this entry; remove it from the exemption list"
			));
		}
	}
	if !bad.is_empty() {
		panic!(
			"{} exemption(s) are stale — the exemption list is shrink-only \
			 (a child PRD's completion removes the entry):\n{}",
			bad.len(),
			bad.join("\n")
		);
	}
}
