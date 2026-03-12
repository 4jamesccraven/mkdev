{
  description = "Save your boilerplate instead of writing it.";

  inputs.nixpkgs.url = "https://channels.nixos.org/nixos-25.11/nixexprs.tar.xz";

  outputs =
    {
      self,
      nixpkgs,
      ...
    }:
    let
      inherit (nixpkgs) lib;

      eachDefaultSystem =
        func: lib.genAttrs lib.systems.flakeExposed (system: func nixpkgs.legacyPackages.${system});
    in
    {

      packages = eachDefaultSystem (
        pkgs:
        let
          inherit (pkgs.stdenv.hostPlatform) system;
        in
        {
          default = self.packages.${system}.mkdev;
          mkdev = pkgs.callPackage ./nix/mkdev.nix { };
          mkf = pkgs.callPackage ./nix/mkf.nix { };
        }
      );

      overlays.default = prev: final: {
          mkdev = prev.callPackage ./nix/mkdev.nix { };
          mkf = prev.callPackage ./nix/mkf.nix { };
        };

      homeManagerModules.default = import ./nix/home-manager.nix;
      homeManagerModule = # .
        lib.warn # .
          "mkdev: The option `homeManagerModule' has been renamed to `homeManagerModules.default'." # .
          self.homeManagerModules.default;

      devShells = eachDefaultSystem (pkgs: {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust
            cargo
            clippy
            libgcc
            rustc

            # Nix
            statix

            # Bash
            argbash

            # Misc.
            gh
          ];

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };
      });

    };
}
