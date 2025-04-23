{ mk, pkgs, ... }:

with pkgs;
writeShellApplication {
  name = "mkf";

  runtimeInputs = [
    mk
    bat
    fzf
  ];

  text = lib.readFile ./scripts/mkf.sh;
}
