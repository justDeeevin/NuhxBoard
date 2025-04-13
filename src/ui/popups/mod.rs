pub mod edit_mode;

pub use edit_mode::*;

use std::error::Error;

use crate::{message::*, nuhxboard::*, types::*};
use iced::{
    widget::{column, container, text},
    window, Theme,
};
use iced_multi_window::Window;

#[derive(Debug, Clone)]
pub struct ErrorPopup {
    // Simple example of the power of using polymorphism for multi-window management. Instead of
    // having some app state dedicated to tracking errors corresponding to window IDs, each
    // instance of an error popup knows its own error. This is put to much greater use for
    // individual key styles.
    // It is important to note, however, that these properties are unable to be changed. I would
    // argue that mutable per-window state would be problematically complex, but it is nevertheless
    // a limitation on the versatility of my multi-window system.
    pub error: NuhxBoardError,
}
impl Window<NuhxBoard, Theme, Message> for ErrorPopup {
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
        let kind = error.to_string();
        let info = error.source().map(|e| e.to_string()).unwrap_or_default();
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
