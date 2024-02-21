use crate::{
    code_convert::*,
    nuhxboard::*,
    types::{config::*, settings::*, style::*},
};
use iced::{
    mouse,
    widget::canvas,
    widget::canvas::{Geometry, Path},
    Color, Rectangle, Renderer, Theme,
};

macro_rules! draw_key {
    ($self: ident, $def: ident, $frame: ident, $content: expr, $pressed_button_list: expr) => {
        let mut boundaries_iter = $def.boundaries.iter();
        let key = Path::new(|builder| {
            builder.move_to((*boundaries_iter.next().unwrap()).clone().into());
            for boundary in boundaries_iter {
                builder.line_to((*boundary).clone().into());
            }
            builder.close();
        });

        let element_style = &$self
            .style
            .element_styles
            .iter()
            .find(|style| style.key == $def.id);

        let style: &KeyStyle;

        if let Some(s) = element_style {
            style = match &s.value {
                ElementStyleUnion::KeyStyle(i_s) => i_s,
                ElementStyleUnion::MouseSpeedIndicatorStyle(_) => unreachable!(),
            };
        } else {
            style = &$self.style.default_key_style;
        }

        let mut pressed = false;

        for keycode in &$def.keycodes {
            if $pressed_button_list.contains_key(keycode) {
                pressed = true;
                break;
            }
        }

        let current_style = match pressed {
            true => &style.pressed,
            false => &style.loose,
        };

        $frame.fill(
            &key,
            Color::from_rgb(
                current_style.background.red / 255.0,
                current_style.background.blue / 255.0,
                current_style.background.green / 255.0,
            ),
        );
        $frame.fill_text(canvas::Text {
            content: $content,
            position: $def.text_position.clone().into(),
            color: Color::from_rgb(
                current_style.text.red / 255.0,
                current_style.text.green / 255.0,
                current_style.text.blue / 255.0,
            ),
            size: iced::Pixels(style.loose.font.size),
            font: iced::Font {
                family: iced::font::Family::Name(
                    // Leak is required because Name requires static lifetime
                    // as opposed to application lifetime :(
                    // I suppose they were just expecting you to pass in a
                    // literal here... damn you!!
                    current_style.font.font_family.clone().leak(),
                ),
                weight: if current_style.font.style & 1 != 0 {
                    iced::font::Weight::Bold
                } else {
                    iced::font::Weight::Normal
                },
                stretch: iced::font::Stretch::Normal,
                style: if current_style.font.style & 0b10 != 0 {
                    iced::font::Style::Italic
                } else {
                    iced::font::Style::Normal
                },
            },
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
            shaping: iced::widget::text::Shaping::Advanced,
            ..Default::default()
        })
    };
}

impl<Message> canvas::Program<Message> for NuhxBoard {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let canvas = self.canvas.draw(renderer, bounds.size(), |frame| {
            for element in &self.config.elements {
                match element {
                    BoardElement::KeyboardKey(def) => {
                        draw_key!(
                            self,
                            def,
                            frame,
                            {
                                let shift_pressed = self
                                    .pressed_keys
                                    .contains_key(&keycode_convert(rdev::Key::ShiftLeft).unwrap())
                                    || self.pressed_keys.contains_key(
                                        &keycode_convert(rdev::Key::ShiftRight).unwrap(),
                                    );
                                match def.change_on_caps {
                                    true => match self.caps
                                        ^ (shift_pressed
                                            && (self.settings.capitalization
                                                == Capitalization::Follow
                                                || self.settings.follow_for_caps_sensitive))
                                    {
                                        true => def.shift_text.clone(),
                                        false => def.text.clone(),
                                    },
                                    false => match shift_pressed
                                        && (self.settings.capitalization == Capitalization::Follow
                                            || self.settings.follow_for_caps_insensitive)
                                    {
                                        true => def.shift_text.clone(),
                                        false => def.text.clone(),
                                    },
                                }
                            },
                            self.pressed_keys
                        );
                    }
                    BoardElement::MouseKey(def) => {
                        draw_key!(
                            self,
                            def,
                            frame,
                            def.text.clone(),
                            self.pressed_mouse_buttons
                        );
                    }
                    BoardElement::MouseScroll(def) => {
                        draw_key!(
                            self,
                            def,
                            frame,
                            def.text.clone(),
                            self.pressed_scroll_buttons
                        );
                    }
                    BoardElement::MouseSpeedIndicator(def) => {
                        let inner = Path::circle(def.location.clone().into(), def.radius / 5.0);
                        let outer = Path::circle(def.location.clone().into(), def.radius);
                        let polar_velocity = (
                            (self.mouse_velocity.0.powi(2) + self.mouse_velocity.1.powi(2)).sqrt(),
                            self.mouse_velocity.1.atan2(self.mouse_velocity.0),
                        );
                        let squashed_magnitude =
                            (self.settings.mouse_sensitivity * 0.000001 * polar_velocity.0).tanh();
                        let ball = Path::circle(
                            iced::Point {
                                x: def.location.x + (def.radius * polar_velocity.1.cos()),
                                y: def.location.y + (def.radius * polar_velocity.1.sin()),
                            },
                            def.radius / 5.0,
                        );

                        // This is a whole lot of trig... just trust the process...
                        // Check out [This Desmos thing](https://www.desmos.com/calculator/wf52bomadb) if you want to see it all workin
                        let triangle = Path::new(|builder| {
                            builder.move_to(iced::Point {
                                x: def.location.x
                                    + (squashed_magnitude
                                        * ((def.radius * polar_velocity.1.cos())
                                            - ((def.radius / 5.0)
                                                * (-polar_velocity.1.tan().powi(-1))
                                                    .atan()
                                                    .cos()))),
                                y: def.location.y
                                    + (squashed_magnitude
                                        * ((def.radius * polar_velocity.1.sin())
                                            - ((def.radius / 5.0)
                                                * (-polar_velocity.1.tan().powi(-1))
                                                    .atan()
                                                    .sin()))),
                            });
                            builder.line_to(iced::Point {
                                x: def.location.x
                                    + (squashed_magnitude
                                        * ((def.radius * polar_velocity.1.cos())
                                            + ((def.radius / 5.0)
                                                * (-polar_velocity.1.tan().powi(-1))
                                                    .atan()
                                                    .cos()))),
                                y: def.location.y
                                    + (squashed_magnitude
                                        * ((def.radius * polar_velocity.1.sin())
                                            + ((def.radius / 5.0)
                                                * (-polar_velocity.1.tan().powi(-1))
                                                    .atan()
                                                    .sin()))),
                            });
                            builder.line_to(def.location.clone().into());
                            builder.close();
                        });

                        let element_style = &self
                            .style
                            .element_styles
                            .iter()
                            .find(|style| style.key == def.id);

                        let style: &MouseSpeedIndicatorStyle;

                        let default_style = &self.style.default_mouse_speed_indicator_style;

                        if let Some(s) = element_style {
                            style = match &s.value {
                                ElementStyleUnion::KeyStyle(_) => unreachable!(),
                                ElementStyleUnion::MouseSpeedIndicatorStyle(i_s) => i_s,
                            };
                        } else {
                            style = default_style;
                        }

                        frame.fill(
                            &inner,
                            Color::from_rgb(
                                style.inner_color.red / 255.0,
                                style.inner_color.green / 255.0,
                                style.inner_color.blue / 255.0,
                            ),
                        );

                        frame.stroke(
                            &outer,
                            canvas::Stroke {
                                width: style.outline_width,
                                line_cap: canvas::LineCap::Round,
                                style: canvas::Style::Solid(Color::from_rgb(
                                    style.inner_color.red / 255.0,
                                    style.inner_color.green / 255.0,
                                    style.inner_color.blue / 255.0,
                                )),
                                line_dash: canvas::LineDash {
                                    segments: &[],
                                    offset: 0,
                                },
                                line_join: canvas::LineJoin::Round,
                            },
                        );
                        let ball_gradient = colorgrad::CustomGradient::new()
                            .colors(&[
                                colorgrad::Color::new(
                                    style.inner_color.red as f64 / 255.0,
                                    style.inner_color.green as f64 / 255.0,
                                    style.inner_color.blue as f64 / 255.0,
                                    1.0,
                                ),
                                colorgrad::Color::new(
                                    style.outer_color.red as f64 / 255.0,
                                    style.outer_color.green as f64 / 255.0,
                                    style.outer_color.blue as f64 / 255.0,
                                    1.0,
                                ),
                            ])
                            .build()
                            .unwrap();
                        let ball_color = ball_gradient.at(squashed_magnitude as f64);
                        frame.fill(
                            &ball,
                            Color::from_rgb(
                                ball_color.r as f32,
                                ball_color.g as f32,
                                ball_color.b as f32,
                            ),
                        );
                        let triangle_gradient = iced::widget::canvas::gradient::Linear::new(
                            def.location.clone().into(),
                            iced::Point {
                                x: def.location.x + (def.radius * polar_velocity.1.cos()),
                                y: def.location.y + (def.radius * polar_velocity.1.sin()),
                            },
                        )
                        .add_stop(
                            0.0,
                            iced::Color::from_rgb(
                                style.inner_color.red / 255.0,
                                style.inner_color.green / 255.0,
                                style.inner_color.blue / 255.0,
                            ),
                        )
                        .add_stop(
                            1.0,
                            iced::Color::from_rgb(
                                style.outer_color.red / 255.0,
                                style.outer_color.green / 255.0,
                                style.outer_color.blue / 255.0,
                            ),
                        );
                        frame.fill(&triangle, triangle_gradient);
                    }
                }
            }
        });
        vec![canvas]
    }
}
