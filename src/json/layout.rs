use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Layout {
    /// Width of the window in logical pixels.
    pub width: f32,
    /// Height of the window in logical pixels.
    pub height: f32,
    pub elements: Vec<Element>,
}

#[derive(Serialize, Deserialize, Deref, DerefMut)]
#[serde(rename_all = "PascalCase")]
pub struct Element {
    pub id: u32,
    #[serde(flatten)]
    #[deref]
    #[deref_mut]
    pub kind: ElementKind,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "__type")]
pub enum ElementKind {
    #[serde(rename_all = "PascalCase")]
    KeyboardKey {
        /// Text to show when shift is held
        ///
        /// Also shows when caps lock is enabled and tracked.
        shift_text: String,
        /// Whether to show the shift text when caps lock is enabled.
        change_on_caps: bool,
        #[serde(flatten)]
        common: CommonElement,
    },
    MouseKey(CommonElement),
    MouseScroll(CommonElement),
    MouseSpeedIndicator(MouseSpeedIndicator),
}

impl ElementKind {
    pub fn translate(&mut self, (dx, dy): (f32, f32)) {
        match self {
            Self::MouseSpeedIndicator(element) => {
                element.location.x += dx;
                element.location.y += dy;
            }
            Self::KeyboardKey {
                common: CommonElement { boundaries, .. },
                ..
            }
            | Self::MouseKey(CommonElement { boundaries, .. })
            | Self::MouseScroll(CommonElement { boundaries, .. }) => {
                for boundary in boundaries {
                    boundary.x += dx;
                    boundary.y += dy;
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NohPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonElement {
    /// Vertices of the element.
    ///
    /// Used to draw a polygon for the background if no image is supplied, and used for hover bounds
    /// in the graphical editor.
    pub boundaries: Vec<NohPoint>,
    /// The position of the top-left corner of the text **within to the window's space**.
    pub text_position: NohPoint,
    pub key_codes: Vec<u32>,
    pub text: String,
}

impl CommonElement {
    pub fn translate_text(&mut self, (dx, dy): (f32, f32)) {
        self.text_position.x += dx;
        self.text_position.y += dy;
    }
}

impl TryFrom<ElementKind> for CommonElement {
    type Error = MouseSpeedIndicator;

    fn try_from(value: ElementKind) -> Result<Self, Self::Error> {
        match value {
            ElementKind::KeyboardKey { common, .. }
            | ElementKind::MouseKey(common)
            | ElementKind::MouseScroll(common) => Ok(common),
            ElementKind::MouseSpeedIndicator(element) => Err(element),
        }
    }
}

impl<'a> TryFrom<&'a ElementKind> for &'a CommonElement {
    type Error = &'a MouseSpeedIndicator;

    fn try_from(value: &'a ElementKind) -> Result<Self, Self::Error> {
        match value {
            ElementKind::KeyboardKey { common, .. }
            | ElementKind::MouseKey(common)
            | ElementKind::MouseScroll(common) => Ok(common),
            ElementKind::MouseSpeedIndicator(element) => Err(element),
        }
    }
}

impl<'a> TryFrom<&'a mut ElementKind> for &'a mut CommonElement {
    type Error = &'a mut MouseSpeedIndicator;

    fn try_from(value: &'a mut ElementKind) -> Result<Self, Self::Error> {
        match value {
            ElementKind::KeyboardKey { common, .. }
            | ElementKind::MouseKey(common)
            | ElementKind::MouseScroll(common) => Ok(common),
            ElementKind::MouseSpeedIndicator(element) => Err(element),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MouseSpeedIndicator {
    /// Position of the center of the indicator.
    pub location: NohPoint,
    /// Radius of the outer ring.
    ///
    /// The inner circle's radius is a fraction of this value.
    pub radius: f32,
}
