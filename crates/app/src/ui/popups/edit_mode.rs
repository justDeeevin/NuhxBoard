use crate::nuhxboard::*;
use crate::ui::components::*;
use iced::{
    font::Weight,
    widget::{
        button, checkbox, column, row, text,
        text::{IntoFragment, Text},
        text_input,
    },
    window, Alignment, Font, Length, Theme,
};
use iced_aw::number_input;
use iced_multi_window::Window;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardProperties;
impl Window<NuhxBoard, Theme, Message> for KeyboardProperties {
    fn id(&self) -> String {
        "keyboard_properties".into()
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
    fn id(&self) -> String {
        "save_definition_as".into()
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
                text_input("", &app.text_input.save_keyboard_as_category,).on_input(|v| {
                    Message::ChangeTextInput(TextInputType::SaveKeyboardAsCategory, v)
                })
            ],
            row![
                text("Name: "),
                text_input("", &app.text_input.save_keyboard_as_name,)
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
    fn id(&self) -> String {
        "save_style_as".into()
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
                text_input("", &app.text_input.save_style_as_name,)
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
    fn id(&self) -> String {
        "keyboard_style".into()
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
                    text_input("", app.text_input.keyboard_background_image.as_str()).on_input(
                        |v| Message::ChangeTextInput(TextInputType::KeyboardBackgroundImage, v)
                    )
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
                        "",
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
                        "",
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
