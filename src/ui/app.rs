use crate::{nuhxboard::*, types::stylesheets::*};
use iced::{
    widget::{
        button, canvas, canvas::Cache, checkbox, column, container, horizontal_space, pick_list,
        radio, row, text, text_input,
    },
    window, Color, Command, Length, Renderer, Subscription, Theme,
};
use iced_aw::{number_input, ContextMenu, SelectionList};

const CONTEXT_MENU_WIDTH: f32 = 160.0;

impl NuhxBoard {
    pub fn draw_main_window(&self) -> iced::Element<'_, Message, Theme, Renderer> {
        let canvas = canvas::<&NuhxBoard, Message, Theme, Renderer>(self)
            .height(Length::Fill)
            .width(Length::Fill);

        ContextMenu::new(canvas, || {
            let load_keyboard_window_message = match self.load_keyboard_window_id {
                Some(_) => None,
                None => Some(Message::OpenLoadKeyboardWindow),
            };

            let settings_window_message = match self.settings_window_id {
                Some(_) => None,
                None => Some(Message::OpenSettingsWindow),
            };

            let toggle_button_label = match self.edit_mode {
                true => "Stop Editing",
                false => "Start Editing",
            };

            let mut menu = vec![
                button("Settings")
                    .on_press_maybe(settings_window_message.clone())
                    .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
                    .width(Length::Fixed(CONTEXT_MENU_WIDTH))
                    .into(),
                button("Load Keyboard")
                    .on_press_maybe(load_keyboard_window_message.clone())
                    .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
                    .width(Length::Fixed(CONTEXT_MENU_WIDTH))
                    .into(),
                button(toggle_button_label)
                    .on_press(Message::ToggleEditMode)
                    .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
                    .width(Length::Fixed(CONTEXT_MENU_WIDTH))
                    .into(),
            ];

            if self.edit_mode {
                menu.append(&mut vec![
                    button("Keyboard Properties")
                        .on_press(Message::OpenKeyboardProperties)
                        .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
                        .width(Length::Fixed(CONTEXT_MENU_WIDTH))
                        .into(),
                    button("Save Keyboard")
                        .on_press(Message::SaveKeyboard(None))
                        .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
                        .width(Length::Fixed(CONTEXT_MENU_WIDTH))
                        .into(),
                    button("Save Style")
                        .on_press(Message::SaveStyle(None))
                        .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
                        .width(Length::Fixed(CONTEXT_MENU_WIDTH))
                        .into(),
                ]);
            }

            menu.push(
                button("Clear Pressed Keys")
                    .on_press(Message::ClearPressedKeys)
                    .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
                    .width(Length::Fixed(CONTEXT_MENU_WIDTH))
                    .into(),
            );
            container(column(menu))
                .style(iced::theme::Container::Custom(Box::new(ContextMenuBox {})))
                .into()
        })
        .into()
    }

    pub fn draw_load_keyboard_window(&self) -> iced::Element<'_, Message, Theme, Renderer> {
        column![
            text("Category:"),
            pick_list(
                self.keyboard_category_options.clone(),
                Some(self.settings.category.clone()),
                Message::ChangeKeyboardCategory,
            ),
            row![
                SelectionList::new_with(
                    self.keyboard_options.clone().leak(),
                    |i, _| Message::LoadKeyboard(i),
                    12.0,
                    5.0,
                    <Theme as iced_aw::style::selection_list::StyleSheet>::Style::default(),
                    self.keyboard,
                    iced::Font::default(),
                ),
                SelectionList::new_with(
                    self.style_options.clone().leak(),
                    |i, _| Message::LoadStyle(i),
                    12.0,
                    5.0,
                    <Theme as iced_aw::style::selection_list::StyleSheet>::Style::default(),
                    self.style_choice,
                    iced::Font::default(),
                ),
            ]
        ]
        .into()
    }

    pub fn draw_error_window(
        &self,
        window: &window::Id,
    ) -> iced::Element<'_, Message, Theme, Renderer> {
        let error = self.error_windows.get(window).unwrap();
        let kind = match error {
            Error::ConfigOpen(_) => "Keyboard file could not be opened.",
            Error::ConfigParse(_) => "Keyboard file could not be parsed.",
            Error::StyleOpen(_) => "Style file could not be opened.",
            Error::StyleParse(_) => "Style file could not be parsed.",
            Error::UnknownKey(_) => "Unknown Key.",
            Error::UnknownButton(_) => "Unknown Mouse Button.",
        };
        let info = match error {
            Error::ConfigParse(e) => {
                if e.is_eof() {
                    format!("Unexpected EOF (End of file) at line {}", e.line())
                } else {
                    format!("{}", e)
                }
            }
            Error::ConfigOpen(e) => format!("{}", e),
            Error::StyleParse(e) => {
                if e.is_eof() {
                    format!("Unexpeted EOF (End of file) at line {}", e.line())
                } else {
                    format!("{}", e)
                }
            }
            Error::StyleOpen(e) => format!("{}", e),
            Error::UnknownKey(key) => format!("Key: {:?}", key),
            Error::UnknownButton(button) => format!("Button: {:?}", button),
        };
        container(
            column![text("Error:"), text(kind), text("More info:"), text(info),]
                .align_items(iced::Alignment::Center),
        )
        .height(iced::Length::Fill)
        .width(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    }
}
