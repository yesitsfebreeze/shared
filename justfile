_default:
	@just --list

# the gate: what a green `shared` means, and the exact command CI runs
check:
	cargo fmt --check --all
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

# the filter in front of the gate — it can say "not yet" and nothing else.
# A green `just fast` is not a green `just check`, and no node closes on one.
fast:
	cargo clippy --workspace --all-targets -- -D warnings

# watch loop; bacon is already on the machine and wired this way in `model`
watch *ARGS:
	bacon {{ if ARGS == "" { "clippy-all" } else { ARGS } }}
