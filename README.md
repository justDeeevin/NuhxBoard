# <img src="NuhxBoard.png" alt="The NuhxBoard logo" width="34"> NuhxBoard

## Contents

1. [Goals](#goals)
2. [Usage](#usage)

## Goals

[Nohboard](https://github.com/ThoNohT/NohBoard) is great! But it's only for Windows. The only alternative is [Tiyenti's KBDisplay](https://github.com/Tiyenti/kbdisplay), which is really great, but limited in functionality. My primary goal with this project is to replicate the functionality of NohBoard in a Linux-compatible manner. More specifically, I want to be able to feed in any NohBoard config file and have near-identical output to NohBoard.

I may add functionality where I think it would fit, but I want to prioritize interoperability with NohBoard. Call it just another incentive for gamers to switch to Linux.

## Usage

Right now, NuhxBoard has to be launched from the command line. In a future release, I plan on porting the graphical interface present in NohBoard. Launch arguments are used to decide the keyboard and style. This means that the app has to be relaunched to change the keyboard or style.

To specify a keyboard layout, provide the group and the keyboard name in the `--keyboard` argument, in the format `[GROUP]/[KEYBOARD]`. To specify a style, just provide the name of the style in the `--style` argument.

Here's the output of the `--help` command:

```
NuhxBoard - The Linux port of NohBoard

Usage: NuhxBoard [OPTIONS] --keyboard <KEYBOARD>

Options:
  -k, --keyboard <KEYBOARD>  The keyboard to use. [GROUP]/[KEYBOARD]
  -s, --style <STYLE>        The style to use. Must be in the same directory as the provided keyboard. If not provided, global default will be used
  -h, --help                 Print help
  -V, --version              Print version

Add keyboard groups to ~/.local/share/NuhxBoard/keyboards/
```

_God, I love `clap`._
