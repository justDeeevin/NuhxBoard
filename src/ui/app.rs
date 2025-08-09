use super::{components::*, keyboard::Keyboard, popups::*};
use crate::{message::*, nuhxboard::*, Args};
use clap::Parser;
use iced::{
    widget::{
        checkbox, column, container, horizontal_space, image::Handle, pick_list, radio, row, text,
        text_input, Image, Scrollable, Stack,
    },
    window, Background, Border, Color, Length, Theme,
};
use iced_aw::{number_input, ContextMenu, SelectionList};
use iced_multi_window::Window;
use nuhxboard_types::settings::*;
use std::sync::Arc;

static IMAGE: &[u8] = include_bytes!("../../media/NuhxBoard.png");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadKeyboard;
impl Window<NuhxBoard, Theme, Message> for LoadKeyboard {
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

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
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
                        &app.layout_options,
                        |i, _| Message::LoadLayout(i),
                        12.0,
                        5.0,
                        iced_aw::style::selection_list::primary,
                        app.layout_choice,
                        iced::Font::default(),
                    )
                ],
                column![
                    text("Keyboard Style:"),
                    SelectionList::new_with(
                        &app.style_options,
                        |i, _| Message::LoadStyle(i),
                        12.0,
                        5.0,
                        iced_aw::style::selection_list::primary,
                        Some(app.style_choice),
                        iced::Font::default(),
                    )
                ],
            ]
        ]
        .into()
    }

    fn title(&self, _app: &NuhxBoard) -> String {
        "Load Keyboard".to_string()
    }

    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Main;
impl Window<NuhxBoard, Theme, Message> for Main {
    fn settings(&self) -> window::Settings {
        let icon_image = image::load_from_memory(IMAGE).unwrap();
        let icon = window::icon::from_rgba(icon_image.to_rgba8().to_vec(), 256, 256).unwrap();
        let icon = if Args::parse().iced_tracing {
            None
        } else {
            Some(icon)
        };

        window::Settings {
            size: DEFAULT_WINDOW_SIZE,
            resizable: cfg!(debug_assertions),
            icon,
            ..window::Settings::default()
        }
    }

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        let keyboard = Keyboard::new(app.layout.width, app.layout.height, app);

        let context_menu = ContextMenu::new(keyboard, || {
            let mut menu = vec![
                context_menu_button("Settings")
                    .on_press_maybe(
                        (!app.windows.any_of(&SettingsWindow))
                            .then_some(Message::Open(Box::new(SettingsWindow))),
                    )
                    .into(),
                context_menu_button("Load Keyboard")
                    .on_press_maybe(
                        (!app.windows.any_of(&LoadKeyboard))
                            .then_some(Message::Open(Box::new(LoadKeyboard))),
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
                menu.push(
                    checkbox("Update Text Position", app.settings.update_text_position)
                        .on_toggle(|_| Message::ChangeSetting(Setting::UpdateTextPosition))
                        .style(|theme, status| match status {
                            checkbox::Status::Active { is_checked } => checkbox::Style {
                                text_color: Some(iced::Color::BLACK),
                                background: Background::Color(match is_checked {
                                    true => iced::Color::from_rgba(0.0, 0.4, 1.0, 0.5),
                                    false => iced::Color::TRANSPARENT,
                                }),
                                border: Border {
                                    color: iced::Color::BLACK,
                                    width: 1.0,
                                    radius: iced::border::Radius::default(),
                                },
                                icon_color: iced::Color::BLACK,
                            },
                            checkbox::Status::Hovered { is_checked: _ } => checkbox::Style {
                                border: Border {
                                    color: iced::Color::BLACK,
                                    width: 2.0,
                                    radius: iced::border::Radius::default(),
                                },
                                text_color: Some(iced::Color::BLACK),
                                ..checkbox::primary(theme, status)
                            },
                            _ => checkbox::primary(theme, status),
                        })
                        .text_size(12)
                        .size(15)
                        .into(),
                );
                if app.hovered_element.is_some() {
                    menu.push(
                        context_menu_button("Remove Element")
                            .on_press(Message::RemoveElement)
                            .into(),
                    );
                } else {
                    menu.push(
                        context_menu_button("Add Keyboard Key")
                            .on_press(Message::AddKeyboardKey)
                            .into(),
                    );
                    menu.push(
                        context_menu_button("Add Mouse Key")
                            .on_press(Message::AddMouseKey)
                            .into(),
                    );
                    menu.push(
                        context_menu_button("Add Mouse Scroll")
                            .on_press(Message::AddMouseScroll)
                            .into(),
                    );
                    menu.push(
                        context_menu_button("Add Mouse Speed Indicator")
                            .on_press(Message::AddMouseSpeedIndicator)
                            .into(),
                    );
                }
                menu.append(&mut vec![
                    seperator().into(),
                    context_menu_button("Keyboard Properties")
                        .on_press_maybe(
                            (!app.windows.any_of(&KeyboardProperties))
                                .then_some(Message::Open(Box::new(KeyboardProperties))),
                        )
                        .into(),
                    context_menu_button("Element Properties")
                        .on_press_maybe(if let Some(index) = app.hovered_element {
                            let window = ElementProperties { index };
                            (!app.windows.any_of(&window))
                                .then_some(Message::Open(Box::new(window)))
                        } else {
                            None
                        })
                        .into(),
                    context_menu_button("Keyboard Style")
                        .on_press_maybe(
                            (!app.windows.any_of(&KeyboardStyle))
                                .then_some(Message::Open(Box::new(KeyboardStyle))),
                        )
                        .into(),
                    context_menu_button("Element Style")
                        .on_press_maybe(if let Some(index) = app.hovered_element {
                            let id = app.layout.elements[index].id();
                            let window = ElementStyle { id };
                            (!app.windows.any_of(&window))
                                .then_some(Message::Open(Box::new(window)))
                        } else {
                            None
                        })
                        .into(),
                ]);
            }

            menu.append(&mut vec![
                seperator().into(),
                context_menu_button("Save Definition")
                    .on_press(Message::SaveLayout(None))
                    .into(),
                context_menu_button("Save Definition As...")
                    .on_press(Message::Open(Box::new(SaveDefinitionAs)))
                    .into(),
                context_menu_button("Save Style")
                    .on_press(Message::SaveStyle(None))
                    .into(),
                context_menu_button("Save Style As...")
                    .on_press(Message::Open(Box::new(SaveStyleAs)))
                    .into(),
                context_menu_button("Clear Pressed Keys")
                    .on_press(Message::ClearPressedKeys)
                    .into(),
                context_menu_button("Exit").on_press(Message::Exit).into(),
            ]);
            container(Scrollable::new(column(menu)))
                .style(|theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::WHITE)),
                    ..container::bordered_box(theme)
                })
                .width(Length::Fixed(150.0))
                .into()
        });
        if app.style.background_image_file_name.is_some() {
            let image = Image::new(Handle::from_path(
                KEYBOARDS_PATH.parent().unwrap().join("background.png"),
            ));
            return Stack::with_children(vec![image.into(), context_menu.into()]).into();
        }
        context_menu.into()
    }

    fn theme(&self, app: &NuhxBoard) -> Theme {
        let red = app.style.background_color.red / 255.0;
        let green = app.style.background_color.green / 255.0;
        let blue = app.style.background_color.blue / 255.0;
        let palette = iced::theme::Palette {
            background: Color::from_rgb(red, green, blue),
            ..iced::theme::Palette::DARK
        };
        Theme::Custom(Arc::new(iced::theme::Custom::new("Custom".into(), palette)))
    }

    fn title(&self, app: &NuhxBoard) -> String {
        app.settings.window_title.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsWindow;
impl Window<NuhxBoard, Theme, Message> for SettingsWindow {
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

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        let display_choices = app
            .display_options
            .iter()
            .map(|display| DisplayChoice {
                id: display.id,
                primary: display.is_primary,
            })
            .collect::<Vec<_>>();

        let mut input = vec![
            row![
                text("Mouse sensitivity: ").size(12),
                horizontal_space(),
                number_input(&app.settings.mouse_sensitivity, 0.0.., |v| {
                    Message::ChangeSetting(Setting::MouseSensitivity(v))
                })
                .size(12.0)
            ]
            .padding(5)
            .align_y(iced::Alignment::Center)
            .into(),
            row![
                text("Scroll hold time (ms): ").size(12),
                horizontal_space(),
                number_input(&app.settings.scroll_hold_time, 0.., |v| {
                    Message::ChangeSetting(Setting::ScrollHoldTime(v))
                })
                .size(12.0)
            ]
            .padding(5)
            .align_y(iced::Alignment::Center)
            .into(),
            checkbox(
                "Calculate mouse speed from center of screen",
                app.settings.mouse_from_center,
            )
            .text_size(12)
            .size(15)
            .on_toggle(|_| Message::ChangeSetting(Setting::CenterMouse))
            .into(),
        ];
        if app.display_options.len() > 1 {
            input.push(
                row![
                    text("Display to use: ").size(12),
                    pick_list(display_choices, Some(&app.settings.display_choice), |v| {
                        Message::ChangeSetting(Setting::DisplayChoice(v))
                    })
                    .text_size(12)
                ]
                .padding(5)
                .align_y(iced::Alignment::Center)
                .into(),
            );
        }
        input.extend([
            text("Show keypresses for at least").size(12).into(),
            row![
                number_input(&app.settings.min_press_time, 0.., |v| {
                    Message::ChangeSetting(Setting::MinPressTime(v))
                })
                .size(12.0)
                .width(Length::Shrink),
                text("ms").size(12)
            ]
            .padding(5)
            .align_y(iced::Alignment::Center)
            .into(),
        ]);
        let input = column(input).align_x(iced::Alignment::Center);

        let follow_for_sensitive_function =
            match app.settings.capitalization != Capitalization::Follow {
                true => Some(|_| Message::ChangeSetting(Setting::FollowForCapsSensitive)),
                false => None,
            };

        let follow_for_caps_insensitive_function = (app.settings.capitalization
            != Capitalization::Follow)
            .then_some(|_: bool| Message::ChangeSetting(Setting::FollowForCapsInsensitive));

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
            input,
            row![
                text("Window title: ").size(12),
                text_input("NuhxBoard", app.settings.window_title.as_str())
                    .size(12)
                    .on_input(|v| Message::ChangeSetting(Setting::WindowTitle(v)))
            ]
            .align_y(iced::Alignment::Center),
            capitalization,
        ]
        .align_x(iced::Alignment::Center)
        .into()
    }

    fn title(&self, _app: &NuhxBoard) -> String {
        "Settings".to_string()
    }

    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }
}
