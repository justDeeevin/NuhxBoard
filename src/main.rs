#![allow(non_snake_case)]

mod code_convert;
mod config;
mod listener;
mod style;
use clap::Parser;
use code_convert::*;
use color_eyre::{
    eyre::{Result, WrapErr},
    Section,
};
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
    pressed_keys: Vec<rdev::Key>,
    pressed_mouse_buttons: Vec<rdev::Button>,
    pressed_scroll_buttons: Vec<u32>,
    mouse_velocity: (f32, f32),
    previous_mouse_position: (f32, f32),
    previous_mouse_time: std::time::SystemTime,
    /// `(up, down, right, left)`
    queued_scrolls: (u32, u32, u32, u32),
    caps: bool,
}

#[derive(Default)]
struct Flags {
    config: Config,
    style: Style,
}

#[derive(Debug)]
enum Message {
    Listener(listener::Event),
    ReleaseScroll(u32),
    SystemInfoRead(iced::system::Information),
}

impl Application for NuhxBoard {
    type Flags = Flags;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new(flags: Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                config: flags.config,
                style: flags.style,
                canvas: Cache::default(),
                pressed_keys: Vec::new(),
                pressed_mouse_buttons: Vec::new(),
                caps: false,
                mouse_velocity: (0.0, 0.0),
                pressed_scroll_buttons: Vec::new(),
                previous_mouse_position: (0.0, 0.0),
                previous_mouse_time: std::time::SystemTime::now(),
                queued_scrolls: (0, 0, 0, 0),
            },
            iced::system::fetch_information(Message::SystemInfoRead),
        )
    }

    fn title(&self) -> String {
        String::from("NuhxBoard")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Listener(listener::Event::KeyReceived(event)) => match event.event_type {
                rdev::EventType::KeyPress(key) => {
                    if key == rdev::Key::CapsLock {
                        self.caps = !self.caps;
                    }
                    if !self.pressed_keys.contains(&key) {
                        self.pressed_keys.push(key);
                    }
                }
                rdev::EventType::KeyRelease(key) => {
                    if self.pressed_keys.contains(&key) {
                        self.pressed_keys.retain(|&x| x != key);
                    }
                }
                rdev::EventType::ButtonPress(button) => {
                    if !self.pressed_mouse_buttons.contains(&button) {
                        self.pressed_mouse_buttons.push(button);
                    }
                }
                rdev::EventType::ButtonRelease(button) => {
                    if self.pressed_mouse_buttons.contains(&button) {
                        self.pressed_mouse_buttons.retain(|&x| x != button);
                    }
                }
                rdev::EventType::Wheel { delta_x, delta_y } => {
                    let button;
                    if delta_x < 0 {
                        button = 3;
                    } else if delta_x > 0 {
                        button = 2;
                    } else if delta_y < 0 {
                        button = 1;
                    } else {
                        button = 0;
                    }

                    match button {
                        0 => self.queued_scrolls.0 += 1,
                        1 => self.queued_scrolls.1 += 1,
                        2 => self.queued_scrolls.2 += 1,
                        3 => self.queued_scrolls.3 += 1,
                        _ => {}
                    }

                    if !self.pressed_scroll_buttons.contains(&button) {
                        self.pressed_scroll_buttons.push(button);
                    }

                    self.canvas.clear();

                    return Command::perform(
                        async_std::task::sleep(std::time::Duration::from_millis(100)),
                        move |_| Message::ReleaseScroll(button),
                    );
                }
                rdev::EventType::MouseMove { x, y } => {
                    let (x, y) = (x as f32, y as f32);
                    let current_time = event.time;
                    let time_diff = current_time
                        .duration_since(self.previous_mouse_time)
                        .unwrap();
                    let position_diff = (
                        x - self.previous_mouse_position.0,
                        y - self.previous_mouse_position.1,
                    );
                    self.mouse_velocity = (
                        position_diff.0 / time_diff.as_secs_f32(),
                        position_diff.1 / time_diff.as_secs_f32(),
                    );
                    self.previous_mouse_position = (x, y);
                    self.previous_mouse_time = current_time;
                }
            },
            Message::ReleaseScroll(button) => match button {
                0 => {
                    self.queued_scrolls.0 -= 1;
                    if self.queued_scrolls.0 == 0 {
                        self.pressed_scroll_buttons.retain(|&x| x != button);
                    }
                }
                1 => {
                    self.queued_scrolls.1 -= 1;
                    if self.queued_scrolls.1 == 0 {
                        self.pressed_scroll_buttons.retain(|&x| x != button);
                    }
                }
                2 => {
                    self.queued_scrolls.2 -= 1;
                    if self.queued_scrolls.2 == 0 {
                        self.pressed_scroll_buttons.retain(|&x| x != button);
                    }
                }
                3 => {
                    self.queued_scrolls.3 -= 1;
                    if self.queued_scrolls.3 == 0 {
                        self.pressed_scroll_buttons.retain(|&x| x != button);
                    }
                }
                _ => {}
            },
            Message::SystemInfoRead(info) => {
                if info.graphics_backend == "Wayland" || info.graphics_backend == "XWayland" {
                    println!("Warning: listening for input through XWayland. Some applications, when focused, may consume input events.");
                }
            }
            _ => {}
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
        listener::bind().map(Message::Listener)
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
                .map(|rdev_code| $keycode_map(*rdev_code))
                .collect::<Vec<u32>>()
                .contains(keycode)
            {
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
            size: style.loose.font.size,
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
                stretch: if current_style.font.style & 0b10 != 0 {
                    iced::font::Stretch::Expanded
                } else {
                    iced::font::Stretch::Normal
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
                            match self.pressed_keys.contains(&rdev::Key::ShiftLeft)
                                || self.pressed_keys.contains(&rdev::Key::ShiftRight)
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
                            unit
                        );
                    }
                    BoardElement::MouseSpeedIndicator(def) => {
                        let inner = Path::circle(def.location.clone().into(), def.radius / 5.0);
                        let outer = Path::circle(def.location.clone().into(), def.radius);
                        let polar_velocity = (
                            (self.mouse_velocity.0.powi(2) + self.mouse_velocity.1.powi(2)).sqrt(),
                            self.mouse_velocity.1.atan2(self.mouse_velocity.0),
                        );
                        let squashed_magnitude = (0.0002 * polar_velocity.0).tanh();
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

fn unit<T>(var: T) -> T {
    var
}

/// NuhxBoard - The cross-platform alternative to NohBoard
#[derive(Parser, Debug)]
#[command(
    version,
    after_help = "Add keyboard groups to ~/.local/share/NuhxBoard/keyboards/"
)]
struct Args {
    /// The keyboard to use. [GROUP]/[KEYBOARD]
    #[arg(short, long)]
    keyboard: String,

    /// The style to use. Must be in the same directory as the provided keyboard. If not provided, global default will be used.
    #[arg(short, long)]
    style: Option<String>,

    /// List available keyboard groups or keyboards in a group specified by `--keyboard`
    #[arg(short, long, conflicts_with("style"))]
    list: bool,
}

static IMAGE: &[u8] = include_bytes!("../NuhxBoard.png");

fn main() -> Result<()> {
    color_eyre::install()?;

    if !home::home_dir()
        .unwrap()
        .join(".local/share/NuhxBoard")
        .exists()
    {
        let make_dir = inquire::Confirm::new(
            "NuhxBoard directory does not exist. Create it? (If no, program will exit)",
        )
        .with_default(true)
        .prompt()?;

        if make_dir {
            std::fs::create_dir_all(
                home::home_dir()
                    .unwrap()
                    .join(".local/share/NuhxBoard/keyboards"),
            )?;
        } else {
            std::process::exit(0);
        }
    }

    let args = Args::parse();

    if args.list {
        let list = std::fs::read_dir(
            home::home_dir()
                .unwrap()
                .join(".local/share/NuhxBoard/keyboards")
                .join(args.keyboard),
        )
        .wrap_err("Error listing keyboards")
        .suggestion("Make sure the given keyboard group exists")?;
        for entry in list {
            let entry = entry?;
            println!("{}", &entry.file_name().to_str().unwrap_or(""));
        }

        return Ok(());
    }

    let mut config_file = File::open(format!(
        "{}/.local/share/NuhxBoard/keyboards/{}/keyboard.json",
        home::home_dir().unwrap().to_str().unwrap(),
        args.keyboard
    ))
    .wrap_err("Error opening keyboard file")
    .suggestion("Make sure the given keyboard file exists")?;
    let mut config_string = String::new();
    config_file
        .read_to_string(&mut config_string)
        .wrap_err("Error reading keyboard file")?;
    let config: Config = serde_json::from_str(&config_string)
        .wrap_err("Error parsing keyboard file")
        .suggestion("Make sure the keyboard file is valid")?;

    let style: Style;
    if let Some(style_name) = &args.style {
        let mut style_file = File::open(format!(
            "{}/.local/share/NuhxBoard/keyboards/{}/{}.style",
            home::home_dir().unwrap().to_str().unwrap(),
            args.keyboard,
            style_name
        ))
        .wrap_err("Error opening style file")
        .suggestion("Make sure the given style file exists")?;
        let mut style_string = String::new();
        style_file
            .read_to_string(&mut style_string)
            .wrap_err("Error reading style file")?;
        style = serde_json::from_str(&style_string)
            .wrap_err("Error parsing style file")
            .suggestion("Make sure the style file is valid")?;
    } else {
        style = Style::default()
    }

    let icon_image = image::load_from_memory(IMAGE)?;
    let icon = iced::window::icon::from_rgba(icon_image.to_rgba8().to_vec(), 256, 256)?;
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
    NuhxBoard::run(settings)?;

    Ok(())
}
