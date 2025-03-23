{ pkgs ? import <nixpkgs> {} }:

let
  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rustVersion = overrides.toolchain.channel;
  rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [
      "rust-src"
      "rust-analyzer"
    ];
    targets = [
      "riscv32imc-unknown-none-elf"
    ];
  };
  buildInputs = (import ./nix/inputs.nix pkgs).buildInputs;
in

pkgs.mkShell {
  inherit buildInputs;
  nativeBuildInputs = [
    pkgs.pkg-config
    rust
  ];
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

  RUST_BACKTRACE = 1;
}