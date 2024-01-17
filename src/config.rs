use iced::Point;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Width")]
    pub width: u32,
    #[serde(rename = "Height")]
    pub height: u32,
    #[serde(rename = "Elements")]
    pub elements: Vec<BoardElement>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "__type")]
pub enum BoardElement {
    KeyboardKey(KeyboardKeyDefinition),
    MouseKey(MouseKeyDefinition),
    MouseScroll(MouseScrollDefinition),
    MouseSpeedIndicator(MouseSpeedIndicatorDefinition),
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct MouseSpeedIndicatorDefinition {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Location")]
    pub location: SerializablePoint,
    #[serde(rename = "Radius")]
    pub radius: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SerializablePoint {
    #[serde(rename = "X")]
    pub x: u32,
    #[serde(rename = "Y")]
    pub y: u32,
}

impl From<SerializablePoint> for Point {
    fn from(point: SerializablePoint) -> Self {
        Point::new(point.x as f32, point.y as f32)
    }
}
