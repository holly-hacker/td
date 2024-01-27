{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ flake-parts, ... }:
    let
      mkTd = { rustPlatform, lib, ... }: rustPlatform.buildRustPackage {
        version = "dev-0.1.0";
        name = "td";
        src = builtins.path {
          path = ./.;
        };
        cargoLock.lockFile = ./Cargo.lock;

        meta = {
          description = "A WIP graph-based TUI TODO app.";
          homepage = "https://github.com/holly-hacker/td";
          license = lib.licenses.bsd2;
          platforms = lib.platforms.unix;
          mainProgram = "td";
        };
      };
    in
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      perSystem = { pkgs, system, self', ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ (import inputs.rust-overlay) ];
        };

        apps.default = {
          type = "app";
          program = self'.packages.default;
        };

        packages.default = pkgs.callPackage mkTd { };

        devShells.default =
          let
            rust-toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
              extensions = [ "rust-src" "rust-analyzer" ];
            };
          in
          pkgs.mkShell {
            packages = [ rust-toolchain ];
          };
      };
    };
}
