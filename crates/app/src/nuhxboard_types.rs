use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct SelectionLists {
    pub boundary: HashMap<usize, usize>,
    pub keycode: HashMap<usize, usize>,
}

#[derive(Default, Clone)]
pub struct NumberInput {
    pub boundary_x: HashMap<usize, f32>,
    pub boundary_y: HashMap<usize, f32>,
    pub keycode: HashMap<usize, u32>,
    pub rectangle_position_x: HashMap<usize, f32>,
    pub rectangle_position_y: HashMap<usize, f32>,
    pub rectangle_size_x: HashMap<usize, f32>,
    pub rectangle_size_y: HashMap<usize, f32>,
}

#[derive(Debug, Clone)]
pub enum NumberInputType {
    BoundaryX(usize, f32),
    BoundaryY(usize, f32),
    Keycode(usize, u32),
    RectanglePositionX(usize, f32),
    RectanglePositionY(usize, f32),
    RectangleSizeX(usize, f32),
    RectangleSizeY(usize, f32),
}

#[derive(Default)]
pub struct TextInput {
    pub keyboard_background_image: String,
    pub save_keyboard_as_category: String,
    pub save_keyboard_as_name: String,
    pub save_style_as_name: String,
    pub default_loose_key_background_image: String,
    pub default_loose_key_font_family: String,
    pub default_pressed_key_background_image: String,
    pub default_pressed_key_font_family: String,
}

#[derive(Clone, Debug)]
pub enum TextInputType {
    KeyboardBackgroundImage,
    SaveKeyboardAsCategory,
    SaveKeyboardAsName,
    SaveStyleAsName,
    DefaultLooseKeyBackgroundImage,
    DefaultLooseKeyFontFamily,
    DefaultPressedKeyBackgroundImage,
    DefaultPressedKeyFontFamily,
}

#[derive(Default)]
pub struct ColorPickers {
    pub keyboard_background: bool,
    pub default_mouse_speed_indicator_1: bool,
    pub default_mouse_speed_indicator_2: bool,
    pub default_loose_background: bool,
    pub default_loose_text: bool,
    pub default_loose_outline: bool,
    pub default_pressed_background: bool,
    pub default_pressed_text: bool,
    pub default_pressed_outline: bool,
    pub loose_background: HashMap<u32, bool>,
    pub loose_text: HashMap<u32, bool>,
    pub loose_outline: HashMap<u32, bool>,
    pub pressed_background: HashMap<u32, bool>,
    pub pressed_text: HashMap<u32, bool>,
    pub pressed_outline: HashMap<u32, bool>,
}

impl ColorPickers {
    pub fn get_mut(&mut self, picker: ColorPicker, id: u32) -> &mut bool {
        match picker {
            ColorPicker::KeyboardBackground => &mut self.keyboard_background,
            ColorPicker::DefaultMouseSpeedIndicator1 => &mut self.default_mouse_speed_indicator_1,
            ColorPicker::DefaultMouseSpeedIndicator2 => &mut self.default_mouse_speed_indicator_2,
            ColorPicker::DefaultLooseBackground => &mut self.default_loose_background,
            ColorPicker::DefaultLooseText => &mut self.default_loose_text,
            ColorPicker::DefaultLooseOutline => &mut self.default_loose_outline,
            ColorPicker::DefaultPressedBackground => &mut self.default_pressed_background,
            ColorPicker::DefaultPressedText => &mut self.default_pressed_text,
            ColorPicker::DefaultPressedOutline => &mut self.default_pressed_outline,
            ColorPicker::LooseBackground => self.loose_background.entry(id).or_insert(false),
            ColorPicker::LooseText => self.loose_text.entry(id).or_insert(false),
            ColorPicker::LooseOutline => self.loose_outline.entry(id).or_insert(false),
            ColorPicker::PressedBackground => self.pressed_background.entry(id).or_insert(false),
            ColorPicker::PressedText => self.pressed_text.entry(id).or_insert(false),
            ColorPicker::PressedOutline => self.pressed_outline.entry(id).or_insert(false),
        }
    }

    pub fn toggle(&mut self, picker: ColorPicker, id: u32) {
        let picker = self.get_mut(picker, id);
        *picker = !*picker;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColorPicker {
    KeyboardBackground,
    DefaultMouseSpeedIndicator1,
    DefaultMouseSpeedIndicator2,
    DefaultLooseBackground,
    DefaultLooseText,
    DefaultLooseOutline,
    DefaultPressedBackground,
    DefaultPressedText,
    DefaultPressedOutline,
    LooseBackground,
    LooseText,
    LooseOutline,
    PressedBackground,
    PressedText,
    PressedOutline,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum StyleChoice {
    Default,
    Global(String),
    Custom(String),
}

impl PartialOrd for StyleChoice {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.to_string().cmp(&other.to_string()))
    }
}

impl Ord for StyleChoice {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl StyleChoice {
    pub fn is_global(&self) -> bool {
        matches!(self, StyleChoice::Global(_))
    }

    pub fn name(&self) -> String {
        match self {
            Self::Global(name) => name.clone(),
            _ => self.to_string(),
        }
    }
}

impl std::fmt::Display for StyleChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StyleChoice::Default => write!(f, "Global Default"),
            StyleChoice::Custom(s) => write!(f, "{}", s),
            StyleChoice::Global(s) => write!(f, "Global: {}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ConfigOpen(String),
    ConfigParse(String),
    StyleOpen(String),
    StyleParse(String),
    UnknownKey(rdev::Key),
    UnknownButton(rdev::Button),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseKey {
    Left,
    Middle,
    Right,
    Forward,
    Back,
}

impl std::fmt::Display for MouseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseKey::Left => write!(f, "Left"),
            MouseKey::Middle => write!(f, "Middle"),
            MouseKey::Right => write!(f, "Right"),
            MouseKey::Forward => write!(f, "Forward"),
            MouseKey::Back => write!(f, "Back"),
        }
    }
}

impl From<MouseKey> for u32 {
    fn from(key: MouseKey) -> Self {
        match key {
            MouseKey::Left => 0,
            MouseKey::Middle => 2,
            MouseKey::Right => 1,
            MouseKey::Forward => 4,
            MouseKey::Back => 3,
        }
    }
}

impl TryFrom<u32> for MouseKey {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, ()> {
        match value {
            0 => Ok(MouseKey::Left),
            2 => Ok(MouseKey::Middle),
            1 => Ok(MouseKey::Right),
            4 => Ok(MouseKey::Forward),
            3 => Ok(MouseKey::Back),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseScroll {
    Up,
    Down,
    Left,
    Right,
}

impl std::fmt::Display for MouseScroll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseScroll::Up => write!(f, "Up"),
            MouseScroll::Down => write!(f, "Down"),
            MouseScroll::Left => write!(f, "Left"),
            MouseScroll::Right => write!(f, "Right"),
        }
    }
}

impl From<MouseScroll> for u32 {
    fn from(key: MouseScroll) -> Self {
        match key {
            MouseScroll::Up => 0,
            MouseScroll::Down => 1,
            MouseScroll::Left => 3,
            MouseScroll::Right => 2,
        }
    }
}

impl TryFrom<u32> for MouseScroll {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, ()> {
        match value {
            0 => Ok(MouseScroll::Up),
            1 => Ok(MouseScroll::Down),
            3 => Ok(MouseScroll::Left),
            2 => Ok(MouseScroll::Right),
            _ => Err(()),
        }
    }
}
