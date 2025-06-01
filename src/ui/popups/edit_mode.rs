use crate::{message::*, nuhxboard::*, types::*, ui::components::*};
use iced::{
    widget::{button, checkbox, column, pick_list, rich_text, row, span, text, text_input},
    window, Alignment, Font, Padding, Theme,
};
use iced_aw::{helpers::selection_list_with, number_input, selection_list};
use iced_multi_window::Window;
use nuhxboard_types::{
    layout::{BoardElement, CommonDefinitionRef, OrderedFloat, SerializablePoint},
    style::{self, FontStyle},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardProperties;
impl Window<NuhxBoard, Theme, Message> for KeyboardProperties {
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
                number_input(&app.layout.width, 0.0.., Message::SetWidth)
            ]
            .align_y(iced::Alignment::Center),
            row![
                text("Height: "),
                number_input(&app.layout.height, 0.0.., Message::SetHeight)
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
                text_input("", &app.save_keyboard_as_category,).on_input(|v| {
                    Message::ChangeTextInput(TextInputType::SaveKeyboardAsCategory, v)
                })
            ],
            row![
                text("Name: "),
                text_input("", &app.save_keyboard_as_name,)
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
                text_input("", &app.save_style_as_name,)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardStyle;

impl Window<NuhxBoard, Theme, Message> for KeyboardStyle {
    fn settings(&self) -> window::Settings {
        window::Settings {
            size: iced::Size {
                width: 800.0,
                height: 350.0,
            },
            resizable: false,
            ..Default::default()
        }
    }

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
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
                    ColorPicker::DefaultMouseSpeedIndicator1,
                ),
                picker_button(
                    "Color 2 (high speed)",
                    app.color_pickers.default_mouse_speed_indicator_2,
                    app.style
                        .default_mouse_speed_indicator_style
                        .outer_color
                        .into(),
                    ColorPicker::DefaultMouseSpeedIndicator2,
                ),
                row![
                    number_input(
                        &app.style.default_mouse_speed_indicator_style.outline_width,
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

        let loose = &app.style.default_key_style.loose;
        let loose_keys = column![
            category_label("Loose Keys"),
            text("Background"),
            gray_box(column![
                picker_button(
                    "Background Color",
                    app.color_pickers.default_loose_background,
                    loose.background.into(),
                    ColorPicker::DefaultLooseBackground,
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
                    loose.text.into(),
                    ColorPicker::DefaultLooseText,
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
                    loose.outline.into(),
                    ColorPicker::DefaultLooseOutline,
                ),
                checkbox("Show Outline", loose.show_outline)
                    .on_toggle(|_| Message::ChangeStyle(StyleSetting::DefaultLooseKeyShowOutline)),
                row![
                    number_input(&loose.outline_width, 1.., |v| {
                        Message::ChangeStyle(StyleSetting::DefaultLooseKeyOutlineWidth(v))
                    }),
                    text(" Outline Width")
                ]
                .align_y(Alignment::Center)
            ])
        ]
        .padding(5);

        let pressed = &app.style.default_key_style.pressed;
        let pressed_keys = column![
            category_label("Pressed Keys"),
            text("Background"),
            gray_box(column![
                picker_button(
                    "Background Color",
                    app.color_pickers.default_pressed_background,
                    pressed.background.into(),
                    ColorPicker::DefaultPressedBackground,
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
                    pressed.text.into(),
                    ColorPicker::DefaultPressedText,
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
                    pressed.outline.into(),
                    ColorPicker::DefaultPressedOutline,
                ),
                checkbox("Show Outline", pressed.show_outline).on_toggle(|_| Message::ChangeStyle(
                    StyleSetting::DefaultPressedKeyShowOutline
                )),
                row![
                    number_input(&pressed.outline_width, 1.., |v| {
                        Message::ChangeStyle(StyleSetting::DefaultPressedKeyOutlineWidth(v))
                    }),
                    text(" Outline Width")
                ]
                .align_y(Alignment::Center)
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
    fn settings(&self) -> window::Settings {
        window::Settings {
            resizable: false,
            size: iced::Size {
                width: 900.0,
                height: 500.0,
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
                        number_input(&def.text_position.x, OrderedFloat(0.0).., move |v| {
                            Message::ChangeElement(index, ElementProperty::TextPositionX(*v))
                        }),
                        number_input(&def.text_position.y, OrderedFloat(0.0).., move |v| {
                            Message::ChangeElement(index, ElementProperty::TextPositionY(*v))
                        }),
                        button("Center").on_press(Message::CenterTextPosition(index)),
                    ]
                    .align_y(Alignment::Center),
                    row![
                        text("Boundaries: "),
                        number_input(
                            &app.number_input
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
                            &app.number_input
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
                                            .unwrap_or_default()
                                            .into(),
                                        y: app
                                            .number_input
                                            .boundary_y
                                            .get(&self.index)
                                            .copied()
                                            .unwrap_or_default()
                                            .into()
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
                                                    .unwrap_or_default()
                                                    .into(),
                                                y: app
                                                    .number_input
                                                    .boundary_y
                                                    .get(&self.index)
                                                    .copied()
                                                    .unwrap_or_default()
                                                    .into(),
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
                            &app.number_input
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
            BoardElement::MouseKey(_) | BoardElement::MouseScroll(_) => {
                let def = CommonDefinitionRef::try_from(element).unwrap();
                column![
                    match element {
                        BoardElement::MouseKey(_) => row![
                            text("Button: "),
                            pick_list(
                                {
                                    use MouseKey::*;
                                    [Left, Middle, Right, Forward, Back]
                                },
                                Some::<MouseKey>(def.key_codes[0].try_into().unwrap()),
                                move |v| Message::ChangeElement(
                                    self.index,
                                    ElementProperty::Keycode(0, Some(v.into()))
                                )
                            )
                        ],
                        BoardElement::MouseScroll(_) => row![
                            text("Scroll Direction: "),
                            pick_list(
                                {
                                    use MouseScroll::*;
                                    [Up, Down, Left, Right]
                                },
                                Some::<MouseScroll>(def.key_codes[0].try_into().unwrap()),
                                move |v| Message::ChangeElement(
                                    self.index,
                                    ElementProperty::Keycode(0, Some(v.into()))
                                )
                            )
                        ],
                        _ => unreachable!(),
                    }
                    .align_y(Alignment::Center),
                    labeled_text_input(
                        "Text: ",
                        text_input("", def.text).on_input(move |v| Message::ChangeElement(
                            index,
                            ElementProperty::Text(v)
                        ))
                    ),
                    row![
                        text("Text Position: "),
                        number_input(&def.text_position.x, OrderedFloat(0.0).., move |v| {
                            Message::ChangeElement(index, ElementProperty::TextPositionX(*v))
                        }),
                        number_input(&def.text_position.y, OrderedFloat(0.0).., move |v| {
                            Message::ChangeElement(index, ElementProperty::TextPositionY(*v))
                        }),
                        button("Center").on_press(Message::CenterTextPosition(index)),
                    ]
                    .align_y(Alignment::Center),
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
                                            .unwrap_or_default()
                                            .into(),
                                        y: app
                                            .number_input
                                            .boundary_y
                                            .get(&self.index)
                                            .copied()
                                            .unwrap_or_default()
                                            .into()
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
                                                    .unwrap_or_default()
                                                    .into(),
                                                y: app
                                                    .number_input
                                                    .boundary_y
                                                    .get(&self.index)
                                                    .copied()
                                                    .unwrap_or_default()
                                                    .into(),
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
                            def.boundaries,
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
                ]
                .into()
            }
            BoardElement::MouseSpeedIndicator(def) => column![
                row![
                    text("Location: "),
                    number_input(&def.location.x, OrderedFloat(0.0).., move |v| {
                        Message::ChangeElement(
                            index,
                            ElementProperty::MouseSpeedIndicatorPositionX(*v),
                        )
                    }),
                    number_input(&def.location.y, OrderedFloat(0.0).., move |v| {
                        Message::ChangeElement(
                            index,
                            ElementProperty::MouseSpeedIndicatorPositionY(*v),
                        )
                    })
                ]
                .align_y(Alignment::Center),
                row![
                    text("Radius: "),
                    number_input(&def.radius, 0.0.., move |v| Message::ChangeElement(
                        index,
                        ElementProperty::MouseSpeedIndicatorRadius(v)
                    ))
                ]
                .align_y(Alignment::Center)
            ]
            .into(),
        }
    }

    fn title(&self, app: &NuhxBoard) -> String {
        match app.layout.elements[self.index] {
            BoardElement::KeyboardKey(_) => "Keyboard Key Properties".to_string(),
            BoardElement::MouseKey(_) => "Mouse Key Properties".to_string(),
            BoardElement::MouseScroll(_) => "Mouse Scroll Properties".to_string(),
            BoardElement::MouseSpeedIndicator(_) => "Mouse Speed Indicator Properties".to_string(),
        }
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
    fn settings(&self) -> window::Settings {
        window::Settings {
            resizable: false,
            size: iced::Size {
                width: 400.0,
                height: 100.0,
            },
            ..Default::default()
        }
    }

    fn view<'a>(&'a self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        let index = self.index;
        column![
            row![
                text("Position: "),
                number_input(
                    &app.number_input
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
                    &app.number_input
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
                    &app.number_input
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
                    &app.number_input
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementStyle {
    pub id: u32,
}

impl Window<NuhxBoard, Theme, Message> for ElementStyle {
    fn settings(&self) -> window::Settings {
        window::Settings {
            resizable: false,
            size: iced::Size {
                width: 500.0,
                height: 400.0,
            },
            ..Default::default()
        }
    }

    fn title(&self, app: &NuhxBoard) -> String {
        let default = match app
            .layout
            .elements
            .iter()
            .find(move |v| v.id() == self.id)
            .unwrap()
        {
            BoardElement::KeyboardKey(_)
            | BoardElement::MouseKey(_)
            | BoardElement::MouseScroll(_) => {
                style::ElementStyle::KeyStyle(app.style.default_key_style.clone().into())
            }
            BoardElement::MouseSpeedIndicator(_) => style::ElementStyle::MouseSpeedIndicatorStyle(
                app.style.default_mouse_speed_indicator_style.clone(),
            ),
        };
        let style = app.style.element_styles.get(&self.id).unwrap_or(&default);
        match style {
            style::ElementStyle::KeyStyle(_) => "Key Style".into(),
            style::ElementStyle::MouseSpeedIndicatorStyle(_) => {
                "Mouse Speed Indicator Style".into()
            }
        }
    }

    fn theme(&self, _app: &NuhxBoard) -> Theme {
        Theme::Light
    }

    fn view<'a>(&'a self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        let id = self.id;
        let default = match app
            .layout
            .elements
            .iter()
            .find(move |v| v.id() == id)
            .unwrap()
        {
            BoardElement::KeyboardKey(_)
            | BoardElement::MouseKey(_)
            | BoardElement::MouseScroll(_) => {
                style::ElementStyle::KeyStyle(app.style.default_key_style.clone().into())
            }
            BoardElement::MouseSpeedIndicator(_) => style::ElementStyle::MouseSpeedIndicatorStyle(
                app.style.default_mouse_speed_indicator_style.clone(),
            ),
        };
        let style = app.style.element_styles.get(&self.id).unwrap_or(&default);

        match style {
            style::ElementStyle::KeyStyle(style) => {
                let default_loose = default.as_key_style().unwrap().loose.as_ref().unwrap();
                let loose = style.loose.as_ref().unwrap_or(default_loose);
                let column_1 = column![
                    category_label("Loose"),
                    text("Background"),
                    gray_box(column![
                        picker_button(
                            "Background Color",
                            app.color_pickers
                                .loose_background
                                .get(&self.id)
                                .copied()
                                .unwrap_or(false),
                            loose.background.into(),
                            ColorPicker::LooseBackground(self.id),
                        ),
                        labeled_text_input(
                            "Image",
                            text_input(
                                "",
                                app.text_input
                                    .loose_background_image
                                    .get(&self.id)
                                    .map(|v| v.as_str())
                                    .unwrap_or_default()
                            )
                            .on_input(move |v| {
                                Message::ChangeTextInput(TextInputType::LooseBackgroundImage(id), v)
                            })
                            .on_submit(Message::ChangeStyle(
                                StyleSetting::LooseKeyBackgroundImage(self.id)
                            ))
                        ),
                    ]),
                    text("Text"),
                    gray_box(column![
                        picker_button(
                            "Text Color",
                            app.color_pickers
                                .loose_text
                                .get(&self.id)
                                .copied()
                                .unwrap_or(false),
                            loose.text.into(),
                            ColorPicker::LooseText(self.id),
                        ),
                        {
                            let font = &loose.font;
                            rich_text![span("Pick a font").font(font.as_iced())]
                        },
                        labeled_text_input(
                            "Font Family: ",
                            text_input(
                                "",
                                app.text_input
                                    .loose_font_family
                                    .get(&self.id)
                                    .map(|v| v.as_str())
                                    .unwrap_or_default()
                            )
                            .on_input(move |v| Message::ChangeTextInput(
                                TextInputType::LooseFontFamily(id),
                                v
                            ))
                            .on_submit(Message::ChangeStyle(StyleSetting::LooseKeyFontFamily(
                                self.id
                            )))
                        ),
                        row![
                            checkbox("Bold ", loose.font.style.contains(FontStyle::BOLD))
                                .on_toggle(move |_| {
                                    Message::ChangeStyle(StyleSetting::LooseKeyFontStyle {
                                        id,
                                        style: FontStyle::BOLD,
                                    })
                                }),
                            checkbox("Italic", loose.font.style.contains(FontStyle::ITALIC))
                                .on_toggle(move |_| {
                                    Message::ChangeStyle(StyleSetting::LooseKeyFontStyle {
                                        id,
                                        style: FontStyle::ITALIC,
                                    })
                                }),
                        ],
                        row![
                            checkbox(
                                "Underline ",
                                loose.font.style.contains(FontStyle::UNDERLINE)
                            )
                            .on_toggle(move |_| Message::ChangeStyle(
                                StyleSetting::LooseKeyFontStyle {
                                    id,
                                    style: FontStyle::UNDERLINE
                                }
                            )),
                            checkbox(
                                "Strikethrough",
                                loose.font.style.contains(FontStyle::STRIKETHROUGH)
                            )
                            .on_toggle(move |_| Message::ChangeStyle(
                                StyleSetting::LooseKeyFontStyle {
                                    id,
                                    style: FontStyle::STRIKETHROUGH
                                }
                            )),
                        ]
                    ]),
                    text("Outline"),
                    gray_box(column![
                        picker_button(
                            "Outline Color",
                            app.color_pickers
                                .loose_outline
                                .get(&self.id)
                                .copied()
                                .unwrap_or(false),
                            loose.outline.into(),
                            ColorPicker::LooseOutline(self.id),
                        ),
                        checkbox("Show Outline", loose.show_outline).on_toggle(move |_| {
                            Message::ChangeStyle(StyleSetting::LooseKeyShowOutline(id))
                        }),
                        row![
                            number_input(&loose.outline_width, 1.., move |v| {
                                Message::ChangeStyle(StyleSetting::LooseKeyOutlineWidth {
                                    id,
                                    width: v,
                                })
                            }),
                            text(" Outline Width"),
                        ]
                    ]),
                ];

                let default_pressed = default.as_key_style().unwrap().pressed.as_ref().unwrap();
                let pressed = style.pressed.as_ref().unwrap_or(default_pressed);
                let column_2 = column![
                    category_label("Pressed"),
                    text("Background"),
                    gray_box(column![
                        picker_button(
                            "Background Color",
                            app.color_pickers
                                .pressed_background
                                .get(&self.id)
                                .copied()
                                .unwrap_or(false),
                            pressed.background.into(),
                            ColorPicker::PressedBackground(self.id),
                        ),
                        labeled_text_input(
                            "Image",
                            text_input(
                                "",
                                app.text_input
                                    .pressed_background_image
                                    .get(&self.id)
                                    .map(|v| v.as_str())
                                    .unwrap_or_default()
                            )
                            .on_input(move |v| {
                                Message::ChangeTextInput(
                                    TextInputType::PressedBackgroundImage(id),
                                    v,
                                )
                            })
                            .on_submit(Message::ChangeStyle(
                                StyleSetting::PressedKeyBackgroundImage(self.id)
                            ))
                        ),
                    ]),
                    text("Text"),
                    gray_box(column![
                        picker_button(
                            "Text Color",
                            app.color_pickers
                                .pressed_text
                                .get(&self.id)
                                .copied()
                                .unwrap_or(false),
                            pressed.text.into(),
                            ColorPicker::PressedText(self.id),
                        ),
                        {
                            let font = &pressed.font;
                            rich_text![span("Pick a font")
                                .font(font.as_iced())
                                .underline(font.style.contains(FontStyle::UNDERLINE))
                                .strikethrough(font.style.contains(FontStyle::STRIKETHROUGH))]
                        },
                        text_input(
                            "",
                            app.text_input
                                .pressed_font_family
                                .get(&self.id)
                                .map(|v| v.as_str())
                                .unwrap_or_default()
                        )
                        .on_input(move |v| Message::ChangeTextInput(
                            TextInputType::PressedFontFamily(id),
                            v
                        ))
                        .on_submit(Message::ChangeStyle(
                            StyleSetting::PressedKeyFontFamily(self.id)
                        )),
                        row![
                            checkbox("Bold ", pressed.font.style.contains(FontStyle::BOLD))
                                .on_toggle(move |_| {
                                    Message::ChangeStyle(StyleSetting::PressedKeyFontStyle {
                                        id,
                                        style: FontStyle::BOLD,
                                    })
                                }),
                            checkbox("Italic", pressed.font.style.contains(FontStyle::ITALIC))
                                .on_toggle(move |_| Message::ChangeStyle(
                                    StyleSetting::PressedKeyFontStyle {
                                        id,
                                        style: FontStyle::ITALIC
                                    }
                                )),
                        ],
                        row![
                            checkbox(
                                "Underline ",
                                pressed.font.style.contains(FontStyle::UNDERLINE)
                            )
                            .on_toggle(move |_| Message::ChangeStyle(
                                StyleSetting::PressedKeyFontStyle {
                                    id,
                                    style: FontStyle::UNDERLINE
                                }
                            )),
                            checkbox(
                                "Strikethrough",
                                pressed.font.style.contains(FontStyle::STRIKETHROUGH)
                            )
                            .on_toggle(move |_| Message::ChangeStyle(
                                StyleSetting::PressedKeyFontStyle {
                                    id,
                                    style: FontStyle::STRIKETHROUGH
                                }
                            )),
                        ]
                    ]),
                    text("Outline"),
                    gray_box(column![
                        picker_button(
                            "Outline Color",
                            app.color_pickers
                                .pressed_outline
                                .get(&self.id)
                                .copied()
                                .unwrap_or(false),
                            pressed.outline.into(),
                            ColorPicker::PressedOutline(self.id),
                        ),
                        checkbox("Show Outline", pressed.show_outline).on_toggle(move |_| {
                            Message::ChangeStyle(StyleSetting::PressedKeyShowOutline(id))
                        }),
                        row![
                            number_input(&pressed.outline_width, 1.., move |v| {
                                Message::ChangeStyle(StyleSetting::PressedKeyOutlineWidth {
                                    id,
                                    width: v,
                                })
                            }),
                            text(" Outline Width"),
                        ]
                    ]),
                ];

                row![column_1, column_2].into()
            }
            style::ElementStyle::MouseSpeedIndicatorStyle(style) => column![
                picker_button(
                    "Color 1 (low speed)",
                    app.color_pickers
                        .mouse_speed_indicator_1
                        .get(&self.id)
                        .copied()
                        .unwrap_or(false),
                    style.inner_color.into(),
                    ColorPicker::MouseSpeedIndicator1(self.id)
                ),
                picker_button(
                    "Color 2 (high speed)",
                    app.color_pickers
                        .mouse_speed_indicator_2
                        .get(&self.id)
                        .copied()
                        .unwrap_or(false),
                    style.outer_color.into(),
                    ColorPicker::MouseSpeedIndicator2(self.id)
                ),
                row![
                    number_input(&style.outline_width, 0.., move |v| {
                        Message::ChangeStyle(StyleSetting::MouseSpeedIndicatorOutlineWidth {
                            id,
                            width: v,
                        })
                    }),
                    text(" Outline Width")
                ]
                .align_y(Alignment::Center)
            ]
            .into(),
        }
    }
}
