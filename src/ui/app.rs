use crate::nuhxboard::*;
use iced::{
    border::Radius,
    font::Weight,
    widget::{
        button, canvas, checkbox, column, container, horizontal_space, image::Handle, pick_list,
        radio, row, text, text::IntoFragment, text_input, Button, Container, Image, Row,
        Scrollable, Stack, Text,
    },
    window, Alignment, Background, Border, Color, Element, Font, Length, Renderer, Shadow, Theme,
};
use iced_aw::{
    color_picker, number_input, quad::Quad, widgets::InnerBounds, ContextMenu, SelectionList,
};
use iced_multi_window::Window;
use std::sync::Arc;
use types::{settings::*, style::NohRgb};

static IMAGE: &[u8] = include_bytes!("../../NuhxBoard.png");

fn labeled_text_input<'a>(
    label: impl IntoFragment<'a>,
    text_input: iced::widget::TextInput<'a, Message>,
) -> Row<'a, Message> {
    row![text(label), text_input].align_y(Alignment::Center)
}

fn gray_box<'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content)
        .style(move |_| container::Style {
            background: None,
            text_color: None,
            border: Border {
                color: NohRgb::DEFAULT_GRAY.into(),
                width: 1.0,
                radius: Radius::from(0.0),
            },
            shadow: Shadow::default(),
        })
        .padding(5)
        .width(Length::Fill)
        .align_x(Alignment::Center)
}

fn picker_button<'a>(
    label: impl IntoFragment<'a>,
    open: bool,
    color: Color,
    picker: ColorPicker,
) -> Row<'a, Message> {
    row![
        color_picker(
            open,
            color,
            button("")
                .width(Length::Fixed(15.0))
                .height(Length::Fixed(15.0))
                .style(move |theme, status| match status {
                    button::Status::Active => button::Style {
                        background: Some(iced::Background::Color(color)),
                        border: Border {
                            color: Color::BLACK,
                            width: 1.0,
                            radius: Radius::new(0)
                        },
                        ..button::primary(theme, status)
                    },
                    _ => button::primary(theme, status),
                })
                .on_press(Message::ToggleColorPicker(picker)),
            Message::ToggleColorPicker(picker),
            move |v| Message::ChangeColor(picker, v)
        ),
        text(label)
    ]
    .align_y(Alignment::Center)
}

fn context_menu_button(label: &str) -> Button<Message> {
    let text = text(label).size(12);
    button(text)
        .style(|theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(iced::Background::Color(iced::Color::WHITE)),
                text_color: iced::Color::BLACK,
                ..button::primary(theme, status)
            },
            button::Status::Hovered => button::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(0.0, 0.0, 1.0),
                    width: 2.0,
                    radius: 0.into(),
                },
                text_color: iced::Color::BLACK,
                background: Some(iced::Background::Color(iced::Color::WHITE)),
                ..button::primary(theme, status)
            },
            button::Status::Disabled => button::Style {
                background: Some(iced::Background::Color(iced::Color::WHITE)),
                text_color: iced::Color::from_rgb(100.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0),
                ..button::primary(theme, status)
            },
            _ => button::primary(theme, status),
        })
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
impl Window<NuhxBoard, Theme, Message> for Main {
    fn id(&self) -> &'static str {
        "main"
    }

    fn settings(&self) -> window::Settings {
        let icon_image = image::load_from_memory(IMAGE).unwrap();
        let icon = window::icon::from_rgba(icon_image.to_rgba8().to_vec(), 256, 256).unwrap();

        window::Settings {
            size: DEFAULT_WINDOW_SIZE,
            resizable: false,
            icon: Some(icon),
            ..window::Settings::default()
        }
    }

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        let canvas = canvas::<&NuhxBoard, Message, Theme, Renderer>(app)
            .height(Length::Fill)
            .width(Length::Fill);

        let context_menu = ContextMenu::new(canvas, || {
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
                menu.append(&mut vec![
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
                    seperator().into(),
                    context_menu_button("Keyboard Properties")
                        .on_press_maybe(
                            (!app.windows.any_of(&KeyboardProperties))
                                .then_some(Message::Open(Box::new(KeyboardProperties))),
                        )
                        .into(),
                    context_menu_button("Element Properties").into(),
                    context_menu_button("Keyboard Style")
                        .on_press_maybe(
                            (!app.windows.any_of(&KeyboardStyle))
                                .then_some(Message::Open(Box::new(KeyboardStyle))),
                        )
                        .into(),
                    context_menu_button("Element Style").into(),
                ]);
            }

            menu.append(&mut vec![
                seperator().into(),
                context_menu_button("Save Definition")
                    .on_press(Message::SaveKeyboard(None))
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
                app.keyboards_path.parent().unwrap().join("background.png"),
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
    fn id(&self) -> &'static str {
        "settings"
    }

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
                number_input(app.settings.mouse_sensitivity, 0.0.., |v| {
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
                number_input(app.settings.scroll_hold_time, 0.., |v| {
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
                number_input(app.settings.min_press_time, 0.., |v| {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadKeyboard;
impl Window<NuhxBoard, Theme, Message> for LoadKeyboard {
    fn id(&self) -> &'static str {
        "load_keyboard"
    }

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
                        app.keyboard_options.clone().leak(),
                        |i, _| Message::LoadLayout(i),
                        12.0,
                        5.0,
                        iced_aw::style::selection_list::primary,
                        app.keyboard_choice,
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
                        iced_aw::style::selection_list::primary,
                        app.style_choice,
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
pub struct ErrorPopup {
    pub error: Error,
}
impl Window<NuhxBoard, Theme, Message> for ErrorPopup {
    fn id(&self) -> &'static str {
        "error"
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardProperties;
impl Window<NuhxBoard, Theme, Message> for KeyboardProperties {
    fn id(&self) -> &'static str {
        "keyboard_properties"
    }

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

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        column![
            row![
                text("Width: "),
                number_input(app.layout.width, 0.0.., Message::SetWidth)
            ]
            .align_y(iced::Alignment::Center),
            row![
                text("Height: "),
                number_input(app.layout.height, 0.0.., Message::SetHeight)
            ]
            .align_y(iced::Alignment::Center)
        ]
        .into()
    }

    fn title(&self, _app: &NuhxBoard) -> String {
        "Keyboard Properties".to_string()
    }

    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveDefinitionAs;
impl Window<NuhxBoard, Theme, Message> for SaveDefinitionAs {
    fn id(&self) -> &'static str {
        "save_definition_as"
    }

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
                text_input(
                    app.settings.category.as_str(),
                    &app.text_input.save_keyboard_as_category,
                )
                .on_input(|v| Message::ChangeTextInput(TextInputType::SaveKeyboardAsCategory, v))
            ],
            row![
                text("Name: "),
                text_input(
                    &app.keyboard_options[app.keyboard_choice.unwrap()],
                    &app.text_input.save_keyboard_as_name,
                )
                .on_input(|v| Message::ChangeTextInput(TextInputType::SaveKeyboardAsName, v))
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
impl Window<NuhxBoard, Theme, Message> for SaveStyleAs {
    fn id(&self) -> &'static str {
        "save_style_as"
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

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        column![
            row![
                text("Name: "),
                text_input(
                    &app.style_options[app.style_choice.unwrap()].name(),
                    &app.text_input.save_style_as_name,
                )
                .on_input(|v| Message::ChangeTextInput(TextInputType::SaveStyleAsName, v))
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardStyle;

impl Window<NuhxBoard, Theme, Message> for KeyboardStyle {
    fn id(&self) -> &'static str {
        "keyboard_style"
    }

    fn settings(&self) -> window::Settings {
        window::Settings {
            ..Default::default()
        }
    }

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        fn category_label<'a>(label: impl IntoFragment<'a>) -> Text<'a> {
            text(label)
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                })
                .align_x(Alignment::Center)
                .width(Length::Fill)
        }

        let keyboard = column![
            category_label("Keyboard"),
            text("Background"),
            gray_box(column![
                picker_button(
                    "Background Color",
                    app.color_pickers.keyboard_background,
                    app.style.background_color.into(),
                    ColorPicker::KeyboardBackground,
                ),
                labeled_text_input(
                    "Image: ",
                    text_input(
                        app.style
                            .background_image_file_name
                            .as_deref()
                            .unwrap_or(""),
                        app.text_input.keyboard_background_image.as_str()
                    )
                    .on_input(|v| Message::ChangeTextInput(
                        TextInputType::KeyboardBackgroundImage,
                        v
                    ))
                )
            ])
        ];

        let mouse_speed_indicator = column![
            category_label("MouseSpeedIndicator"),
            text("General"),
            gray_box(column![
                picker_button(
                    "Color 1 (low speed)",
                    app.color_pickers.default_mouse_speed_indicator_1,
                    app.style
                        .default_mouse_speed_indicator_style
                        .inner_color
                        .into(),
                    ColorPicker::DefaultMouseSpeedIndicator1
                ),
                picker_button(
                    "Color 2 (high speed)",
                    app.color_pickers.default_mouse_speed_indicator_2,
                    app.style
                        .default_mouse_speed_indicator_style
                        .outer_color
                        .into(),
                    ColorPicker::DefaultMouseSpeedIndicator2
                ),
                row![
                    number_input(
                        app.style.default_mouse_speed_indicator_style.outline_width,
                        1..,
                        |v| Message::ChangeStyle(
                            StyleSetting::DefaultMouseSpeedIndicatorOutlineWidth(v)
                        )
                    ),
                    text(" Outline Width")
                ]
                .align_y(Alignment::Center)
            ])
        ];

        let loose_keys = column![
            category_label("Loose Keys"),
            text("Background"),
            gray_box(column![
                picker_button(
                    "Background Color",
                    app.color_pickers.default_loose_background,
                    app.style.default_key_style.loose.background.into(),
                    ColorPicker::DefaultLooseBackground
                ),
                labeled_text_input(
                    "Image: ",
                    text_input(
                        app.style
                            .default_key_style
                            .loose
                            .background_image_file_name
                            .as_deref()
                            .unwrap_or(""),
                        app.text_input.default_loose_key_background_image.as_str()
                    )
                    .on_input(|v| Message::ChangeTextInput(
                        TextInputType::DefaultLooseKeyBackgroundImage,
                        v
                    ))
                )
            ]),
            text("Text"),
            gray_box(column![
                picker_button(
                    "Text Color",
                    app.color_pickers.default_loose_text,
                    app.style.default_key_style.loose.text.into(),
                    ColorPicker::DefaultLooseText
                ),
                labeled_text_input(
                    "Font Family: ",
                    text_input(
                        "",
                        app.style.default_key_style.loose.font.font_family.as_str()
                    )
                    .on_input(|v| Message::ChangeStyle(StyleSetting::DefaultLooseKeyFontFamily(v)))
                )
            ]),
            text("Outline"),
            gray_box(column![
                picker_button(
                    "Outline Color",
                    app.color_pickers.default_loose_outline,
                    app.style.default_key_style.loose.outline.into(),
                    ColorPicker::DefaultLooseOutline
                ),
                checkbox(
                    "Show Outline",
                    app.style.default_key_style.loose.show_outline
                )
                .on_toggle(|_| Message::ChangeStyle(StyleSetting::DefaultLooseKeyShowOutline)),
                row![
                    number_input(app.style.default_key_style.loose.outline_width, 1.., |v| {
                        Message::ChangeStyle(StyleSetting::DefaultLooseKeyOutlineWidth(v))
                    }),
                    text(" Outline Width")
                ]
            ])
        ]
        .padding(5);

        let pressed_keys = column![
            category_label("Pressed Keys"),
            text("Background"),
            gray_box(column![
                picker_button(
                    "Background Color",
                    app.color_pickers.default_pressed_background,
                    app.style.default_key_style.pressed.background.into(),
                    ColorPicker::DefaultPressedBackground
                ),
                labeled_text_input(
                    "Image: ",
                    text_input(
                        app.style
                            .default_key_style
                            .pressed
                            .background_image_file_name
                            .as_deref()
                            .unwrap_or(""),
                        app.text_input.default_pressed_key_background_image.as_str()
                    )
                    .on_input(|v| Message::ChangeTextInput(
                        TextInputType::DefaultPressedKeyBackgroundImage,
                        v
                    ))
                )
            ]),
            text("Text"),
            gray_box(column![
                picker_button(
                    "Text Color",
                    app.color_pickers.default_pressed_text,
                    app.style.default_key_style.pressed.text.into(),
                    ColorPicker::DefaultPressedText
                ),
                labeled_text_input(
                    "Font Family: ",
                    text_input(
                        "",
                        app.style
                            .default_key_style
                            .pressed
                            .font
                            .font_family
                            .as_str()
                    )
                    .on_input(|v| Message::ChangeStyle(
                        StyleSetting::DefaultPressedKeyFontFamily(v)
                    ))
                )
            ]),
            text("Outline"),
            gray_box(column![
                picker_button(
                    "Outline Color",
                    app.color_pickers.default_pressed_outline,
                    app.style.default_key_style.pressed.outline.into(),
                    ColorPicker::DefaultPressedOutline
                ),
                checkbox(
                    "Show Outline",
                    app.style.default_key_style.pressed.show_outline
                )
                .on_toggle(|_| Message::ChangeStyle(StyleSetting::DefaultPressedKeyShowOutline)),
                row![
                    number_input(
                        app.style.default_key_style.pressed.outline_width,
                        1..,
                        |v| {
                            Message::ChangeStyle(StyleSetting::DefaultPressedKeyOutlineWidth(v))
                        }
                    ),
                    text(" Outline Width")
                ]
            ])
        ]
        .padding(5);

        row![
            column![keyboard, mouse_speed_indicator].padding(5),
            loose_keys,
            pressed_keys,
        ]
        .into()
    }

    fn title<'a>(&'a self, _app: &'a NuhxBoard) -> String {
        "Keyboard Style".to_string()
    }

    fn theme<'a>(&'a self, _app: &'a NuhxBoard) -> Theme {
        Theme::Light
    }
}
