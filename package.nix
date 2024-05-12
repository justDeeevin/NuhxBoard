{
  lib,
  copyDesktopItems,
  makeDesktopItem,
  makeWrapper,
  pkg-config,
  rustPlatform,
  libs,
}: let
  package = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
in
  rustPlatform.buildRustPackage rec {
    name = package.name;
    version = package.version;

    src = lib.cleanSource ./.;

    cargoLock = {
      lockFile = ./Cargo.lock;
      outputHashes = {
        "iced-multi-window-2.0.0" = "sha256-AiVxvxg6nGpnKUsHRgT5qKgPKRYkdee3Nnc/1wL69hU=";
      };
    };

    nativeBuildInputs = [
      copyDesktopItems
      pkg-config
      makeWrapper
    ];

    buildInputs = libs;

    desktopItems = [
      (makeDesktopItem {
        name = "NuhxBoard";
        desktopName = name;
        comment = package.description;
        icon = "NuhxBoard.png";
        exec = package.name;
        terminal = false;
        keywords = ["Keyboard"];
        startupWMClass = "NuhxBoard";
      })
    ];

    postInstall = ''
      install -Dm644 NuhxBoard.png $out/share/icons/hicolor/128x128/apps/NuhxBoard.png
      wrapProgram $out/bin/${name} --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath libs}"
    '';

    meta = {
      description = package.description;
      homepage = package.repository;
      changelog = "${package.repository}/blob/${version}/CHANGELOG.md";
      license = package.license;
    };
  }
