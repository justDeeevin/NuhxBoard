use derive_more::Display;
use nuhxboard_json::{
    PathSegment,
    layout::{CommonElement, Layout, NohPoint},
    style::{ElementStyle, FontStyle, KeyStyle, Style},
};
use slint::{Image, LogicalPosition, LogicalSize, Model, SharedString, StyledText, ToSharedString};
use std::fmt::Display;
use std::{
    num::{ParseFloatError, ParseIntError},
    path::PathBuf,
    sync::LazyLock,
};

slint::include_modules!();

pub static NUHXBOARD_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    confy::get_configuration_file_path("NuhxBoard", None)
        .expect("Failed to get config path")
        .parent()
        .expect("Config file has no parent?")
        .into()
});

impl Main {
    pub fn apply_layout(&self, layout: &Layout) {
        self.set_w(layout.width);
        self.set_h(layout.height);

        let mut keys = Vec::default();
        let mut speedometers = Vec::new();

        for element in &layout.elements {
            match <&CommonElement>::try_from(&element.kind) {
                Ok(def) => {
                    // For some reason, the orientation of the path changes how slint decides to
                    // fill, eliminating the pixel gap upon which layouts tend to rely.
                    let should_flip = signed_area(&def.boundaries) > 0.0;
                    let first = if should_flip {
                        *def.boundaries.last().unwrap()
                    } else {
                        def.boundaries[0]
                    };
                    let mut commands = format!("M {} {} ", first.x, first.y).to_shared_string();
                    if should_flip {
                        add_points(&mut commands, def.boundaries.iter().rev().skip(1));
                    } else {
                        add_points(&mut commands, &def.boundaries[1..]);
                    }
                    let (bounding_box_pos, bounding_box_size) = bounding_box(&def.boundaries);
                    commands += "Z";
                    keys.push(Key {
                        id: element.id as i32,
                        commands,
                        text: def.text.to_shared_string(),
                        styled_text: StyledText::from_plain_text(&def.text),
                        text_position: LogicalPosition {
                            x: def.text_position.x,
                            y: def.text_position.y,
                        },
                        bounding_box_pos,
                        bounding_box_size,
                        ..Default::default()
                    });
                }
                Err(def) => speedometers.push(Speedometer {
                    position: LogicalPosition {
                        x: def.location.x,
                        y: def.location.y,
                    },
                    radius: def.radius,
                }),
            }
        }

        self.set_keys(keys.as_slice().into());
        self.set_speedometers(speedometers.as_slice().into());
    }

    pub fn apply_style(
        &self,
        source: StyleSource<'_, '_>,
        style_name: &PathSegment,
        style: &Style,
    ) {
        self.set_background_color(style.background_color.into());
        let images_path = match source {
            StyleSource::Global => format!("global/{style_name}"),
            StyleSource::Local { category, .. } => format!("{category}/images"),
        };
        let images_path = NUHXBOARD_PATH.join("keyboards").join(images_path);
        if let Some(image) = &style.background_image_file_name {
            if let Ok(i) = Image::load_from_path(&images_path.join(image)) {
                self.set_background_image(i);
            } else {
                ErrorPopup::pop("Failed to load background image.")
            }
        }
        self.set_keys(
            self.get_keys()
                .iter()
                .map(|key| {
                    let key_style = match style.element_styles.get(&(key.id as u32)) {
                        Some(ElementStyle::KeyStyle(KeyStyle {
                            loose: Some(style), ..
                        })) => style,
                        _ => &style.default_key_style.loose,
                    };
                    let background_image = key_style
                        .background_image_file_name
                        .as_ref()
                        .and_then(|name| {
                            if name.is_empty() {
                                return None;
                            }
                            match Image::load_from_path(&images_path.join(name)) {
                                Ok(image) => Some(image),
                                Err(_) => {
                                    ErrorPopup::pop("Failed to open background image.");
                                    None
                                }
                            }
                        })
                        .unwrap_or_default();

                    let mut text = key.text.replace('#', "\\#");
                    if !text.is_empty() {
                        if key_style.font.style.contains(FontStyle::BOLD) {
                            text = format!("**{text}**");
                        }
                        if key_style.font.style.contains(FontStyle::ITALIC) {
                            text = format!("*{text}*");
                        }
                        if key_style.font.style.contains(FontStyle::UNDERLINE) {
                            text = format!("<u>{text}</u>");
                        }
                        if key_style.font.style.contains(FontStyle::STRIKETHROUGH) {
                            text = format!("~~{text}~~");
                        }
                    }
                    Key {
                        background: key_style.background.into(),
                        background_image: background_image.clone(),
                        underline: key_style.font.style.contains(FontStyle::UNDERLINE),
                        styled_text: StyledText::from_markdown(&text).unwrap(),
                        text_color: key_style.text.into(),
                        font_family: key_style.font.font_family.to_shared_string(),
                        font_size: key_style.font.size,
                        outline_color: key_style.outline.into(),
                        outline_width: if key_style.show_outline {
                            key_style.outline_width
                        } else {
                            0.0
                        },
                        ..key
                    }
                })
                .collect::<Vec<_>>()
                .as_slice()
                .into(),
        );
    }
}

impl From<&nuhxboard_json::settings::Settings> for Settings {
    fn from(value: &nuhxboard_json::settings::Settings) -> Self {
        Self {
            capitalization: value.capitalization.into(),
            follow_for_caps_sensitive: value.follow_for_caps_sensitive,
            follow_for_caps_insensitive: value.follow_for_caps_insensitive,
            mouse_from_center: value.mouse_from_center,
            mouse_sensitivity: value.mouse_sensitivity.to_shared_string(),
            min_press_time: value.min_press_time.to_shared_string(),
            scroll_hold_time: value.scroll_hold_time.to_shared_string(),
            window_title: value.window_title.to_shared_string(),
        }
    }
}

#[derive(Debug, Display)]
pub enum ApplyError {
    #[display("Failed to parse integer: {_0}")]
    Int(ParseIntError),
    #[display("Failed to parse   float: {_0}")]
    Float(ParseFloatError),
}

impl From<ParseFloatError> for ApplyError {
    fn from(value: ParseFloatError) -> Self {
        Self::Float(value)
    }
}

impl From<ParseIntError> for ApplyError {
    fn from(value: ParseIntError) -> Self {
        Self::Int(value)
    }
}

impl Settings {
    pub fn apply_to(
        self,
        settings: &mut nuhxboard_json::settings::Settings,
    ) -> Result<(), Vec<ApplyError>> {
        let mut out: Result<_, Vec<ApplyError>> = Ok(());

        macro_rules! mytry {
            ($left:expr => $right:expr) => {
                match $right {
                    Ok(v) => $left = v,
                    Err(e) => match &mut out {
                        Ok(()) => out = Err(vec![e.into()]),
                        Err(es) => es.push(e.into()),
                    },
                }
            };
        }

        settings.capitalization = self.capitalization.into();
        settings.follow_for_caps_sensitive = self.follow_for_caps_sensitive;
        settings.follow_for_caps_insensitive = self.follow_for_caps_insensitive;
        settings.mouse_from_center = self.mouse_from_center;
        mytry!(settings.mouse_sensitivity => self.mouse_sensitivity.parse());
        mytry!(settings.min_press_time => self.min_press_time.parse());
        mytry!(settings.scroll_hold_time => self.scroll_hold_time.parse());
        settings.window_title = self.window_title.into();

        out
    }
}

impl From<Capitalization> for nuhxboard_json::settings::Capitalization {
    fn from(value: Capitalization) -> Self {
        match value {
            Capitalization::Lower => Self::Lower,
            Capitalization::Upper => Self::Upper,
            Capitalization::Follow => Self::Follow,
        }
    }
}

impl From<nuhxboard_json::settings::Capitalization> for Capitalization {
    fn from(value: nuhxboard_json::settings::Capitalization) -> Self {
        match value {
            nuhxboard_json::settings::Capitalization::Lower => Self::Lower,
            nuhxboard_json::settings::Capitalization::Upper => Self::Upper,
            nuhxboard_json::settings::Capitalization::Follow => Self::Follow,
        }
    }
}

fn signed_area(points: &[NohPoint]) -> f32 {
    let n = points.len();
    (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            (a.x * b.y) - (b.x * a.y)
        })
        .sum::<f32>()
        / 2.0
}

fn add_points<'a>(commands: &mut SharedString, points: impl IntoIterator<Item = &'a NohPoint>) {
    for point in points {
        *commands += &format!("L {} {} ", point.x, point.y);
    }
}

fn bounding_box(points: &[NohPoint]) -> (LogicalPosition, LogicalSize) {
    let (min_x, min_y, max_x, max_y) = points.iter().fold(
        (points[0].x, points[0].y, points[0].x, points[0].y),
        |(min_x, min_y, max_x, max_y), point| {
            (
                min_x.min(point.x),
                min_y.min(point.y),
                max_x.max(point.x),
                max_y.max(point.y),
            )
        },
    );

    (
        LogicalPosition { x: min_x, y: min_y },
        LogicalSize {
            width: max_x - min_x,
            height: max_y - min_y,
        },
    )
}

#[derive(Clone, Copy)]
pub enum StyleSource<'a, 'b> {
    Global,
    Local {
        category: &'a PathSegment,
        layout: &'b PathSegment,
    },
}

impl ErrorPopup {
    fn with_message(message: impl Display) -> Result<Self, slint::PlatformError> {
        let out = Self::new()?;
        out.set_message(message.to_shared_string());
        Ok(out)
    }

    #[track_caller]
    pub fn pop(message: impl Display) {
        eprintln!("Error at {}", std::panic::Location::caller());
        let _ = Self::with_message(message).and_then(|window| window.show());
    }
}
