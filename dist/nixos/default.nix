let
  pkgs = ((import <nixpkgs>) {});
in
  pkgs.callPackage ./orderly-0_5_0.nix {}