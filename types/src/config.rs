use iced::Point;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, JsonSchema)]
pub struct Layout {
    #[serde(rename = "Version")]
    pub version: Option<u8>,
    #[serde(rename = "Width")]
    pub width: f32,
    #[serde(rename = "Height")]
    pub height: f32,
    #[serde(rename = "Elements")]
    pub elements: Vec<BoardElement>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "__type")]
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
                    _ => return,
                };
                for boundary in boundaries {
                    *boundary += delta;
                }
                if move_text {
                    let text_position = match self {
                        BoardElement::KeyboardKey(key) => &mut key.text_position,
                        BoardElement::MouseKey(key) => &mut key.text_position,
                        BoardElement::MouseScroll(key) => &mut key.text_position,
                        _ => return,
                    };

                    *text_position += delta;
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct KeyboardKeyDefinition {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Boundaries")]
    pub boundaries: Vec<SerializablePoint>,
    #[serde(rename = "TextPosition")]
    pub text_position: SerializablePoint,
    #[serde(rename = "KeyCodes")]
    pub keycodes: Vec<u32>,
    #[serde(rename = "Text")]
    pub text: String,
    #[serde(rename = "ShiftText")]
    pub shift_text: String,
    #[serde(rename = "ChangeOnCaps")]
    pub change_on_caps: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct MouseKeyDefinition {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Boundaries")]
    pub boundaries: Vec<SerializablePoint>,
    #[serde(rename = "TextPosition")]
    pub text_position: SerializablePoint,
    #[serde(rename = "KeyCodes")]
    pub keycodes: Vec<u32>,
    #[serde(rename = "Text")]
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
            keycodes: val.keycodes,
        }
    }
}

impl From<MouseKeyDefinition> for CommonDefinition {
    fn from(val: MouseKeyDefinition) -> Self {
        CommonDefinition {
            id: val.id,
            text_position: val.text_position,
            boundaries: val.boundaries,
            keycodes: val.keycodes,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct MouseSpeedIndicatorDefinition {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Location")]
    pub location: SerializablePoint,
    #[serde(rename = "Radius")]
    pub radius: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct SerializablePoint {
    #[serde(rename = "X")]
    pub x: f32,
    #[serde(rename = "Y")]
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
