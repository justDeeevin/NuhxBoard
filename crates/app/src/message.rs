use crate::{nuhxboard::*, nuhxboard_types::*};
use geo::Coord;
use iced::{window, Color, Theme};
use iced_multi_window::Window;
use logic::listener;
use types::{
    config::SerializablePoint,
    settings::{Capitalization, DisplayChoice},
};

#[derive(Clone, Debug)]
pub enum Message {
    Open(Box<dyn Window<NuhxBoard, Theme, Message>>),
    Close(Box<dyn Window<NuhxBoard, Theme, Message>>),
    Exit,
    Closed(window::Id),
    Listener(listener::Event),
    ReleaseScroll(u32),
    LoadStyle(usize),
    ChangeKeyboardCategory(String),
    LoadLayout(usize),
    ChangeSetting(Setting),
    ChangeStyle(StyleSetting),
    ClearPressedKeys,
    ToggleEditMode,
    MoveElement { index: usize, delta: Coord<f32> },
    SaveKeyboard(Option<std::path::PathBuf>),
    SaveStyle(Option<std::path::PathBuf>),
    SetHeight(f32),
    SetWidth(f32),
    PushChange(Change),
    Undo,
    Redo,
    ToggleSaveStyleAsGlobal,
    ChangeColor(ColorPicker, Color),
    ToggleColorPicker(ColorPicker),
    UpdateCanvas,
    ChangeTextInput(TextInputType, String),
    ChangeNumberInput(NumberInputType),
    ChangeSelection(usize, SelectionType, usize),
    SwapBoundaries(usize, usize, usize),
    UpdateHoveredElement(Option<usize>),
    ChangeElement(usize, ElementProperty),
    CenterTextPosition(usize),
    MakeRectangle(usize),
}

#[derive(Debug, Clone)]
pub enum SelectionType {
    Boundary,
    Keycode,
}

#[derive(Debug, Clone)]
pub enum ElementProperty {
    Text(String),
    ShiftText(String),
    TextPositionX(f32),
    TextPositionY(f32),
    FollowCaps,
    Boundary(usize, Option<SerializablePoint>),
    Keycode(usize, Option<u32>),
}

#[derive(Debug, Clone)]
pub enum StyleSetting {
    DefaultMouseSpeedIndicatorOutlineWidth(u32),
    DefaultLooseKeyFontFamily,
    DefaultLooseKeyShowOutline,
    DefaultLooseKeyOutlineWidth(u32),
    DefaultLooseKeyBackgroundImage,
    DefaultPressedKeyFontFamily,
    DefaultPressedKeyShowOutline,
    DefaultPressedKeyOutlineWidth(u32),
    DefaultPressedKeyBackgroundImage,
    KeyboardBackgroundImage,
}

// TODO: Are window resized undoable in NohBoard?
#[derive(Debug, Clone)]
pub enum Change {
    MoveElement {
        index: usize,
        delta: Coord<f32>,
        move_text: bool,
    },
}

#[derive(Debug, Clone)]
pub enum Setting {
    MouseSensitivity(f32),
    ScrollHoldTime(u64),
    CenterMouse,
    DisplayChoice(DisplayChoice),
    MinPressTime(u128),
    WindowTitle(String),
    Capitalization(Capitalization),
    FollowForCapsSensitive,
    FollowForCapsInsensitive,
    UpdateTextPosition,
}

impl Message {
    pub fn key_release(key: rdev::Key) -> Self {
        Message::Listener(listener::Event::KeyReceived(rdev::Event {
            event_type: rdev::EventType::KeyRelease(key),
            time: std::time::SystemTime::now(),
            name: None,
        }))
    }

    pub fn button_release(button: rdev::Button) -> Self {
        Message::Listener(listener::Event::KeyReceived(rdev::Event {
            event_type: rdev::EventType::ButtonRelease(button),
            time: std::time::SystemTime::now(),
            name: None,
        }))
    }

    pub fn none() -> Self {
        Message::Listener(listener::Event::None)
    }
}
