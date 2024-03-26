use display_info::DisplayInfo;

use crate::ui::app::DisplayChoice;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub capitalization: Capitalization,
    pub follow_for_caps_sensitive: bool,
    pub follow_for_caps_insensitive: bool,
    pub category: String,
    pub keyboard: usize,
    pub style: usize,
    pub mouse_from_center: bool,
    pub mouse_sensitivity: f32,
    pub min_press_time: u128,
    pub scroll_hold_time: u64,
    pub window_title: String,
    pub display_choice: DisplayChoice,
    pub auto_desktop_entry: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let displays = DisplayInfo::all().unwrap();

        let mut display_id = 0;

        for display in displays {
            if display.is_primary {
                display_id = display.id;
            }
        }

        Self {
            capitalization: Capitalization::Follow,
            follow_for_caps_sensitive: false,
            follow_for_caps_insensitive: false,
            category: String::new(),
            keyboard: 0,
            style: 0,
            mouse_from_center: false,
            mouse_sensitivity: 50.0,
            min_press_time: 0,
            scroll_hold_time: 100,
            window_title: "NuhxBoard".into(),
            display_choice: DisplayChoice {
                id: display_id,
                primary: true,
            },
            auto_desktop_entry: true,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Capitalization {
    Lower,
    Upper,
    Follow,
}
