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
          mkdev = pkgs.callPackage ./mkdev.nix { };
          mkf = pkgs.callPackage ./mkf.nix { mk = self.packages.${system}.mkdev; };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            libgcc
          ];

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          # Can enable when needed by running `eval $ALIAS_CARGO_CLEAN_ALL`
          ALIAS_CARGO_CLEAN_ALL = "alias cargo-clean-all='find -name Cargo.toml -execdir cargo clean \\;'";
        };
      }
    );
}
