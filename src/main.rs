#![allow(unused)]
mod config;
mod listener;
mod style;
use clap::Parser;
use colors_transform::{AlphaColor, Color as ColorTrait, Rgb};
use config::*;
use iced::{
    keyboard, mouse,
    widget::{
        canvas,
        canvas::{Cache, Geometry, Path},
        container,
    },
    Application, Color, Command, Event, Length, Point, Rectangle, Renderer, Subscription, Theme,
};
use serde::Deserialize;
use std::{fs::File, io::prelude::*};
use style::*;

struct NuhxBoard {
    config: Config,
    style: Style,
    canvas: Cache,
    pressed_keys: Vec<u32>,
    caps: bool,
}

#[derive(Debug)]
pub enum Message {
    KeyPress { keycode: u32, caps: bool },
    KeyRelease { keycode: u32, caps: bool },
    MouseButtonPress(u32),
    MouseButtonRelease(u32),
    Motion { x: f32, y: f32 },
}

#[derive(Default)]
struct Flags {
    config: Config,
    style: Style,
}

impl Application for NuhxBoard {
    type Flags = Flags;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new(flags: Flags) -> (Self, Command<Message>) {
        (
            Self {
                config: flags.config,
                style: flags.style,
                canvas: Cache::default(),
                pressed_keys: Vec::new(),
                caps: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("NuhxBoard")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let canvas = canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill);

        container(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        let red = self.style.background_color.red / 255.0;
        let green = self.style.background_color.green / 255.0;
        let blue = self.style.background_color.blue / 255.0;
        let palette: iced::theme::Palette = iced::theme::Palette {
            background: Color::from_rgb(red, green, blue),
            ..iced::theme::Palette::DARK
        };
        Theme::Custom(Box::new(iced::theme::Custom::new(palette)))
    }
}

macro_rules! draw_key {
    ($self: ident, $def: ident, $frame: ident) => {
        let mut boundaries_iter = $def.boundaries.iter();
        let key = Path::new(|builder| {
            builder.move_to((*boundaries_iter.next().unwrap()).clone().into());
            for boundary in boundaries_iter {
                builder.line_to((*boundary).clone().into());
            }
            builder.close()
        });

        let element_style = &$self
            .style
            .element_styles
            .iter()
            .find(|style| style.key == $def.id);

        let mut style: &KeyStyle;

        if let Some(s) = element_style {
            style = match &s.value {
                ElementStyleUnion::KeyStyle(i_s) => i_s,
                ElementStyleUnion::MouseSpeedIndicatorStyle(_) => unreachable!(),
            };
        } else {
            style = &$self.style.default_key_style;
        }

        let fill_color = match $self.pressed_keys.contains(&$def.id) {
            true => &style.pressed.background,
            false => &style.loose.background,
        };
        $frame.fill(
            &key,
            Color::from_rgb(
                fill_color.red / 255.0,
                fill_color.blue / 255.0,
                fill_color.green / 255.0,
            ),
        );
        $frame.fill_text(canvas::Text {
            content: $def.text.clone(),
            position: $def.text_position.clone().into(),
            color: match $self.pressed_keys.contains(&$def.id) {
                true => Color::from_rgb(
                    style.pressed.text.red,
                    style.pressed.text.green,
                    style.pressed.text.blue,
                ),
                false => Color::from_rgb(
                    style.loose.text.red,
                    style.loose.text.green,
                    style.loose.text.blue,
                ),
            },
            size: style.loose.font.size,
            font: iced::Font {
                family: iced::font::Family::Name(match $self.pressed_keys.contains(&$def.id) {
                    // Leak is required because Name requires static lifetime
                    // as opposed to application lifetime :(
                    // I suppose they were just expecting you to pass in a
                    // literal here... damn you!!
                    true => style.pressed.font.font_family.clone().leak(),
                    false => style.loose.font.font_family.clone().leak(),
                }),
                weight: match $self.pressed_keys.contains(&$def.id) {
                    true => {
                        if style.pressed.font.style & 1 != 0 {
                            iced::font::Weight::Bold
                        } else {
                            iced::font::Weight::Normal
                        }
                    }
                    false => {
                        if style.loose.font.style & 1 != 0 {
                            iced::font::Weight::Bold
                        } else {
                            iced::font::Weight::Normal
                        }
                    }
                },
                stretch: match $self.pressed_keys.contains(&$def.id) {
                    true => {
                        if style.pressed.font.style & 0b10 != 0 {
                            iced::font::Stretch::Expanded
                        } else {
                            iced::font::Stretch::Normal
                        }
                    }
                    false => {
                        if style.loose.font.style & 0b10 != 0 {
                            iced::font::Stretch::Expanded
                        } else {
                            iced::font::Stretch::Normal
                        }
                    }
                },
                monospaced: false,
            },
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        })
    };
}

impl<Message> canvas::Program<Message, Renderer> for NuhxBoard {
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
                        let mut boundaries_iter = def.boundaries.iter();
                        let key = Path::new(|builder| {
                            builder.move_to((*boundaries_iter.next().unwrap()).clone().into());
                            for boundary in boundaries_iter {
                                builder.line_to((*boundary).clone().into());
                            }
                            builder.close()
                        });

                        let element_style = &self
                            .style
                            .element_styles
                            .iter()
                            .find(|style| style.key == def.id);

                        let mut style: &KeyStyle;

                        if let Some(s) = element_style {
                            style = match &s.value {
                                ElementStyleUnion::KeyStyle(i_s) => i_s,
                                ElementStyleUnion::MouseSpeedIndicatorStyle(_) => unreachable!(),
                            };
                        } else {
                            style = &self.style.default_key_style;
                        }

                        let fill_color = match self.pressed_keys.contains(&def.id) {
                            true => &style.pressed.background,
                            false => &style.loose.background,
                        };
                        frame.fill(
                            &key,
                            Color::from_rgb(
                                fill_color.red / 255.0,
                                fill_color.blue / 255.0,
                                fill_color.green / 255.0,
                            ),
                        );
                        frame.fill_text(canvas::Text {
                            content: match self.pressed_keys.contains(&def.id)
                                || (self.caps && def.change_on_caps)
                            {
                                true => def.shift_text.clone(),
                                false => def.text.clone(),
                            },
                            position: def.text_position.clone().into(),
                            color: match self.pressed_keys.contains(&def.id) {
                                true => Color::from_rgb(
                                    style.pressed.text.red,
                                    style.pressed.text.green,
                                    style.pressed.text.blue,
                                ),
                                false => Color::from_rgb(
                                    style.loose.text.red,
                                    style.loose.text.green,
                                    style.loose.text.blue,
                                ),
                            },
                            size: style.loose.font.size,
                            font: iced::Font {
                                family: iced::font::Family::Name(
                                    match self.pressed_keys.contains(&def.id) {
                                        // Leak is required because Name requires static lifetime
                                        // as opposed to application lifetime :(
                                        // I suppose they were just expecting you to pass in a
                                        // literal here... damn you!!
                                        true => style.pressed.font.font_family.clone().leak(),
                                        false => style.loose.font.font_family.clone().leak(),
                                    },
                                ),
                                weight: match self.pressed_keys.contains(&def.id) {
                                    true => {
                                        if style.pressed.font.style & 1 != 0 {
                                            iced::font::Weight::Bold
                                        } else {
                                            iced::font::Weight::Normal
                                        }
                                    }
                                    false => {
                                        if style.loose.font.style & 1 != 0 {
                                            iced::font::Weight::Bold
                                        } else {
                                            iced::font::Weight::Normal
                                        }
                                    }
                                },
                                stretch: match self.pressed_keys.contains(&def.id) {
                                    true => {
                                        if style.pressed.font.style & 0b10 != 0 {
                                            iced::font::Stretch::Expanded
                                        } else {
                                            iced::font::Stretch::Normal
                                        }
                                    }
                                    false => {
                                        if style.loose.font.style & 0b10 != 0 {
                                            iced::font::Stretch::Expanded
                                        } else {
                                            iced::font::Stretch::Normal
                                        }
                                    }
                                },
                                monospaced: false,
                            },
                            horizontal_alignment: iced::alignment::Horizontal::Center,
                            vertical_alignment: iced::alignment::Vertical::Center,
                            ..canvas::Text::default()
                        })
                    }
                    BoardElement::MouseKey(def) => {
                        draw_key!(self, def, frame);
                    }
                    BoardElement::MouseScroll(def) => {
                        draw_key!(self, def, frame);
                    }
                    BoardElement::MouseSpeedIndicator(def) => {
                        let inner = Path::rectangle(
                            Point::new(def.location.x - 5.0, def.location.y - 5.0),
                            iced::Size::new(10.0, 10.0),
                        );

                        let outer = Path::circle(def.location.clone().into(), def.radius as f32);

                        let element_style = &self
                            .style
                            .element_styles
                            .iter()
                            .find(|style| style.key == def.id);

                        let mut style: &MouseSpeedIndicatorStyle;

                        let global_default_style =
                            Style::default().default_mouse_indicator_style.unwrap();
                        let default_style = &self
                            .style
                            .default_mouse_indicator_style
                            .as_ref()
                            .unwrap_or(&global_default_style);

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
                                style.inner_color.red,
                                style.inner_color.green,
                                style.inner_color.blue,
                            ),
                        );

                        frame.stroke(
                            &outer,
                            canvas::Stroke {
                                width: style.outline_width,
                                line_cap: canvas::LineCap::Round,
                                style: canvas::Style::Solid(Color::from_rgb(
                                    style.outer_color.red,
                                    style.outer_color.green,
                                    style.outer_color.blue,
                                )),
                                line_dash: canvas::LineDash {
                                    segments: &[],
                                    offset: 0,
                                },
                                line_join: canvas::LineJoin::Round,
                            },
                        )
                    }
                }
            }
        });
        vec![canvas]
    }
}

#[derive(Parser, Debug)]
#[command(author = "justDeeevin", version = "0.1.0")]
struct Args {
    #[arg(short, long)]
    config_path: String,

    #[arg(short, long)]
    style_path: Option<String>,
}

fn main() -> iced::Result {
    let args = Args::parse();

    let mut config_file = match File::open(&args.config_path) {
        Err(why) => panic!(
            "Error opening config file (given path: {}): {}",
            args.config_path, why
        ),
        Ok(file) => file,
    };
    let mut config_string = String::new();
    if let Err(why) = config_file.read_to_string(&mut config_string) {
        panic!("Error reading config file: {}", why)
    };
    let config: Config = match serde_json::from_str(&config_string) {
        Err(why) => panic!("Error parsing config file: {}", why),
        Ok(config) => config,
    };

    let mut style: Style;
    if let Some(style_path) = &args.style_path {
        let mut style_file = match File::open(style_path) {
            Err(why) => panic!(
                "Error opening style file (given path: {}): {}",
                style_path, why
            ),
            Ok(file) => file,
        };
        let mut style_string = String::new();
        if let Err(why) = style_file.read_to_string(&mut style_string) {
            panic!("Error reading style file: {}", why)
        };
        style = match serde_json::from_str(&style_string) {
            Err(why) => panic!("Error parsing style file: {}", why),
            Ok(style) => style,
        };
    } else {
        style = Style::default()
    }

    let icon = iced::window::icon::from_file(std::path::Path::new("NuhxBoard.png")).unwrap();
    let flags = Flags { config, style };
    let settings = iced::Settings {
        window: iced::window::Settings {
            size: (flags.config.width, flags.config.height),
            resizable: true,
            icon: Some(icon),
            ..iced::window::Settings::default()
        },
        flags,
        ..iced::Settings::default()
    };
    NuhxBoard::run(settings)
}
