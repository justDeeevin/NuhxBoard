use std::{collections::HashSet, ops::Deref};

use colorgrad::Gradient;
use geo::{BoundingRect, Coord, Distance, Euclidean, LineString, Polygon, Within};
use iced::{
    advanced::{layout::Node, widget::tree, Renderer as _, Widget},
    mouse,
    widget::{
        canvas::{self, event::Status, Geometry},
        image::Handle,
    },
    Color, Element, Event, Length, Rectangle, Renderer, Size,
};
use iced_graphics::geometry::{Image, Path, Renderer as _};
use image::ImageReader;
use nalgebra::{Vector2, Vector3};
use nuhxboard_types::{
    layout::{BoardElement, CommonDefinitionRef},
    settings::Capitalization,
};
use redev::keycodes::windows::code_from_key as win_code_from_key;
use tracing::trace;

use crate::{
    message::{Change, Message},
    nuhxboard::{NuhxBoard, FONTS, KEYBOARDS_PATH},
};

const BALL_TO_RADIUS_RATIO: f32 = 0.2;
const HOVER_EDGE_DISTANCE: f32 = 5.0;

pub struct Keyboard<'a> {
    app: &'a NuhxBoard,
    width: f32,
    height: f32,
}

impl Deref for Keyboard<'_> {
    type Target = NuhxBoard;

    fn deref(&self) -> &Self::Target {
        self.app
    }
}

#[derive(Default)]
struct State {
    held_element: Option<usize>,
    hovered_face: Option<usize>,
    hovered_vertex: Option<usize>,
    selected_element: Option<usize>,
    interaction: Interaction,
    previous_cursor_position: Coord<f32>,
    delta_accumulator: Coord<f32>,
}

impl State {
    fn set_hovered_face(&mut self, face: usize) -> bool {
        let mut out = false;
        if self.hovered_face.is_none() {
            self.hovered_face = Some(face);
            out = true;
        }
        if self.hovered_vertex.is_some() {
            self.hovered_vertex = None;
            out = true;
        }
        out
    }

    fn set_hovered_vertex(&mut self, vertex: usize) -> bool {
        let mut out = false;
        if self.hovered_vertex.is_none() {
            self.hovered_vertex = Some(vertex);
            out = true;
        }
        if self.hovered_face.is_some() {
            self.hovered_face = None;
            out = true;
        }
        out
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
enum Interaction {
    #[default]
    None,
    Dragging,
}

impl<'a> Keyboard<'a> {
    pub fn new(width: f32, height: f32, app: &'a NuhxBoard) -> Self {
        Self { width, height, app }
    }

    /// Returns `(fg, bg)`
    fn draw_element(
        &self,
        state: &State,
        renderer: &Renderer,
        index: usize,
    ) -> (Geometry, Geometry) {
        match &self.layout.elements[index] {
            BoardElement::KeyboardKey(def) => self.draw_key(
                state,
                renderer,
                {
                    let shift_pressed = self
                        .pressed_keys
                        .contains_key(&win_code_from_key(redev::Key::ShiftLeft).unwrap())
                        || self
                            .pressed_keys
                            .contains_key(&win_code_from_key(redev::Key::ShiftRight).unwrap());
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
            ),
            BoardElement::MouseKey(def) => self.draw_key(
                state,
                renderer,
                def.text.clone(),
                self.pressed_mouse_buttons.keys().cloned().collect(),
                index,
            ),
            BoardElement::MouseScroll(def) => self.draw_key(
                state,
                renderer,
                def.text.clone(),
                self.pressed_scroll_buttons.keys().cloned().collect(),
                index,
            ),
            BoardElement::MouseSpeedIndicator(def) => {
                let cache = &self.caches[index];
                let inner = Path::circle(
                    def.location.clone().into(),
                    def.radius * BALL_TO_RADIUS_RATIO,
                );
                let outer = Path::circle(def.location.clone().into(), def.radius);

                let style = self
                    .style
                    .element_styles
                    .get(&def.id)
                    .map_or(&self.style.default_mouse_speed_indicator_style, |v| {
                        v.as_mouse_speed_indicator_style().unwrap()
                    });

                let fg = cache.fg.draw(renderer, self.bounds().size(), |frame| {
                    frame.fill(&inner, Color::from(style.inner_color));

                    frame.stroke(
                        &outer,
                        canvas::Stroke {
                            width: style.outline_width as f32,
                            style: canvas::Style::Solid(style.inner_color.into()),
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

                    let squashed_magnitude =
                        (self.settings.mouse_sensitivity * 0.000005 * velocity.magnitude()).tanh();

                    let ball = Path::circle(
                        {
                            iced::Point {
                                x: *(def.location.x + (def.radius * normalized_velocity.x)),
                                y: *(def.location.y + (def.radius * normalized_velocity.y)),
                            }
                        },
                        def.radius * BALL_TO_RADIUS_RATIO,
                    );

                    let triangle = indicator_triangle(
                        velocity,
                        def.radius,
                        BALL_TO_RADIUS_RATIO,
                        squashed_magnitude,
                        def.location.clone().into(),
                    );

                    let ball_gradient = colorgrad::GradientBuilder::new()
                        .colors(&[style.inner_color.into(), style.outer_color.into()])
                        .build::<colorgrad::LinearGradient>()
                        .unwrap();
                    let ball_color = ball_gradient.at(squashed_magnitude);
                    trace!(index = index, "Redrawing foreground");

                    frame.fill(
                        &ball,
                        Color::from_rgb(ball_color.r, ball_color.g, ball_color.b),
                    );
                    let triangle_gradient = iced::widget::canvas::gradient::Linear::new(
                        def.location.clone().into(),
                        iced::Point {
                            x: *(def.location.x + (def.radius * normalized_velocity.x)),
                            y: *(def.location.y + (def.radius * normalized_velocity.y)),
                        },
                    )
                    .add_stop(0.0, style.inner_color.into())
                    .add_stop(1.0, style.outer_color.into());
                    frame.fill(&triangle, triangle_gradient);
                });

                (fg, cache.bg.draw(renderer, self.bounds().size(), |_| {}))
            }
        }
    }

    /// Returns `(fg, bg)`
    fn draw_key(
        &self,
        state: &State,
        renderer: &Renderer,
        text: String,
        pressed_button_list: HashSet<u32>,
        index: usize,
    ) -> (Geometry, Geometry) {
        let cache = &self.caches[index];
        let def = CommonDefinitionRef::try_from(&self.layout.elements[index]).unwrap();
        let element_style = &self.style.element_styles.get(def.id);

        let style = match element_style {
            Some(s) => s.as_key_style().unwrap(),
            None => &self.style.default_key_style.clone().into(),
        };

        let pressed = pressed_button_list
            .difference(&HashSet::from_iter(def.key_codes.iter().copied()))
            .cloned()
            .collect::<HashSet<_>>()
            != pressed_button_list;

        let current_style = if pressed {
            style
                .pressed
                .as_ref()
                .unwrap_or(&self.style.default_key_style.pressed)
        } else {
            style
                .loose
                .as_ref()
                .unwrap_or(&self.style.default_key_style.loose)
        };

        let mut boundaries = def.boundaries.clone();
        boundaries.push(boundaries[0].clone());
        let shape = Polygon::new(LineString::from(boundaries), vec![]);

        let key = Path::new(|builder| {
            builder.move_to(def.boundaries[0].clone().into());
            for boundary in def.boundaries.iter().skip(1) {
                builder.line_to(boundary.clone().into());
            }
            builder.close();
        });

        let fg = cache.fg.draw(renderer, self.bounds().size(), |frame| {
            trace!(index = index, "Redrawing foreground");

            frame.fill_text(canvas::Text {
                content: text,
                position: def.text_position.clone().into(),
                color: current_style.text.into(),
                size: iced::Pixels(current_style.font.size),
                font: current_style.font.as_iced(&FONTS).unwrap(),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                shaping: iced::widget::text::Shaping::Advanced,
                ..Default::default()
            });

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
                let face = shape.exterior().lines().nth(face).unwrap();
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
                            width: 5.0,
                            ..Default::default()
                        },
                    );
                } else if self.hovered_element == Some(index) {
                    frame.stroke(
                        &path,
                        canvas::Stroke {
                            style: canvas::Style::Solid(Color::BLACK),
                            width: 5.0,
                            ..Default::default()
                        },
                    );
                }
            }
        });

        let bg = cache.bg.draw(renderer, self.bounds().size(), |frame| {
            trace!(index = index, "Redrawing background");

            if let Some(name) = &current_style.background_image_file_name {
                let path = KEYBOARDS_PATH
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
                frame.fill(&key, Color::from(current_style.background));
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
                            self.style.default_key_style.pressed.background.into()
                        }
                    },
                );
            }
        });

        (fg, bg)
    }

    fn bounds(&self) -> Rectangle {
        Rectangle::with_size(Size::new(self.width, self.height))
    }
}

impl<Theme> Widget<Message, Theme, Renderer> for Keyboard<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.width), Length::Fixed(self.height))
    }

    fn layout(
        &self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &iced::Renderer,
        _limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        Node::new(Size::new(self.width, self.height))
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        _layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let mut fgs = Vec::new();
        let mut bgs = Vec::new();

        for i in 0..self.caches.len() {
            if state.held_element == Some(i) {
                continue;
            }
            let (fg, bg) = self.draw_element(state, renderer, i);
            fgs.push(fg);
            bgs.push(bg);
        }

        renderer.with_layer(self.bounds(), |renderer| {
            for geo in bgs {
                renderer.draw_geometry(geo);
            }
        });

        renderer.with_layer(self.bounds(), |renderer| {
            for geo in fgs {
                renderer.draw_geometry(geo);
            }
        });

        if let Some(i) = state.held_element.or(state.selected_element) {
            let (fg, bg) = self.draw_element(state, renderer, i);
            renderer.with_layer(self.bounds(), |renderer| {
                renderer.draw_geometry(bg);
            });

            renderer.with_layer(self.bounds(), |renderer| {
                renderer.draw_geometry(fg);
            });
        }
    }

    fn on_event(
        &mut self,
        state: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        _layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        let state = state.state.downcast_mut::<State>();
        if !self.edit_mode {
            return if self.hovered_element.is_some() {
                shell.publish(Message::UpdateHoveredElement(None));
                Status::Captured
            } else if state.selected_element.is_some() {
                state.selected_element = None;
                Status::Captured
            } else if state.held_element.is_some() {
                state.held_element = None;
                Status::Captured
            } else {
                Status::Ignored
            };
        }

        let Some(cursor_position) = cursor.position_in(self.bounds()) else {
            return Status::Ignored;
        };

        let cursor_position = Coord {
            x: cursor_position.x,
            y: cursor_position.y,
        };

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                match state.interaction {
                    Interaction::None => {
                        state.previous_cursor_position = cursor_position;
                        for (index, element) in self.layout.elements.iter().enumerate() {
                            match element {
                                BoardElement::MouseSpeedIndicator(def) => {
                                    if Euclidean.distance(
                                        cursor_position,
                                        Coord::from(def.location.clone()),
                                    ) < def.radius
                                    {
                                        if self.hovered_element != Some(index) {
                                            shell.publish(Message::UpdateHoveredElement(Some(
                                                index,
                                            )));
                                        }

                                        return Status::Captured;
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
                                        let lines = bounds.exterior().lines().collect::<Vec<_>>();
                                        if !lines
                                            .windows(2)
                                            .chain(std::iter::once(
                                                [*lines.last().unwrap(), *lines.first().unwrap()]
                                                    .as_slice(),
                                            ))
                                            .enumerate()
                                            .any(|(i, window)| {
                                                let left = window[0];
                                                let right = window[1];

                                                if Euclidean.distance(cursor_position, &left)
                                                    <= HOVER_EDGE_DISTANCE
                                                    && Euclidean.distance(cursor_position, &right)
                                                        <= HOVER_EDGE_DISTANCE
                                                {
                                                    if state.set_hovered_vertex(i + 1) {
                                                        shell.publish(Message::ClearCache(index));
                                                    }
                                                    true
                                                } else if Euclidean.distance(cursor_position, &left)
                                                    <= HOVER_EDGE_DISTANCE
                                                {
                                                    if state.set_hovered_face(i) {
                                                        shell.publish(Message::ClearCache(index));
                                                    }
                                                    true
                                                } else if Euclidean
                                                    .distance(cursor_position, &right)
                                                    <= HOVER_EDGE_DISTANCE
                                                {
                                                    if state.set_hovered_face(i + 1) {
                                                        shell.publish(Message::ClearCache(index));
                                                    }
                                                    true
                                                } else {
                                                    false
                                                }
                                            })
                                        {
                                            state.hovered_face = None;
                                            shell.publish(Message::ClearCache(index));
                                        }
                                        if self.hovered_element != Some(index) {
                                            shell.publish(Message::UpdateHoveredElement(Some(
                                                index,
                                            )));
                                        }
                                        return Status::Captured;
                                    }
                                }
                            }
                        }

                        if self.hovered_element.is_some() {
                            shell.publish(Message::UpdateHoveredElement(None));
                            return Status::Captured;
                        }
                    }
                    Interaction::Dragging => {
                        if state.held_element.is_some() {
                            let delta = cursor_position - state.previous_cursor_position;
                            state.previous_cursor_position = cursor_position;
                            state.delta_accumulator.x += delta.x;
                            state.delta_accumulator.y += delta.y;
                            shell.publish(Message::MoveElement {
                                index: state.held_element.unwrap(),
                                delta,
                            });
                            return Status::Captured;
                        }
                    }
                }
                state.previous_cursor_position = cursor_position;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.interaction = Interaction::Dragging;

                if state.selected_element.is_none() {
                    state.held_element = self.hovered_element;
                    if let Some(i) = state.held_element {
                        shell.publish(Message::ClearCache(i));
                    }
                } else {
                    state.held_element = state.selected_element;
                    if let Some(i) = state.selected_element {
                        shell.publish(Message::ClearCache(i));
                    }
                    state.selected_element = None;
                }

                return Status::Captured;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.delta_accumulator != Coord::default() {
                    if let Some(message) = state.held_element.map(|index| {
                        Message::PushChange(Change::MoveElement {
                            index,
                            delta: state.delta_accumulator,
                        })
                    }) {
                        shell.publish(message);
                    }
                    state.delta_accumulator = Coord::default();
                    state.selected_element = state.held_element;
                    if let Some(i) = state.selected_element {
                        shell.publish(Message::ClearCache(i));
                    }
                } else {
                    state.selected_element = self.hovered_element;
                    if let Some(i) = state.selected_element {
                        shell.publish(Message::ClearCache(i));
                    }
                }

                if let Some(i) = state.held_element {
                    shell.publish(Message::ClearCache(i));
                }
                state.held_element = None;

                state.interaction = Interaction::None;
            }
            _ => {}
        }
        Status::Ignored
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

impl<'a> From<Keyboard<'a>> for Element<'a, Message> {
    fn from(value: Keyboard<'a>) -> Self {
        Element::new(value)
    }
}
