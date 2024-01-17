use iced::Point;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    pub version: String,
    pub width: u32,
    pub height: u32,
    pub elements: Vec<BoardElement>,
}

#[derive(Deserialize, Debug)]
pub enum BoardElement {
    KeyboardKey(KeyboardKeyDefinition),
    MouseKey(MouseKeyDefinition),
    MouseScroll(MouseScrollDefinition),
    MouseSpeedIndicator(MouseSpeedIndicatorDefinition),
}

#[derive(Deserialize, Debug)]
pub struct KeyboardKeyDefinition {
    pub id: u32,
    pub boundaries: Vec<SerializablePoint>,
    pub text_position: SerializablePoint,
    pub keycodes: Vec<u32>,
    pub text: String,
    pub shift_text: String,
    pub change_on_caps: bool,
}

#[derive(Deserialize, Debug)]
pub struct MouseKeyDefinition {
    pub id: u32,
    pub boundaries: Vec<SerializablePoint>,
    pub text_position: SerializablePoint,
    pub keycodes: Vec<u32>,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct MouseScrollDefinition {
    pub id: u32,
    pub boundaries: Vec<SerializablePoint>,
    pub text_position: SerializablePoint,
    pub keycodes: Vec<u32>,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct MouseSpeedIndicatorDefinition {
    pub id: u32,
    pub location: SerializablePoint,
    pub radius: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SerializablePoint {
    pub x: u32,
    pub y: u32,
}

impl From<SerializablePoint> for Point {
    fn from(point: SerializablePoint) -> Self {
        Point::new(point.x as f32, point.y as f32)
    }
}
