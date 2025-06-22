{
  description = "Save your boilerplate instead of writing it.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages = {
          default = self.packages.${system}.mkdev;
          mkdev = pkgs.callPackage ./nix/mkdev.nix { };
          mkf = pkgs.callPackage ./nix/mkf.nix { mk = self.packages.${system}.mkdev; };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            argbash
            cargo
            clippy
            gh
            libgcc
            rustc
            rustfmt
          ];

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };
      }
    )
    // flake-utils.lib.eachDefaultSystemPassThrough (system: {
      homeManagerModule = import ./nix/home-manager.nix { inherit (self.packages.${system}) mkdev; };
    });
}
