{
  description = "The Lamuda language";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs@{
      flake-parts,
      systems,
      fenix,
      crane,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import systems;
      perSystem =
        { pkgs, system, ... }:
        let
          toolchain = fenix.packages.${system}.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-+xUr6Al1wZSqX8aEsq7DpuFOK4Ve/i+qTl+K5uWQ9GM=";
          };
          craneLibBase = crane.mkLib pkgs;
          craneLib = craneLibBase.overrideToolchain toolchain;
          src = craneLib.cleanCargoSource ./.;
          commonArgs = {
            inherit src;
            strictDeps = true;
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          crate = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
        in
        rec {
          packages.default = crate;
          checks = {
            inherit crate;
            tests = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; });
            clippy =
              let
                args = {
                  inherit cargoArtifacts;
                  cargoClippyExtraArgs = "--all-targets -- --deny warnings";
                };
              in
              craneLib.cargoClippy (commonArgs // args);

          };
          devShells.default = craneLib.devShell {
            inputsFrom = [ packages.default ];
            packages = with pkgs; [
              nixd
              nixfmt
              taplo
            ];
          };
        };
    };
}
