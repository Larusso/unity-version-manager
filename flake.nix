{
  description = "Unity Version Manager - CLI tool for managing Unity installations";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
    openspec = {
      url = "github:Fission-AI/OpenSpec";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin" ];
      perSystem = { config, self', pkgs, lib, system, ... }:
        let
          runtimeDeps = with pkgs; [
            p7zip
            openssl
          ];
          buildDeps = with pkgs; [
            pkg-config
            rustPlatform.bindgenHook
            makeWrapper
          ];
          devDeps = [
            inputs.openspec.packages.${system}.default
          ];
          libPath = with pkgs; lib.makeLibraryPath [
            openssl
          ];

          cargoToml = builtins.fromTOML (builtins.readFile ./uvm/Cargo.toml);
          msrv = cargoToml.package.rust-version or null;

          rustPackage = features:
            (pkgs.makeRustPlatform {
              cargo = pkgs.rust-bin.stable.latest.minimal;
              rustc = pkgs.rust-bin.stable.latest.minimal;
            }).buildRustPackage {
              inherit (cargoToml.package) name version;
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;
              buildFeatures = features;
              buildInputs = runtimeDeps;
              nativeBuildInputs = buildDeps;
              doCheck = false;
              postInstall = ''
                wrapProgram "$out/bin/uvm" --prefix LD_LIBRARY_PATH : "${libPath}"
              '';
            };

          mkDevShell = rustc:
            pkgs.mkShell {
              shellHook = ''
                export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}

                echo "┌────────────────────────────────────────────────────────────┐"
                echo "│  Unity Version Manager - Development Environment           │"
                echo "└────────────────────────────────────────────────────────────┘"
                echo ""
                echo "Rust: $(rustc --version)"
                echo "Cargo: $(cargo --version)"
                echo ""
                echo "Build Commands:"
                echo "  cargo build --workspace       - Build all crates"
                echo "  cargo build --release         - Build release version"
                echo "  cargo run --bin uvm -- <cmd>  - Run development binary"
                echo ""
                echo "Test & Lint:"
                echo "  cargo test --workspace        - Run all tests"
                echo "  cargo fmt                     - Format code"
                echo "  cargo clippy --workspace      - Run linter"
                echo ""
              '';
              buildInputs = runtimeDeps;
              nativeBuildInputs = buildDeps ++ devDeps ++ [ rustc ];
            };
        in {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };

          packages.default = self'.packages.uvm;
          devShells.default = self'.devShells.stable;

          packages.uvm = (rustPackage "");

          devShells.nightly = (mkDevShell (pkgs.rust-bin.selectLatestNightlyWith
            (toolchain: toolchain.default)));
          devShells.stable = (mkDevShell pkgs.rust-bin.stable.latest.default);
          devShells.msrv = (mkDevShell (
            if msrv != null
            then pkgs.rust-bin.stable.${msrv}.default
            else pkgs.rust-bin.stable.latest.default
          ));
        };
    };
}