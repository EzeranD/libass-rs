{
  description = "libass-rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, fenix, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;

        rustToolchain = fenix.packages.${system}.latest.withComponents [
          "cargo"
          "clippy"
          "rust-analyzer"
          "rustc"
          "rustfmt"
          "rust-src"
        ];

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        crateInfo = craneLib.crateNameFromCargoToml {
          cargoToml = ./crates/libass/Cargo.toml;
        };

        unfilteredRoot = ./.;

        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.maybeMissing ./crates/libass-sys/data)
          ];
        };

        commonArgs = {
          inherit src;
          pname = "libass-rs";
          version = crateInfo.version;
          strictDeps = true;

          nativeBuildInputs = [ pkgs.pkg-config pkgs.libclang.lib ];

          buildInputs = [ pkgs.libass ];

          BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.stdenv.cc.libc.dev}/include";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      in {
        checks = {
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          fmt = craneLib.cargoFmt (commonArgs // { inherit src; });

          toml-fmt = craneLib.taploFmt (commonArgs // {
            src = pkgs.lib.sources.sourceFilesBySuffices src [".toml"];
          });

          audit = craneLib.cargoAudit (commonArgs // {
            inherit src advisory-db;
          });

          deny = craneLib.cargoDeny (commonArgs // { inherit src; });

          nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
            cargoNextestPartitionsExtraArgs = "--no-tests=pass";
          });
        };

        formatter = pkgs.alejandra;

        devShells.default = craneLib.devShell (commonArgs // {
          checks = self.checks.${system};

          version = null;

          packages = [];
        });
      });
}
