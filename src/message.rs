use crate::{nuhxboard::*, types::*};
use geo::Coord;
use iced::{window, Color, Theme};
use iced_multi_window::Window;
use nuhxboard_types::{
    config::SerializablePoint,
    settings::{Capitalization, DisplayChoice},
};
use rdev::Event;

#[derive(Clone, Debug)]
pub enum Message {
    Open(Box<dyn Window<NuhxBoard, Theme, Message>>),
    CloseAllOf(Box<dyn Window<NuhxBoard, Theme, Message>>),
    Exit,
    Closed(window::Id),
    Listener(Event),
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
    StartDetecting(usize),
    None,
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
    MouseSpeedIndicatorPositionX(f32),
    MouseSpeedIndicatorPositionY(f32),
    MouseSpeedIndicatorRadius(f32),
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
    LooseKeyFontFamily(u32),
    LooseKeyShowOutline(u32),
    LooseKeyOutlineWidth { id: u32, width: u32 },
    LooseKeyBackgroundImage(u32),
    LooseKeyBold(u32),
    LooseKeyItalic(u32),
    LooseKeyUnderline(u32),
    LooseKeyStrikethrough(u32),
    PressedKeyFontFamily(u32),
    PressedKeyShowOutline(u32),
    PressedKeyOutlineWidth { id: u32, width: u32 },
    PressedKeyBackgroundImage(u32),
    PressedKeyBold(u32),
    PressedKeyItalic(u32),
    PressedKeyUnderline(u32),
    PressedKeyStrikethrough(u32),
    MouseSpeedIndicatorOutlineWidth { id: u32, width: u32 },
}

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
    MinPressTime(u64),
    WindowTitle(String),
    Capitalization(Capitalization),
    FollowForCapsSensitive,
    FollowForCapsInsensitive,
    UpdateTextPosition,
}

impl Message {
    pub fn key_release(key: rdev::Key) -> Self {
        Message::Listener(rdev::Event {
            event_type: rdev::EventType::KeyRelease(key),
            time: std::time::SystemTime::now(),
            unicode: None,
            platform_code: 0,
            position_code: 0,
            usb_hid: 0,
        })
    }

    pub fn button_release(button: rdev::Button) -> Self {
        Message::Listener(rdev::Event {
            event_type: rdev::EventType::ButtonRelease(button),
            time: std::time::SystemTime::now(),
            unicode: None,
            platform_code: 0,
            position_code: 0,
            usb_hid: 0,
        })
    }
}
