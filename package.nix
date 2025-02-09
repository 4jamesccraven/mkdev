{lib, pkgs}:

with pkgs;
rustPlatform.buildRustPackage {
  pname = "mkdev";
  version = "3.0.1";

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    license = lib.licenses.mit;
    mainProgram = "mk";
  };
}
