use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Style {
    #[serde(rename = "BackgrounColor")]
    pub background_color: NohRgb,
    #[serde(rename = "BackgroundImageFileName")]
    pub background_image_file_name: String,
    #[serde(rename = "DefaultKeyStyle")]
    pub default_key_style: KeyStyle,
    #[serde(rename = "DefaultMouseIndicatorStyle")]
    pub default_mouse_indicator_style: MouseSpeedIndicatorStyle,
    #[serde(rename = "ElementStyles")]
    pub element_styles: Vec<ElementStyle>,
}

#[derive(Deserialize, Debug)]
pub struct NohRgb {
    #[serde(rename = "Red")]
    pub red: u8,
    #[serde(rename = "Green")]
    pub green: u8,
    #[serde(rename = "Blue")]
    pub blue: u8,
}

#[derive(Deserialize, Debug)]
pub struct KeyStyle {
    #[serde(rename = "Loose")]
    pub loose: KeySubStyle,
    #[serde(rename = "Pressed")]
    pub pressed: KeySubStyle,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Font {
    #[serde(rename = "FontFamily")]
    pub font_family: String,
    #[serde(rename = "Size")]
    pub size: f32,
    #[serde(rename = "Style")]
    pub style: FontStyle,
}

#[derive(Deserialize, Debug)]
pub enum FontStyle {
    Regular,
    Bold,
    Italic,
    Underline,
    Strikeout,
}

impl TryFrom<u8> for FontStyle {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FontStyle::Regular),
            1 => Ok(FontStyle::Bold),
            2 => Ok(FontStyle::Italic),
            4 => Ok(FontStyle::Underline),
            8 => Ok(FontStyle::Strikeout),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct MouseSpeedIndicatorStyle {
    #[serde(rename = "InnerColor")]
    pub inner_color: NohRgb,
    #[serde(rename = "OuterColor")]
    pub outer_color: NohRgb,
    #[serde(rename = "OutlineWidth")]
    pub outline_width: u32,
}

#[derive(Deserialize, Debug)]
pub struct ElementStyle {
    #[serde(rename = "Key")]
    pub key: u32,
    #[serde(rename = "Value")]
    pub value: ElementStyleUnion,
}

#[derive(Deserialize, Debug)]
pub enum ElementStyleUnion {
    KeyStyle(KeyStyle),
    MouseSpeedIndicatorStyle(MouseSpeedIndicatorStyle),
}
