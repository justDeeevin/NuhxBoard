use crate::{
    logic::code_convert::*,
    nuhxboard::*,
    types::{config::*, settings::*, style::*},
};
use geo::{Coord, LineString, Polygon, Within};
use iced::{
    mouse,
    widget::canvas::{self, event::Status, Geometry, Path},
    Color, Rectangle, Renderer, Theme,
};

macro_rules! draw_key {
    ($self: ident, $state:ident, $def: ident, $frame: ident, $content: expr, $pressed_button_list: expr, $index: expr) => {
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
                    // as opposed to application lifetime.
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
        });

        if $state.hovered_element == Some($index) {
            $frame.fill(&key, Color::from_rgba(0.0, 0.0, 1.0, 0.5));
        }
    };
}

#[derive(Default)]
pub struct CanvasState {
    hovered_element: Option<usize>,
    selected_element: Option<usize>,
    interaction: Interaction,
    previous_cursor_position: Coord,
}

#[derive(Default, Debug)]
pub enum Interaction {
    #[default]
    None,
    Dragging,
}

impl canvas::Program<Message> for NuhxBoard {
    type State = CanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        if !self.edit_mode {
            if state.hovered_element.is_some() {
                state.hovered_element = None;
            }
            return (Status::Ignored, None);
        }

        let Some(cursor_position) = cursor.position_in(bounds) else {
            return (Status::Ignored, None);
        };

        let cursor_position_geo = Coord {
            x: cursor_position.x as f64,
            y: cursor_position.y as f64,
        };

        if let canvas::Event::Mouse(event) = event {
            match event {
                mouse::Event::CursorMoved { position: _ } => {
                    match state.interaction {
                        Interaction::None => {
                            for (index, element) in self.config.elements.iter().enumerate() {
                                match element {
                                    BoardElement::MouseSpeedIndicator(def) => {
                                        if cursor_position.distance(def.location.clone().into())
                                            < def.radius
                                        {
                                            if state.hovered_element != Some(index) {
                                                state.hovered_element = Some(index);
                                            }
                                            state.previous_cursor_position = cursor_position_geo;
                                            return (Status::Captured, None);
                                        }
                                    }
                                    _ => {
                                        let mut vertices = match element {
                                            BoardElement::KeyboardKey(def) => {
                                                def.boundaries.clone()
                                            }
                                            BoardElement::MouseKey(def) => def.boundaries.clone(),
                                            BoardElement::MouseScroll(def) => {
                                                def.boundaries.clone()
                                            }
                                            BoardElement::MouseSpeedIndicator(_) => {
                                                unreachable!()
                                            }
                                        };

                                        vertices.push(vertices[0].clone());

                                        let bounds =
                                            Polygon::new(LineString::from(vertices), vec![]);

                                        if cursor_position_geo.is_within(&bounds) {
                                            if state.hovered_element != Some(index) {
                                                state.hovered_element = Some(index);
                                            }
                                            state.previous_cursor_position = cursor_position_geo;
                                            return (Status::Captured, None);
                                        }
                                    }
                                }
                            }

                            if state.hovered_element.is_some() {
                                state.hovered_element = None;
                                state.previous_cursor_position = cursor_position_geo;
                                return (Status::Captured, None);
                            }
                        }
                        Interaction::Dragging => {
                            if state.selected_element.is_some() {
                                let delta = cursor_position_geo - state.previous_cursor_position;

                                match &self.config.elements[state.selected_element.unwrap()] {
                                    BoardElement::MouseSpeedIndicator(def) => {
                                        let new_location = SerializablePoint {
                                            x: def.location.x + delta.x as f32,
                                            y: def.location.y + delta.y as f32,
                                        };

                                        state.previous_cursor_position = cursor_position_geo;
                                        return (
                                            Status::Captured,
                                            Some(Message::UpdateElement {
                                                index: state.selected_element.unwrap(),
                                                new_element: BoardElement::MouseSpeedIndicator(
                                                    MouseSpeedIndicatorDefinition {
                                                        id: def.id,
                                                        location: new_location,
                                                        radius: def.radius,
                                                    },
                                                ),
                                            }),
                                        );
                                    }
                                    _ => {
                                        let mut boundaries = match &self.config.elements
                                            [state.selected_element.unwrap()]
                                        {
                                            BoardElement::KeyboardKey(def) => {
                                                def.boundaries.clone()
                                            }
                                            BoardElement::MouseKey(def) => def.boundaries.clone(),
                                            BoardElement::MouseScroll(def) => {
                                                def.boundaries.clone()
                                            }
                                            BoardElement::MouseSpeedIndicator(_) => unreachable!(),
                                        };

                                        for boundary in &mut boundaries {
                                            boundary.x += delta.x as f32;
                                            boundary.y += delta.y as f32;
                                        }

                                        let mut text_position = match &self.config.elements
                                            [state.selected_element.unwrap()]
                                        {
                                            BoardElement::KeyboardKey(def) => {
                                                def.text_position.clone()
                                            }
                                            BoardElement::MouseKey(def) => {
                                                def.text_position.clone()
                                            }
                                            BoardElement::MouseScroll(def) => {
                                                def.text_position.clone()
                                            }
                                            BoardElement::MouseSpeedIndicator(_) => unreachable!(),
                                        };

                                        text_position.x += delta.x as f32;
                                        text_position.y += delta.y as f32;

                                        let new_element = match &self.config.elements
                                            [state.selected_element.unwrap()]
                                        {
                                            BoardElement::KeyboardKey(def) => {
                                                BoardElement::KeyboardKey(KeyboardKeyDefinition {
                                                    id: def.id,
                                                    boundaries,
                                                    text_position,
                                                    keycodes: def.keycodes.clone(),
                                                    text: def.text.clone(),
                                                    shift_text: def.shift_text.clone(),
                                                    change_on_caps: def.change_on_caps,
                                                })
                                            }
                                            BoardElement::MouseKey(def) => {
                                                BoardElement::MouseKey(MouseKeyDefinition {
                                                    id: def.id,
                                                    boundaries,
                                                    text_position,
                                                    keycodes: def.keycodes.clone(),
                                                    text: def.text.clone(),
                                                })
                                            }
                                            BoardElement::MouseScroll(def) => {
                                                BoardElement::MouseScroll(MouseScrollDefinition {
                                                    id: def.id,
                                                    boundaries,
                                                    text_position,
                                                    keycodes: def.keycodes.clone(),
                                                    text: def.text.clone(),
                                                })
                                            }
                                            BoardElement::MouseSpeedIndicator(_) => unreachable!(),
                                        };

                                        state.previous_cursor_position = cursor_position_geo;
                                        return (
                                            Status::Captured,
                                            Some(Message::UpdateElement {
                                                index: state.selected_element.unwrap(),
                                                new_element,
                                            }),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    state.previous_cursor_position = cursor_position_geo;
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    state.selected_element = state.hovered_element;
                    state.interaction = Interaction::Dragging;
                    return (Status::Captured, None);
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    state.interaction = Interaction::None;
                    return (Status::Ignored, None);
                }
                _ => {}
            }
        }
        (Status::Ignored, None)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let canvas = self.canvas.draw(renderer, bounds.size(), |frame| {
            for (index, element) in self.config.elements.iter().enumerate() {
                match element {
                    BoardElement::KeyboardKey(def) => {
                        draw_key!(
                            self,
                            state,
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
                            self.pressed_keys,
                            index
                        );
                    }
                    BoardElement::MouseKey(def) => {
                        draw_key!(
                            self,
                            state,
                            def,
                            frame,
                            def.text.clone(),
                            self.pressed_mouse_buttons,
                            index
                        );
                    }
                    BoardElement::MouseScroll(def) => {
                        draw_key!(
                            self,
                            state,
                            def,
                            frame,
                            def.text.clone(),
                            self.pressed_scroll_buttons,
                            index
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

                        if state.hovered_element == Some(index) {
                            frame.fill(&outer, Color::from_rgba(0.0, 0.0, 1.0, 0.5));
                        }

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
