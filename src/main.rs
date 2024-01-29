mod code_convert;
mod config;
mod listener;
mod style;
use clap::Parser;
use code_convert::*;
use config::*;
use iced::{
    mouse,
    widget::{
        canvas,
        canvas::{Cache, Geometry, Path},
        container,
    },
    Application, Color, Command, Length, Rectangle, Renderer, Subscription, Theme,
};
use std::{fs::File, io::prelude::*};
use style::*;

struct NuhxBoard {
    config: Config,
    style: Style,
    canvas: Cache,
    pressed_keys: Vec<u32>,
    pressed_mouse_buttons: Vec<u32>,
    pressed_scroll_buttons: Vec<u32>,
    mouse_velocity: (f32, f32),
    caps: bool,
    queued_scrolls: u32,
}

#[derive(Debug, PartialEq)]
pub enum Message {
    KeyPress { keycode: u32, caps: bool },
    KeyRelease { keycode: u32, caps: bool },
    MouseButtonPress(u32),
    MouseButtonRelease(u32),
    ScrollButtonPress(u32),
    ScrollButtonRelease(u32),
    Motion { x: f32, y: f32 },
    Dummy,
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
                pressed_mouse_buttons: Vec::new(),
                pressed_scroll_buttons: Vec::new(),
                caps: false,
                queued_scrolls: 0,
                mouse_velocity: (0.0, 0.0),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("NuhxBoard")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::KeyPress { keycode, caps } => {
                if !self.pressed_keys.contains(&keycode) {
                    self.pressed_keys.push(keycode);
                }
                if keycode == 66 {
                    self.caps = !self.caps;
                } else if self.caps != caps {
                    self.caps = caps;
                }
            }
            Message::KeyRelease { keycode, caps } => {
                if self.pressed_keys.contains(&keycode) {
                    self.pressed_keys.retain(|&x| x != keycode);
                } else if self.caps != caps {
                    self.caps = caps;
                }
            }
            Message::MouseButtonPress(keycode) => {
                if !self.pressed_mouse_buttons.contains(&keycode) {
                    self.pressed_mouse_buttons.push(keycode);
                }
            }
            Message::MouseButtonRelease(keycode) => {
                if self.pressed_mouse_buttons.contains(&keycode) {
                    self.pressed_mouse_buttons.retain(|&x| x != keycode);
                }
            }
            Message::ScrollButtonPress(keycode) => {
                // Scroll up and down release way too early to even be displayed, so instead of
                // unhighlighting them when xinput sends the release, we unhighlight them on a
                // delay.
                self.queued_scrolls += 1;
                if !self.pressed_scroll_buttons.contains(&keycode) {
                    self.pressed_scroll_buttons.push(keycode);
                }
                self.canvas.clear();
                return Command::perform(
                    async_std::task::sleep(std::time::Duration::from_millis(100)),
                    move |_| Message::ScrollButtonRelease(keycode),
                );
            }
            Message::ScrollButtonRelease(keycode) => {
                self.queued_scrolls -= 1;
                if self.pressed_scroll_buttons.contains(&keycode) && (self.queued_scrolls == 0) {
                    self.pressed_scroll_buttons.retain(|&x| x != keycode);
                }
            }
            Message::Motion { x, y } => self.mouse_velocity = (x, y),
            Message::Dummy => {
                return Command::none();
            }
        }
        self.canvas.clear();
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

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::from_recipe(listener::InputSubscription)
    }
}

macro_rules! draw_key {
    ($self: ident, $def: ident, $frame: ident, $content: expr, $pressed_button_list: expr, $keycode_map: expr) => {
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
            if $pressed_button_list
                .iter()
                .map(|xinput_code: &u32| $keycode_map(*xinput_code))
                .collect::<Vec<u32>>()
                .contains(keycode)
            {
                pressed = true;
                break;
            }
        }

        let fill_color = match pressed {
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
            content: $content,
            position: $def.text_position.clone().into(),
            color: match pressed {
                true => Color::from_rgb(
                    style.pressed.text.red / 255.0,
                    style.pressed.text.green / 255.0,
                    style.pressed.text.blue / 255.0,
                ),
                false => Color::from_rgb(
                    style.loose.text.red / 255.0,
                    style.loose.text.green / 255.0,
                    style.loose.text.blue / 255.0,
                ),
            },
            size: style.loose.font.size,
            font: iced::Font {
                family: iced::font::Family::Name(match pressed {
                    // Leak is required because Name requires static lifetime
                    // as opposed to application lifetime :(
                    // I suppose they were just expecting you to pass in a
                    // literal here... damn you!!
                    true => style.pressed.font.font_family.clone().leak(),
                    false => style.loose.font.font_family.clone().leak(),
                }),
                weight: match pressed {
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
                stretch: match pressed {
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
                        draw_key!(
                            self,
                            def,
                            frame,
                            match self.pressed_keys.contains(&50)
                                || self.pressed_keys.contains(&62)
                                || (self.caps && def.change_on_caps)
                            {
                                true => def.shift_text.clone(),
                                false => def.text.clone(),
                            },
                            self.pressed_keys,
                            keycode_convert
                        );
                    }
                    BoardElement::MouseKey(def) => {
                        draw_key!(
                            self,
                            def,
                            frame,
                            def.text.clone(),
                            self.pressed_mouse_buttons,
                            mouse_button_code_convert
                        );
                    }
                    BoardElement::MouseScroll(def) => {
                        draw_key!(
                            self,
                            def,
                            frame,
                            def.text.clone(),
                            self.pressed_scroll_buttons,
                            mouse_button_code_convert
                        );
                    }
                    BoardElement::MouseSpeedIndicator(def) => {
                        let inner = Path::circle(def.location.clone().into(), def.radius / 5.0);
                        let outer = Path::circle(def.location.clone().into(), def.radius);
                        let polar_velocity = (
                            (self.mouse_velocity.0.powi(2) + self.mouse_velocity.1.powi(2)).sqrt(),
                            self.mouse_velocity.1.atan2(self.mouse_velocity.0),
                        );
                        let squashed_magnitude = (0.01 * polar_velocity.0).tanh();
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
                                colorgrad::Color::new(1.0, 1.0, 1.0, 1.0),
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

#[derive(Parser, Debug)]
#[command(author = "justDeeevin", version = "0.1.0")]
struct Args {
    #[arg(short, long)]
    config_path: String,

    #[arg(short, long)]
    style_path: Option<String>,
}

fn main() {
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

    let style: Style;
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
            resizable: false,
            icon: Some(icon),
            ..iced::window::Settings::default()
        },
        flags,
        ..iced::Settings::default()
    };
    NuhxBoard::run(settings).unwrap();

    std::process::Command::new("killall")
        .arg("xinput")
        .spawn()
        .unwrap();
}
