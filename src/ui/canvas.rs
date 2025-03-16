use crate::{message::*, nuhxboard::*};
use colorgrad::Gradient;
use geo::{
    algorithm::line_measures::Euclidean, BoundingRect, Coord, Distance, LineString, Polygon, Within,
};
use iced::{
    mouse,
    widget::{
        canvas::{self, event::Status, Frame, Geometry, Image, Path},
        image::Handle,
    },
    Color, Rectangle, Renderer, Theme,
};
use image::ImageReader;
use nalgebra::{Vector2, Vector3};
use nuhxboard_types::{config::*, settings::*, style::*};
use rdev::win_code_from_key;
use std::collections::HashSet;

fn captured_message() -> Option<Message> {
    if cfg!(target_os = "linux") && std::env::var("XDG_SESSION_TYPE").unwrap() == "wayland" {
        Some(Message::UpdateCanvas)
    } else {
        None
    }
}

#[derive(Default)]
pub struct CanvasState {
    held_element: Option<usize>,
    hovered_face: Option<usize>,
    hovered_vertex: Option<usize>,
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
            let mut clear_canvas = false;
            if self.hovered_element.is_some() {
                return (Status::Captured, Some(Message::UpdateHoveredElement(None)));
            }
            if state.selected_element.is_some() {
                state.selected_element = None;
                clear_canvas = true;
            }
            if state.held_element.is_some() {
                state.held_element = None;
                clear_canvas = true;
            }
            if clear_canvas {
                return (Status::Captured, captured_message());
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
                                    if Euclidean::distance(
                                        cursor_position,
                                        Coord::from(def.location.clone()),
                                    ) < def.radius
                                    {
                                        if self.hovered_element != Some(index) {
                                            return (
                                                Status::Captured,
                                                Some(Message::UpdateHoveredElement(Some(index))),
                                            );
                                        }

                                        return (Status::Captured, captured_message());
                                    }
                                }
                                _ => {
                                    let mut vertices = CommonDefinitionRef::try_from(element)
                                        .unwrap()
                                        .boundaries
                                        .clone();

                                    vertices.push(vertices[0].clone());

                                    let bounds = Polygon::new(LineString::from(vertices), vec![]);

                                    if cursor_position.is_within(&bounds) {
                                        if !bounds
                                            // I love iterators -_-
                                            .exterior()
                                            .lines()
                                            .collect::<Vec<_>>()
                                            .as_slice()
                                            .windows(2)
                                            .enumerate()
                                            .any(|(i, window)| {
                                                let left = window[0];
                                                let right = window[1];

                                                if Euclidean::distance(cursor_position, &left)
                                                    <= 5.0
                                                    && Euclidean::distance(cursor_position, &right)
                                                        <= 5.0
                                                {
                                                    state.hovered_vertex = Some(i + 1);
                                                    state.hovered_face = None;
                                                } else if Euclidean::distance(
                                                    cursor_position,
                                                    &left,
                                                ) <= 5.0
                                                {
                                                    state.hovered_face = Some(i);
                                                } else if Euclidean::distance(
                                                    cursor_position,
                                                    &right,
                                                ) <= 5.0
                                                {
                                                    state.hovered_face = Some(i + 1);
                                                } else {
                                                    return false;
                                                }
                                                true
                                            })
                                        {
                                            state.hovered_face = None;
                                        }
                                        if self.hovered_element != Some(index) {
                                            return (
                                                Status::Captured,
                                                Some(Message::UpdateHoveredElement(Some(index))),
                                            );
                                        }
                                        return (Status::Captured, captured_message());
                                    }
                                }
                            }
                        }

                        if self.hovered_element.is_some() {
                            return (Status::Captured, Some(Message::UpdateHoveredElement(None)));
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
                    state.held_element = self.hovered_element;
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
                    state.selected_element = self.hovered_element;
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

            // I can only have so much control over the layering of the elements. Geometry, text,
            // and images are rendered in their own distinct passes, and their order over one
            // another cannot be specified (to my knowledge).
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
                            .contains_key(&win_code_from_key(rdev::Key::ShiftLeft).unwrap())
                            || self
                                .pressed_keys
                                .contains_key(&win_code_from_key(rdev::Key::ShiftRight).unwrap());
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
                    def.clone(),
                    frame,
                    def.text.clone(),
                    self.pressed_mouse_buttons.keys().cloned().collect(),
                    index,
                );
            }
            BoardElement::MouseScroll(def) => {
                self.draw_key(
                    state,
                    def.clone(),
                    frame,
                    def.text.clone(),
                    self.pressed_scroll_buttons.keys().cloned().collect(),
                    index,
                );
            }
            BoardElement::MouseSpeedIndicator(def) => {
                let inner = Path::circle(def.location.clone().into(), def.radius / 5.0);
                let outer = Path::circle(def.location.clone().into(), def.radius);

                let style = self
                    .style
                    .element_styles
                    .get(&def.id)
                    .map(|v| {
                        let ElementStyle::MouseSpeedIndicatorStyle(ref style) = v else {
                            unreachable!()
                        };
                        style
                    })
                    .unwrap_or(&self.style.default_mouse_speed_indicator_style);

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
                        width: style.outline_width as f32,
                        style: canvas::Style::Solid(Color::from_rgb(
                            style.inner_color.red / 255.0,
                            style.inner_color.green / 255.0,
                            style.inner_color.blue / 255.0,
                        )),
                        ..Default::default()
                    },
                );

                // TODO: Still highlight when an element is selected. While the current code is
                // closer to NohBoard's behavior, I'm not a fan of it.
                if self.hovered_element == Some(index)
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

                let velocity = Vector2::new(self.mouse_velocity.0, self.mouse_velocity.1);
                let normalized_velocity = velocity.normalize();

                let ball_to_radius_ratio = 0.2;

                let squashed_magnitude =
                    (self.settings.mouse_sensitivity * 0.000005 * velocity.magnitude()).tanh();

                let ball = Path::circle(
                    {
                        iced::Point {
                            x: def.location.x + (def.radius * normalized_velocity.x),
                            y: def.location.y + (def.radius * normalized_velocity.y),
                        }
                    },
                    def.radius * ball_to_radius_ratio,
                );

                let triangle = indicator_triangle(
                    velocity,
                    def.radius,
                    ball_to_radius_ratio,
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
                        x: def.location.x + (def.radius * normalized_velocity.x),
                        y: def.location.y + (def.radius * normalized_velocity.y),
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
        let element_style = &self.style.element_styles.get(&def.id);

        let style = match element_style {
            Some(s) => match s {
                ElementStyle::KeyStyle(i_s) => i_s,
                ElementStyle::MouseSpeedIndicatorStyle(_) => unreachable!(),
            },
            None => &self.style.default_key_style,
        };

        let pressed = pressed_button_list
            .difference(&HashSet::from_iter(def.key_codes))
            .cloned()
            .collect::<HashSet<_>>()
            != pressed_button_list;

        let current_style = match pressed {
            true => {
                if let Some(pressed) = &style.pressed {
                    pressed
                } else {
                    &KeySubStyle::default_pressed()
                }
            }
            false => {
                if let Some(loose) = &style.loose {
                    loose
                } else {
                    &KeySubStyle::default_loose()
                }
            }
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
                    // This takes a static string (I suppose the developers of the library were
                    // expecting the use of a literal), so to avoid re-leaking the font family
                    // every frame, I use a static hashset.
                    FONTS
                        .read()
                        .unwrap()
                        .get(current_style.font.font_family.as_str())
                        .unwrap(),
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
        let mut boundaries = def.boundaries.clone();
        boundaries.push(boundaries[0].clone());
        let shape = Polygon::new(LineString::from(boundaries), vec![]);

        if let Some(name) = &current_style.background_image_file_name {
            let path = self
                .keyboards_path
                .join(&self.settings.category)
                .join("images")
                .join(name);

            if !name.is_empty() && path.exists() {
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

        if current_style.show_outline {
            frame.stroke(
                &key,
                canvas::Stroke {
                    style: canvas::Style::Solid(current_style.outline.into()),
                    width: current_style.outline_width as f32,
                    ..Default::default()
                },
            );
        }

        if self.hovered_element == Some(index)
            && state.held_element.is_none()
            && state.selected_element.is_none()
        {
            frame.fill(&key, Color::from_rgba(0.0, 0.0, 1.0, 0.5));
        } else if state.held_element == Some(index) {
            frame.fill(
                &key,
                Color {
                    a: 0.5,
                    ..if let Some(pressed) = &style.pressed {
                        pressed.background.into()
                    } else {
                        KeySubStyle::default_pressed().background.into()
                    }
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
        if let Some(face) = state.hovered_face {
            let face = shape.exterior().lines().collect::<Vec<_>>()[face];
            let path = Path::line(
                iced::Point {
                    x: face.start.x,
                    y: face.start.y,
                },
                iced::Point {
                    x: face.end.x,
                    y: face.end.y,
                },
            );
            if state.selected_element == Some(index) {
                frame.stroke(
                    &path,
                    canvas::Stroke {
                        // #FF4500
                        style: canvas::Style::Solid(Color::from_rgb(1.0, 45.0 / 255.0, 0.0)),
                        width: 10.0,
                        ..Default::default()
                    },
                );
            } else if self.hovered_element == Some(index) {
                frame.stroke(
                    &path,
                    canvas::Stroke {
                        // #FF4500
                        style: canvas::Style::Solid(Color::BLACK),
                        width: 20.0,
                        ..Default::default()
                    },
                );
            }
        }
    }
}

// A previous iteration of this function used a bunch of messy trig. This vector math solution is
// actually slower (by a whopping microsecond), but the improved readability is definitely worth it.
fn indicator_triangle(
    velocity: Vector2<f32>,
    radius: f32,
    ball_to_ring_ratio: f32,
    magnitude: f32,
    center: iced::Point,
) -> Path {
    let v = Vector3::new(velocity.x, velocity.y, 0.0);
    let r = radius;
    let b = ball_to_ring_ratio;
    let t = magnitude;
    let c = center;

    let u = Vector3::new(0.0, 0.0, 1.0);

    // i and j components allow me to create a coordinate space relative to the velocity
    // j points in the direction of the velocity
    let j = v.normalize();
    // i is orthogonal to the velocity
    let i = j.cross(&u).normalize();

    // points to the center of the ball
    let ball_vector = r * j;

    // points to the leftmost side of the ball
    let left_vector = t * (ball_vector - (b * r * i));
    // points to the rightmost side of the ball
    let right_vector = t * (ball_vector + (b * r * i));

    Path::new(|builder| {
        builder.move_to(c);
        builder.line_to(iced::Point {
            x: c.x + left_vector.x,
            y: c.y + left_vector.y,
        });
        builder.line_to(iced::Point {
            x: c.x + right_vector.x,
            y: c.y + right_vector.y,
        });
        builder.close();
    })
}
