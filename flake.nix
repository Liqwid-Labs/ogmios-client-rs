{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        toolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        unfilteredRoot = ./.; # The original, unfiltered source
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          # tests require live DB and SaaS blockfrost, so it requires network access
          doCheck = false;
        };
        ogmios-client-rs = craneLib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          }
        );
      in
      {
        checks = { inherit ogmios-client-rs; };

        packages.default = ogmios-client-rs;

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = with pkgs; [
            cargo
            pkg-config
            openssl
            rustc
            cargo-watch
            rust-analyzer
          ];
        };
      }
    );
}
