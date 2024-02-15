use iced::widget::*;

pub struct WhiteButton;

impl button::StyleSheet for WhiteButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(iced::Color::WHITE)),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            border: iced::Border {
                color: iced::Color::BLACK,
                width: 2.0,
                radius: 0.into(),
            },
            ..self.active(style)
        }
    }
}
