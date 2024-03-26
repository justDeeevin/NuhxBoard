use std::fmt::Display;

use crate::{
    nuhxboard::*,
    types::{settings::*, stylesheets::*},
};
use iced::{
    widget::{
        button, canvas, checkbox, column, container, horizontal_space, image, pick_list, radio,
        row, text, text_input,
    },
    window, Length, Renderer, Theme,
};
use iced_aw::{native::FloatingElement, number_input, ContextMenu, SelectionList};
use serde::{Deserialize, Serialize};

const CONTEXT_MENU_WIDTH: f32 = 160.0;

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct DisplayChoice {
    pub id: u32,
    pub primary: bool,
}

impl Display for DisplayChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.primary {
            write!(f, "{} (primary)", self.id)
        } else {
            write!(f, "{}", self.id)
        }
    }
}

impl NuhxBoard {
    pub fn draw_main_window(&self) -> iced::Element<'_, Message, Theme, Renderer> {
        let canvas = canvas::<&NuhxBoard, Message, Theme, Renderer>(self)
            .height(Length::Fill)
            .width(Length::Fill);

        let context_menu = ContextMenu::new(canvas, || {
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

            menu.append(&mut vec![
                button("Clear Pressed Keys")
                    .on_press(Message::ClearPressedKeys)
                    .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
                    .width(Length::Fixed(CONTEXT_MENU_WIDTH))
                    .into(),
                button("Exit")
                    .on_press(Message::Quitting)
                    .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
                    .width(Length::Fixed(CONTEXT_MENU_WIDTH))
                    .into(),
            ]);
            container(column(menu))
                .style(iced::theme::Container::Custom(Box::new(ContextMenuBox {})))
                .into()
        });
        if self.style.background_image_file_name.is_some() {
            let image_path = self.keyboards_path.parent().unwrap().join("background.png");
            FloatingElement::new(
                image(image_path).height(Length::Fill).width(Length::Fill),
                context_menu,
            )
            .into()
        } else {
            context_menu.into()
        }
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

    pub fn draw_settings_window(&self) -> iced::Element<'_, Message, Theme, Renderer> {
        let display_choices = self
            .display_options
            .iter()
            .map(|display| DisplayChoice {
                id: display.id,
                primary: display.is_primary,
            })
            .collect::<Vec<_>>();

        let input = column![
            row![
                text("Mouse sensitivity: ").size(12),
                horizontal_space(),
                number_input(self.settings.mouse_sensitivity, f32::MAX, |v| {
                    Message::ChangeSetting(Setting::MouseSensitivity(v))
                })
                .size(12.0)
            ]
            .padding(5)
            .align_items(iced::Alignment::Center),
            row![
                text("Scroll hold time (ms): ").size(12),
                horizontal_space(),
                number_input(self.settings.scroll_hold_time, u64::MAX, |v| {
                    Message::ChangeSetting(Setting::ScrollHoldTime(v))
                })
                .size(12.0)
            ]
            .padding(5)
            .align_items(iced::Alignment::Center),
            checkbox(
                "Calculate mouse speed from center of screen",
                self.settings.mouse_from_center
            )
            .text_size(12)
            .size(15)
            .on_toggle(|_| { Message::ChangeSetting(Setting::CenterMouse) }),
            row![
                text("Display to use: ").size(12),
                pick_list(display_choices, Some(&self.settings.display_choice), |v| {
                    Message::ChangeSetting(Setting::DisplayChoice(v))
                })
                .text_size(12)
            ]
            .padding(5)
            .align_items(iced::Alignment::Center),
            text("Show keypresses for at least").size(12),
            row![
                number_input(self.settings.min_press_time, u128::MAX, |v| {
                    Message::ChangeSetting(Setting::MinPressTime(v))
                })
                .size(12.0)
                .width(Length::Shrink),
                text("ms").size(12)
            ]
            .padding(5)
            .align_items(iced::Alignment::Center),
        ]
        .align_items(iced::Alignment::Center);

        let follow_for_sensitive_function =
            match self.settings.capitalization != Capitalization::Follow {
                true => Some(|_| Message::ChangeSetting(Setting::FollowForCapsSensitive)),
                false => None,
            };

        let follow_for_caps_insensitive_function =
            match self.settings.capitalization != Capitalization::Follow {
                true => Some(|_: bool| Message::ChangeSetting(Setting::FollowForCapsInsensitive)),
                false => None,
            };

        let capitalization = row![
            column![
                radio(
                    "Follow Caps-Lock and Shift",
                    Capitalization::Follow,
                    Some(self.settings.capitalization),
                    |v| { Message::ChangeSetting(Setting::Capitalization(v)) }
                )
                .text_size(12)
                .size(15),
                radio(
                    "Show all buttons capitalized",
                    Capitalization::Upper,
                    Some(self.settings.capitalization),
                    |v| { Message::ChangeSetting(Setting::Capitalization(v)) }
                )
                .text_size(12)
                .size(15),
                radio(
                    "Show all buttons lowercase",
                    Capitalization::Lower,
                    Some(self.settings.capitalization),
                    |v| { Message::ChangeSetting(Setting::Capitalization(v)) }
                )
                .text_size(12)
                .size(15),
            ],
            horizontal_space(),
            column![
                text("Still follow shift for").size(12),
                checkbox(
                    "Caps Lock insensitive keys",
                    self.settings.follow_for_caps_insensitive
                )
                .text_size(12)
                .size(15)
                .on_toggle_maybe(follow_for_caps_insensitive_function),
                checkbox(
                    "Caps Lock sensitive keys",
                    self.settings.follow_for_caps_sensitive
                )
                .text_size(12)
                .size(15)
                .on_toggle_maybe(follow_for_sensitive_function),
            ]
        ];

        column![
            checkbox(
                "Automatically create a desktop entry if none exists",
                self.settings.auto_desktop_entry
            )
            .text_size(12)
            .size(15)
            .on_toggle(|_| Message::ChangeSetting(Setting::AutoDesktopEntry)),
            input,
            row![
                text("Window title: ").size(12),
                text_input("NuhxBoard", self.settings.window_title.as_str())
                    .size(12)
                    .on_input(|v| Message::ChangeSetting(Setting::WindowTitle(v)))
            ]
            .align_items(iced::Alignment::Center),
            capitalization,
        ]
        .align_items(iced::Alignment::Center)
        .into()
    }

    pub fn draw_keyboard_properties_window(&self) -> iced::Element<'_, Message, Theme, Renderer> {
        column![
            row![
                text("Width: "),
                number_input(self.config.width, f32::MAX, Message::SetWidth)
            ]
            .align_items(iced::Alignment::Center),
            row![
                text("Height: "),
                number_input(self.config.height, f32::MAX, Message::SetHeight)
            ]
            .align_items(iced::Alignment::Center)
        ]
        .into()
    }
}
