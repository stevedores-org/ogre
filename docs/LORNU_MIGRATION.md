# lornu-ai migration

Mirrored from `stevedores-org/ogre` @ `main` into private `lornu-ai/ogre`.

- Canonical org: **lornu-ai** only (no `stevedores-org` / `stevedores.org` references).
- Data plane successor: **`lornu-ai/data-mesh`** (replaces `data-fabric`).
- AIVCS monorepo: **`lornu-ai/aivcs.io`**.
- Local CI gate: **`lornu-ai/worry-free-crab`** / `propel.toml` (replacing stevedores `local-ci`).
- OCI builds: register in `lornu-ai/oci-builds` when a `flake.nix` OCI output lands.
- Deploy: `lornu-ai/infra-code` GitOps paths only.

Follow-up: add `flake.nix` OCI image output and drop any remaining shell-only tooling.
