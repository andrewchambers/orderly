let
  pkgs = ((import <nixpkgs>) {});
in
  pkgs.callPackage ./orderly-0_5_1.nix {}