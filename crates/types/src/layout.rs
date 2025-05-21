use geo::Coord;
pub use ordered_float::OrderedFloat;
use schemars::{json_schema, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct Layout {
    /// No actual meaning. Kept for parity with NohBoard layout files.
    pub version: Option<u8>,
    /// Width of the window in pixels
    pub width: f32,
    /// Height of the window in pixels
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

    pub fn translate(&mut self, delta: Coord<f32>, move_text: bool) {
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
    /// Unique id of the element. Used by style files.
    pub id: u32,
    /// Vertices of the element. Used to draw a polygon for the background if no image is
    /// supplied, and always used for graphical editing.
    pub boundaries: Vec<SerializablePoint>,
    /// The position of the top-left corner of the text. **Window-relative, not
    /// element-relative**.
    pub text_position: SerializablePoint,
    pub key_codes: Vec<u32>,
    pub text: String,
    /// Text to display when Shift is held.
    pub shift_text: String,
    pub change_on_caps: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct CommonDefinition {
    /// Unique id of the element. Used by style files.
    pub id: u32,
    /// Vertices of the element. Used to draw a polygon for the background if no image is
    /// supplied, and always used for graphical editing.
    pub boundaries: Vec<SerializablePoint>,
    /// The position of the top-left corner of the text. **Window-relative, not
    /// element-relative**.
    pub text_position: SerializablePoint,
    pub key_codes: Vec<u32>,
    pub text: String,
}

impl CommonDefinition {
    pub fn translate_face(&mut self, face: usize, delta: Coord<f32>) {
        self.boundaries[face] += delta;
        if face == self.boundaries.len() - 1 {
            self.boundaries[0] += delta;
        } else {
            self.boundaries[face + 1] += delta;
        }
    }
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

impl CommonDefinitionMut<'_> {
    pub fn translate_face(&mut self, face: usize, delta: Coord<f32>) {
        self.boundaries[face] += delta;
        if face == self.boundaries.len() - 1 {
            self.boundaries[0] += delta;
        } else {
            self.boundaries[face + 1] += delta;
        }
    }
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
    type Error = &'a mut MouseSpeedIndicatorDefinition;

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
    /// Unique id of the element. Used by style files.
    pub id: u32,
    /// Position of the center of the indicator.
    pub location: SerializablePoint,
    /// Radius of the outer ring.
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SerializablePoint {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
}

impl JsonSchema for SerializablePoint {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "SerializablePoint".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        "nuhxboard_types::layout::SerializablePoint".into()
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "$schema": "https://json-schema.org/draft/2029-12/schema",
            "properties": {
                "X": {
                    "format": "float",
                    "type": "number"
                },
                "Y": {
                    "format": "float",
                    "type": "number"
                }
            },
            "required": ["X", "Y"],
            "title": "SerializablePoint",
            "type": "object"
        })
    }
}

impl From<SerializablePoint> for iced::Point {
    fn from(point: SerializablePoint) -> Self {
        iced::Point::new(*point.x, *point.y)
    }
}

impl From<iced::Point> for SerializablePoint {
    fn from(point: iced::Point) -> Self {
        Self {
            x: OrderedFloat(point.x),
            y: OrderedFloat(point.y),
        }
    }
}

impl From<SerializablePoint> for Coord<f32> {
    fn from(value: SerializablePoint) -> Self {
        Self {
            x: *value.x,
            y: *value.y,
        }
    }
}

impl From<Coord<f32>> for SerializablePoint {
    fn from(value: Coord<f32>) -> Self {
        Self {
            x: OrderedFloat(value.x),
            y: OrderedFloat(value.y),
        }
    }
}

impl std::ops::AddAssign<Coord<f32>> for SerializablePoint {
    fn add_assign(&mut self, rhs: Coord<f32>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::SubAssign<Coord<f32>> for SerializablePoint {
    fn sub_assign(&mut self, rhs: Coord<f32>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::fmt::Display for SerializablePoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
