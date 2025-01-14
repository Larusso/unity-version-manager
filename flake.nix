{
  description = "Development environment";

  inputs = {
    naersk = { url = "github:nmattia/naersk/master"; };
    nixpkgs = { url = "github:NixOS/nixpkgs/nixos-unstable"; };
    utils = { url = "github:numtide/flake-utils"; };
    flake-compat = {
      url = github:edolstra/flake-compat;
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, naersk, ... }:
   utils.lib.eachDefaultSystem (system:
      let
        inherit (nixpkgs.lib) optional;
        naersk-lib = pkgs.callPackage naersk { };
        pkgs = import nixpkgs { inherit system; };
        libPath = with pkgs; lib.makeLibraryPath [
            openssl
        ];
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          doCheck = false;
          pname = "uvm";
          nativeBuildInputs = [ pkgs.makeWrapper pkgs.pkg-config ];
          buildInputs = with pkgs; [
            p7zip
            glibc
            openssl
          ];
          postInstall = ''
            wrapProgram "$out/bin/uvm" --prefix LD_LIBRARY_PATH : "${libPath}"
          '';
        };

        defaultApp = utils.lib.mkApp {
          drv = self.defaultPackage."${system}";
        };

        devShell = with pkgs; mkShell {
          
          buildInputs = [
            cargo
            cargo-insta
            pre-commit
            rust-analyzer
            rustPackages.clippy
            rustc
            rustfmt
            openssl
            pkg-config
          ];

          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LD_LIBRARY_PATH = libPath;
        };

      });
}

