---
complexity: 30
footprint:
  - ../mitosys/.cargo/config.toml
  - ../mitosys/vendor/
  - ../mitosys/Cargo.lock
  - ../mitosys/Dockerfile
  - ../mitosys/docker-compose.yml
  - ../mitosys/.mi/docs/DOCKER.md
---

# spec01 — the offline dev container can resolve dependencies with no network

Give mitosys's dev container a mechanism (`cargo vendor` into a committed
`vendor/` directory with a `[source]` replacement in `.cargo/config.toml`, or
a pre-populated registry/git cache seeded by a committed script) so `cargo
build`/`cargo test` succeed inside the container with cargo's `--offline`
flag, before `conserved` is ever added as a dependency. This is this PRD's
first spec by explicit instruction: p0's distribution memo scoped it as
"mitosys-side follow-up designed before p5's mitosys adoption step," and once
`conserved` is pinned by rev it must be addable to the existing mechanism as
one more vendored/cached entry, not a new one.

Ground truth measured today: `docker-compose.yml` gives the `dev` service no
`ports:` and no `network_mode: host`, and mounts `cargo-registry` and
`cargo-git` as Docker-managed volumes (not the bind-mounted repo) — so those
caches persist across `exec` sessions but do not yet survive a volume reset,
and `../mitosys/.cargo/config.toml` does not exist. `CLAUDE.md` states
outright: "no host network, no host credentials, no bind mount beyond the
repo itself." Whether that means zero outbound internet at container-build
time, or only no host-reachable ports, was not independently re-verified in
this session (unmeasured) — `cargo build --offline` is deliberately chosen
below as the proof because it is self-certifying: it refuses any network
fetch regardless of what is actually reachable, so passing it is proof the
vendor/cache mechanism supplied everything, not the network.

## Acceptance

- [ ] `docker compose down -v` (drops the `cargo-registry`/`cargo-git`
      volumes, simulating a machine that has never built this repo) followed
      by `docker compose up -d dev` and `docker compose exec dev cargo build
      --workspace --offline` succeeds.
- [ ] `docker compose exec dev cargo test --workspace --offline` succeeds
      (dev-dependencies are covered too, not just the build graph).
- [ ] The provisioning step that fills `vendor/` or the registry/git cache is
      a committed script or `just` target, not a manual one-off — running it
      again after a `Cargo.toml` dependency change updates the cache/vendor
      tree without hand-editing.
- [ ] `docker compose exec dev just check` still passes, unmodified — the
      offline mechanism changes how dependencies are fetched, not the gate
      itself.

## Verify and Proof

```sh
cd ../mitosys
docker compose down -v
docker compose up -d dev
docker compose exec dev cargo build --workspace --offline
docker compose exec dev cargo test --workspace --offline
docker compose exec dev just check
```
