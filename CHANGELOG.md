# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.6.2](https://github.com/justdeeevin/nuhxboard/compare/v0.6.1...HEAD) - Unrelased

### Added

- Elements can be added and removed from the layout in the graphical editor.

### Fixed

- Being able to add duplicate keycodes to a keyboard key element. [860176c](https://github.com/justdeeevin/nuhxboard/commit/860176c72ca21106c3a71146a7304e33f2110227)
- Keycode detection behavior not matching NohBoard. [4aa8985](https://github.com/justdeeevin/nuhxboard/commit/4aa8985500e9d425426abd44e720a49126d75c66)
- Behavior of elements with multiple keycodes. They now require all keycodes to be pressed to be highlighted. [84aeb64](https://github.com/justdeeevin/nuhxboard/commit/84aeb645efbc22cf55cac23b8b86a06476b0a234)
- Regression: Mouse speed indicators not being re-drawn when the mouse moves. [797fd8e](https://github.com/justdeeevin/nuhxboard/commit/797fd8ef346f97755382540b6c4a0761a7b72ebb)
- Automatically loading the first layout when changing categories if the cached category is an empty string. [bed6047](https://github.com/justdeeevin/nuhxboard/commit/bed6047a6b40822e1fc62b18dc601fd399c64292)

## [v0.6.1](https://github.com/justDeeevin/NuhxBoard/releases/v0.6.1) - 2025-05-19

This release doesn't change too much for you guys, but it includes a lot of under-the-hood improvements.

### Added

- Element dimensions can now be changed by clicking and dragging.

### Changed

- Config now lives in `~/.config/nuhxboard/` (Linux) or `~/Library/Application Suport/rs.NuhxBoard/` (macOS) or `~\AppData\Roaming\nuhxboard\config\` (Windows) instead of `~/.local/share/nuhxboard/`. [a3d72c2](https://github.com/justdeeevin/nuhxboard/commit/a3d72c29c8191630602dd32a4c0a0d8aa7248c13)

### Fixed

- Handling of null values for `KeyStyle.pressed` and `KeyStyle.loose`. [#13](https://github.com/justdeeevin/nuhxboard/pull/13)
- Improper layering behavior with image backgrounds. [034646d](https://github.com/justdeeevin/nuhxboard/commit/034646d01be4a8ae00a81ef9efcdf5178b03e05b)
- macOS support (PROBABLY. I don't have a modern Mac to test on, and I had to rely on virtualization for testing).

## [v0.6.0](https://github.com/justdeeevin/nuhxboard/releases/v0.6.0) - 2025-12-24

Not quite a 1.0, because faces and vertices cannot yet be dragged.

### Added

#### Edit mode!

There's a new toggle for edit mode in the global context menu. You can graphically manipulate every part of a keyboard _and_ its style. No more typing away at a JSON file.

#### Other additions

- Nix support. [#7](https://github.com/justdeeevin/nuhxboard/pull/7)
- Some example keyboards are downloaded if none are present. [e7bc762](https://github.com/justdeeevin/nuhxboard/commit/e7bc7623d35056860e2005927a31eb115346a0a0)
- Key background images. [e4bd63](https://github.com/justdeeevin/nuhxboard/commit/e4bd63b86414d6dbcadf3033c3296da0c00e3e87)
- Layout background images. [912027](https://github.com/justdeeevin/nuhxboard/commit/9120279b59bdc0b64dbc693bcb3edf5024ea238f)
- Exit button in global context menu. [f1c04a6](https://github.com/justdeeevin/nuhxboard/commit/f1c04a61dff29c7406cd297261292a60f80ad74d)
- Version info in settings menu. [c46943d](https://github.com/justdeeevin/nuhxboard/commit/c46943d0d1c25499b3382aca8d29a7e5b74c2e4f)
- Labels for the layout and style picklists in the load keyboard menu. [2805261](https://github.com/justdeeevin/nuhxboard/commit/28052612b8a0a120e8f0f5c47b372d69922e108d)
- The context menu is now scrollable in case the window is too small for it to fully display. [880650a](https://github.com/justdeeevin/nuhxboard/commit/880650afe3e5881f67a97531adbd8e8707ace0ad)

### Fixed

- The global styles directory appearing in the keyboard categories list. [6594a75](https://github.com/justdeeevin/nuhxboard/commit/6594a7553115938b6de4f15411f5c930939bce74)
- Holding keys for an extended time period causing them to never be released. [bb12d99](https://github.com/justdeeevin/nuhxboard/commit/bb12d9960a3d6bf003662af6445794d4f8472a23)
- Font size not changing with key pressed/loose state. [6726d1c](https://github.com/justdeeevin/nuhxboard/commit/6726d1cb5c86770177ae62bf5902bfc6ad98c2cd)
- Selected style not being remembered between launches. [ef73d9f](https://github.com/justdeeevin/nuhxboard/commit/ef73d9f56414fa1d9067050e0ef79d51dab98451)

### Changed

- Primary display is marked as such in list of displays in settings menu. [f2e0a00](https://github.com/justdeeevin/nuhxboard/commit/f2e0a00d6fd93f47f6cd9111f8452e2e2d344fec)
- `Version` property of keyboard layout is now optional. [79a3162](https://github.com/justdeeevin/nuhxboard/commit/79a3162acdf7de5606f48b6aa7535f39d321cb06)
- Global default style's mouse speed indicator's outer color now matches NohBoard's (white). [fda0006](https://github.com/justdeeevin/nuhxboard/commit/fda000645e15ff3cc2ac7e6f894c821c9b1b1e7a)
- Mouse speed indicator's sensitivity is closer to NohBoard's. [1fde6e6](https://github.com/justdeeevin/nuhxboard/commit/1fde6e6cb385aa77930eda06268e5a8127c7bb05)
- Display option dropdown is not displayed if there is only one option. [1db5fc1](https://github.com/justdeeevin/nuhxboard/commit/1db5fc1b3d3b16467d0beaf7e32816292bbb8326)

### Removed

- Automatic desktop entry creation. [5c6465f](https://github.com/justdeeevin/nuhxboard/commit/5c6465f1fe8f4928f9f4b5e07fd5876ca6e24a59)

## [v0.5.3](https://github.com/justdeeevin/nuhxboard/releases/v0.5.2) - 2024-2-27

#### We're back on [crates.io](https://crates.io/crates/nuhxboard)!

As of [iced_aw#209](https://github.com/iced-rs/iced_aw/pull/209), `iced_aw` is on `iced` v0.12.0. This means that I don't have to depend on their Git repo to have a v0.12 context menu, which means I can finally publish on crates.io again. Hurrah!

### Added

- Support for global styles. [1ee37b9](https://github.com/justdeeevin/nuhxboard/commit/1ee37b973497ce78d4371c17a6d6053c0dd0f055)
- Creation of global styles folder to install behavior. [321c8b9](https://github.com/justdeeevin/nuhxboard/commit/321c8b9175c0f42ac86a0cdf77d86485ee6c4787)

## [v0.5.2](https://github.com/justdeeevin/nuhxboard/releases/v0.5.2) - 2024-2-21

### Added

- Button to clear the pressed keys list in case of emergencies. [ef7bd95](https://github.com/justdeeevin/nuhxboard/commit/ef7bd95608701274f3c1125e7f292de8df6f1eb9)

### Fixed

- Certain characters not rendering. [dfa8f08](https://github.com/justdeeevin/nuhxboard/commit/dfa8f08d577b4296bc0f8a478bcbc4349c5ed8f8)

## [v0.5.1](https://github.com/justdeeevin/nuhxboard/releases/v0.5.1) - 2024-2-20

### Changed

- There is no more `--install` command-line argument. If NuhxBoard sees it doesn't have any settings file to read, it'll make one and also create the start menu entry.

### Fixed

- Crash where a key release was received with no preceding key press. [d5cc1aa](https://github.com/justdeeevin/nuhxboard/commit/d5cc1aa31924f9087a7d7e6b60254253cf0b5c81)

## [v0.5.0](https://github.com/justDeeevin/NuhxBoard/releases/v0.5.0) - 2024-2-19

#### NuhxBoard is now a fully graphical application!

No more command-line arguments. The future is now.  
On launch, you'll be prompted to load a keyboard and select its style through a menu that's as close to NohBoard as I could get. If you want to change the keyboard or style, you can right-click anywhere in the main window and select "Load keyboard" to change your selection.

### Added

- `--install` command to put NuhxBoard in your OS's app list (Not implemented for MacOS).
- Settings and graphical menu to change them.

### Changed

- No longer captures inputs when window is focused. [93aa6d0](https://github.com/justDeeevin/NuhxBoard/commit/93aa6d08bd9a002a472f58ec9cb90b1b41cab91c)

## [v0.4.0](https://github.com/justDeeevin/NuhxBoard/releases/v0.4.0) - 2024-2-13

### Changed

- Receiving an unknown keycode no longer causes NuhxBoard to crash, but a message is still printed to stderr. [87621b5](https://github.com/justDeeevin/NuhxBoard/commit/87621b52b6c16978bce3cdef1b3808fca7f79668)

### Fixed

- Missing iced keycodes `LWin` and `RWin`. [13951ab](https://github.com/justDeeevin/NuhxBoard/commit/13951ab8190ce8f93b1372219a87d65262a74b77)

## [v0.3.2](https://github.com/justdeeevin/NuhxBoard/releases/v0.3.2) - 2024-2-13

### Fixed

- Crash when either meta key or scroll lock was pressed. [4b6dc1b1](https://github.com/justDeeevin/NuhxBoard/commit/4b6dc1b17a4984a592457436dc2132727f28960a)
- Crash when caps lock was pressed with window focused. [ca073ab](https://github.com/justDeeevin/NuhxBoard/commit/ca073ab4739a75f974e153d6c8a8a029fa271b1b)
- Shift text behaving incorrectly. [a677332](https://github.com/justDeeevin/NuhxBoard/commit/a677332c34b92c7f935272dd041ec65bf3116c20)

## [v0.3.1](https://github.com/justdeeevin/NuhxBoard/releases/v0.3.1) - 2024-2-2

### Fixed

- Window size under Wayland. [6af88ac](https://github.com/justdeeevin/NuhxBoard/commit/6af88ac0ec2c390a60ce4f1a2648284dd271be9c)
- Occasional duration_since error while window is focused. [f0ef286](https://github.com/justdeeevin/NuhxBoard/commit/f0ef286c50a18dec68cf8395c97c20f97799356a)

## [v0.3.0](https://github.com/justdeeevin/NuhxBoard/releases/v0.3.0) - 2024-2-1

### Added

- Function to list keyboards and keyboard groups. [67e5308](https://github.com/justdeeevin/NuhxBoard/commit/67e5308a3b7b76253bef1b7de5dd8d830190d35c)
- Warning when running under Wayland. [b389725](https://github.com/justdeeevin/NuhxBoard/commit/b3897255979f55006802939eee9dab4bcc03c478)

### Changed

- Help message tagline now matches NuhxBoard's cross-platform capability. [d300ee9](https://github.com/justdeeevin/NuhxBoard/commit/d300ee9f8902d8c745b47662c9319061c317b7e7)

### Fixed

- Occasional panic on window close. [aad1d09](https://github.com/justdeeevin/NuhxBoard/commit/aad1d0997be01f58092feb43fcc81625b717c450)
- Events not being handled when NuhxBoard is focused. [#4](https://github.com/justdeeevin/NuhxBoard/issues/4)

## [v0.2.1](https://github.com/thepyrotf2/nuhxboard/releases/v0.2.1) - 2024-01-31

### Added

- Cross-platform functionality with [`rdev`](https://crates.io/crates/rdev). [#3](https://github.com/justdeeevin/nuhxboard/pull/3)
