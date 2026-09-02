//! Every `state: done` PRD on this tree's board carries a closed board file,
//! enforced.
//!
//! The family rule, from `shared/learnings/done-means-done.md`: a `done` PRD
//! requires evidence per box, and a box that cannot close is struck with a
//! reason rather than ticked. The population the rule counts is settled by
//! `shared/learnings/exemptions-name-their-reason.md` § The scope rule and by
//! the user's decision of 2026-08-27, recorded as
//! `../.pearde/memos/done-counts-which-boxes.md` on the master board.
//!
//! # What this file counts
//!
//! **The whole `prd.md`, every heading.** An open box under `## Requirements`,
//! `## Out of scope`, or under no heading at all counts exactly as one under
//! `## Acceptance` does. The narrower `## Acceptance`-only reading this file
//! carried until 2026-08-28 let a `state: done` PRD hold open boxes anywhere
//! above the acceptance section; a gate reporting green on a condition that
//! does not hold is worse than no gate.
//!
//! # What an open box is spelled like
//!
//! Any Markdown list marker — `-`, `*`, `+`, or an ordered `1.`/`1)` — then a
//! bracket pair holding nothing but whitespace. The ordered arm arrived on
//! 2026-08-28 from `@infra/gates-adopt-the-best-matcher`, which adopted this
//! file's matcher on the other three trees and measured the one spelling it
//! did not close. All four gates carry the identical function and the
//! identical fixture table now.
//!
//! `- [x]` is a closure. `- [~]` is a closure too: it is a box struck with a
//! reason, per `shared/learnings/done-means-done.md` § The three forms an
//! unclosable box may take. A strike records a bar the code did not clear, not
//! work that is still owed.
//!
//! # What this file does NOT count
//!
//! - Specs: each spec lives in its own file under `specs/specNN.md`, named
//!   with the `specNN.md` spelling, so the walk's `name == "prd.md"` filter
//!   never reaches them. The scope rule keeps them out on purpose.
//!
//! # The exemption list
//!
//! `EXEMPT` is empty and has never held an entry. The contract every entry
//! owes — the PRD, the commit, and the observable condition that removes it —
//! is `shared/learnings/exemptions-name-their-reason.md` § The exemption
//! contract, in this repository, two directories up from this file. **The list
//! is shrink-only:** a completed child PRD removes an entry, and a fresh
//! regression never adds one. An entry that cannot name all three fields means
//! the PRD's `state` is wrong, and the state is what gets corrected — not the
//! list, not the box, not the gate.

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

/// Every `prd.md` under `<root>/.pearde/prds/`, as paths relative to the root.
fn board_files(root: &Path) -> Vec<String> {
	let mut out = Vec::new();
	walk(&root.join(".pearde").join("prds"), &mut out);
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

/// PRDs excused from the whole-file count, by path relative to the repository
/// root. Each entry names the PRD whose completion removes it, the commit that
/// justifies the claim the exempt PRD is making, and the observable condition
/// that removes the entry -- all three, per
/// `shared/learnings/exemptions-name-their-reason.md` (the exemption
/// contract). The 3-tuple is that contract in the type: an entry that cannot
/// fill all three fields will not compile.
///
/// **Shrink-only.** Entries leave this list when a child PRD closes them; they
/// never enter it to silence a regression. Widening an exemption to make a
/// tree green is the move the family rule forbids by name.
///
/// This list is `&[]` and has never held an entry. It is not given its first
/// one by the change that widened the count to the whole file: the seven boxes
/// that widening exposed are closed per box, on this board's two
/// `p5-adoption` nodes. Neither could have filled the removal-condition field
/// -- those boxes were not waiting on an observable event, only on evidence
/// that already existed being written down.
const EXEMPT: &[(&str, &str, &str)] = &[];

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

/// Strips one Markdown list marker from the front of `rest`, returning what
/// follows it, or `None` when `rest` does not open a list item.
///
/// The three bullets (`-`, `*`, `+`) are this file's own, unchanged. The
/// ordered arm (`1.`, `2)`) came back from mitosys, model and realm on
/// 2026-08-28 (`@infra/gates-adopt-the-best-matcher`, spec03 and spec05): this
/// matcher was the widest of the four and was adopted by the other three, and
/// the port measured a spelling it does not close. It strips a bullet or gives
/// up, so an ordered task list reaches neither branch and `1. [ ]` walks past
/// it exactly as it walks past the single literal `- [ ]`. GitHub renders an
/// ordered task list as checkboxes exactly like an unordered one, so the same
/// rendered open box was invisible to all four gates.
///
/// Zero ordered-marker boxes exist on any of the four boards, measured
/// 2026-08-28 over every tracked board file on mitosys (122), model (75),
/// realm (19) and this tree (16): no hits, on any tree, in any state. The arm
/// changes no verdict today.
///
/// `digits > 9` is GFM's own bound on an ordered marker, and it is what keeps
/// a year or a version number from being read as a list marker. `)` is
/// admitted beside `.` because GFM admits both.
///
/// Kept as its own arm so it can be removed on its own if the ordered form is
/// ruled out of scope.
fn strip_list_marker(rest: &str) -> Option<&str> {
	if let Some(after) = rest
		.strip_prefix('-')
		.or_else(|| rest.strip_prefix('*'))
		.or_else(|| rest.strip_prefix('+'))
	{
		return Some(after);
	}
	let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
	if digits == 0 || digits > 9 {
		return None;
	}
	let rest = &rest[digits..];
	rest.strip_prefix('.').or_else(|| rest.strip_prefix(')'))
}

/// True when `line` opens an unticked checkbox: a list marker, then a bracket
/// pair holding nothing but whitespace.
///
/// The marker is any of Markdown's three bullets or an ordered marker (see
/// [`strip_list_marker`]) and the gap between marker and bracket is any run of
/// spaces, because all of those render as the same open box in every viewer
/// the board is read in. A gate matching one spelling only is a gate a stray
/// `*`-bulleted box walks past, and a board file is prose, written by hand, in
/// four repositories.
///
/// A ticked box and a struck box are closures and do not match: their brackets
/// are not empty. The struck form is a box whose bar the code did not clear,
/// closed with a reason beside it -- never work that is merely still owed.
///
/// This body is byte-identical to mitosys's, model's and realm's, which is the
/// point: the four gates disagreeing about what a box is was the defect
/// `@infra/gates-adopt-the-best-matcher` existed to remove.
fn opens_an_unticked_box(line: &str) -> bool {
	let Some(rest) = strip_list_marker(line.trim_start()) else {
		return false;
	};
	let rest = rest.trim_start_matches(' ');
	let Some(rest) = rest.strip_prefix('[') else {
		return false;
	};
	match rest.find(']') {
		Some(end) => rest[..end].trim().is_empty(),
		None => false,
	}
}

/// Counts unticked boxes anywhere in the PRD -- every heading, and the lines
/// under no heading at all.
///
/// The population is `shared/learnings/exemptions-name-their-reason.md`
/// (the scope rule): the whole board file. This counter read only the run
/// between the acceptance heading and the next one until 2026-08-28, which is
/// why two `state: done` PRDs on this board carried seven open boxes under
/// their requirements heading while the gate reported green.
fn unticked_boxes_in_file(text: &str) -> usize {
	text
		.lines()
		.filter(|line| opens_an_unticked_box(line))
		.count()
}

/// Every `state: done` PRD on this tree's board carries no unticked box,
/// anywhere in the file.
///
/// Renamed from `every_done_prd_has_a_ticked_acceptance` on 2026-08-28, with
/// the counter it calls, because the population is no longer the acceptance
/// section. A test whose name says `acceptance` while its body reads the whole
/// file is the same defect as a doc comment that lies about its own code.
///
/// A gate that reads nothing must fail: a future PRDs/ that has been moved
/// or deleted would otherwise turn this check silently off (law 3: a rule
/// that cannot run is a wish — so the rule runs).
#[test]
fn every_done_prd_has_no_unticked_box() {
	let root = root();
	let files = board_files(&root);
	assert!(
		!files.is_empty(),
		"found no prd.md under {}/.pearde/prds — a gate that reads nothing must fail, \
		 because a moved board would otherwise turn this check silently off",
		root.display()
	);

	let exempt_paths: Vec<&str> = EXEMPT.iter().map(|(p, _, _)| *p).collect();

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
		let n = unticked_boxes_in_file(&text);
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
			"{} `state: done` PRD(s) carry unticked boxes — the count is the \
			 whole file, every heading, per \
			 `shared/learnings/exemptions-name-their-reason.md`. Either tick \
			 the box with quoted evidence, strike it with a measured reason, \
			 or correct the PRD's state; an exemption entry that cannot name a \
			 PRD, a commit and the condition that removes it is not \
			 written:\n{detail}",
			bad.len()
		);
	}
}

/// The matcher, pinned to a fixture rather than to the board.
///
/// The board carries zero of these spellings today, on all four trees, so
/// nothing about the widening is held by the walk above: it reads the same
/// green it read yesterday. Only this fixture fails if the matcher narrows
/// back to the single literal `- [ ]`, or if the ordered arm is dropped.
///
/// The rows are identical to mitosys's, model's and realm's, so "proven by the
/// same fixture run" is literal rather than inherited.
#[test]
fn the_matcher_reads_every_spelling_of_one_rendered_box() {
	// (line, is an open box)
	let cases: &[(&str, bool)] = &[
		// The five that walked past the literal `- [ ]` matcher.
		("* [ ] a star bullet", true),
		("+ [ ] a plus bullet", true),
		("- [] no space inside the brackets", true),
		("-  [ ] two spaces after the bullet", true),
		("1. [ ] an ordered task list", true),
		("1) [ ] an ordered task list, paren marker", true),
		// Already red before the widening, and still red.
		("- [ ] the literal spelling", true),
		("  - [ ] indented, under no heading", true),
		("\t- [ ] tab-indented", true),
		// Closures. `- [~]` is struck-with-a-reason, not work still owed.
		("- [x] ticked, on evidence", false),
		("- [X] ticked, capital", false),
		("- [~] ~~struck~~ — with the reason beside it", false),
		("* [x] ticked under a star bullet", false),
		// Not boxes at all.
		("prose quoting an inline `- [ ]` mid-sentence", false),
		("- [a link](https://example.invalid) is not a box", false),
		("- not a box at all", false),
		("1234567890. [ ] ten digits, past GFM's bound", false),
		("state: done", false),
	];
	let wrong: Vec<&str> = cases
		.iter()
		.filter(|(line, open)| opens_an_unticked_box(line) != *open)
		.map(|(line, _)| *line)
		.collect();
	assert!(
		wrong.is_empty(),
		"{} line(s) read the wrong way — every row here renders as the same \
		 open box, or as no box at all, in every viewer this board is read \
		 in:\n  {}",
		wrong.len(),
		wrong.join("\n  ")
	);
}

/// The exemption list is shrink-only. An entry added today that names a
/// PRD which is no longer `state: done` (i.e. the lane has already closed
/// it) is a defect: it hides a real tick from the gate.
#[test]
fn exemption_list_only_names_done_prds() {
	let root = root();
	let mut bad: Vec<String> = Vec::new();
	for (rel, _commit, _removal_condition) in EXEMPT {
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
