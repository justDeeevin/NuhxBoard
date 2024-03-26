use iced::{widget::*, Background, Border};

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

pub struct ContextMenuBox;

impl container::StyleSheet for ContextMenuBox {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(iced::Color::WHITE)),
            ..Default::default()
        }
    }
}

pub struct ContextMenuCheckBox;

impl checkbox::StyleSheet for ContextMenuCheckBox {
    type Style = iced::Theme;
    fn active(&self, _style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
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
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            border: Border {
                color: iced::Color::BLACK,
                width: 2.0,
                radius: iced::border::Radius::default(),
            },
            ..self.active(style, is_checked)
        }
    }
}
