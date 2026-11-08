{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        craneLib = crane.mkLib pkgs;

        dylibs = with pkgs; [ libxcb ];

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          # TODO: darwin deps
          buildInputs = [
          ]
          ++ dylibs;
          nativeBuildInputs = with pkgs; [ pkg-config ];
        };

        nuhxboard = craneLib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          }
        );
      in
      {
        checks = {
          inherit nuhxboard;
        };

        packages.default = nuhxboard;

        apps.default = flake-utils.lib.mkApp {
          drv = nuhxboard;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath dylibs;
        };
      }
    );
}
