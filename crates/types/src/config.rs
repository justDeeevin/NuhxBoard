use iced::Point;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct Layout {
    /// Doesn't do anything
    pub version: Option<u8>,
    /// Window width
    pub width: f32,
    /// Window height
    pub height: f32,
    pub elements: Vec<BoardElement>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "__type")]
/// Union for different element types
pub enum BoardElement {
    KeyboardKey(KeyboardKeyDefinition),
    MouseKey(MouseKeyDefinition),
    MouseScroll(MouseKeyDefinition),
    MouseSpeedIndicator(MouseSpeedIndicatorDefinition),
}

impl BoardElement {
    pub fn translate(&mut self, delta: geo::Coord<f32>, move_text: bool) {
        match self {
            BoardElement::MouseSpeedIndicator(key) => {
                key.location += delta;
            }
            _ => {
                let boundaries = match self {
                    BoardElement::KeyboardKey(key) => &mut key.boundaries,
                    BoardElement::MouseKey(key) => &mut key.boundaries,
                    BoardElement::MouseScroll(key) => &mut key.boundaries,
                    _ => unreachable!(),
                };
                for boundary in boundaries {
                    *boundary += delta;
                }
                if move_text {
                    let text_position = match self {
                        BoardElement::KeyboardKey(key) => &mut key.text_position,
                        BoardElement::MouseKey(key) => &mut key.text_position,
                        BoardElement::MouseScroll(key) => &mut key.text_position,
                        _ => unreachable!(),
                    };

                    *text_position += delta;
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct KeyboardKeyDefinition {
    pub id: u32,
    pub boundaries: Vec<SerializablePoint>,
    pub text_position: SerializablePoint,
    pub key_codes: Vec<u32>,
    pub text: String,
    pub shift_text: String,
    pub change_on_caps: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct MouseKeyDefinition {
    pub id: u32,
    pub boundaries: Vec<SerializablePoint>,
    pub text_position: SerializablePoint,
    pub key_codes: Vec<u32>,
    pub text: String,
}

pub struct CommonDefinition {
    pub id: u32,
    pub text_position: SerializablePoint,
    pub boundaries: Vec<SerializablePoint>,
    pub keycodes: Vec<u32>,
}

impl From<KeyboardKeyDefinition> for CommonDefinition {
    fn from(val: KeyboardKeyDefinition) -> Self {
        CommonDefinition {
            id: val.id,
            text_position: val.text_position,
            boundaries: val.boundaries,
            keycodes: val.key_codes,
        }
    }
}

impl From<MouseKeyDefinition> for CommonDefinition {
    fn from(val: MouseKeyDefinition) -> Self {
        CommonDefinition {
            id: val.id,
            text_position: val.text_position,
            boundaries: val.boundaries,
            keycodes: val.key_codes,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct MouseSpeedIndicatorDefinition {
    pub id: u32,
    pub location: SerializablePoint,
    pub radius: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct SerializablePoint {
    pub x: f32,
    pub y: f32,
}

impl From<SerializablePoint> for Point {
    fn from(point: SerializablePoint) -> Self {
        Point::new(point.x, point.y)
    }
}

impl From<SerializablePoint> for geo::Coord<f32> {
    fn from(value: SerializablePoint) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl std::ops::AddAssign<geo::Coord<f32>> for SerializablePoint {
    fn add_assign(&mut self, rhs: geo::Coord<f32>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
