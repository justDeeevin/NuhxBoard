use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct Style {
    pub background_color: NohRgb,
    pub background_image_file_name: Option<String>,
    pub default_key_style: KeyStyle,
    pub default_mouse_speed_indicator_style: MouseSpeedIndicatorStyle,
    pub element_styles: Vec<ElementStyle>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
pub struct NohRgb {
    #[serde(rename = "Red")]
    pub red: f32,
    #[serde(rename = "Green")]
    pub green: f32,
    #[serde(rename = "Blue")]
    pub blue: f32,
}

impl NohRgb {
    pub const BLACK: NohRgb = NohRgb {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };

    pub const WHITE: NohRgb = NohRgb {
        red: 255.0,
        green: 255.0,
        blue: 255.0,
    };

    pub const DEFAULT_GRAY: NohRgb = NohRgb {
        red: 100.0,
        green: 100.0,
        blue: 100.0,
    };
}

impl From<NohRgb> for iced::Color {
    fn from(val: NohRgb) -> Self {
        iced::Color::from_rgba(val.red / 255.0, val.green / 255.0, val.blue / 255.0, 1.0)
    }
}

impl From<iced::Color> for NohRgb {
    fn from(val: iced::Color) -> Self {
        NohRgb {
            red: val.r * 255.0,
            green: val.g * 255.0,
            blue: val.b * 255.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct KeyStyle {
    pub loose: KeySubStyle,
    pub pressed: KeySubStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct KeySubStyle {
    pub background: NohRgb,
    pub text: NohRgb,
    pub outline: NohRgb,
    pub show_outline: bool,
    pub outline_width: u32,
    pub font: Font,
    pub background_image_file_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct Font {
    pub font_family: String,
    pub size: f32,
    pub style: u8,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct MouseSpeedIndicatorStyle {
    pub inner_color: NohRgb,
    pub outer_color: NohRgb,
    pub outline_width: f32,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct ElementStyle {
    pub key: u32,
    pub value: ElementStyleUnion,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(tag = "__type")]
pub enum ElementStyleUnion {
    KeyStyle(KeyStyle),
    MouseSpeedIndicatorStyle(MouseSpeedIndicatorStyle),
}

impl Default for Style {
    fn default() -> Self {
        Style {
            background_color: NohRgb {
                red: 0.0,
                green: 0.0,
                blue: 100.0,
            },
            background_image_file_name: None,
            default_key_style: KeyStyle {
                loose: KeySubStyle {
                    background: NohRgb::DEFAULT_GRAY,
                    text: NohRgb::BLACK,
                    outline: NohRgb {
                        red: 0.0,
                        green: 255.0,
                        blue: 0.0,
                    },
                    show_outline: false,
                    outline_width: 1,
                    font: Font {
                        font_family: "Courier New".into(),
                        size: 10.0,
                        style: 0,
                    },
                    background_image_file_name: None,
                },
                pressed: KeySubStyle {
                    background: NohRgb::WHITE,
                    text: NohRgb::BLACK,
                    outline: NohRgb {
                        red: 0.0,
                        green: 255.0,
                        blue: 0.0,
                    },
                    show_outline: false,
                    outline_width: 1,
                    font: Font {
                        font_family: "Courier New".into(),
                        size: 10.0,
                        style: 0,
                    },
                    background_image_file_name: None,
                },
            },
            default_mouse_speed_indicator_style: MouseSpeedIndicatorStyle {
                inner_color: NohRgb::DEFAULT_GRAY,
                outer_color: NohRgb::WHITE,
                outline_width: 1.0,
            },
            element_styles: vec![],
        }
    }
}
