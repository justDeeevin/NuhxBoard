{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = inputs:
    with inputs; let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = fenix.packages.${system}.complete.toolchain;
      libs = with pkgs.pkgs;
        [
          libGL
          libxkbcommon
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          wayland
          openssl
          xorg.libXtst
          libevdev
        ]
        ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.AppKit
          darwin.apple_sdk.frameworks.CoreFoundation
          darwin.apple_sdk.frameworks.CoreGraphics
          darwin.apple_sdk.frameworks.Foundation
          darwin.apple_sdk.frameworks.Metal
          darwin.apple_sdk.frameworks.QuartzCore
          darwin.apple_sdk.frameworks.Security
        ];
    in {
      packages.x86_64-linux = rec {
        default = nuxhxboard;
        nuxhxboard = pkgs.callPackage ./package.nix {
          libs = libs;
        };
      };
      devShells.x86_64-linux = {
        default = pkgs.mkShell {
          # Get dependencies from the main package
          inputsFrom = [
            (pkgs.callPackage ./package.nix {
              libs = libs;
            })
          ];
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libs}";
          # Additional tooling
          packages = [toolchain];
        };
      };
    };
}
