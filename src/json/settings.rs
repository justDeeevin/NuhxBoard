use std::num::{ParseFloatError, ParseIntError};

use crate::{SCapitalization, UserSettings};

use super::PathSegment;
#[cfg(doc)]
use super::layout::ElementKind;
use derive_more::Display;
use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};
use slint::ToSharedString;
use smart_default::SmartDefault;

#[derive(Serialize, Deserialize, SmartDefault)]
pub struct Settings {
    pub capitalization: Capitalization,
    /// When [`capitalization`](Self::capitalization) is not [`Follow`](Capitalization::Follow),
    /// whether keys with [`change_on_caps`](ElementKind::KeyboardKey::change_on_caps) set to true should respond to the state of the shift key.
    pub follow_for_caps_sensitive: bool,
    /// When [`capitalization`](Self::capitalization) is not [`Follow`](Capitalization::Follow),
    /// whether keys with [`change_on_caps`](ElementKind::KeyboardKey::change_on_caps) set to false should respond to the state of the shift key.
    pub follow_for_caps_insensitive: bool,
    /// The name of the previously-opened category.
    pub category: Option<PathSegment>,
    /// The name of the previously-opened keyboard layout.
    pub layout: Option<PathSegment>,
    /// The name of the previously-opened style.
    pub style: Option<PathSegment>,
    /// Whether the previously-opened style was a global.
    pub global_style: bool,
    /// Whether to treat the cursor's distance from the center of the screen as its velocity.
    pub mouse_from_center: bool,
    #[default = 50.0]
    pub mouse_sensitivity: f32,
    #[default = 0]
    /// The minimum number of milliseconds for which a pressed key should be highlighted.
    pub min_press_time: u64,
    #[default = 100]
    /// The number of milliseconds for which [`MouseScroll`](ElementKind::MouseScroll) elements
    /// should be highlighted following a scroll event.
    pub scroll_hold_time: u64,
    #[serde(default)]
    pub window_title: String,
    #[default(get_default_display())]
    pub display_id: Option<u32>,
    #[default = true]
    /// Whether to update the text position of an element when it is moved in the graphical editor.
    pub update_text_position: bool,
}

impl From<&Settings> for UserSettings {
    fn from(value: &Settings) -> Self {
        Self {
            capitalization: value.capitalization.into(),
            follow_for_caps_sensitive: value.follow_for_caps_sensitive,
            follow_for_caps_insensitive: value.follow_for_caps_insensitive,
            mouse_from_center: value.mouse_from_center,
            mouse_sensitivity: value.mouse_sensitivity.to_shared_string(),
            min_press_time: value.min_press_time.to_shared_string(),
            scroll_hold_time: value.scroll_hold_time.to_shared_string(),
            window_title: value.window_title.to_shared_string(),
        }
    }
}

#[derive(Debug, Display)]
pub enum ApplyError {
    #[display("Failed to parse integer: {_0}")]
    Int(ParseIntError),
    #[display("Failed to parse   float: {_0}")]
    Float(ParseFloatError),
}

impl From<ParseFloatError> for ApplyError {
    fn from(value: ParseFloatError) -> Self {
        Self::Float(value)
    }
}

impl From<ParseIntError> for ApplyError {
    fn from(value: ParseIntError) -> Self {
        Self::Int(value)
    }
}

impl Settings {
    pub fn apply(&mut self, settings: UserSettings) -> Result<(), Vec<ApplyError>> {
        let mut out: Result<_, Vec<ApplyError>> = Ok(());

        macro_rules! mytry {
            ($left:expr => $right:expr) => {
                match $right {
                    Ok(v) => $left = v,
                    Err(e) => match &mut out {
                        Ok(()) => out = Err(vec![e.into()]),
                        Err(es) => es.push(e.into()),
                    },
                }
            };
        }

        self.capitalization = settings.capitalization.into();
        self.follow_for_caps_sensitive = settings.follow_for_caps_sensitive;
        self.follow_for_caps_insensitive = settings.follow_for_caps_insensitive;
        self.mouse_from_center = settings.mouse_from_center;
        mytry!(self.mouse_sensitivity => settings.mouse_sensitivity.parse());
        mytry!(self.min_press_time => settings.min_press_time.parse());
        mytry!(self.scroll_hold_time => settings.scroll_hold_time.parse());
        self.window_title = settings.window_title.into();

        out
    }
}

fn get_default_display() -> Option<u32> {
    DisplayInfo::all().ok()?.into_iter().find_map(|display| {
        if display.is_primary {
            Some(display.id)
        } else {
            None
        }
    })
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
/// How [`KeyboardKey`](ElementKind::KeyboardKey)s should
/// respond to caps lock state.
///
/// For any value other than [`Follow`](Self::Follow), keys will also ignore the state of the shift
/// key by default unless [`follow_for_caps_sensitive`](Settings::follow_for_caps_sensitive) and/or
/// [`follow_for_caps_insensitive`](Settings::follow_for_caps_insensitive) are enabled.
pub enum Capitalization {
    /// Keys will assume caps lock is always disabled.
    Lower,
    /// Keys will assume caps lock is always enabled.
    Upper,
    #[default]
    /// Keys will follow caps lock state.
    Follow,
}

impl From<Capitalization> for SCapitalization {
    fn from(value: Capitalization) -> Self {
        match value {
            Capitalization::Lower => Self::Lower,
            Capitalization::Upper => Self::Upper,
            Capitalization::Follow => Self::Follow,
        }
    }
}

impl From<SCapitalization> for Capitalization {
    fn from(value: SCapitalization) -> Self {
        match value {
            SCapitalization::Lower => Self::Lower,
            SCapitalization::Upper => Self::Upper,
            SCapitalization::Follow => Self::Follow,
        }
    }
}
