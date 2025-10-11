use crate::{message::*, types::*};
use iced::{
    font::Weight,
    widget::{button, container, row, text, text::IntoFragment, Button, Container, Row, Text},
    Alignment, Background, Color, Element, Font, Length,
};
use iced_aw::{color_picker, widget::InnerBounds, Quad};
pub fn labeled_text_input<'a>(
    label: impl IntoFragment<'a>,
    text_input: iced::widget::TextInput<'a, Message>,
) -> Row<'a, Message> {
    row![text(label), text_input].align_y(Alignment::Center)
}

pub fn gray_box<'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content)
        .padding(5)
        .width(Length::Fill)
        .align_x(Alignment::Center)
}

pub fn picker_button<'a>(
    label: impl std::fmt::Display,
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
                .on_press(Message::ToggleColorPicker(picker))
                .style(move |theme, status| button::Style {
                    background: Some(Background::Color(color)),
                    ..button::primary(theme, status)
                }),
            Message::ToggleColorPicker(picker),
            move |v| Message::ChangeColor(picker, v)
        ),
        text(format!(" {label}"))
    ]
    .align_y(Alignment::Center)
}

pub fn context_menu_button(label: &str) -> Button<'_, Message> {
    let text = text(label).size(12);
    button(text)
        .style(|theme, status| {
            let primary = button::primary(theme, status);
            button::Style {
                text_color: Color::BLACK,
                background: Some(Background::Color(Color::WHITE)),
                ..primary
            }
        })
        .width(Length::Fill)
}

pub fn seperator() -> Quad {
    Quad {
        quad_color: Background::Color(Color::from_rgb8(204, 204, 204)),
        height: Length::Fixed(5.0),
        inner_bounds: InnerBounds::Ratio(0.95, 0.2),
        ..Default::default()
    }
}
pub fn category_label<'a>(label: impl IntoFragment<'a>) -> Text<'a> {
    text(label)
        .font(Font {
            weight: Weight::Bold,
            ..Default::default()
        })
        .align_x(Alignment::Center)
        .width(Length::Fill)
}
