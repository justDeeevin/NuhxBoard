#[derive(derivative::Derivative, serde::Serialize, serde::Deserialize)]
#[derivative(Default)]
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
    #[derivative(Default(value = "100"))]
    pub scroll_hold_time: u64,
    pub window_title: String,
    pub display_id: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Capitalization {
    Lower,
    Upper,
    #[default]
    Follow,
}
