{
  description = "Save your boilerplate instead of writing it.";

  inputs.nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.xz";

  outputs =
    {
      nixpkgs,
      ...
    }:
    let
      inherit (nixpkgs) lib;

      eachDefaultSystem =
        func: lib.genAttrs lib.systems.flakeExposed (system: func nixpkgs.legacyPackages.${system});
    in
    {

      packages = eachDefaultSystem (pkgs: rec {
        default = mkdev;
        mkdev = pkgs.callPackage ./nix/mkdev.nix { };
      });

      overlays.default = prev: final: {
        mkdev = prev.callPackage ./nix/mkdev.nix { };
      };

      homeManagerModules.default = import ./nix/home-manager.nix;

      devShells = eachDefaultSystem (pkgs: {
        default = pkgs.mkShell {
          buildInputs = builtins.attrValues {
            inherit (pkgs)
              # Rust
              cargo
              clippy
              libgcc
              rustc
              rustfmt

              # Nix
              statix
              ;
          };

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };
      });

    };
}
