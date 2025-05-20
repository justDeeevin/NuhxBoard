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

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
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

          buildInputs =
            with pkgs;
            [
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
              wayland
              libxkbcommon
              libevdev
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];

          nativeBuildInputs = with pkgs; [
            copyDesktopItems
            pkg-config
            makeWrapper
          ];
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
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
              install -Dm644 ${src}/NuhxBoard.png $out/share/icons/hicolor/128x128/apps/NuhxBoard.png
              wrapProgram $out/bin/nuhxboard --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath commonArgs.buildInputs}"
            '';
          }
        );
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          nuhxboard = nuhxboard;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          nuhxboard-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          nuhxboard-doc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

          # Check formatting
          nuhxboard-fmt = craneLib.cargoFmt { inherit src; };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `nuhxboard` if you do not want
          # the tests to run twice
          nuhxboard-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            }
          );
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
            cargo-release
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath commonArgs.buildInputs;
        };
      }
    );
}
