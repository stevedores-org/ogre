# Packaging (OGRE stack)

**Org policy: no Dockerfiles.** Container images are **OCI-standard** artifacts built from **Nix** flakes and published with **[dockworker.ai](https://dockworker.ai)** + **skopeo**. Kubernetes deploys those images via **Kustomize** (no Helm).

OGRE does not ship application binaries in this repo. Each component repo owns its image; OGRE composes them at deploy time.

## Pipeline (per component repo)

| Step | Tool | Output |
|------|------|--------|
| Build | `nix build .#<image-output>` | OCI tarball (`./result`) from `pkgs.dockerTools.buildLayeredImage` |
| Manifest | `dockworker.toml` | Maps flake output → registry name/tag |
| Push | `dockworker build` or `skopeo copy` | `ghcr.io/stevedores-org/<component>/...` |

Example (oxidizedgraph):

```bash
nix build .#server-image -L
dockworker build   # reads dockworker.toml, pushes via skopeo
```

## dockworker.toml

Each service repo includes a `dockworker.toml` at the root:

```toml
version = 1
flake = ".#"

[[images]]
name = "oxidizedgraph/server"
nix_output = "server-image"
push = true
tag = "0.2.0"

[registry]
default = "ghcr.io/stevedores-org"

[transport]
tool = "skopeo"
```

dockworker.ai orchestrates `nix build` per `[[images]]` entry, then transports the tarball to the registry. No `docker build`, no Dockerfile.

## Why no Dockerfiles

- **Hermetic**: Nix owns deps; flakes are reproducible and cache-friendly (stevedores substituter opt-in — see component `docs/PACKAGING.md`).
- **OCI without Docker**: Images are standard OCI layouts; runtime is containerd/CRI on GKE, not Docker Engine.
- **Single source of truth**: `flake.nix` + `Cargo.lock` (for Rust) — no drift between Dockerfile layers and native builds.

Legacy Dockerfiles in component repos (if any) are **deprecated** and must not be used for CI or production.

## OGRE’s role

| Repo | Builds image? | OGRE role |
|------|---------------|-----------|
| oxidizedgraph | Yes (`server-image`) | Wire URL + Kustomize component |
| oxidizedRAG | Yes (when packaged) | Same |
| data-fabric | Yes (OCI path per repo docs) | Same |
| **ogre** | **No** | Stack Kustomize, contracts, env matrix — [DEPLOY_STACK.md](./DEPLOY_STACK.md) |

## References

- [oxidizedgraph — PACKAGING](https://github.com/stevedores-org/oxidizedgraph/blob/main/docs/PACKAGING.md) (once merged)
- [lornu-finops — PACKAGING](https://github.com/lornu-ai/lornu-finops/blob/main/docs/PACKAGING.md) (reference pattern)
