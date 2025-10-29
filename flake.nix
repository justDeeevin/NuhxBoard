{
  description = "NuhxBoard flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        inherit (pkgs) lib;

        rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        iconFilter = path: (builtins.match ".*NuhxBoard.png$" path) != null;
        iconOrCargo = path: type: (iconFilter path) || (craneLib.filterCargoSources path type);
        src = lib.cleanSourceWith {
          src = ./.;
          filter = iconOrCargo;
          name = "source";
        };
        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = with pkgs; [
            expat
            fontconfig
            freetype
            freetype.dev
            libGL
            pkg-config
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            xorg.libXtst
            xorg.libxcb
            wayland
            libxkbcommon
            libevdev
          ];

          nativeBuildInputs = with pkgs; [
            copyDesktopItems
            pkg-config
            makeWrapper
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        nuhxboard = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
          // {
            desktopItems = [
              (pkgs.makeDesktopItem {
                name = "NuhxBoard";
                desktopName = "NuhxBoard";
                comment = "Cross-platform input visualizer";
                icon = "NuhxBoard";
                exec = "nuhxboard";
                terminal = false;
                keywords = [ "Keyboard" ];
                startupWMClass = "NuhxBoard";
              })
            ];

            postInstall = ''
              install -Dm644 ${src}/media/NuhxBoard.png $out/share/icons/hicolor/128x128/apps/NuhxBoard.png
              wrapProgram $out/bin/nuhxboard --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath commonArgs.buildInputs}"
            '';
          }
        );
      in
      {
        checks = {
          nuhxboard = nuhxboard;

          nuhxboard-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          nuhxboard-fmt = craneLib.cargoFmt { inherit src; };
        };

        packages = {
          default = nuhxboard;
        };

        apps.default = flake-utils.lib.mkApp { drv = nuhxboard; };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          packages = with pkgs; [
            cargo-dist
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath commonArgs.buildInputs;

          RUSTFLAGS = "-Z threads=8";
        };
      }
    );
}
