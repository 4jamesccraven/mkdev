{ lib, pkgs }:

with pkgs;
let
  manifest = (pkgs.lib.importTOML ../Cargo.toml).package;
in
rustPlatform.buildRustPackage {
  pname = manifest.name;
  version = manifest.version;

  src = ../.;

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ installShellFiles ];

  postInstall = ''
    installShellCompletion --cmd mk \
      --bash <(COMPLETE=bash $out/bin/mk) \
      --zsh <(COMPLETE=zsh $out/bin/mk) \
      --fish <(COMPLETE=fish $out/bin/mk)

    MANPAGE=true $out/bin/mk
    installManPage ./mkdev-man/*
  '';

  meta = {
    license = lib.licenses.mit;
    mainProgram = "mk";
  };
}
