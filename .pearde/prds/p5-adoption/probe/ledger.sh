#!/bin/sh
# p5-adoption — the cross-tree ledger probe.
#
# Read-only. Measures, in each consumer tree and in this one, the evidence
# each `## Requirements` box of prds/p5-adoption/prd.md rests on, and the
# `## Acceptance` sentence clause by clause. Prints one line per fact,
# `ok` / `GAP` / `FAIL`, and exits 1 if any line is FAIL.
#
#   ok    the box's clause holds and here is where the proof lives
#   GAP   the clause does not hold as written; the line names the owner
#         (a strike-with-reason in the ledger, per learnings/done-means-done.md)
#   FAIL  a proof this ledger relies on is missing or red — stop, do not tick
#
# Writes nothing anywhere. `cargo test` and `just check` in the consumer trees
# are the only commands that take time; everything else is grep.
#
# Usage: sh prds/p5-adoption/probe/ledger.sh     (from the shared repo root)

set -u

here=$(cd "$(dirname "$0")/../../.." && pwd)
mitosys="$here/../mitosys"
model="$here/../model"
realm="$here/../realm"

fails=0
ok()   { printf 'ok    %s\n' "$*"; }
gap()  { printf 'GAP   %s\n' "$*"; }
fail() { printf 'FAIL  %s\n' "$*"; fails=$((fails + 1)); }

# non-test `.rs` files under a dir: anything not under a `tests/` directory
src_files() { find "$1" -name '*.rs' -not -path '*/tests/*' -not -path '*/target/*' -not -path '*/vendor/*'; }
count_in()  { src_files "$1" | xargs grep -h "$2" 2>/dev/null | wc -l | tr -d ' '; }

echo "== pins: which rev each consumer holds, and whether a clone can fetch it"
for tree in "$mitosys" "$model" "$realm"; do
	name=$(basename "$tree")
	rev=$(grep -oE '^conserved = \{[^}]*rev = "[0-9a-f]+"' "$tree/Cargo.toml" | grep -oE '[0-9a-f]{40}')
	if [ -z "$rev" ]; then fail "$name: no conserved rev pin in Cargo.toml"; continue; fi
	remotes=$(git -C "$here" branch -r --contains "$rev" 2>/dev/null | grep -c 'origin/main')
	if [ "$remotes" -ge 1 ]; then
		ok "$name pins conserved rev ${rev} — on origin/main ($(git -C "$here" log -1 --format=%ad --date=short "$rev"))"
	else
		fail "$name pins conserved rev ${rev} — NOT on origin/main; a clone cannot fetch it"
	fi
	if [ -d "$tree/vendor/conserved-0.1.0" ]; then
		ok "$name carries vendor/conserved-0.1.0 with a source replacement in .cargo/config.toml"
	else
		fail "$name has no vendor/conserved-0.1.0 — offline/CI build depends on the private remote"
	fi
done
m_rev=$(grep -oE '[0-9a-f]{40}' "$mitosys/Cargo.toml" | head -1)
l_rev=$(grep -oE 'rev = "[0-9a-f]{40}"' "$model/Cargo.toml" | grep -oE '[0-9a-f]{40}' | head -1)
r_rev=$(grep -oE 'rev = "[0-9a-f]{40}"' "$realm/Cargo.toml" | grep -oE '[0-9a-f]{40}' | head -1)
if [ "$m_rev" = "$l_rev" ] && [ "$l_rev" = "$r_rev" ]; then
	ok "all three consumers pin the same rev"
else
	gap "consumers pin two revs (mitosys ${m_rev%???????????????????????????????????} / model+realm ${l_rev%???????????????????????????????????}) — Answer 1 asked for one; both are pre-rename and on the remote, so nothing is broken; the rename nodes re-pin all three"
fi
if grep -q '^name = "shared"' "$here/shared/Cargo.toml"; then
	unpushed=$(git -C "$here" log --oneline origin/main..HEAD | wc -l | tr -d ' ')
	gap "this repo's crate is \`shared\` (renamed dfc98fb), $unpushed commit(s) ahead of origin/main; consumers still pin \`conserved\` — the ledger's text names \`conserved\`, which is what every pinned rev holds"
fi

echo
echo "== box 5: the load proof, in this crate"
out=$(cd "$here" && cargo test -p shared load -- --include-ignored 2>&1)
if printf '%s' "$out" | grep -qE 'test result: .*[1-9][0-9]* passed' && ! printf '%s' "$out" | grep -qE '[1-9][0-9]* failed' && ! printf '%s' "$out" | grep -q '^error'; then
	printf '%s\n' "$out" | grep -E '^     Running tests/load_|^test result: ok\. [1-9]' | sed 's/^/      /'
	ok "cargo test -p shared load -- --include-ignored: every load_* binary green"
else
	fail "cargo test -p shared load -- --include-ignored is red"
fi
out=$(cd "$here" && cargo test -p shared --test done_boxes_are_ticked --test landed_rev_is_published 2>&1)
if printf '%s' "$out" | grep -qE '^test result: ok' && ! printf '%s' "$out" | grep -qE 'test result: FAILED|^error'; then
	ok "shared gates done_boxes_are_ticked + landed_rev_is_published green"
else
	fail "a shared board gate is red — a done transition here would turn it red again"
fi

echo
echo "== box 4: the ratchets — each tree's own named check, run in that tree"
run_gate() {
	tree=$1; shift; label=$1; shift
	out=$(cd "$tree" && cargo test "$@" 2>&1)
	if printf '%s' "$out" | grep -q 'wall_clock_reads_may_only_decrease ... ok' && printf '%s' "$out" | grep -q 'monotonic_reads_are_never_counted ... ok' && ! printf '%s' "$out" | grep -qE 'FAILED|^error'; then
		summary=$(printf '%s' "$out" | grep -E '^test result' | tail -1 | sed 's/test result: //; s/; 0 ignored.*//')
		ok "$label — $summary; wall_clock_reads_may_only_decrease + monotonic_reads_are_never_counted"
	else
		fail "$label is red or missing wall_clock_reads_may_only_decrease"
	fi
}
run_gate "$mitosys" "mitosys cargo test -p mitosys-gates --test write_path_reads_no_clock" -p mitosys-gates --test write_path_reads_no_clock
run_gate "$model"   "model   cargo test -p gates --test clock_read_ratchet"                -p gates --test clock_read_ratchet
run_gate "$realm"   "realm   cargo test -p realm-gates --test clock_read_ratchet"          -p realm-gates --test clock_read_ratchet
for spec in "$mitosys/src/mitosys/gates/tests/write_path_reads_no_clock.rs:mitosys" "$model/gates/tests/clock_read_ratchet.rs:model" "$realm/src/gates/tests/clock_read_ratchet.rs:realm"; do
	f=${spec%%:*}; n=${spec##*:}
	c=$(grep -oE '^const RATCHET_CEILING: usize = [0-9]+' "$f" | grep -oE '[0-9]+$')
	ok "$n RATCHET_CEILING = $c  ($f:$(grep -nE '^const RATCHET_CEILING' "$f" | cut -d: -f1))"
done

echo
echo "== box 1: mitosys, clause by clause"
if grep -q '^pub use conserved::scope::{Closed, Disposer, Scope, Undo};' "$mitosys/src/mitosys/util/effect/effect.rs"; then
	ok "util/effect is a re-export: effect.rs:$(grep -n '^pub use conserved::scope' "$mitosys/src/mitosys/util/effect/effect.rs" | cut -d: -f1), $(wc -l < "$mitosys/src/mitosys/util/effect/effect.rs" | tr -d ' ') lines, no implementation left"
else
	fail "mitosys util/effect is not a re-export of conserved::scope"
fi
if grep -q 'conserved::ContentId::of(s.as_bytes()).to_string()' "$mitosys/src/mitosys/engine/util/util.rs"; then
	ok "content_hash is conserved::ContentId (engine/util/util.rs:$(grep -n 'conserved::ContentId::of' "$mitosys/src/mitosys/engine/util/util.rs" | head -1 | cut -d: -f1)) — SHA-256 ids migrated; \`digest\` (util.rs:$(grep -n '^pub fn digest' "$mitosys/src/mitosys/engine/util/util.rs" | cut -d: -f1)) stays SHA-256 as a documented non-identity"
else
	fail "mitosys content_hash does not delegate to conserved::ContentId"
fi
if grep -q 'the one place an `ed25519:` prefix is tolerated' "$mitosys/src/mitosys/engine/util/util.rs"; then
	ok "ed25519: shim lives at engine/util/util.rs:$(grep -n 'one place an `ed25519:` prefix is tolerated' "$mitosys/src/mitosys/engine/util/util.rs" | cut -d: -f1)"
else
	fail "ed25519: shim not found in mitosys engine/util"
fi
if ! src_files "$mitosys/src" | xargs grep -q 'percentile_sorted' 2>/dev/null; then
	ok "percentile_sorted: 0 occurrences in non-test mitosys source"
else
	fail "percentile_sorted still present in mitosys"
fi
if grep -q '"conserved",' "$mitosys/src/mitosys/gates/tests/dependency_tree.rs"; then
	ok "dependency_tree.rs accepts the crate (line $(grep -n '"conserved",' "$mitosys/src/mitosys/gates/tests/dependency_tree.rs" | head -1 | cut -d: -f1))"
	out=$(cd "$mitosys" && cargo test -p mitosys-gates --test dependency_tree 2>&1)
	if printf '%s' "$out" | grep -qE '^test result: ok'; then
		ok "mitosys cargo test -p mitosys-gates --test dependency_tree — $(printf '%s' "$out" | grep -E '^test result' | sed 's/test result: //; s/; 0 ignored.*//')"
	else
		fail "mitosys dependency_tree gate is red"
	fi
else
	fail "dependency_tree.rs does not list conserved"
fi
if grep -q 'conserved::SystemClock\|use conserved::Clock' "$mitosys/src/mitosys/engine/util/util.rs"; then
	ok "mitosys clock helpers route through conserved::Clock"
else
	n=$(count_in "$mitosys/src" 'SystemTime::now')
	gap "mitosys has NOT adopted Clock: now_nanos/now_ms/now_secs (engine/util/util.rs:$(grep -n '^pub fn now_nanos' "$mitosys/src/mitosys/engine/util/util.rs" | cut -d: -f1)-$(grep -n '^pub fn now_secs' "$mitosys/src/mitosys/engine/util/util.rs" | cut -d: -f1)) read SystemTime::now; $n direct wall reads in non-test source, held by the ratchet; owner: mitosys/prds/adopt-conserved (open) box \"Clock reads route through conserved::Clock\""
fi
ok "just check in the container: not runnable from here; the child's record is 2d04000d — 2139/0/21, EXIT=0, empty /usr/local/cargo/git/ (prds/p5-adoption/mitosys/prd.md § Done 2026-08-28)"

echo
echo "== box 2: llm (../model), clause by clause"
if grep -q 'SystemClock.now().as_unix_secs()' "$model/src/record/mod.rs" && grep -q 'pub created: Instant,' "$model/src/record/mod.rs"; then
	ok "rec_now() reads SystemClock (record/mod.rs:$(grep -n 'SystemClock.now().as_unix_secs()' "$model/src/record/mod.rs" | head -1 | cut -d: -f1)); Record.created: Instant (record/mod.rs:$(grep -n 'pub created: Instant,' "$model/src/record/mod.rs" | cut -d: -f1)) — the content-hash preimage is on Clock"
else
	fail "model rec_now()/Record.created not on conserved::Clock"
fi
if grep -q 'conserved::stats::min_median_max(' "$model/src/grade/measure.rs"; then
	ok "grade::measure::aggregate calls min_median_max (grade/measure.rs:$(grep -n 'conserved::stats::min_median_max(' "$model/src/grade/measure.rs" | head -1 | cut -d: -f1))"
else
	fail "model aggregate does not call conserved::stats::min_median_max"
fi
if grep -q 'use conserved::scope::{Scope, Undo};' "$model/src/daemon/mod.rs" && grep -q 'let scope = Scope::new();' "$model/src/daemon/mod.rs"; then
	ok "Scope adopted at boot: daemon/mod.rs:$(grep -n 'let scope = Scope::new();' "$model/src/daemon/mod.rs" | head -1 | cut -d: -f1); main.rs's DOGMA-13 prose site now points at it (main.rs:$(grep -n 'conserved::Scope' "$model/src/main.rs" | head -1 | cut -d: -f1))"
else
	fail "model has no conserved::Scope at the boot path"
fi
if grep -q '^edition = "2024"' "$model/Cargo.toml" && grep -q 'RECORDED, not fixed: this tree is edition 2024' "$model/Cargo.toml"; then
	ok "edition-2024 tree consuming an edition-2021 crate is recorded (model/Cargo.toml:$(grep -n 'RECORDED, not fixed' "$model/Cargo.toml" | cut -d: -f1))"
else
	fail "model's edition note is missing"
fi
inst=$(count_in "$model/src" 'Instant::now')
wall=$(count_in "$model/src" 'SystemTime::now')
gap "the box's \"~65 wall-clock reads\" were monotonic: $inst Instant::now in non-test source today and none may become conserved::Instant (llm child box 3); the real wall-clock count went 15 -> ceiling 10 (gates/tests/clock_read_ratchet.rs), $wall SystemTime::now lines by plain grep; owner of the remainder: model's ratchet + model/prds/adopt-conserved (open)"

echo
echo "== box 3: realm"
sites=$(src_files "$realm/src" | xargs grep -c 'let scope = Scope::new();' 2>/dev/null | grep -v ':0$')
n=$(printf '%s\n' "$sites" | awk -F: '{s+=$2} END {print s+0}')
if [ "$n" -ge 3 ]; then
	ok "Scope adopted at $n sites: $(printf '%s' "$sites" | sed "s#$realm/##" | tr '\n' ' ')"
else
	fail "realm has $n Scope::new() sites (expected the three files' worth)"
fi
if ! grep -rqE 'blake3|sha2|Sha256' --include='*.rs' --include='*.toml' "$realm/src" "$realm/Cargo.toml" && ! grep -rqE 'median|percentile' --include='*.rs' --include='*.toml' "$realm/src" "$realm/Cargo.toml"; then
	ok "ContentId and stats have no call site in realm (grep for blake3|sha2|Sha256 and median|percentile both exit 1) — refusal recorded, admission criterion 1"
else
	fail "realm grew a hash or median site — the refusal in the realm child is stale"
fi
forced=$(git -C "$here" show --stat --format= 795f1df ad1b3b4 | grep -vc '^ prds/\|^ [0-9]* files\? changed' )
if [ "$forced" -eq 0 ]; then
	ok "realm's adoption forced no change to the crate: shared commits 795f1df + ad1b3b4 touch prds/ only — 'distributable to any repo' held"
else
	gap "realm's adoption touched $forced non-prds path(s) in shared — a missing requirement, per the box"
fi
out=$(cd "$realm" && just check 2>&1)
if [ $? -eq 0 ] && printf '%s' "$out" | grep -q 'conserved-vendor-check: ok   content match'; then
	ok "cd ../realm && just check — exit 0 (vendor-check rev agreement + content match, cargo fmt --check, cargo check --workspace)"
else
	fail "realm just check is red"
fi

echo
echo "== acceptance sentence, clause by clause, across the three trees"
if ! src_files "$mitosys/src" "$model/src" "$realm/src" | xargs grep -lE '^(pub )?struct (Scope|Disposer)\b' >/dev/null 2>&1; then
	ok "scope: no second Scope/Disposer type in any tree"
else
	fail "a second Scope/Disposer type exists"
fi
if ! src_files "$mitosys/src" "$model/src" "$realm/src" | xargs grep -lE 'fn (median|min_median_max|percentile_sorted)\b' >/dev/null 2>&1; then
	ok "median: no second implementation in any tree"
else
	fail "a second median implementation exists"
fi
copies=$(src_files "$model/src" | xargs grep -l '^fn blake3_hash\|^pub fn content_id\|^pub fn rec_id' 2>/dev/null | sed "s#$model/##" | tr '\n' ' ')
gap "content hashing: mitosys is on ContentId; model still carries local blake3 copies — ${copies}— owner: model/prds/adopt-conserved (open), its ContentId substitution boxes"
gap "clock reads outside SystemClock: realm 0 (ceiling 0); model ceiling 10; mitosys ceiling 48 — held by the ratchets (box 4), which is the mechanism learnings/clock.md chose; the counts descend on the consumers' own boards"

echo
echo "== the learnings this board closed"
for l in shared-crate clock; do
	if grep -q '^status: decided' "$here/learnings/$l.md"; then ok "learnings/$l.md status: decided"; else fail "learnings/$l.md is not decided"; fi
done

echo
if [ "$fails" -eq 0 ]; then
	echo "ledger: every proof this node relies on is green; GAP lines are strikes-with-reason, each with its owner"
	exit 0
else
	echo "ledger: $fails FAIL line(s) — do not tick; the proof is missing"
	exit 1
fi
