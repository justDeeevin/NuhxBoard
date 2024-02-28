{ 
callPackage
, lib
, libGL
, libxkbcommon
, vulkan-loader
, xorg
, mkShell
, cargo
, cargo-watch
, rustc
, rustup
, clippy
, rust-analyzer
, pkg-config
}:
let
  runtimeLibs = [
    libGL
    libxkbcommon
    vulkan-loader
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];
in
mkShell {
  # Get dependencies from the main package
  inputsFrom = [ (callPackage ./package.nix { }) ];
  LD_LIBRARY_PATH = "${lib.makeLibraryPath runtimeLibs}";
  # Additional tooling
  buildInputs = with pkgs; [
    cargo
    cargo-watch
    rustc
    rustup
    clippy
    rust-analyzer
    pkg-config
  ];
}