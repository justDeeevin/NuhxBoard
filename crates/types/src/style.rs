use bitflags::bitflags;
use schemars::JsonSchema;
use serde::{
    de::Deserializer,
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};
use std::{
    collections::{HashMap, HashSet},
    sync::{LazyLock, RwLock},
};
use tracing::debug;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct Style {
    /// Background color of the window. Will be overridden by background image if present.
    pub background_color: NohRgb,
    pub background_image_file_name: Option<String>,
    pub default_key_style: DefaultKeyStyle,
    pub default_mouse_speed_indicator_style: MouseSpeedIndicatorStyle,
    #[serde(with = "CustomMap")]
    pub element_styles: HashMap<u32, ElementStyle>,
}

// This allows `HashMap<u32, ElementStyle>` to be serialized as a list of `{Key: u32, Value: ElementStyle}`
struct CustomMap;
impl CustomMap {
    pub fn serialize<S>(map: &HashMap<u32, ElementStyle>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(map.len()))?;
        for (key, value) in map {
            seq.serialize_element(&KeyValue {
                key: *key,
                value: value.clone(),
            })?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<u32, ElementStyle>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<KeyValue>::deserialize(deserializer)?;
        let mut map = HashMap::new();
        for item in vec {
            map.insert(item.key, item.value);
        }
        Ok(map)
    }
}

impl JsonSchema for CustomMap {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "CustomMap".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        generator.subschema_for::<Vec<KeyValue>>()
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
struct KeyValue {
    key: u32,
    value: ElementStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct NohRgb {
    pub red: f32,
    pub green: f32,
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

impl From<NohRgb> for colorgrad::Color {
    fn from(value: NohRgb) -> Self {
        colorgrad::Color::new(
            value.red / 255.0,
            value.green / 255.0,
            value.blue / 255.0,
            1.0,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct DefaultKeyStyle {
    pub loose: KeySubStyle,
    pub pressed: KeySubStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct KeyStyle {
    pub loose: Option<KeySubStyle>,
    pub pressed: Option<KeySubStyle>,
}

impl From<DefaultKeyStyle> for KeyStyle {
    fn from(val: DefaultKeyStyle) -> Self {
        Self {
            loose: Some(val.loose),
            pressed: Some(val.pressed),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct KeySubStyle {
    pub background: NohRgb,
    pub text: NohRgb,
    pub outline: NohRgb,
    pub show_outline: bool,
    /// Outline thickness in pixels.
    pub outline_width: u32,
    pub font: Font,
    pub background_image_file_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct Font {
    pub font_family: String,
    /// Font size in pixels.
    pub size: f32,
    pub style: FontStyle,
}

impl From<FontStyle> for iced::font::Weight {
    fn from(val: FontStyle) -> Self {
        if val.contains(FontStyle::BOLD) {
            iced::font::Weight::Bold
        } else {
            iced::font::Weight::Normal
        }
    }
}

impl From<FontStyle> for iced::font::Style {
    fn from(val: FontStyle) -> Self {
        if val.contains(FontStyle::ITALIC) {
            iced::font::Style::Italic
        } else {
            iced::font::Style::Normal
        }
    }
}

// Iced requires font family names to have a static lifetime. This means that `String`s read from config
// must be leaked to be used. This data structure acts as a cache of fonts that have been used, so
// the names only need to be leaked once.
static FONTS: LazyLock<RwLock<HashSet<&'static str>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

impl Font {
    pub fn as_iced(&self) -> iced::Font {
        let name: &'static str = {
            let read = FONTS.read().unwrap();
            if let Some(name) = read.get(self.font_family.as_str()) {
                name
            } else {
                drop(read);

                let out = self.font_family.clone().leak();
                debug!(name = out, "Leaked font family");
                FONTS.write().unwrap().insert(out);
                out
            }
        };
        iced::Font {
            family: iced::font::Family::Name(name),
            weight: self.style.into(),
            style: self.style.into(),
            stretch: iced::font::Stretch::Normal,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FontStyle: u8 {
        const BOLD = 0b0001;
        const ITALIC = 0b0010;
        const UNDERLINE = 0b0100;
        const STRIKETHROUGH = 0b1000;
    }
}

impl Serialize for FontStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.bits())
    }
}

impl<'de> Deserialize<'de> for FontStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        FontStyle::from_bits(bits).ok_or_else(|| serde::de::Error::custom("Extraneous bits set"))
    }
}

impl JsonSchema for FontStyle {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FontStyle".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        generator.subschema_for::<u8>()
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MouseSpeedIndicatorStyle {
    pub inner_color: NohRgb,
    pub outer_color: NohRgb,
    pub outline_width: u32,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(tag = "__type")]
pub enum ElementStyle {
    KeyStyle(KeyStyle),
    MouseSpeedIndicatorStyle(MouseSpeedIndicatorStyle),
}

impl ElementStyle {
    pub fn as_key_style(&self) -> Option<&KeyStyle> {
        match self {
            ElementStyle::KeyStyle(key_style) => Some(key_style),
            _ => None,
        }
    }

    pub fn as_mouse_speed_indicator_style(&self) -> Option<&MouseSpeedIndicatorStyle> {
        match self {
            ElementStyle::MouseSpeedIndicatorStyle(mouse_speed_indicator_style) => {
                Some(mouse_speed_indicator_style)
            }
            _ => None,
        }
    }
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
            default_key_style: DefaultKeyStyle {
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
                    font: Font::default(),
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
                    font: Font::default(),
                    background_image_file_name: None,
                },
            },
            default_mouse_speed_indicator_style: MouseSpeedIndicatorStyle {
                inner_color: NohRgb::DEFAULT_GRAY,
                outer_color: NohRgb::WHITE,
                outline_width: 1,
            },
            element_styles: HashMap::new(),
        }
    }
}

impl Default for Font {
    fn default() -> Self {
        Self {
            font_family: "Courier New".into(),
            size: 10.0,
            style: FontStyle::empty(),
        }
    }
}
