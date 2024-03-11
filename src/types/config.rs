use iced::Point;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    #[serde(rename = "Version")]
    pub version: u8,
    #[serde(rename = "Width")]
    pub width: f32,
    #[serde(rename = "Height")]
    pub height: f32,
    #[serde(rename = "Elements")]
    pub elements: Vec<BoardElement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "__type")]
pub enum BoardElement {
    KeyboardKey(KeyboardKeyDefinition),
    MouseKey(MouseKeyDefinition),
    MouseScroll(MouseScrollDefinition),
    MouseSpeedIndicator(MouseSpeedIndicatorDefinition),
}

impl BoardElement {
    pub fn translate(&mut self, delta: geo::Coord) {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseScrollDefinition {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseSpeedIndicatorDefinition {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Location")]
    pub location: SerializablePoint,
    #[serde(rename = "Radius")]
    pub radius: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl From<SerializablePoint> for geo::Coord {
    fn from(value: SerializablePoint) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
        }
    }
}

impl std::ops::AddAssign<geo::Coord> for SerializablePoint {
    fn add_assign(&mut self, rhs: geo::Coord) {
        self.x += rhs.x as f32;
        self.y += rhs.y as f32;
    }
}
