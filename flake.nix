{
  description = "Service to limit power consumption on ryzen cpus";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };
  outputs = {
    nixpkgs,
    treefmt-nix,
    self,
    ...
  }: let
    pkgs = nixpkgs.legacyPackages.x86_64-linux;

    treefmtEval = treefmt-nix.lib.evalModule pkgs {
      projectRootFile = "flake.nix";
      programs = {
        alejandra.enable = true;
        deadnix.enable = true;
        statix.enable = true;
        rustfmt.enable = true;
      };
    };

    treefmt = treefmtEval.config.build;

    buildInputs = [pkgs.pciutils];

    nativeBuildInputs = with pkgs; [
      clang
      pkg-config
      cmake
      rustPlatform.bindgenHook
    ];

    ryzencap = let
      name = "ryzencap";
    in
      pkgs.rustPlatform.buildRustPackage {
        inherit buildInputs nativeBuildInputs name;
        cargoLock.lockFile = ./Cargo.lock;
        src = ./.;
        meta.mainProgram = name;
      };
  in {
    nixosModules.ryzencap = import ./modules self;

    formatter.${pkgs.system} = treefmt.wrapper;

    checks.${pkgs.system}.formatting = treefmt.check self;

    devShells.${pkgs.system}.default = pkgs.mkShell {
      inherit buildInputs nativeBuildInputs;
      inputsFrom = [treefmt.devShell];
      packages = with pkgs; [
        nil
        rustc
        cargo
        clippy
        rust-analyzer
      ];
    };

    packages.${pkgs.system} = {
      inherit ryzencap;
      default = ryzencap;
    };
  };
}
