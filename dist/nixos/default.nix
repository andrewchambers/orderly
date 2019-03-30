let
  pkgs = ((import <nixpkgs>) {});
in
  pkgs.callPackage ./orderly-0_2_0.nix {}