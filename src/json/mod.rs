use derive_more::Deref;
use serde::{Deserialize, Deserializer, Serialize};

pub mod layout;
pub mod settings;
pub mod style;

#[derive(Deref, Serialize)]
/// An individual path segment with no separators.
pub struct PathSegment(String);

impl<'de> Deserialize<'de> for PathSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        if raw.contains("/") || (cfg!(target_os = "windows") && raw.contains("\\")) {
            Err(serde::de::Error::custom(
                "Path fragment cannot include a path separator.",
            ))
        } else {
            Ok(Self(raw))
        }
    }
}
