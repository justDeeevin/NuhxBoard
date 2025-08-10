pub mod edit_mode;

pub use edit_mode::*;

use std::error::Error;

use crate::{message::*, nuhxboard::*, types::*};
use iced::{
    widget::{button, column, container, row, text},
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

#[derive(Debug, Clone)]
pub struct UnsavedChangesPopup(pub Action);

#[derive(Debug, Clone)]
pub enum Action {
    LoadKeyboard,
    Exit,
}

impl Action {
    pub fn present_tense(&self) -> &'static str {
        match self {
            Action::LoadKeyboard => "Loading a new keyboard",
            Action::Exit => "Exiting",
        }
    }

    pub fn future_tense(&self) -> &'static str {
        match self {
            Action::LoadKeyboard => "load a new keyboard",
            Action::Exit => "exit",
        }
    }
}

impl Window<NuhxBoard, Theme, Message> for UnsavedChangesPopup {
    fn settings(&self) -> window::Settings {
        window::Settings {
            size: iced::Size {
                width: 700.0,
                height: 100.0,
            },
            resizable: false,
            ..window::Settings::default()
        }
    }

    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }

    fn title(&self, _app: &NuhxBoard) -> String {
        "Discard changes".to_string()
    }

    fn view<'a>(&'a self, _app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        row![column![
            text(format!(
                "You have unsaved changes. {} will undo them. Are you sure you want to {}?",
                self.0.present_tense(),
                self.0.future_tense()
            )),
            row![
                button("Yes").on_press(Message::Commit(self.0.clone())),
                button("Cancel").on_press(Message::CancelDiscard(self.0.clone()))
            ]
        ]
        .align_x(iced::Alignment::Center)
        .width(iced::Length::Fill)]
        .align_y(iced::Alignment::Center)
        .height(iced::Length::Fill)
        .into()
    }
}
