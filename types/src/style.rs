use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Style {
    #[serde(rename = "BackgroundColor")]
    pub background_color: NohRgb,
    #[serde(rename = "BackgroundImageFileName")]
    pub background_image_file_name: Option<String>,
    #[serde(rename = "DefaultKeyStyle")]
    pub default_key_style: KeyStyle,
    #[serde(rename = "DefaultMouseSpeedIndicatorStyle")]
    pub default_mouse_speed_indicator_style: MouseSpeedIndicatorStyle,
    #[serde(rename = "ElementStyles")]
    pub element_styles: Vec<ElementStyle>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
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
pub struct KeyStyle {
    #[serde(rename = "Loose")]
    pub loose: KeySubStyle,
    #[serde(rename = "Pressed")]
    pub pressed: KeySubStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct KeySubStyle {
    #[serde(rename = "Background")]
    pub background: NohRgb,
    #[serde(rename = "Text")]
    pub text: NohRgb,
    #[serde(rename = "Outline")]
    pub outline: NohRgb,
    #[serde(rename = "ShowOutline")]
    pub show_outline: bool,
    #[serde(rename = "OutlineWidth")]
    pub outline_width: u32,
    #[serde(rename = "Font")]
    pub font: Font,
    #[serde(rename = "BackgroundImageFileName")]
    pub background_image_file_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct Font {
    #[serde(rename = "FontFamily")]
    pub font_family: String,
    #[serde(rename = "Size")]
    pub size: f32,
    #[serde(rename = "Style")]
    pub style: u8,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct MouseSpeedIndicatorStyle {
    #[serde(rename = "InnerColor")]
    pub inner_color: NohRgb,
    #[serde(rename = "OuterColor")]
    pub outer_color: NohRgb,
    #[serde(rename = "OutlineWidth")]
    pub outline_width: f32,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct ElementStyle {
    #[serde(rename = "Key")]
    pub key: u32,
    #[serde(rename = "Value")]
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
