use crate::nuhxboard::*;
use colorgrad::Gradient;
use geo::{BoundingRect, Coord, EuclideanDistance, LineString, Polygon, Vector2DOps, Within};
use iced::{
    mouse,
    widget::{
        canvas::{self, event::Status, Frame, Geometry, Image, Path},
        image::Handle,
    },
    Color, Rectangle, Renderer, Theme,
};
use image::ImageReader;
use logic::code_convert::*;
use std::collections::HashSet;
use types::{config::*, settings::*, style::*};

fn captured_message() -> Option<Message> {
    if cfg!(target_os = "linux") && std::env::var("XDG_SESSION_TYPE").unwrap() == "wayland" {
        Some(Message::UpdateCanvas)
    } else {
        None
    }
}

#[derive(Default)]
pub struct CanvasState {
    hovered_element: Option<usize>,
    held_element: Option<usize>,
    selected_element: Option<usize>,
    interaction: Interaction,
    previous_cursor_position: Coord<f32>,
    delta_accumulator: Coord<f32>,
}

#[derive(Default, Debug, PartialEq, Eq)]
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

        let cursor_position = Coord {
            x: cursor_position.x,
            y: cursor_position.y,
        };

        match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                match state.interaction {
                    Interaction::None => {
                        state.previous_cursor_position = cursor_position;
                        for (index, element) in self.layout.elements.iter().enumerate() {
                            match element {
                                BoardElement::MouseSpeedIndicator(def) => {
                                    if cursor_position
                                        .euclidean_distance(&Coord::from(def.location.clone()))
                                        < def.radius
                                    {
                                        if state.hovered_element != Some(index) {
                                            state.hovered_element = Some(index);
                                        }

                                        return (Status::Captured, captured_message());
                                    }
                                }
                                _ => {
                                    let mut vertices = match element {
                                        BoardElement::KeyboardKey(def) => def.boundaries.clone(),
                                        BoardElement::MouseKey(def) => def.boundaries.clone(),
                                        BoardElement::MouseScroll(def) => def.boundaries.clone(),
                                        BoardElement::MouseSpeedIndicator(_) => {
                                            unreachable!()
                                        }
                                    };

                                    vertices.push(vertices[0].clone());

                                    let bounds = Polygon::new(LineString::from(vertices), vec![]);

                                    if cursor_position.is_within(&bounds) {
                                        if state.hovered_element != Some(index) {
                                            state.hovered_element = Some(index);
                                        }
                                        return (Status::Captured, captured_message());
                                    }
                                }
                            }
                        }

                        if state.hovered_element.is_some() {
                            state.hovered_element = None;
                            return (Status::Captured, captured_message());
                        }
                    }
                    Interaction::Dragging => {
                        if state.held_element.is_some() {
                            let delta = cursor_position - state.previous_cursor_position;
                            state.previous_cursor_position = cursor_position;
                            state.delta_accumulator.x += delta.x;
                            state.delta_accumulator.y += delta.y;
                            return (
                                Status::Captured,
                                Some(Message::MoveElement {
                                    index: state.held_element.unwrap(),
                                    delta,
                                }),
                            );
                        }
                    }
                }
                state.previous_cursor_position = cursor_position;
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.interaction = Interaction::Dragging;

                if state.selected_element.is_none() {
                    state.held_element = state.hovered_element;
                } else {
                    state.held_element = state.selected_element;
                    state.selected_element = None;
                }

                return (Status::Captured, captured_message());
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let message = if state.delta_accumulator != Coord::default() {
                    let out = state.held_element.map(|index| {
                        Message::PushChange(Change::MoveElement {
                            index,
                            delta: state.delta_accumulator,
                            move_text: self.settings.update_text_position,
                        })
                    });
                    state.delta_accumulator = Coord::default();
                    state.selected_element = state.held_element;
                    out
                } else {
                    state.selected_element = state.hovered_element;
                    Some(Message::UpdateCanvas)
                };

                state.held_element = None;

                state.interaction = Interaction::None;
                return (Status::Ignored, message);
            }
            _ => {}
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
            for (index, element) in self.layout.elements.iter().enumerate() {
                if state.selected_element == Some(index) || state.held_element == Some(index) {
                    continue;
                }
                self.draw_element(element, state, frame, index);
            }
            let top_element = if state.selected_element.is_some() {
                state.selected_element
            } else {
                state.held_element
            };
            if let Some(top_element) = top_element {
                self.draw_element(
                    &self.layout.elements[top_element],
                    state,
                    frame,
                    top_element,
                );
            }
        });
        vec![canvas]
    }
}

impl NuhxBoard {
    fn draw_element(
        &self,
        element: &BoardElement,
        state: &CanvasState,
        frame: &mut Frame,
        index: usize,
    ) {
        match element {
            BoardElement::KeyboardKey(def) => {
                self.draw_key(
                    state,
                    def.clone().into(),
                    frame,
                    {
                        let shift_pressed = self
                            .pressed_keys
                            .contains_key(&keycode_convert(rdev::Key::ShiftLeft).unwrap())
                            || self
                                .pressed_keys
                                .contains_key(&keycode_convert(rdev::Key::ShiftRight).unwrap());
                        match def.change_on_caps {
                            true => match self.caps
                                ^ (shift_pressed
                                    && (self.settings.capitalization == Capitalization::Follow
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
                    self.pressed_keys.keys().cloned().collect(),
                    index,
                );
            }
            BoardElement::MouseKey(def) => {
                self.draw_key(
                    state,
                    def.clone().into(),
                    frame,
                    def.text.clone(),
                    self.pressed_mouse_buttons.keys().cloned().collect(),
                    index,
                );
            }
            BoardElement::MouseScroll(def) => {
                self.draw_key(
                    state,
                    def.clone().into(),
                    frame,
                    def.text.clone(),
                    self.pressed_scroll_buttons.keys().cloned().collect(),
                    index,
                );
            }
            BoardElement::MouseSpeedIndicator(def) => {
                let inner = Path::circle(def.location.clone().into(), def.radius / 5.0);
                let outer = Path::circle(def.location.clone().into(), def.radius);

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
                        ElementStyleUnion::MouseSpeedIndicatorStyle(style) => style,
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
                        style: canvas::Style::Solid(Color::from_rgb(
                            style.inner_color.red / 255.0,
                            style.inner_color.green / 255.0,
                            style.inner_color.blue / 255.0,
                        )),
                        ..Default::default()
                    },
                );

                // TODO: Still highlight when an element is selected. I dislike this
                // behavior of NohBoard.
                if state.hovered_element == Some(index)
                    && state.held_element.is_none()
                    && state.selected_element.is_none()
                {
                    frame.fill(&outer, Color::from_rgba(0.0, 0.0, 1.0, 0.5));
                } else if state.held_element == Some(index) {
                    frame.fill(&outer, Color::from_rgba(1.0, 1.0, 1.0, 0.5));
                }
                if state.selected_element == Some(index) {
                    frame.stroke(
                        &outer,
                        canvas::Stroke {
                            width: 2.0,
                            style: canvas::Style::Solid(Color::from_rgba(1.0, 0.0, 1.0, 1.0)),
                            ..Default::default()
                        },
                    );
                }

                if self.mouse_velocity.0 == 0.0 && self.mouse_velocity.1 == 0.0 {
                    return;
                }

                let polar_velocity = (
                    (self.mouse_velocity.0.powi(2) + self.mouse_velocity.1.powi(2)).sqrt(),
                    self.mouse_velocity.1.atan2(self.mouse_velocity.0),
                );
                let squashed_magnitude =
                    (self.settings.mouse_sensitivity * 0.000005 * polar_velocity.0).tanh();
                let ball = Path::circle(
                    {
                        let normalized_velocity =
                            Coord::from(self.mouse_velocity).try_normalize().unwrap();
                        iced::Point {
                            x: def.location.x + (def.radius * normalized_velocity.x),
                            y: def.location.y + (def.radius * normalized_velocity.y),
                        }
                    },
                    def.radius / 5.0,
                );

                let triangle = indicator_triangle(
                    def.radius,
                    polar_velocity.1,
                    1.0 / 5.0,
                    squashed_magnitude,
                    def.location.clone().into(),
                );

                let ball_gradient = colorgrad::GradientBuilder::new()
                    .colors(&[
                        colorgrad::Color::new(
                            style.inner_color.red / 255.0,
                            style.inner_color.green / 255.0,
                            style.inner_color.blue / 255.0,
                            1.0,
                        ),
                        colorgrad::Color::new(
                            style.outer_color.red / 255.0,
                            style.outer_color.green / 255.0,
                            style.outer_color.blue / 255.0,
                            1.0,
                        ),
                    ])
                    .build::<colorgrad::LinearGradient>()
                    .unwrap();
                let ball_color = ball_gradient.at(squashed_magnitude);
                frame.fill(
                    &ball,
                    Color::from_rgb(ball_color.r, ball_color.g, ball_color.b),
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

    fn draw_key(
        &self,
        state: &CanvasState,
        def: CommonDefinition,
        frame: &mut Frame,
        text: String,
        pressed_button_list: HashSet<u32>,
        index: usize,
    ) {
        let element_style = &self
            .style
            .element_styles
            .iter()
            .find(|style| style.key == def.id);

        let style = match element_style {
            Some(s) => match &s.value {
                ElementStyleUnion::KeyStyle(i_s) => i_s,
                ElementStyleUnion::MouseSpeedIndicatorStyle(_) => unreachable!(),
            },
            None => &self.style.default_key_style,
        };

        let pressed = pressed_button_list
            .difference(&HashSet::from_iter(def.keycodes))
            .cloned()
            .collect::<HashSet<_>>()
            != pressed_button_list;

        let current_style = match pressed {
            true => &style.pressed,
            false => &style.loose,
        };

        frame.fill_text(canvas::Text {
            content: text,
            position: def.text_position.clone().into(),
            color: Color::from_rgb(
                current_style.text.red / 255.0,
                current_style.text.green / 255.0,
                current_style.text.blue / 255.0,
            ),
            size: iced::Pixels(current_style.font.size),
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

        let key = Path::new(|builder| {
            builder.move_to(def.boundaries[0].clone().into());
            for boundary in def.boundaries.iter().skip(1) {
                builder.line_to(boundary.clone().into());
            }
            builder.close();
        });

        if let Some(name) = &current_style.background_image_file_name {
            let path = self
                .keyboards_path
                .join(&self.settings.category)
                .join("images")
                .join(name);

            if !name.is_empty() && path.exists() {
                let mut boundaries = def.boundaries.clone();
                boundaries.push(boundaries[0].clone());
                let shape = Polygon::new(LineString::from(boundaries), vec![]);
                let rect = shape.bounding_rect().unwrap();
                let width = rect.width();
                let height = rect.height();

                let image = ImageReader::open(path).unwrap();
                let image = image.decode().unwrap();
                let image = image.resize_exact(
                    width as u32,
                    height as u32,
                    image::imageops::FilterType::Nearest,
                );
                let pos = rect.min();

                frame.draw_image(
                    iced::Rectangle::new(
                        iced::Point { x: pos.x, y: pos.y },
                        iced::Size::new(width, height),
                    ),
                    Image::new(Handle::from_rgba(
                        width as u32,
                        height as u32,
                        image.to_rgba8().to_vec(),
                    )),
                );
            }
        } else {
            frame.fill(
                &key,
                Color::from_rgb(
                    current_style.background.red / 255.0,
                    current_style.background.blue / 255.0,
                    current_style.background.green / 255.0,
                ),
            );
        }

        if state.hovered_element == Some(index)
            && state.held_element.is_none()
            && state.selected_element.is_none()
        {
            frame.fill(&key, Color::from_rgba(0.0, 0.0, 1.0, 0.5));
        } else if state.held_element == Some(index) {
            frame.fill(
                &key,
                Color {
                    a: 0.5,
                    ..style.pressed.background.clone().into()
                },
            );
        }
        if state.selected_element == Some(index) {
            frame.stroke(
                &key,
                canvas::Stroke {
                    style: canvas::Style::Solid(Color::from_rgba(1.0, 0.0, 1.0, 1.0)),
                    width: 2.0,
                    ..Default::default()
                },
            );
        }
    }
}

/// This is a whole lot of trig... just trust the process...
/// Check out [This Desmos thing](https://www.desmos.com/calculator/lij5p4ptfo) if you want to see it all working
fn indicator_triangle(
    radius: f32,
    angle: f32,
    ball_to_ring_ratio: f32,
    magnitude: f32,
    center: iced::Point,
) -> Path {
    let r = radius;
    let n = angle;
    let b = ball_to_ring_ratio;
    let t = magnitude;

    fn cos(n: f32) -> f32 {
        n.cos()
    }

    fn sin(n: f32) -> f32 {
        n.sin()
    }

    fn cot(n: f32) -> f32 {
        n.tan().powi(-1)
    }

    fn atan(n: f32) -> f32 {
        n.atan()
    }

    Path::new(|builder| {
        builder.move_to(center);
        builder.line_to(iced::Point {
            x: center.x + (t * ((r * cos(n)) - (b * r * cos(atan(-cot(n)))))),
            y: center.y + (t * ((r * sin(n)) - (b * r * sin(atan(-cot(n)))))),
        });
        builder.line_to(iced::Point {
            x: center.x + (t * ((r * cos(n)) + (b * r * cos(atan(-cot(n)))))),
            y: center.y + (t * ((r * sin(n)) + (b * r * sin(atan(-cot(n)))))),
        })
    })
}
