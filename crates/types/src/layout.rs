use iced::Point;
use ordered_float::OrderedFloat;
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
    MouseKey(CommonDefinition),
    MouseScroll(CommonDefinition),
    MouseSpeedIndicator(MouseSpeedIndicatorDefinition),
}

impl BoardElement {
    pub fn id(&self) -> u32 {
        if let Ok(def) = CommonDefinitionRef::try_from(self) {
            *def.id
        } else if let Self::MouseSpeedIndicator(def) = self {
            def.id
        } else {
            unreachable!()
        }
    }

    pub fn translate(&mut self, delta: geo::Coord<f32>, move_text: bool) {
        match self {
            BoardElement::MouseSpeedIndicator(key) => {
                key.location += delta;
            }
            _ => {
                let common = CommonDefinitionMut::try_from(self).unwrap();
                for boundary in common.boundaries {
                    *boundary += delta;
                }
                if move_text {
                    *common.text_position += delta;
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
pub struct CommonDefinition {
    pub id: u32,
    pub boundaries: Vec<SerializablePoint>,
    pub text_position: SerializablePoint,
    pub key_codes: Vec<u32>,
    pub text: String,
}

impl From<KeyboardKeyDefinition> for CommonDefinition {
    fn from(val: KeyboardKeyDefinition) -> Self {
        CommonDefinition {
            id: val.id,
            text: val.text,
            text_position: val.text_position,
            boundaries: val.boundaries,
            key_codes: val.key_codes,
        }
    }
}

impl TryFrom<BoardElement> for CommonDefinition {
    type Error = MouseSpeedIndicatorDefinition;

    fn try_from(value: BoardElement) -> Result<Self, Self::Error> {
        match value {
            BoardElement::KeyboardKey(key) => Ok(key.into()),
            BoardElement::MouseKey(key) | BoardElement::MouseScroll(key) => Ok(key),
            BoardElement::MouseSpeedIndicator(key) => Err(key),
        }
    }
}

pub struct CommonDefinitionRef<'a> {
    pub id: &'a u32,
    pub text: &'a String,
    pub text_position: &'a SerializablePoint,
    pub boundaries: &'a Vec<SerializablePoint>,
    pub key_codes: &'a Vec<u32>,
}

impl<'a> From<&'a KeyboardKeyDefinition> for CommonDefinitionRef<'a> {
    fn from(val: &'a KeyboardKeyDefinition) -> Self {
        CommonDefinitionRef {
            id: &val.id,
            text: &val.text,
            text_position: &val.text_position,
            boundaries: &val.boundaries,
            key_codes: &val.key_codes,
        }
    }
}

impl<'a> From<&'a CommonDefinition> for CommonDefinitionRef<'a> {
    fn from(val: &'a CommonDefinition) -> Self {
        CommonDefinitionRef {
            id: &val.id,
            text: &val.text,
            text_position: &val.text_position,
            boundaries: &val.boundaries,
            key_codes: &val.key_codes,
        }
    }
}

impl<'a> TryFrom<&'a BoardElement> for CommonDefinitionRef<'a> {
    type Error = &'a MouseSpeedIndicatorDefinition;

    fn try_from(value: &'a BoardElement) -> Result<Self, Self::Error> {
        match value {
            BoardElement::KeyboardKey(key) => Ok(key.into()),
            BoardElement::MouseKey(key) | BoardElement::MouseScroll(key) => Ok(key.into()),
            BoardElement::MouseSpeedIndicator(def) => Err(def),
        }
    }
}

pub struct CommonDefinitionMut<'a> {
    pub id: &'a mut u32,
    pub text: &'a mut String,
    pub text_position: &'a mut SerializablePoint,
    pub boundaries: &'a mut Vec<SerializablePoint>,
    pub key_codes: &'a mut Vec<u32>,
}

impl<'a> From<&'a mut KeyboardKeyDefinition> for CommonDefinitionMut<'a> {
    fn from(val: &'a mut KeyboardKeyDefinition) -> Self {
        CommonDefinitionMut {
            id: &mut val.id,
            text: &mut val.text,
            text_position: &mut val.text_position,
            boundaries: &mut val.boundaries,
            key_codes: &mut val.key_codes,
        }
    }
}

impl<'a> From<&'a mut CommonDefinition> for CommonDefinitionMut<'a> {
    fn from(val: &'a mut CommonDefinition) -> Self {
        CommonDefinitionMut {
            id: &mut val.id,
            text: &mut val.text,
            text_position: &mut val.text_position,
            boundaries: &mut val.boundaries,
            key_codes: &mut val.key_codes,
        }
    }
}

impl<'a> TryFrom<&'a mut BoardElement> for CommonDefinitionMut<'a> {
    type Error = &'a MouseSpeedIndicatorDefinition;

    fn try_from(value: &'a mut BoardElement) -> Result<Self, Self::Error> {
        match value {
            BoardElement::KeyboardKey(key) => Ok(key.into()),
            BoardElement::MouseKey(key) | BoardElement::MouseScroll(key) => Ok(key.into()),
            BoardElement::MouseSpeedIndicator(key) => Err(key),
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

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq)]
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

impl From<geo::Coord<f32>> for SerializablePoint {
    fn from(value: geo::Coord<f32>) -> Self {
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

impl std::fmt::Display for SerializablePoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Eq for SerializablePoint {}

impl std::hash::Hash for SerializablePoint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        OrderedFloat(self.x).hash(state);
        OrderedFloat(self.y).hash(state);
    }
}
