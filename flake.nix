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
        lib = pkgs.lib;

        craneLib = crane.mkLib pkgs;

        libs = with pkgs; [
          fontconfig
          libgcc
          libxcb
        ];

        dylibs = with pkgs; [
          # wayland
          wayland
          libxkbcommon

          # x11
          libx11
          libxcursor
          libxi

          # gpu
          libGL
          vulkan-loader
        ];

        commonArgs = {
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources ./.)
              ./ui
            ];
          };
          strictDeps = true;

          # TODO: darwin deps
          buildInputs = [ ] ++ libs;
          nativeBuildInputs = with pkgs; [
            pkg-config
            autoPatchelfHook
          ];
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
          env = {
            LD_LIBRARY_PATH = lib.makeLibraryPath dylibs;
            SLINT_LIVE_PREVIEW = 1;
          };
        };
      }
    );
}
