{ pkgs, ... }:

with pkgs;
writeShellApplication {
  name = "mkf";

  runtimeInputs = [
    mkdev
    bat
    fzf
  ];

  text = lib.readFile ../scripts/mkf.sh;
}
