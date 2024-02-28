{ lib
, stdenv
, darwin
, fetchFromGitHub
, copyDesktopItems
, makeDesktopItem
, libGL
, makeWrapper
, libxkbcommon
, openssl
, pkg-config
, rustPlatform
, vulkan-loader
, wayland
, xorg
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
rustPlatform.buildRustPackage rec {
  name = "nuhxboard";
  version = "0.5.3";

  src = lib.cleanSource ./.;

  cargoHash = "sha256-cjaiCSQ83C/YOSLARoBdO1t/D9dRWT/3tShVTv2KAME=";

  nativeBuildInputs = [
    copyDesktopItems
    pkg-config
    makeWrapper
  ];

  buildInputs = [
    libxkbcommon
    openssl
    vulkan-loader
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    xorg.libXtst
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.AppKit
    darwin.apple_sdk.frameworks.CoreFoundation
    darwin.apple_sdk.frameworks.CoreGraphics
    darwin.apple_sdk.frameworks.Foundation
    darwin.apple_sdk.frameworks.Metal
    darwin.apple_sdk.frameworks.QuartzCore
    darwin.apple_sdk.frameworks.Security
  ] ++ lib.optionals stdenv.isLinux [
    wayland
  ];

  desktopItems = [
    (makeDesktopItem {
      name = "NuhxBoard";
      desktopName = "NuhxBoard";
      comment = "A cross-platform alternative to NohBoard";
      icon = "NuhxBoard.png";
      exec = name;
      terminal = false;
      # mimeTypes = [ "x-scheme-handler/irc" "x-scheme-handler/ircs" ];
      # categories = [ "Keyboard"];
      keywords = [ "Keyboard" ];
      startupWMClass = "NuxBoard";
    })
  ];

  postInstall = ''
    install -Dm644 NuhxBoard.png $out/share/icons/hicolor/128x128/apps/NuhxBoard.png
    wrapProgram $out/bin/${name} --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath runtimeLibs}"
  '';

  # meta = with lib; { #TODO change these
  #   description = "IRC application";
  #   homepage = "https://github.com/squidowl/halloy";
  #   changelog = "https://github.com/squidowl/halloy/blob/${version}/CHANGELOG.md";
  #   license = licenses.gpl3Only;
  #   maintainers = with maintainers; [ fab ];
  # };
}