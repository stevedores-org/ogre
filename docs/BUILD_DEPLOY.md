# OGRE build & deploy

## CI (FFT)

Pull requests and pushes to `main` / `develop` run the shared **FFT** gate from `lornu-ai/ci-checks`:

- `cargo fmt` + `clippy -D warnings` (workspace)
- `cargo test --workspace`
- gitleaks
- `kustomize build deploy/overlays/gke-prod`

Local parity:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
kustomize build deploy/overlays/gke-prod
```

## OCI build (main)

Every push to `main` runs `.github/workflows/build-on-main.yml`, which delegates to `lornu-ai/oci-builds` and builds `.#ogre-svc-image` from `flake.nix`.

Published tags (GAR `us-central1-docker.pkg.dev/gcp-lornu-ai/lornu/ogre-svc`):

- `main`, `main-<UTCyyyymmddHHMMSS>`, `sha-<short>`, `0.1.0`

Dry-run locally:

```bash
nix build .#ogre-svc-image
```

## Deploy (GitOps)

Kubernetes manifests live under `deploy/`:

| Path | Purpose |
|------|---------|
| `deploy/base/` | Namespace, Deployment, Service for `ogre-svc` |
| `deploy/overlays/gke-prod/` | GKE prod overlay — image tag pin |

Render:

```bash
kustomize build deploy/overlays/gke-prod
```

Wire into `lornu-ai/infra-code` Flux (follow-up): point a `Kustomization` at this repo path and add an `ImagePolicy` on `^main-([0-9]+)$` matching the oci-builds tag contract.

## Dependencies

`ogre-core` depends on private `lornu-ai/oxidizedgraph` via a Cargo git dependency. CI fetches it using the org OIDC → `secrets.aivcs.io` token exchange (see `fft-rust-gate.yml`).
