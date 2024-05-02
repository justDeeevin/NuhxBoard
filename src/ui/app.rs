use crate::{
    nuhxboard::*,
    types::{settings::*, stylesheets::*},
};
use iced::{
    widget::{
        button, canvas, checkbox, column, container, horizontal_space, image, pick_list, radio,
        row, text, text_input, Button,
    },
    window, Color, Length, Renderer, Theme,
};
use iced_aw::{
    native::{FloatingElement, InnerBounds},
    number_input,
    quad::Quad,
    ContextMenu, SelectionList,
};
use iced_multi_window::{window, Window};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};

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

fn context_menu_button(label: &str) -> Button<Message> {
    let text = text(label).size(12);
    button(text)
        .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
        .width(Length::Fill)
}

fn seperator() -> Quad {
    Quad {
        quad_color: iced::Background::Color(Color::from_rgb8(204, 204, 204)),
        height: Length::Fixed(5.0),
        inner_bounds: InnerBounds::Ratio(0.95, 0.2),
        ..Default::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Main;
impl Window<NuhxBoard> for Main {
    fn settings(&self) -> window::Settings {
        window::Settings::default()
    }

    fn view<'a>(
        &'a self,
        app: &'a NuhxBoard,
        _id: window::Id,
    ) -> iced::Element<
        '_,
        <NuhxBoard as iced::multi_window::Application>::Message,
        <NuhxBoard as iced::multi_window::Application>::Theme,
    > {
        let canvas = canvas::<&NuhxBoard, Message, Theme, Renderer>(app)
            .height(Length::Fill)
            .width(Length::Fill);

        let context_menu = ContextMenu::new(canvas, || {
            let mut menu = vec![
                context_menu_button("Settings")
                    .on_press_maybe(
                        (!app.windows.any_of(window!(SettingsWindow)))
                            .then_some(Message::Open(window!(SettingsWindow))),
                    )
                    .into(),
                context_menu_button("Load Keyboard")
                    .on_press_maybe(
                        (!app.windows.any_of(window!(LoadKeyboard)))
                            .then_some(Message::Open(window!(LoadKeyboard))),
                    )
                    .into(),
                seperator().into(),
                context_menu_button(match app.edit_mode {
                    true => "Stop Editing",
                    false => "Start Editing",
                })
                .on_press(Message::ToggleEditMode)
                .into(),
            ];

            if app.edit_mode {
                menu.append(&mut vec![
                    checkbox("Update Text Position", app.settings.update_text_position)
                        .on_toggle(|_| Message::ToggleUpdateTextPosition)
                        .style(iced::theme::Checkbox::Custom(Box::new(
                            ContextMenuCheckBox {},
                        )))
                        .text_size(12)
                        .size(15)
                        .into(),
                    seperator().into(),
                    context_menu_button("Keyboard Properties")
                        .on_press_maybe(
                            (!app.windows.any_of(window!(KeyboardProperties)))
                                .then_some(Message::Open(window!(KeyboardProperties))),
                        )
                        .into(),
                ]);
            }

            menu.append(&mut vec![
                seperator().into(),
                context_menu_button("Save Definition")
                    .on_press(Message::SaveKeyboard(None))
                    .into(),
                context_menu_button("Save Definition As...")
                    .on_press(Message::Open(window!(SaveKeyboardAs)))
                    .into(),
                context_menu_button("Save Style")
                    .on_press(Message::SaveStyle(None))
                    .into(),
                context_menu_button("Save Style As...")
                    .on_press(Message::Open(window!(SaveStyleAs)))
                    .into(),
                context_menu_button("Clear Pressed Keys")
                    .on_press(Message::ClearPressedKeys)
                    .into(),
                context_menu_button("Exit")
                    .on_press(Message::ClosingMain)
                    .into(),
            ]);
            container(column(menu))
                .style(iced::theme::Container::Custom(Box::new(ContextMenuBox {})))
                .width(Length::Fixed(150.0))
                .into()
        });
        if app.style.background_image_file_name.is_some() {
            let image_path = app.keyboards_path.parent().unwrap().join("background.png");
            FloatingElement::new(
                image(image_path).height(Length::Fill).width(Length::Fill),
                context_menu,
            )
            .into()
        } else {
            context_menu.into()
        }
    }

    fn theme(
        &self,
        app: &NuhxBoard,
        _id: window::Id,
    ) -> <NuhxBoard as iced::multi_window::Application>::Theme {
        let red = app.style.background_color.red / 255.0;
        let green = app.style.background_color.green / 255.0;
        let blue = app.style.background_color.blue / 255.0;
        let palette = iced::theme::Palette {
            background: Color::from_rgb(red, green, blue),
            ..iced::theme::Palette::DARK
        };
        Theme::Custom(Arc::new(iced::theme::Custom::new("Custom".into(), palette)))
    }

    fn title(&self, app: &NuhxBoard, _id: window::Id) -> String {
        app.settings.window_title.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsWindow;
impl Window<NuhxBoard> for SettingsWindow {
    fn settings(&self) -> window::Settings {
        window::Settings {
            resizable: false,
            size: iced::Size {
                width: 420.0,
                height: 300.0,
            },
            ..Default::default()
        }
    }

    fn view<'a>(
        &'a self,
        app: &'a NuhxBoard,
        _id: window::Id,
    ) -> iced::Element<
        '_,
        <NuhxBoard as iced::multi_window::Application>::Message,
        <NuhxBoard as iced::multi_window::Application>::Theme,
    > {
        let display_choices = app
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
                number_input(app.settings.mouse_sensitivity, f32::MAX, |v| {
                    Message::ChangeSetting(Setting::MouseSensitivity(v))
                })
                .size(12.0)
            ]
            .padding(5)
            .align_items(iced::Alignment::Center),
            row![
                text("Scroll hold time (ms): ").size(12),
                horizontal_space(),
                number_input(app.settings.scroll_hold_time, u64::MAX, |v| {
                    Message::ChangeSetting(Setting::ScrollHoldTime(v))
                })
                .size(12.0)
            ]
            .padding(5)
            .align_items(iced::Alignment::Center),
            checkbox(
                "Calculate mouse speed from center of screen",
                app.settings.mouse_from_center
            )
            .text_size(12)
            .size(15)
            .on_toggle(|_| { Message::ChangeSetting(Setting::CenterMouse) }),
            row![
                text("Display to use: ").size(12),
                pick_list(display_choices, Some(&app.settings.display_choice), |v| {
                    Message::ChangeSetting(Setting::DisplayChoice(v))
                })
                .text_size(12)
            ]
            .padding(5)
            .align_items(iced::Alignment::Center),
            text("Show keypresses for at least").size(12),
            row![
                number_input(app.settings.min_press_time, u128::MAX, |v| {
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
            match app.settings.capitalization != Capitalization::Follow {
                true => Some(|_| Message::ChangeSetting(Setting::FollowForCapsSensitive)),
                false => None,
            };

        let follow_for_caps_insensitive_function =
            match app.settings.capitalization != Capitalization::Follow {
                true => Some(|_: bool| Message::ChangeSetting(Setting::FollowForCapsInsensitive)),
                false => None,
            };

        let capitalization = row![
            column![
                radio(
                    "Follow Caps-Lock and Shift",
                    Capitalization::Follow,
                    Some(app.settings.capitalization),
                    |v| { Message::ChangeSetting(Setting::Capitalization(v)) }
                )
                .text_size(12)
                .size(15),
                radio(
                    "Show all buttons capitalized",
                    Capitalization::Upper,
                    Some(app.settings.capitalization),
                    |v| { Message::ChangeSetting(Setting::Capitalization(v)) }
                )
                .text_size(12)
                .size(15),
                radio(
                    "Show all buttons lowercase",
                    Capitalization::Lower,
                    Some(app.settings.capitalization),
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
                    app.settings.follow_for_caps_insensitive
                )
                .text_size(12)
                .size(15)
                .on_toggle_maybe(follow_for_caps_insensitive_function),
                checkbox(
                    "Caps Lock sensitive keys",
                    app.settings.follow_for_caps_sensitive
                )
                .text_size(12)
                .size(15)
                .on_toggle_maybe(follow_for_sensitive_function),
            ]
        ];

        column![
            text(format!("NuhxBoard v{}", env!("CARGO_PKG_VERSION"))).size(20),
            checkbox(
                "Automatically create a desktop entry if none exists",
                app.settings.auto_desktop_entry
            )
            .text_size(12)
            .size(15)
            .on_toggle(|_| Message::ChangeSetting(Setting::AutoDesktopEntry)),
            input,
            row![
                text("Window title: ").size(12),
                text_input("NuhxBoard", app.settings.window_title.as_str())
                    .size(12)
                    .on_input(|v| Message::ChangeSetting(Setting::WindowTitle(v)))
            ]
            .align_items(iced::Alignment::Center),
            capitalization,
        ]
        .align_items(iced::Alignment::Center)
        .into()
    }

    fn title(&self, _app: &NuhxBoard, _id: window::Id) -> String {
        "Settings".to_string()
    }

    fn theme(
        &self,
        _app: &NuhxBoard,
        _id: window::Id,
    ) -> <NuhxBoard as iced::multi_window::Application>::Theme {
        Theme::Light
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadKeyboard;
impl Window<NuhxBoard> for LoadKeyboard {
    fn settings(&self) -> window::Settings {
        window::Settings {
            resizable: false,
            size: iced::Size {
                width: 300.0,
                height: 250.0,
            },
            ..Default::default()
        }
    }

    fn view(&self, app: &NuhxBoard, _id: window::Id) -> iced::Element<'_, Message, Theme> {
        column![
            text("Category:"),
            pick_list(
                app.keyboard_category_options.clone(),
                Some(app.settings.category.clone()),
                Message::ChangeKeyboardCategory,
            ),
            row![
                column![
                    text("Keyboard Layout:"),
                    SelectionList::new_with(
                        app.keyboard_options.clone().leak(),
                        |i, _| Message::LoadKeyboard(i),
                        12.0,
                        5.0,
                        <Theme as iced_aw::style::selection_list::StyleSheet>::Style::default(),
                        app.keyboard,
                        iced::Font::default(),
                    )
                ],
                column![
                    text("Keyboard Style:"),
                    SelectionList::new_with(
                        app.style_options.clone().leak(),
                        |i, _| Message::LoadStyle(i),
                        12.0,
                        5.0,
                        <Theme as iced_aw::style::selection_list::StyleSheet>::Style::default(),
                        app.style_choice,
                        iced::Font::default(),
                    )
                ],
            ]
        ]
        .into()
    }

    fn title(&self, _app: &NuhxBoard, _id: window::Id) -> String {
        "Load Keyboard".to_string()
    }

    fn theme(&self, _app: &NuhxBoard, _id: window::Id) -> Theme {
        Theme::Light
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorPopup;
impl Window<NuhxBoard> for ErrorPopup {
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

    fn theme(&self, _app: &NuhxBoard, _id: window::Id) -> Theme {
        Theme::Light
    }

    fn title(&self, _app: &NuhxBoard, _id: window::Id) -> String {
        "Error".to_string()
    }

    fn view(&self, app: &NuhxBoard, id: window::Id) -> iced::Element<'_, Message, Theme> {
        let error = app.error_windows.get(&id).unwrap();
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardProperties;
impl Window<NuhxBoard> for KeyboardProperties {
    fn settings(&self) -> window::Settings {
        window::Settings {
            size: iced::Size {
                width: 200.0,
                height: 100.0,
            },
            resizable: false,
            ..Default::default()
        }
    }

    fn view(&self, app: &NuhxBoard, _id: window::Id) -> iced::Element<'_, Message, Theme> {
        column![
            row![
                text("Width: "),
                number_input(app.config.width, f32::MAX, Message::SetWidth)
            ]
            .align_items(iced::Alignment::Center),
            row![
                text("Height: "),
                number_input(app.config.height, f32::MAX, Message::SetHeight)
            ]
            .align_items(iced::Alignment::Center)
        ]
        .into()
    }

    fn title(&self, _app: &NuhxBoard, _id: window::Id) -> String {
        "Keyboard Properties".to_string()
    }

    fn theme(&self, _app: &NuhxBoard, _id: window::Id) -> Theme {
        Theme::Light
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveKeyboardAs;
impl Window<NuhxBoard> for SaveKeyboardAs {
    fn theme(&self, _app: &NuhxBoard, _id: window::Id) -> Theme {
        Theme::Light
    }

    fn title(&self, _app: &NuhxBoard, _id: window::Id) -> String {
        "Save Keyboard As".to_string()
    }

    fn view(&self, app: &NuhxBoard, _id: window::Id) -> iced::Element<'_, Message, Theme> {
        column![
            row![
                text("Category: "),
                text_input(
                    app.settings.category.as_str(),
                    &app.save_keyboard_as_category,
                )
                .on_input(Message::ChangeSaveKeyboardAsCategory)
            ],
            row![
                text("Name: "),
                text_input(
                    &app.keyboard_options[app.keyboard.unwrap()],
                    &app.save_keyboard_as_name,
                )
                .on_input(Message::ChangeSaveKeyboardAsName)
            ],
            button("Save").on_press(Message::SaveKeyboard(Some(
                app.keyboards_path
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
impl Window<NuhxBoard> for SaveStyleAs {
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

    fn view(&self, app: &NuhxBoard, _id: window::Id) -> iced::Element<'_, Message, Theme> {
        column![
            row![
                text("Name: "),
                text_input(
                    &app.style_options[app.style_choice.unwrap()].name(),
                    &app.save_style_as_name,
                )
                .on_input(Message::ChangeSaveStyleAsName)
            ],
            checkbox("Save as global", app.save_style_as_global)
                .on_toggle(|_| Message::ToggleSaveStyleAsGlobal),
            button("Save").on_press(Message::SaveStyle(Some(match app.save_style_as_global {
                true => app
                    .keyboards_path
                    .join("global")
                    .join(format!("{}.style", &app.save_style_as_name)),
                false => app
                    .keyboards_path
                    .join(&app.settings.category)
                    .join(&app.keyboard_options[app.keyboard.unwrap()])
                    .join(format!("{}.style", &app.save_style_as_name)),
            }))),
        ]
        .into()
    }

    fn title(&self, _app: &NuhxBoard, _id: window::Id) -> String {
        "Save Style As".to_string()
    }

    fn theme(&self, _app: &NuhxBoard, _id: window::Id) -> Theme {
        Theme::Light
    }
}
