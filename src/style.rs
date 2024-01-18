use serde::Deserialize;

#[derive(Default, Deserialize, Debug)]
pub struct Style {
    #[serde(rename = "BackgroundColor")]
    pub background_color: NohRgb,
    #[serde(rename = "BackgroundImageFileName")]
    pub background_image_file_name: Option<String>,
    #[serde(rename = "DefaultKeyStyle")]
    pub default_key_style: KeyStyle,
    #[serde(rename = "DefaultMouseIndicatorStyle")]
    pub default_mouse_indicator_style: Option<MouseSpeedIndicatorStyle>,
    #[serde(rename = "ElementStyles")]
    pub element_styles: Vec<ElementStyle>,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct NohRgb {
    #[serde(rename = "Red")]
    pub red: u8,
    #[serde(rename = "Green")]
    pub green: u8,
    #[serde(rename = "Blue")]
    pub blue: u8,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct KeyStyle {
    #[serde(rename = "Loose")]
    pub loose: KeySubStyle,
    #[serde(rename = "Pressed")]
    pub pressed: KeySubStyle,
}

#[derive(Default, Deserialize, Debug, Clone)]
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

#[derive(Default, Deserialize, Debug, Clone)]
pub struct Font {
    #[serde(rename = "FontFamily")]
    pub font_family: String,
    #[serde(rename = "Size")]
    pub size: f32,
    #[serde(rename = "Style")]
    pub style: u8,
}

#[derive(Default, Deserialize, Debug)]
pub struct MouseSpeedIndicatorStyle {
    #[serde(rename = "InnerColor")]
    pub inner_color: NohRgb,
    #[serde(rename = "OuterColor")]
    pub outer_color: NohRgb,
    #[serde(rename = "OutlineWidth")]
    pub outline_width: u32,
}

#[derive(Default, Deserialize, Debug)]
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

impl Default for ElementStyleUnion {
    fn default() -> Self {
        ElementStyleUnion::KeyStyle(KeyStyle::default())
    }
}
