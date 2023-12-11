{ nixpkgs ? import <nixpkgs> { }}:

let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
in
with pkgs;
mkShell {
    buildInputs = [
      rust-bin.stable.latest.default
      rust-analyzer
      openssl
      pkg-config
      p7zip

    ];

    RUST_BACKTRACE = 1;
  }