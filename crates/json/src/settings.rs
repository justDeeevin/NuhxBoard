#[cfg(doc)]
use super::layout::ElementKind;
use crate::PathSegment;
use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};
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
