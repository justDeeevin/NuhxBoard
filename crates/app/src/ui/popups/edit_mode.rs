use crate::{message::*, nuhxboard::*, nuhxboard_types::*, ui::components::*};
use iced::{
    font::Weight,
    widget::{
        button, checkbox, column, row, text,
        text::{IntoFragment, Text},
        text_input,
    },
    window, Alignment, Font, Length, Padding, Theme,
};
use iced_aw::{helpers::selection_list_with, number_input, selection_list};
use iced_multi_window::Window;
use types::config::{BoardElement, SerializablePoint};

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
                    text_input("", app.text_input.keyboard_background_image.as_str())
                        .on_input(|v| Message::ChangeTextInput(
                            TextInputType::KeyboardBackgroundImage,
                            v
                        ))
                        .on_submit(Message::ChangeStyle(StyleSetting::KeyboardBackgroundImage))
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
                    .on_submit(Message::ChangeStyle(
                        StyleSetting::DefaultLooseKeyBackgroundImage
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
                    text_input("", app.text_input.default_loose_key_font_family.as_str())
                        .on_input(|v| Message::ChangeTextInput(
                            TextInputType::DefaultLooseKeyFontFamily,
                            v
                        ))
                        .on_submit(Message::ChangeStyle(
                            StyleSetting::DefaultLooseKeyFontFamily
                        ))
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
                    .on_submit(Message::ChangeStyle(
                        StyleSetting::DefaultPressedKeyBackgroundImage
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
                    text_input("", app.text_input.default_pressed_key_font_family.as_str())
                        .on_input(|v| Message::ChangeTextInput(
                            TextInputType::DefaultPressedKeyFontFamily,
                            v
                        ))
                        .on_submit(Message::ChangeStyle(
                            StyleSetting::DefaultPressedKeyFontFamily
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementProperties {
    // An even more powerful example of window state. You can have multiple `ElementProperty`
    // menus open at once, and they each will remember their associated element.
    pub index: usize,
}

impl Window<NuhxBoard, Theme, Message> for ElementProperties {
    fn id(&self) -> String {
        format!("element_properties_{}", self.index)
    }

    fn settings(&self) -> window::Settings {
        window::Settings {
            // TODO: Window size
            // resizable: false,
            size: iced::Size {
                width: 400.0,
                height: 100.0,
            },
            ..Default::default()
        }
    }

    fn view<'a>(&'a self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        let element = &app.layout.elements[self.index];
        let index = self.index;
        match element {
            BoardElement::KeyboardKey(def) => {
                let column_1 = column![
                    labeled_text_input(
                        "Text: ",
                        text_input("", &def.text).on_input(|v| Message::ChangeElement(
                            self.index,
                            ElementProperty::Text(v)
                        ))
                    ),
                    labeled_text_input(
                        "Shift Text: ",
                        text_input("", &def.shift_text).on_input(|v| Message::ChangeElement(
                            self.index,
                            ElementProperty::ShiftText(v)
                        ))
                    ),
                    row![
                        text("Text Position: "),
                        number_input(def.text_position.x, 0.0.., move |v| {
                            Message::ChangeElement(index, ElementProperty::TextPositionX(v))
                        }),
                        number_input(def.text_position.y, 0.0.., move |v| {
                            Message::ChangeElement(index, ElementProperty::TextPositionY(v))
                        }),
                        button("Center").on_press(Message::CenterTextPosition(index)),
                    ]
                    .align_y(Alignment::Center),
                    row![
                        text("Boundaries: "),
                        number_input(
                            app.number_input
                                .boundary_x
                                .get(&index)
                                .copied()
                                .unwrap_or_default(),
                            0.0..,
                            move |v| {
                                Message::ChangeNumberInput(NumberInputType::BoundaryX(index, v))
                            }
                        ),
                        number_input(
                            app.number_input
                                .boundary_y
                                .get(&index)
                                .copied()
                                .unwrap_or_default(),
                            0.0..,
                            move |v| {
                                Message::ChangeNumberInput(NumberInputType::BoundaryY(index, v))
                            }
                        ),
                    ],
                    row![
                        column![
                            button("Add").on_press(Message::ChangeElement(
                                self.index,
                                ElementProperty::Boundary(
                                    def.boundaries.len(),
                                    Some(SerializablePoint {
                                        x: app
                                            .number_input
                                            .boundary_x
                                            .get(&self.index)
                                            .copied()
                                            .unwrap_or_default(),
                                        y: app
                                            .number_input
                                            .boundary_y
                                            .get(&self.index)
                                            .copied()
                                            .unwrap_or_default()
                                    })
                                )
                            )),
                            button("Update").on_press_maybe(
                                app.selections.boundary.get(&self.index).map(|v| {
                                    Message::ChangeElement(
                                        self.index,
                                        ElementProperty::Boundary(
                                            *v,
                                            Some(SerializablePoint {
                                                x: app
                                                    .number_input
                                                    .boundary_x
                                                    .get(&self.index)
                                                    .copied()
                                                    .unwrap_or_default(),
                                                y: app
                                                    .number_input
                                                    .boundary_y
                                                    .get(&self.index)
                                                    .copied()
                                                    .unwrap_or_default(),
                                            }),
                                        ),
                                    )
                                })
                            ),
                            button("Remove").on_press_maybe(
                                app.selections.boundary.get(&self.index).map(|v| {
                                    Message::ChangeElement(
                                        self.index,
                                        ElementProperty::Boundary(*v, None),
                                    )
                                })
                            ),
                            button("Up").on_press_maybe(
                                app.selections
                                    .boundary
                                    .get(&self.index)
                                    .filter(|v| **v != 0)
                                    .map(move |v| Message::SwapBoundaries(index, *v, v - 1))
                            ),
                            button("Down").on_press_maybe(
                                app.selections
                                    .boundary
                                    .get(&self.index)
                                    .filter(|v| **v != def.boundaries.len() - 1)
                                    .map(move |v| Message::SwapBoundaries(index, *v, v + 1))
                            ),
                            button("Rectangle").on_press_maybe(
                                if app.windows.any_of(&RectangleDialog { index: self.index }) {
                                    None
                                } else {
                                    Some(Message::Open(Box::new(RectangleDialog {
                                        index: self.index,
                                    })))
                                }
                            ),
                        ],
                        selection_list_with(
                            &def.boundaries,
                            move |i, _| {
                                Message::ChangeSelection(index, SelectionType::Boundary, i)
                            },
                            12.0,
                            Padding {
                                top: 5.0,
                                bottom: 5.0,
                                ..Default::default()
                            },
                            iced_aw::style::selection_list::primary,
                            app.selections.boundary.get(&self.index).copied(),
                            Font::default()
                        )
                    ]
                ];

                let column_2 = column![
                    checkbox("Change capitalization on Caps Lock key", def.change_on_caps)
                        .on_toggle(move |_| Message::ChangeElement(
                            index,
                            ElementProperty::FollowCaps
                        )),
                    row![
                        text("Key codes: "),
                        number_input(
                            app.number_input
                                .keycode
                                .get(&self.index)
                                .copied()
                                .unwrap_or_default(),
                            0..,
                            move |v| Message::ChangeNumberInput(NumberInputType::Keycode(index, v))
                        )
                    ]
                    .align_y(Alignment::Center),
                    row![
                        column![
                            button("Add").on_press(Message::ChangeElement(
                                self.index,
                                ElementProperty::Keycode(
                                    0,
                                    Some(
                                        app.number_input
                                            .keycode
                                            .get(&self.index)
                                            .copied()
                                            .unwrap_or_default()
                                    )
                                )
                            )),
                            button("Remove").on_press_maybe(
                                app.selections.keycode.get(&self.index).map(move |v| {
                                    Message::ChangeElement(
                                        index,
                                        ElementProperty::Keycode(*v, None),
                                    )
                                })
                            ),
                            button("Detect").on_press_maybe(
                                (!app.detecting.contains(&self.index))
                                    .then_some(Message::StartDetecting(self.index))
                            )
                        ],
                        selection_list(&def.key_codes, move |i, _| Message::ChangeSelection(
                            index,
                            SelectionType::Keycode,
                            i
                        ))
                    ]
                ];

                row![column_1, column_2].into()
            }
            _ => todo!(),
        }
    }

    fn title(&self, _app: &NuhxBoard) -> String {
        "Keyboard Key Properties".to_string()
    }

    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RectangleDialog {
    pub index: usize,
}

impl Window<NuhxBoard, Theme, Message> for RectangleDialog {
    fn id(&self) -> String {
        format!("rectangle_dialog_{}", self.index)
    }

    fn settings(&self) -> window::Settings {
        window::Settings {
            // TODO: Window size
            size: iced::Size {
                width: 400.0,
                height: 100.0,
            },
            // resizable: false,
            ..Default::default()
        }
    }

    fn view<'a>(&'a self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        let index = self.index;
        column![
            row![
                text("Position: "),
                number_input(
                    app.number_input
                        .rectangle_position_x
                        .get(&self.index)
                        .copied()
                        .unwrap_or_default(),
                    0.0..,
                    move |v| {
                        Message::ChangeNumberInput(NumberInputType::RectanglePositionX(index, v))
                    }
                ),
                number_input(
                    app.number_input
                        .rectangle_position_y
                        .get(&self.index)
                        .copied()
                        .unwrap_or_default(),
                    0.0..,
                    move |v| {
                        Message::ChangeNumberInput(NumberInputType::RectanglePositionY(index, v))
                    }
                ),
            ]
            .align_y(Alignment::Center),
            row![
                text("Size: "),
                number_input(
                    app.number_input
                        .rectangle_size_x
                        .get(&self.index)
                        .copied()
                        .unwrap_or_default(),
                    0.0..,
                    move |v| {
                        Message::ChangeNumberInput(NumberInputType::RectangleSizeX(index, v))
                    }
                ),
                number_input(
                    app.number_input
                        .rectangle_size_y
                        .get(&self.index)
                        .copied()
                        .unwrap_or_default(),
                    0.0..,
                    move |v| {
                        Message::ChangeNumberInput(NumberInputType::RectangleSizeY(index, v))
                    }
                ),
            ]
            .align_y(Alignment::Center),
            row![
                button("Cancel").on_press(Message::CloseAllOf(Box::new(self.clone()))),
                button("Apply").on_press(Message::MakeRectangle(self.index))
            ]
        ]
        .into()
    }

    fn title(&self, _app: &NuhxBoard) -> String {
        "Rectangle".to_string()
    }

    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }
}
