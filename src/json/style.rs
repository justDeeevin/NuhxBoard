use crate::json::PathSegment;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, SmartDefault)]
#[serde(rename_all = "PascalCase")]
pub struct Style {
    /// The background color of the window.
    ///
    /// This will be overriden by the [background image](Self::background_image_file_name) if present.
    #[default(NohRgb { blue: 100.0, ..NohRgb::BLACK })]
    pub background_color: NohRgb,
    pub background_image_file_name: Option<PathSegment>,
    pub default_key_style: DefaultKeyStyle,
    pub default_mouse_speed_indicator_style: MouseSpeedIndicatorStyle,
    #[serde(with = "custom_map")]
    pub element_styles: HashMap<u32, ElementStyle>,
}

mod custom_map {
    use super::ElementStyle;
    use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct KeyValue<K, V> {
        key: K,
        value: V,
    }

    pub fn serialize<S: Serializer>(
        map: &HashMap<u32, ElementStyle>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(map.len()))?;
        for (key, value) in map {
            seq.serialize_element(&KeyValue { key, value })?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<u32, ElementStyle>, D::Error> {
        Ok(
            Vec::<KeyValue<u32, ElementStyle>>::deserialize(deserializer)?
                .into_iter()
                .map(|entry| (entry.key, entry.value))
                .collect(),
        )
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NohRgb {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl NohRgb {
    pub const BLACK: Self = Self {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };

    pub const RED: Self = Self {
        red: Self::WHITE.red,
        ..Self::BLACK
    };

    pub const GREEN: Self = Self {
        green: Self::WHITE.green,
        ..Self::BLACK
    };

    pub const BLUE: Self = Self {
        blue: Self::WHITE.blue,
        ..Self::BLACK
    };

    pub const WHITE: Self = Self {
        red: 255.0,
        green: 255.0,
        blue: 255.0,
    };

    pub const DEFAULT_GRAY: Self = Self {
        red: 100.0,
        green: 100.0,
        blue: 100.0,
    };
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DefaultKeyStyle {
    /// The style to show when an element is not highlighted.
    pub loose: KeySubStyle,
    /// The style to show when an element is highlighted.
    pub pressed: KeySubStyle,
}

impl Default for DefaultKeyStyle {
    fn default() -> Self {
        Self {
            loose: KeySubStyle {
                background: NohRgb::DEFAULT_GRAY,
                text: NohRgb::BLACK,
                outline: NohRgb::GREEN,
                show_outline: false,
                outline_width: 1,
                font: Font::default(),
                background_image_file_name: None,
            },
            pressed: KeySubStyle {
                background: NohRgb::WHITE,
                text: NohRgb::BLACK,
                outline: NohRgb::GREEN,
                show_outline: false,
                outline_width: 1,
                font: Font::default(),
                background_image_file_name: None,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyStyle {
    /// If `None`, uses the default loose style.
    pub loose: Option<KeySubStyle>,
    /// If `None`, uses the default pressed style.
    pub pressed: Option<KeySubStyle>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeySubStyle {
    pub background: NohRgb,
    pub text: NohRgb,
    pub outline: NohRgb,
    pub show_outline: bool,
    /// The outline's width in pixels.
    outline_width: u32,
    pub font: Font,
    pub background_image_file_name: Option<PathSegment>,
}

#[derive(Serialize, Deserialize, SmartDefault)]
#[serde(rename_all = "PascalCase")]
pub struct Font {
    #[default = "Courier New"]
    pub font_family: String,
    #[default = 10.0]
    /// The font size in pixels.
    pub size: f32,
    pub style: FontStyle,
}

#[derive(Serialize, Deserialize, Default)]
pub struct FontStyle(u8);

bitflags::bitflags! {
    impl FontStyle: u8 {
        const BOLD = 1 << 0;
        const ITALIC = 1 << 1;
        const UNDERLINE = 1 << 2;
        const STRIKETHROUGH = 1 << 3;
    }
}

#[derive(Serialize, Deserialize, SmartDefault)]
#[serde(rename_all = "PascalCase")]
pub struct MouseSpeedIndicatorStyle {
    #[default(NohRgb::DEFAULT_GRAY)]
    /// The starting color of the gradient.
    pub inner_color: NohRgb,
    #[default(NohRgb::WHITE)]
    /// The ending color of the gradient.
    ///
    /// This is only shown at the maximum possible speed.
    // TODO: check above
    pub outer_color: NohRgb,
    #[default = 1]
    /// The outline's width in pixels.
    pub outline_width: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "__type")]
pub enum ElementStyle {
    KeyStyle(KeyStyle),
    MouseSpeedIndicatorStyle(MouseSpeedIndicatorStyle),
}
