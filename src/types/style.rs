use serde::Deserialize;

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
pub struct KeyStyle {
    #[serde(rename = "Loose")]
    pub loose: KeySubStyle,
    #[serde(rename = "Pressed")]
    pub pressed: KeySubStyle,
}

#[derive(Deserialize, Debug, Clone)]
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
    pub background_image_file_name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Font {
    #[serde(rename = "FontFamily")]
    pub font_family: String,
    #[serde(rename = "Size")]
    pub size: f32,
    #[serde(rename = "Style")]
    pub style: u8,
}

#[derive(Deserialize, Debug)]
pub struct MouseSpeedIndicatorStyle {
    #[serde(rename = "InnerColor")]
    pub inner_color: NohRgb,
    #[serde(rename = "OuterColor")]
    pub outer_color: NohRgb,
    #[serde(rename = "OutlineWidth")]
    pub outline_width: f32,
}

#[derive(Deserialize, Debug)]
pub struct ElementStyle {
    #[serde(rename = "Key")]
    pub key: u32,
    #[serde(rename = "Value")]
    pub value: ElementStyleUnion,
}

#[derive(Deserialize, Debug)]
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
                    outline: NohRgb::BLACK,
                    show_outline: false,
                    outline_width: 0,
                    font: Font {
                        font_family: "Consolas".into(),
                        size: 15.0,
                        style: 0,
                    },
                    background_image_file_name: "".into(),
                },
                pressed: KeySubStyle {
                    background: NohRgb::WHITE,
                    text: NohRgb::BLACK,
                    outline: NohRgb::BLACK,
                    show_outline: false,
                    outline_width: 0,
                    font: Font {
                        font_family: "Consolas".into(),
                        size: 15.0,
                        style: 0,
                    },
                    background_image_file_name: "".into(),
                },
            },
            default_mouse_speed_indicator_style: MouseSpeedIndicatorStyle {
                inner_color: NohRgb::DEFAULT_GRAY,
                outer_color: NohRgb::DEFAULT_GRAY,
                outline_width: 1.0,
            },
            element_styles: vec![],
        }
    }
}
