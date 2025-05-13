use crate::{message::*, nuhxboard::*, types::*};
use iced::{
    widget::{button, checkbox, column, row, text, text_input},
    window, Theme,
};
use iced_multi_window::Window;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveDefinitionAs;
impl Window<NuhxBoard, Theme, Message> for SaveDefinitionAs {
    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }

    fn title(&self, _app: &NuhxBoard) -> String {
        "Save Definition As".to_string()
    }

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        column![
            row![
                text("Category: "),
                text_input("", &app.text_input.save_keyboard_as_category,).on_input(|v| {
                    Message::ChangeTextInput(TextInputType::SaveKeyboardAsCategory, v)
                })
            ],
            row![
                text("Name: "),
                text_input("", &app.text_input.save_keyboard_as_name,)
                    .on_input(|v| Message::ChangeTextInput(TextInputType::SaveKeyboardAsName, v))
            ],
            button("Save").on_press(Message::SaveLayout(Some(
                KEYBOARDS_PATH
                    .join(&app.save_keyboard_as_category)
                    .join(&app.save_keyboard_as_name)
                    .join("keyboard.json")
            ))),
        ]
        .into()
    }

    fn settings(&self) -> window::Settings {
        window::Settings {
            size: iced::Size {
                width: 400.0,
                height: 100.0,
            },
            resizable: false,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveStyleAs;
impl Window<NuhxBoard, Theme, Message> for SaveStyleAs {
    fn settings(&self) -> window::Settings {
        window::Settings {
            size: iced::Size {
                width: 400.0,
                height: 100.0,
            },
            resizable: false,
            ..Default::default()
        }
    }

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        column![
            row![
                text("Name: "),
                text_input("", &app.text_input.save_style_as_name,)
                    .on_input(|v| Message::ChangeTextInput(TextInputType::SaveStyleAsName, v))
            ],
            checkbox("Save as global", app.save_style_as_global)
                .on_toggle(|_| Message::ToggleSaveStyleAsGlobal),
            button("Save").on_press(Message::SaveStyle(Some(match app.save_style_as_global {
                true => KEYBOARDS_PATH
                    .join("global")
                    .join(format!("{}.style", &app.save_style_as_name)),
                false => KEYBOARDS_PATH
                    .join(&app.settings.category)
                    .join(&app.keyboard_options[app.keyboard_choice.unwrap()])
                    .join(format!("{}.style", &app.save_style_as_name)),
            }))),
        ]
        .into()
    }

    fn title(&self, _app: &NuhxBoard) -> String {
        "Save Style As".to_string()
    }

    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }
}
