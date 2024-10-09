{
  description = "flake for fuzzer as it needs nightly";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };
  outputs = {self,  nixpkgs, flake-utils, rust-overlay, ... }:
  flake-utils.lib.eachDefaultSystem (system:
  let
    overlays = [ (import rust-overlay) ];
    pkgs = import nixpkgs {
      inherit system overlays;
    };
  in
  with pkgs;
  {
    devShell = mkShell {
      buildInputs = [
        (rust-bin.nightly.latest.default.override { extensions = [ "llvm-tools-preview" ]; })
        cargo-fuzz
        rustfilt
      ];
    };
  });
}
