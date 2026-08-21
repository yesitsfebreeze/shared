//! Linkage smoke test.
//!
//! `conserved` is empty at p0, so there is nothing to assert about behaviour.
//! What is worth proving is that an integration test at `conserved/tests/` —
//! the test shape this crate committed to — actually links the crate, rather
//! than merely compiling a test binary that never names it.

use conserved as _;

#[test]
fn conserved_links() {
	// The `use` above resolves the crate at compile time; reaching this line
	// means the test binary linked against `conserved`, empty or not.
}
