# OGRE deploy stack (component mash-up)

OGRE production deploy is a **composition of component OCI images**, not a monolithic OGRE container.

## Components

| Service | Image (example) | Built in | Deploy manifests |
|---------|-----------------|----------|------------------|
| oxidizedgraph | `ghcr.io/stevedores-org/oxidizedgraph/server:<tag>` | [oxidizedgraph](https://github.com/stevedores-org/oxidizedgraph) — Nix + dockworker | `deploy/overlays/gke-autopilot` |
| oxidizedRAG | TBD | [oxidizedRAG](https://github.com/stevedores-org/oxidizedRAG) | TBD (same pattern) |
| data-fabric | per repo OCI docs | [data-fabric](https://github.com/stevedores-org/data-fabric) | TBD |

Packaging rules: [PACKAGING.md](./PACKAGING.md) — **no Dockerfiles**, dockworker.ai + skopeo only.

## OGRE repo (future)

```
ogre/deploy/
  components/          # optional: remote bases or image pins
  base/kustomization.yaml
  overlays/gke-autopilot/
```

`ogre/deploy` will:

1. Reference each component’s Kustomize overlay (or pinned `images:` transforms).
2. Publish a shared ConfigMap for inter-service URLs.
3. Add cross-namespace NetworkPolicy only where the stack requires it.

OGRE does **not** add a fourth “fat” image unless a thin coordinator is needed later.

## Service URL matrix (planned)

```yaml
# ogre deploy ConfigMap (illustrative)
OXIDIZEDGRAPH_URL: "http://oxidizedgraph.oxidizedgraph.svc.cluster.local:8080"
OXIDIZEDRAG_URL: "http://oxidizedrag.oxidizedrag.svc.cluster.local:8080"
DATA_FABRIC_URL: "https://<fabric-host>/v1"
```

oxidizedgraph → data-fabric: graph execution events (`/v1/integrations/oxidizedgraph/events`).

## Apply order (GKE Autopilot)

```bash
# Per component (from each repo, after dockworker push):
kubectl apply -k deploy/overlays/gke-autopilot   # oxidizedgraph, etc.

# Future: single entry from ogre
kubectl apply -k ogre/deploy/overlays/gke-autopilot
```

## GitOps

- **Kustomize + overlays** (no Helm), aligned with lornu.ai / stevedores GKE Autopilot conventions.
- Image tags: semver or git SHA via dockworker / `IMAGE_TAG`; pin digest in overlay for prod.
