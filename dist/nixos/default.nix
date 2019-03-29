let
  pkgs = ((import <nixpkgs>) {});
in
  pkgs.callPackage ./orderly-0_1_0.nix {}