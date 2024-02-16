#![allow(non_snake_case)]

mod code_convert;
mod config;
mod listener;
mod style;
mod stylesheets;
use clap::Parser;
use code_convert::*;
use color_eyre::eyre::Result;
use config::*;
use iced::{
    mouse,
    multi_window::Application,
    widget::{
        button, canvas,
        canvas::{Cache, Geometry, Path},
        column, container, pick_list, row, text,
    },
    Color, Command, Length, Rectangle, Renderer, Subscription, Theme,
};
use iced_aw::{ContextMenu, SelectionList};
use owo_colors::OwoColorize;
use std::sync::Arc;
use std::{
    fs::{self, File},
    io::prelude::*,
};
use style::*;
use stylesheets::*;

struct NuhxBoard {
    config: Config,
    style: Style,
    canvas: Cache,
    pressed_keys: Vec<u32>,
    pressed_mouse_buttons: Vec<u32>,
    pressed_scroll_buttons: Vec<u32>,
    mouse_velocity: (f32, f32),
    previous_mouse_position: (f32, f32),
    previous_mouse_time: std::time::SystemTime,
    /// `(up, down, right, left)`
    queued_scrolls: (u32, u32, u32, u32),
    caps: bool,
    verbose: bool,
    load_keyboard_window_id: Option<iced::window::Id>,
    keyboard_category: Option<String>,
    keyboard: Option<String>,
}

#[derive(Default)]
struct Flags {
    verbose: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Listener(listener::Event),
    ReleaseScroll(u32),
    LoadStyle(String),
    OpenLoadKeyboardMenu,
    WindowClosed(iced::window::Id),
    ChangeKeyboardCategory(String),
    LoadKeyboard(String),
}

const DEFAULT_WINDOW_SIZE: iced::Size = iced::Size {
    width: 200.0,
    height: 200.0,
};

const LOAD_KEYBOARD_WINDOW_SIZE: iced::Size = iced::Size {
    width: 300.0,
    height: 250.0,
};

impl Application for NuhxBoard {
    type Flags = Flags;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new(flags: Flags) -> (Self, Command<Self::Message>) {
        #[cfg(target_os = "linux")]
        {
            if std::env::var("XDG_SESSION_TYPE").unwrap() == "wayland" {
                println!("Warning: grabbing input events throuh XWayland. Some windows may consume input events.");
            }
        }
        let (id, command) = iced::window::spawn::<Message>(iced::window::Settings {
            resizable: false,
            size: LOAD_KEYBOARD_WINDOW_SIZE,
            ..Default::default()
        });

        let config = Config {
            version: 2,
            width: DEFAULT_WINDOW_SIZE.width,
            height: DEFAULT_WINDOW_SIZE.height,
            elements: vec![],
        };

        (
            Self {
                config,
                style: Style::default(),
                canvas: Cache::default(),
                pressed_keys: Vec::new(),
                pressed_mouse_buttons: Vec::new(),
                caps: false,
                mouse_velocity: (0.0, 0.0),
                pressed_scroll_buttons: Vec::new(),
                previous_mouse_position: (0.0, 0.0),
                previous_mouse_time: std::time::SystemTime::now(),
                queued_scrolls: (0, 0, 0, 0),
                verbose: flags.verbose,
                load_keyboard_window_id: Some(id),
                keyboard_category: None,
                keyboard: None,
            },
            command,
        )
    }

    fn title(&self, _window: iced::window::Id) -> String {
        String::from("NuhxBoard")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        if self.verbose {
            dbg!(&message);
        }
        match message {
            Message::Listener(listener::Event::KeyReceived(event)) => match event.event_type {
                rdev::EventType::KeyPress(key) => {
                    if let Err(bad_key) = keycode_convert(key) {
                        eprintln!("{}{:?}", "Unknown rdev keycode: ".red(), bad_key.red());
                        return Command::none();
                    }
                    if key == rdev::Key::CapsLock {
                        self.caps = !self.caps;
                    }
                    let key = keycode_convert(key).unwrap();
                    if !self.pressed_keys.contains(&key) {
                        self.pressed_keys.push(key);
                    }
                }
                rdev::EventType::KeyRelease(key) => {
                    if let Err(bad_key) = keycode_convert(key) {
                        eprintln!("{}{:?}", "Unknown rdev keycode: ".red(), bad_key.red());
                        return Command::none();
                    }
                    let key = keycode_convert(key).unwrap();
                    if self.pressed_keys.contains(&key) {
                        self.pressed_keys.retain(|&x| x != key);
                    }
                }
                rdev::EventType::ButtonPress(button) => {
                    if let Err(bad_button) = mouse_button_code_convert(button) {
                        eprintln!(
                            "{}{:?}",
                            "Unknown rdev mouse button code: ".red(),
                            bad_button.red()
                        );
                        return Command::none();
                    }
                    let button = mouse_button_code_convert(button).unwrap();
                    if !self.pressed_mouse_buttons.contains(&button) {
                        self.pressed_mouse_buttons.push(button);
                    }
                }
                rdev::EventType::ButtonRelease(button) => {
                    if let Err(bad_button) = mouse_button_code_convert(button) {
                        eprintln!(
                            "{}{:?}",
                            "Unknown rdev mouse button code: ".red(),
                            bad_button.red()
                        );
                        return Command::none();
                    }
                    let button = mouse_button_code_convert(button).unwrap();
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
                    let time_diff = match current_time.duration_since(self.previous_mouse_time) {
                        Ok(diff) => diff,
                        Err(_) => return Command::none(),
                    };
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
            Message::OpenLoadKeyboardMenu => {
                let (id, command) = iced::window::spawn::<Message>(iced::window::Settings {
                    resizable: false,
                    size: LOAD_KEYBOARD_WINDOW_SIZE,
                    ..Default::default()
                });
                self.load_keyboard_window_id = Some(id);
                return command;
            }
            Message::ChangeKeyboardCategory(category) => {
                self.keyboard_category = Some(category);
                self.keyboard = None;
            }
            Message::LoadKeyboard(keyboard) => {
                self.keyboard = Some(keyboard.clone());
                self.style = Style::default();

                let mut path = home::home_dir().unwrap();
                path.push(".local/share/NuhxBoard/keyboards");
                path.push(self.keyboard_category.as_ref().unwrap());
                path.push(keyboard);
                path.push("keyboard.json");
                let mut config_file = File::open(path).unwrap();
                let mut config_string = "".into();
                config_file.read_to_string(&mut config_string).unwrap();

                self.config = serde_json::from_str(config_string.as_str()).unwrap();
                return iced::window::resize(
                    iced::window::Id::MAIN,
                    iced::Size {
                        width: self.config.width,
                        height: self.config.height,
                    },
                );
            }
            Message::LoadStyle(style) => {
                let mut path = home::home_dir().unwrap();
                path.push(".local/share/NuhxBoard/keyboards");
                path.push(self.keyboard_category.as_ref().unwrap());
                path.push(self.keyboard.as_ref().unwrap());
                path.push(format!("{}.style", style));
                dbg!(&path);

                let mut style_file = File::open(path).unwrap();
                let mut style_string = "".into();
                style_file.read_to_string(&mut style_string).unwrap();
                self.style = serde_json::from_str(style_string.as_str()).unwrap();
            }
            Message::WindowClosed(id) => {
                if let Some(load_keyboard_window_id) = self.load_keyboard_window_id {
                    if id == load_keyboard_window_id {
                        self.load_keyboard_window_id = None;
                    } else if id == iced::window::Id::MAIN {
                        return iced::window::close(load_keyboard_window_id);
                    }
                }
            }
            _ => {}
        }
        self.canvas.clear();
        Command::none()
    }

    fn view(
        &self,
        window: iced::window::Id,
    ) -> iced::Element<'_, Self::Message, Self::Theme, crate::Renderer> {
        if window == iced::window::Id::MAIN {
            let canvas = canvas::<&NuhxBoard, Message, Theme, Renderer>(self)
                .height(Length::Fill)
                .width(Length::Fill);

            let load_keyboard_menu_message = match self.load_keyboard_window_id {
                Some(_) => None,
                None => Some(Message::OpenLoadKeyboardMenu),
            };
            ContextMenu::new(canvas, move || {
                container(column([button("Load Keyboard")
                    .on_press_maybe(load_keyboard_menu_message.clone())
                    .style(iced::theme::Button::Custom(Box::new(WhiteButton {})))
                    .into()]))
                .into()
            })
            .into()
        } else if let Some(load_keyboard_window) = self.load_keyboard_window_id {
            if load_keyboard_window == window {
                let mut path = home::home_dir().unwrap();
                path.push(".local/share/NuhxBoard/keyboards");

                let keyboard_category_options = fs::read_dir(&path)
                    .unwrap()
                    .map(|r| r.unwrap())
                    .filter(|entry| entry.file_type().unwrap().is_dir())
                    .map(|entry| entry.file_name().to_str().unwrap().to_owned())
                    .collect::<Vec<_>>();

                let keyboard_options = if let Some(category) = &self.keyboard_category {
                    path.push(category);
                    fs::read_dir(&path)
                        .unwrap()
                        .map(|r| r.unwrap())
                        .filter(|entry| entry.file_type().unwrap().is_dir())
                        .map(|entry| entry.file_name().to_str().unwrap().to_owned())
                        .collect()
                } else {
                    vec![]
                };

                let style_options = if let Some(keyboard) = &self.keyboard {
                    path.push(keyboard);
                    fs::read_dir(&path)
                        .unwrap()
                        .map(|r| r.unwrap())
                        .filter(|entry| entry.file_type().unwrap().is_file())
                        .filter(|entry| {
                            entry.path().extension() == Some(std::ffi::OsStr::new("style"))
                        })
                        .map(|entry| {
                            entry
                                .path()
                                .file_stem()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_owned()
                        })
                        .collect()
                } else {
                    vec![]
                };

                column([
                    text("Category:").into(),
                    pick_list(
                        keyboard_category_options,
                        self.keyboard_category.clone(),
                        Message::ChangeKeyboardCategory,
                    )
                    .into(),
                    row([
                        SelectionList::new(keyboard_options.leak(), |_, selection| {
                            Message::LoadKeyboard(selection)
                        })
                        .into(),
                        SelectionList::new(style_options.leak(), |_, selection| {
                            Message::LoadStyle(selection)
                        })
                        .into(),
                    ])
                    .into(),
                ])
                .into()
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }

    fn theme(&self, window: iced::window::Id) -> Self::Theme {
        if window == iced::window::Id::MAIN {
            let red = self.style.background_color.red / 255.0;
            let green = self.style.background_color.green / 255.0;
            let blue = self.style.background_color.blue / 255.0;
            let palette = iced::theme::Palette {
                background: Color::from_rgb(red, green, blue),
                ..iced::theme::Palette::DARK
            };
            Theme::Custom(Arc::new(iced::theme::Custom::new("Custom".into(), palette)))
        } else if let Some(load_keyboard_window) = self.load_keyboard_window_id {
            if load_keyboard_window == window {
                Theme::Light
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            listener::bind().map(Message::Listener),
            iced::event::listen_with(|event, _| match event {
                iced::Event::Window(id, iced::window::Event::Closed) => {
                    Some(Message::WindowClosed(id))
                }
                _ => None,
            }),
        ])
    }
}

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
            if $pressed_button_list.contains(keycode) {
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
            ..canvas::Text::default()
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
                                    .contains(&keycode_convert(rdev::Key::ShiftLeft).unwrap())
                                    || self
                                        .pressed_keys
                                        .contains(&keycode_convert(rdev::Key::ShiftRight).unwrap());
                                match def.change_on_caps {
                                    true => match self.caps ^ shift_pressed {
                                        true => def.shift_text.clone(),
                                        false => def.text.clone(),
                                    },
                                    false => match shift_pressed {
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

/// NuhxBoard - The cross-platform alternative to NohBoard
#[derive(Parser, Debug)]
#[command(
    version,
    after_help = "Add keyboard categorys to ~/.local/share/NuhxBoard/keyboards/"
)]
struct Args {
    /// Display debug info
    #[arg(short, long)]
    verbose: bool,
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

    let icon_image = image::load_from_memory(IMAGE)?;
    let icon = iced::window::icon::from_rgba(icon_image.to_rgba8().to_vec(), 256, 256)?;
    let flags = Flags {
        verbose: args.verbose,
    };

    let settings = iced::Settings {
        window: iced::window::Settings {
            size: DEFAULT_WINDOW_SIZE,
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
