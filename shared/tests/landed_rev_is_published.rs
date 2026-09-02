//! The rev the family pins is published, and it is the renamed one.
//!
//! `learnings/crate-name.md` §"A vendored consumer build does not force the rev
//! bump" records the whole risk in the rename: `mitosys`, `model` and `realm`
//! each pin this repository by git sha, all three carry a committed `vendor/`
//! and a source replacement, and **cargo takes whatever the replacement
//! directory provides**. A consumer therefore builds green against a rev that
//! was never published and against a package that was never renamed. No
//! compiler catches it. `model` and `realm` have a vendor-check script that
//! does; `mitosys` has nothing.
//!
//! This is the producing side of that gate. It asserts, from inside the repo
//! that makes the rev, that `.pearde/prds/rename-conserved-to-shared/prd.md` §Landed
//! names a sha which
//!
//! 1. exists in this repository,
//! 2. is reachable from a branch on a **remote** — so a clone can fetch it, and
//! 3. carries `shared/Cargo.toml` with `name = "shared"` at that sha — so it is
//!    the renamed rev and not an older one copied in by hand.
//!
//! It fires only once the PRD is `state: done`, because §Landed cannot be
//! filled before the commit it names exists. Marking the PRD done is what
//! arms it, and that is the step the three consumers wait on.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The PRD that produces the rev, relative to the repository root.
const PRD: &str = ".pearde/prds/rename-conserved-to-shared/prd.md";

/// The repository root — the workspace directory above this crate.
fn repo_root() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.expect("the crate directory has a parent")
		.to_path_buf()
}

/// `state:` from a PRD's frontmatter, or `None` when there is no frontmatter.
fn frontmatter_state(body: &str) -> Option<&str> {
	let rest = body.strip_prefix("---\n")?;
	let end = rest.find("\n---")?;
	rest[..end]
		.lines()
		.find_map(|l| l.strip_prefix("state:"))
		.map(str::trim)
}

/// The `- rev: …` entry under `## Landed`, unwrapped from its backticks.
///
/// Returns `None` when the section is absent; returns `Some("TBD")` for the
/// placeholder the PRD ships with, so the caller can name that case exactly.
fn landed_rev(body: &str) -> Option<String> {
	let section = body.split("\n## Landed").nth(1)?;
	// stop at the next heading so a later section cannot be misread as this one
	let section = section.split("\n## ").next().unwrap_or(section);
	let line = section
		.lines()
		.find_map(|l| l.trim().strip_prefix("- rev:"))?;
	Some(line.trim().trim_matches('`').trim().to_string())
}

/// Run a git command in the repository root, returning stdout on success.
fn git(args: &[&str]) -> Result<String, String> {
	let out = Command::new("git")
		.arg("-C")
		.arg(repo_root())
		.args(args)
		.output()
		.map_err(|e| format!("could not run git: {e}"))?;
	if !out.status.success() {
		return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
	}
	Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

#[test]
fn landed_rev_is_a_published_rename_commit() {
	let prd_path = repo_root().join(PRD);
	let Ok(body) = std::fs::read_to_string(&prd_path) else {
		// The PRD is this gate's whole subject. If it is gone the work is done
		// and archived, and there is nothing left to hold open.
		return;
	};

	if frontmatter_state(&body) != Some("done") {
		// Not claiming to have landed yet, so there is no claim to check.
		return;
	}

	let rev = landed_rev(&body).unwrap_or_else(|| {
		panic!(
			"{PRD} is `state: done` but has no `- rev:` entry under `## Landed`. \
			 mitosys, model and realm each pin this repository by sha and read that \
			 line to know which one; without it they cannot be dispatched."
		)
	});

	assert!(
		rev.len() == 40 && rev.chars().all(|c| c.is_ascii_hexdigit()),
		"{PRD} §Landed names rev `{rev}`, which is not a 40-character sha. \
		 The placeholder is `TBD`; a done PRD must have replaced it with the \
		 commit the three consumers pin."
	);

	git(&["cat-file", "-e", &format!("{rev}^{{commit}}")]).unwrap_or_else(|e| {
		panic!("{PRD} §Landed names rev `{rev}`, which is not a commit in this repository: {e}")
	});

	// The check `model/scripts/*-vendor-check.sh` runs from the other side.
	// `cargo vendor` resolves a rev out of the *local* git db, so a commit that
	// never left this machine vendors in looking healthy while naming something
	// no clone can fetch.
	let remotes = git(&["branch", "-r", "--contains", &rev]).unwrap_or_default();
	assert!(
		remotes.lines().any(|l| !l.trim().is_empty()),
		"{PRD} §Landed names rev `{rev}`, which is on NO remote branch of this \
		 repository — it exists only on this machine, so nothing that clones the \
		 repo can ever fetch it. Push it before the consumers pin it.\n\
		 `git branch -r --contains {rev}` returned nothing."
	);

	// …and it must be the *renamed* rev. A sha that predates the rename is on a
	// remote and is a real commit, and pinning it forks the family onto two
	// package names at once — exactly what this PRD exists to prevent.
	let manifest = git(&["show", &format!("{rev}:shared/Cargo.toml")]).unwrap_or_else(|e| {
		panic!(
			"{PRD} §Landed names rev `{rev}`, which has no `shared/Cargo.toml`. \
			 That rev predates the rename, so a consumer pinning it gets the crate \
			 under its old name: {e}"
		)
	});
	assert!(
		manifest.lines().any(|l| l.trim() == r#"name = "shared""#),
		"{PRD} §Landed names rev `{rev}`, whose `shared/Cargo.toml` does not read \
		 `name = \"shared\"`. The directory moved but the package did not, which is \
		 the one-thing-two-names defect `learnings/crate-name.md` exists to remove."
	);
}

#[cfg(test)]
mod parser {
	use super::*;

	#[test]
	fn the_placeholder_is_not_mistaken_for_a_sha() {
		let doc = "---\nstate: done\n---\n\n## Landed\n\n- rev: `TBD`\n";
		assert_eq!(landed_rev(doc).as_deref(), Some("TBD"));
	}

	#[test]
	fn a_filled_rev_is_read_out_of_its_backticks() {
		let doc = "---\nstate: done\n---\n\n## Landed\n\n- rev: `0123456789abcdef0123456789abcdef01234567`\n- pushed: yes\n";
		assert_eq!(
			landed_rev(doc).as_deref(),
			Some("0123456789abcdef0123456789abcdef01234567")
		);
	}

	#[test]
	fn a_later_section_is_not_read_as_landed() {
		let doc =
			"---\nstate: done\n---\n\n## Landed\n\nnothing here\n\n## Notes\n\n- rev: `deadbeef`\n";
		assert_eq!(landed_rev(doc), None);
	}

	#[test]
	fn state_is_read_out_of_the_frontmatter() {
		assert_eq!(
			frontmatter_state("---\nstate: analyzing\n---\n"),
			Some("analyzing")
		);
		assert_eq!(frontmatter_state("no frontmatter\n"), None);
	}
}
