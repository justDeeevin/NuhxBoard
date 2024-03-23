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
  - [Keyboard Layouts](#keyboard-layouts)
    - [Top-Level Properties](#top-level-properties)
    - [Elements](#elements)
      - [KeyboardKeys](#keyboardkeys)
3. [Installation](#installation)
4. [Demo](#demo)

## Goals

[Nohboard](https://github.com/ThoNohT/NohBoard) is great! But it's only for Windows. The only alternative is [Tiyenti's KBDisplay](https://github.com/Tiyenti/kbdisplay), which is really great, but limited in functionality. My primary goal with this project is to replicate the functionality of NohBoard in a cross-compatible manner. More specifically, I want to be able to feed in any NohBoard config file and have near-identical output to NohBoard.

I may add functionality where I think it would fit, but I want to prioritize interoperability with NohBoard. Call it just another incentive for gamers to switch to Linux.

## Usage

NuhxBoard is made with customizability in mind. Every part of its appearance and behavior is configurable. At its core, NuhxBoard is an app that loads keyboard layouts and styles. A keyboard layout defines the positions, shapes, and behaviors of keys. It also defines the dimensions of the window. A style defines colors, fonts, and (in a future release) images for keys.

Keyboard layouts are grouped into categories, and styles (aside from global ones) correspond to specific keyboard layouts.

Keyboards are located in `~/.local/share/NuhxBoard/keyboards`. Here's the general structure of that directory:

- keyboards/
  - \[CATEGORY NAME\]/
    - \[KEYBOARD NAME\]/
      - keyboard.json
      - \[STYLE NAME\].style
  - global/
    - \[STYLE NAME\].style

This folder will be populated on first run with some example keyboards and categories. You can inspect it yourself to get a good idea of how this looks in practice.

To load a keyboard and style, right-click anywhere in NuhxBoard to open the global context menu and click on "Load Keyboard". This will open a new window. The drop-down list labeled "Categories" allows you to select a category. When a category has been selected, the keyboards available in that category will appear in a list on the left side of the vertical line. When you click on one of these options, your selection of keyboard layout will be loaded, and that keyboard layout's available styles will appear in a list on the right side of the vertical line. When you click on one of these options, your selection of style will be loaded. You can change you selection of keyboard layout and style at any time through this interface.

### Keyboard Layouts

As previously stated, keyboard layouts define key positions, shapes, and behaviors, as well as window dimensions. Keyboard layouts are defined by a JSON file, `keyboard.json`, in their corresponding named directory. Here's what the type definition for a keyboard layout looks like in rust:

```rust
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    #[serde(rename = "Version")]
    pub version: Option<u8>,
    #[serde(rename = "Width")]
    pub width: f32,
    #[serde(rename = "Height")]
    pub height: f32,
    #[serde(rename = "Elements")]
    pub elements: Vec<BoardElement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "__type")]
pub enum BoardElement {
    KeyboardKey(KeyboardKeyDefinition),
    MouseKey(MouseKeyDefinition),
    MouseScroll(MouseScrollDefinition),
    MouseSpeedIndicator(MouseSpeedIndicatorDefinition),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyboardKeyDefinition {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Boundaries")]
    pub boundaries: Vec<SerializablePoint>,
    #[serde(rename = "TextPosition")]
    pub text_position: SerializablePoint,
    #[serde(rename = "KeyCodes")]
    pub keycodes: Vec<u32>,
    #[serde(rename = "Text")]
    pub text: String,
    #[serde(rename = "ShiftText")]
    pub shift_text: String,
    #[serde(rename = "ChangeOnCaps")]
    pub change_on_caps: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseKeyDefinition {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Boundaries")]
    pub boundaries: Vec<SerializablePoint>,
    #[serde(rename = "TextPosition")]
    pub text_position: SerializablePoint,
    #[serde(rename = "KeyCodes")]
    pub keycodes: Vec<u32>,
    #[serde(rename = "Text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseScrollDefinition {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Boundaries")]
    pub boundaries: Vec<SerializablePoint>,
    #[serde(rename = "TextPosition")]
    pub text_position: SerializablePoint,
    #[serde(rename = "KeyCodes")]
    pub keycodes: Vec<u32>,
    #[serde(rename = "Text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseSpeedIndicatorDefinition {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Location")]
    pub location: SerializablePoint,
    #[serde(rename = "Radius")]
    pub radius: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializablePoint {
    #[serde(rename = "X")]
    pub x: f32,
    #[serde(rename = "Y")]
    pub y: f32,
}
```

If you can make sense of that, then good for you! Otherwise, here's an actual explanation of how a keyboard layout is defined.

#### Top-Level Properties

There are four properties of a keyboard layout:

- Version  
  No actual meaning. Kept for parity with NohBoard layout files.
- Width  
  Width of the window
- Height  
  Height of the window
- Elements  
  Array of elements in the layout

#### Elements

There are four kinds of elements: KeyboardKeys, MouseKeys, MouseScrolls, and MouseSpeedIndicators. Each item in the list of elements indicates what kind it is by having a `__type` property.

##### KeyboardKeys

- Id  
  Each element has a unique Id. Style files can apply styles to specific keys by referring to their Id.
- Boundaries  
  Elements' shapes are defined by an array of points, their vertices. When no image is specified for an element, it is drawn by connecting lines between each point in the order they appear in the list (including closing the shape by connecting the last vertex to the first), then filling the polygon formed. Even if an element has an image specified, the boundaries are used for the graphical layout editor to know when your cursor is hovering over an element.
- TextPosition  
  The point where the top-left corner of the element's text is to be. Technically, this can be anywhere in the window.
- KeyCodes  
  An array containing the keycodes (just integers) this key should track. You can have one element listen for multiple keys! In a future release, there will be a tool in the element properties menu of the graphical layout editor that will help to figure out which key corresponds to which keycode. For the time being, you chan check [this document](./KEYCODES.md) for conversion.
- Text  
  The text to display on the key
- ShiftText  
  The text to display when shift is held
- ChangeOnCaps  
  Whether or not to follow the state of caps lock (generally, this is `true` for letters and `false` for symbols)

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
