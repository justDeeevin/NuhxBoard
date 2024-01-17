# <img src="NuhxBoard.png" alt="The NuhxBoard logo" width="34"> NuhxBoard

## Contents

1. [Goals](#goals)
2. [Limitations](#limitations)

## Goals

[Nohboard](https://github.com/ThoNohT/NohBoard) is great! But it's only for Windows. The only alternative is [Tiyenti's KBDisplay](https://github.com/Tiyenti/kbdisplay), which is really great, but limited in functionality. My primary goal with this project is to replicate the functionality of NohBoard in a Linux-compatible manner. More specifically, I want to be able to feed in any NohBoard config file and have near-identical output to NohBoard.

I may add functionality where I think it would fit, but I want to prioritize interoperability with NohBoard. Call it just another incentive for gamers to switch to Linux.

## Limitations

Unfortunately, due to the nature of data structuring in Rust and the behavior of `serde_json`, There are a couple schema tweaks I've had to make.

1. The `elements` property of a keyboard file is a list of multiple different types. I can replicate that behavior well enough with enums. However, when it comes to representing the data structure of a list of enums in json, NuhxBoard and NohBoard take different paths. NohBoard gets type information from the `__type` property. `serde_json` represents enums with fields by making an object with one property (the variant name) containing all the fields. I'm not sure of any alternatives to `serde_json` and I don't think it'd be worthwhile to make a fork of `serde_json` and go to the effort of updating its behavior, so we're stuck with this minor difference. I'd be open to ideas for fixing this difference.
