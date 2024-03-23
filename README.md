# <img src="NuhxBoard.png" alt="The NuhxBoard logo" width="34"> NuhxBoard

![Crates.io Version](https://img.shields.io/crates/v/nuhxboard)
![Crates.io License](https://img.shields.io/crates/l/nuhxboard)
![Crates.io Total Downloads](https://img.shields.io/crates/d/nuhxboard)

<a href="https://github.com/iced-rs/iced">
  <img src="https://gist.githubusercontent.com/hecrj/ad7ecd38f6e47ff3688a38c79fd108f0/raw/74384875ecbad02ae2a926425e9bcafd0695bade/color.svg" width="130px">
</a>

## Contents

1. [Goals](#goals)
2. [Usage](#usage)
3. [Installation](#installation)
4. [Demo](#demo)

## Goals

[Nohboard](https://github.com/ThoNohT/NohBoard) is great! But it's only for Windows. The only alternative is [Tiyenti's KBDisplay](https://github.com/Tiyenti/kbdisplay), which is really great, but limited in functionality. My primary goal with this project is to replicate the functionality of NohBoard in a cross-compatible manner. More specifically, I want to be able to feed in any NohBoard config file and have near-identical output to NohBoard.

I may add functionality where I think it would fit, but I want to prioritize interoperability with NohBoard. Call it just another incentive for gamers to switch to Linux.

## Usage

As per the above stated goal, your likely best source of info will be [NohBoard's documentation](https://github.com/ThoNohT/NohBoard/wiki/How-To-Use). The directory structure in `keyboards` is the same. `keyboards` itself lives in `~/.local/share/NuhxBoard/`. Nonetheless, there is a work-in-progress NuhxBoard GitHub wiki.

## Installation

NuhxBoard is currently only on [crates.io](https://crates.io/crates/nuhxboard). It can also be installed with [`cargo-binstall`](https://crates.io/crates/cargo-binstall). You can also install NuhxBoard using the option matching your platform on the [latest release page](https://github.com/thepyrotf2/nuhxboard/releases/latest).

NuhxBoard will detect if any app files are missing and replace them automatically. This includes

- The main settings  
  If the `NuhxBoard.json` file containing app settings and saved state doesn't exist, it'll be populated with defaults.
- Installed keyboards  
  If the `keyboards` directory is empty or doesn't exist, then nuhxboard will download a pack of example keyboards to use.
- Optionally, desktop entries  
  On windows, if there isn't a listing in the start menu, or on Linux, if there isn't a desktop entry in the proper place, NuhxBoard will create one. This behavior can be disabled in case you want to make your own desktop entry.

## Demo

https://github.com/justDeeevin/NuhxBoard/assets/90054389/36dc9cf6-3b23-435c-a742-18dddf9c7c19

Configurable like NohBoard:

https://github.com/justDeeevin/NuhxBoard/assets/90054389/80c69a52-e76d-4715-a22c-78db34743959
