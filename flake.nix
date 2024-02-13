{
  description = "NixOS configuration";

  # All inputs for the system
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };
  outputs = {flake-parts, ...} @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} ({...}: {
      systems = ["x86_64-linux"];

      imports = [
        inputs.treefmt-nix.flakeModule
      ];

      perSystem = {
        config,
        pkgs,
        ...
      }: let
        buildInputs = [pkgs.pciutils];

        nativeBuildInputs = with pkgs; [
          clang
          pkg-config
          cmake
          rustPlatform.bindgenHook
        ];

        pwr-cap-rs = pkgs.rustPlatform.buildRustPackage {
          inherit buildInputs nativeBuildInputs;
          name = "pwr-cap-rs";
          cargoLock.lockFile = ./Cargo.lock;
          src = ./.;
        };
      in {
        treefmt.config = {
          projectRootFile = "flake.nix";
          programs = {
            alejandra.enable = true;
            deadnix.enable = true;
            statix.enable = true;
            rustfmt.enable = true;
          };
        };
        devShells.default = pkgs.mkShell {
          inputsFrom = [config.treefmt.build.devShell];

          packages = with pkgs; [
            nil
            rustc
            cargo
          ];

          inherit buildInputs nativeBuildInputs;

          BINDGEN_EXTRA_CLANG_ARGS = "-I ${pkgs.pciutils}/Include";
        };

        packages.default = pwr-cap-rs;
      };
    });
}