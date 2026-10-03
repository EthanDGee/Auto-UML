{
  description = "Auto-UML — an automatic UML diagram generator for a variety of programming languages";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-26.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        auto-uml = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;

          src = self;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.pkg-config ];

          # git2 pulls in libgit2-sys, which links against zlib/openssl/libssh2.
          buildInputs = [
            pkgs.zlib
            pkgs.openssl
            pkgs.libssh2
          ];

          meta = {
            description = cargoToml.package.description;
            homepage = cargoToml.package.repository;
            license = pkgs.lib.licenses.mit;
            mainProgram = "auto-uml";
            platforms = pkgs.lib.platforms.linux;
          };
        };
      in
      {
        packages = {
          inherit auto-uml;
          default = auto-uml;
        };

        apps.default = {
          type = "app";
          program = "${auto-uml}/bin/auto-uml";
          meta = {
            inherit (auto-uml.meta) description homepage license;
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ auto-uml ];
          packages = [
            pkgs.cargo
            pkgs.rustc
            pkgs.rustfmt
            pkgs.clippy
            pkgs.rust-analyzer
          ];
          env.RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };

        formatter = pkgs.nixfmt;
      }
    );
}
