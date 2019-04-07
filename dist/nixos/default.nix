let
  pkgs = ((import <nixpkgs>) {});
in
  pkgs.callPackage ./orderly-0_4_0.nix {}