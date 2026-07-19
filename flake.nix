{
  description = "ogre - Orchestrated GraphRAG Engineering wiring layer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        imageTag = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
          nativeBuildInputs = with pkgs; [ pkg-config cmake ];
          buildInputs = with pkgs; [ openssl libgit2 ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        ogre-svc = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "ogre-svc";
          cargoExtraArgs = "--bin ogre-svc";
        });

        ogre-svc-image = pkgs.dockerTools.buildLayeredImage {
          name = "ogre-svc";
          tag = imageTag;
          contents = [ ogre-svc pkgs.cacert ];
          config = {
            Cmd = [ "${ogre-svc}/bin/ogre-svc" ];
            ExposedPorts = { "8080/tcp" = {}; };
            User = "1000:1000";
            Env = [
              "PORT=8080"
              "RUST_LOG=info"
              "OGRE_LISTEN_ADDR=0.0.0.0:8080"
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            ];
          };
        };
      in
      {
        packages = {
          default = ogre-svc;
          inherit ogre-svc ogre-svc-image;
        };

        checks = {
          inherit ogre-svc;
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--workspace --all-targets -- -D warnings";
          });
          fmt = craneLib.cargoFmt { src = craneLib.cleanCargoSource ./.; };
          test = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; });
        };
      }
    );
}
