use derive_more::{Deref, Display};
use serde::{Deserialize, Deserializer, Serialize};

pub mod layout;
pub mod settings;
pub mod style;

#[derive(Deref, Serialize, Display)]
/// An individual path segment with no separators.
pub struct PathSegment(String);

impl<'de> Deserialize<'de> for PathSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?)
            .ok_or_else(|| serde::de::Error::custom("Path segment cannot contain a separator."))
    }
}

impl AsRef<std::path::Path> for PathSegment {
    fn as_ref(&self) -> &std::path::Path {
        self.0.as_ref()
    }
}

impl AsRef<std::ffi::OsStr> for PathSegment {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_ref()
    }
}

impl PathSegment {
    pub fn new(string: impl Into<String>) -> Option<Self> {
        let raw = string.into();
        if raw.contains('/') || (cfg!(target_os = "windows") && raw.contains('\\')) {
            None
        } else {
            Some(Self(raw))
        }
    }
}
