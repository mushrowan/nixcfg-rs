{
  description = "nixcfg-rs - rust driver for nixcfg (schemars + #[nixcfg] macro)";

  inputs = {
    nixpkgs.url = "git+https://github.com/nixos/nixpkgs?shallow=1&ref=nixos-unstable";

    crane.url = "github:ipetkov/crane";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts = {
      url = "git+https://github.com/hercules-ci/flake-parts?shallow=1";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.git-hooks.flakeModule
      ];

      perSystem = {
        system,
        self',
        config,
        ...
      }: let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [(import inputs.rust-overlay)];
        };

        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = ["rust-src" "rust-analyzer"];
        };

        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

        craneOutputs = import ./nix/packages.nix {
          inherit pkgs craneLib;
          src = craneLib.cleanCargoSource ./.;
        };
      in {
        packages.default = craneOutputs.package;

        checks = {
          inherit (craneOutputs) package clippy test deny doctest schemaCheck;
        };

        devShells.default = craneLib.devShell {
          inherit (craneOutputs) cargoArtifacts;
          inherit (self') checks;
          shellHook = config.pre-commit.installationScript;

          packages = with pkgs; [
            cargo-deny
            cargo-edit
            cargo-machete
            cargo-nextest
            cargo-semver-checks
            cargo-watch
            jujutsu
            # cargo publish's verify step needs a C linker for proc-macro
            # build scripts. rust-overlay toolchain ships rustc/cargo but
            # not cc
            stdenv.cc
          ];
        };

        pre-commit.settings.hooks = {
          treefmt.enable = true;
          treefmt.package = config.treefmt.build.wrapper;
        };

        treefmt = {
          projectRootFile = "flake.nix";
          programs = {
            alejandra.enable = true;
            deadnix.enable = true;
            statix.enable = true;
            rustfmt = {
              enable = true;
              package = rustToolchain;
            };
            taplo.enable = true;
          };
        };
      };
    };
}
