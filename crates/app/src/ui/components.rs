use crate::{message::*, nuhxboard_types::*};
use iced::{
    border::Radius,
    widget::{button, container, row, text, text::IntoFragment, Button, Container, Row},
    Alignment, Border, Color, Element, Length, Shadow,
};
use iced_aw::{color_picker, widget::InnerBounds, Quad};
use types::style::NohRgb;

pub fn labeled_text_input<'a>(
    label: impl IntoFragment<'a>,
    text_input: iced::widget::TextInput<'a, Message>,
) -> Row<'a, Message> {
    row![text(label), text_input].align_y(Alignment::Center)
}

pub fn gray_box<'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
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

pub fn picker_button<'a>(
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
                    button::Status::Active | button::Status::Hovered => button::Style {
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

pub fn context_menu_button(label: &str) -> Button<Message> {
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

pub fn seperator() -> Quad {
    Quad {
        quad_color: iced::Background::Color(Color::from_rgb8(204, 204, 204)),
        height: Length::Fixed(5.0),
        inner_bounds: InnerBounds::Ratio(0.95, 0.2),
        ..Default::default()
    }
}
