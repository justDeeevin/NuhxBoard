pub mod edit_mode;

pub use edit_mode::*;

use crate::{message::*, nuhxboard::*, nuhxboard_types::*};
use iced::{
    widget::{column, container, text},
    window, Theme,
};
use iced_multi_window::Window;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorPopup {
    // Simple example of the power of using polymorphism for multi-window management. Instead of
    // having some app state dedicated to tracking errors corresponding to window IDs, each
    // instance of an error popup knows its own error. This is put to much greater use for
    // individual key styles.
    // It is important to note, however, that these properties are unable to be changed. I would
    // argue that mutable per-window state would be problematically complex, but it is nevertheless
    // a limitation on the versatility of my multi-window system.
    pub error: Error,
}
impl Window<NuhxBoard, Theme, Message> for ErrorPopup {
    fn class(&self) -> &'static str {
        "error"
    }

    fn id(&self) -> String {
        format!("{}_{:?}", self.class(), self.error)
    }

    fn settings(&self) -> window::Settings {
        window::Settings {
            size: iced::Size {
                width: 400.0,
                height: 150.0,
            },
            resizable: false,
            ..Default::default()
        }
    }

    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }

    fn title(&self, _app: &NuhxBoard) -> String {
        "Error".to_string()
    }

    fn view<'a>(&self, _app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        let error = &self.error;
        let kind = match error {
            Error::ConfigOpen(_) => "Keyboard file could not be opened.",
            Error::ConfigParse(_) => "Keyboard file could not be parsed.",
            Error::StyleOpen(_) => "Style file could not be opened.",
            Error::StyleParse(_) => "Style file could not be parsed.",
            Error::UnknownKey(_) => "Unknown Key.",
            Error::UnknownButton(_) => "Unknown Mouse Button.",
        };
        let info = match error {
            Error::ConfigParse(e) => e.clone(),
            Error::ConfigOpen(e) => e.clone(),
            Error::StyleParse(e) => e.clone(),
            Error::StyleOpen(e) => e.clone(),
            Error::UnknownKey(key) => format!("Key: {:?}", key),
            Error::UnknownButton(button) => format!("Button: {:?}", button),
        };
        container(
            column![text("Error:"), text(kind), text("More info:"), text(info),]
                .align_x(iced::Alignment::Center),
        )
        .height(iced::Length::Fill)
        .width(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    }
}
