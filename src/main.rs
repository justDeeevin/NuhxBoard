#![allow(non_snake_case)]
#![windows_subsystem = "windows"]
#![feature(let_chains)]

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
    window, Color, Command, Length, Rectangle, Renderer, Subscription, Theme,
};
use iced_aw::{ContextMenu, SelectionList};
use std::sync::Arc;
use std::{
    collections::HashMap,
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
    load_keyboard_window_id: Option<window::Id>,
    keyboard_category: Option<String>,
    keyboard: Option<usize>,
    style_choice: Option<usize>,
    error_windows: HashMap<window::Id, Error>,
    keyboard_options: Vec<String>,
    keyboard_category_options: Vec<String>,
    style_options: Vec<StyleChoice>,
    keyboards_path: std::path::PathBuf,
}

#[derive(Default)]
struct Flags {
    verbose: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum StyleChoice {
    Default,
    Custom(String),
}

impl std::fmt::Display for StyleChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StyleChoice::Default => write!(f, "Global Default"),
            StyleChoice::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Listener(listener::Event),
    ReleaseScroll(u32),
    LoadStyle(usize),
    OpenLoadKeyboardMenu,
    WindowClosed(window::Id),
    ChangeKeyboardCategory(String),
    LoadKeyboard(usize),
}

#[derive(Debug)]
enum Error {
    ConfigOpen(std::io::Error),
    ConfigParse(serde_json::Error),
    StyleOpen(std::io::Error),
    StyleParse(serde_json::Error),
    UnknownKey(rdev::Key),
    UnknownButton(rdev::Button),
}

const DEFAULT_WINDOW_SIZE: iced::Size = iced::Size {
    width: 200.0,
    height: 200.0,
};

const LOAD_KEYBOARD_WINDOW_SIZE: iced::Size = iced::Size {
    width: 300.0,
    height: 250.0,
};

async fn noop() {}

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

        let mut path = home::home_dir().unwrap();
        path.push(".local/share/NuhxBoard/keyboards");
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
                load_keyboard_window_id: None,
                keyboard_category: None,
                keyboard: None,
                style_choice: None,
                error_windows: HashMap::new(),
                keyboard_options: vec![],
                keyboard_category_options: vec![],
                style_options: vec![],
                keyboards_path: path,
            },
            Command::perform(noop(), |_| Message::OpenLoadKeyboardMenu),
        )
    }

    fn title(&self, _window: window::Id) -> String {
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
                        return self.error(Error::UnknownKey(bad_key));
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
                        return self.error(Error::UnknownKey(bad_key));
                    }
                    let key = keycode_convert(key).unwrap();
                    if self.pressed_keys.contains(&key) {
                        self.pressed_keys.retain(|&x| x != key);
                    }
                }
                rdev::EventType::ButtonPress(button) => {
                    if let Err(bad_button) = mouse_button_code_convert(button) {
                        return self.error(Error::UnknownButton(bad_button));
                    }
                    let button = mouse_button_code_convert(button).unwrap();
                    if !self.pressed_mouse_buttons.contains(&button) {
                        self.pressed_mouse_buttons.push(button);
                    }
                }
                rdev::EventType::ButtonRelease(button) => {
                    if let Err(bad_button) = mouse_button_code_convert(button) {
                        return self.error(Error::UnknownButton(bad_button));
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
                let path = self.keyboards_path.clone();
                let (id, command) = window::spawn::<Message>(window::Settings {
                    resizable: false,
                    size: LOAD_KEYBOARD_WINDOW_SIZE,
                    ..Default::default()
                });
                self.load_keyboard_window_id = Some(id);

                self.keyboard_category_options = fs::read_dir(path)
                    .unwrap()
                    .map(|r| r.unwrap())
                    .filter(|entry| entry.file_type().unwrap().is_dir())
                    .map(|entry| entry.file_name().to_str().unwrap().to_owned())
                    .collect::<Vec<_>>();

                return command;
            }
            Message::ChangeKeyboardCategory(category) => {
                let mut path = self.keyboards_path.clone();

                self.keyboard_category = Some(category);
                self.keyboard = None;
                self.style_choice = None;
                self.style_options = vec![];
                self.keyboard_options = if let Some(category) = &self.keyboard_category {
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
            }
            Message::LoadKeyboard(keyboard) => {
                self.keyboard = Some(keyboard);
                self.style = Style::default();

                let mut path = self.keyboards_path.clone();
                path.push(self.keyboard_category.as_ref().unwrap());
                path.push(self.keyboard_options[keyboard].clone());
                path.push("keyboard.json");
                let mut config_file = match File::open(path) {
                    Ok(file) => file,
                    Err(e) => {
                        return self.error(Error::ConfigOpen(e));
                    }
                };
                let mut config_string = "".into();
                config_file.read_to_string(&mut config_string).unwrap();

                self.config = match serde_json::from_str(config_string.as_str()) {
                    Ok(config) => config,
                    Err(e) => return self.error(Error::ConfigParse(e)),
                };

                let mut path = self.keyboards_path.clone();
                path.push(self.keyboard_category.as_ref().unwrap());
                path.push(self.keyboard_options[keyboard].clone());

                self.style_options.push(StyleChoice::Default);
                self.style_options.append(
                    &mut fs::read_dir(&path)
                        .unwrap()
                        .map(|r| r.unwrap())
                        .filter(|entry| entry.file_type().unwrap().is_file())
                        .filter(|entry| {
                            entry.path().extension() == Some(std::ffi::OsStr::new("style"))
                        })
                        .map(|entry| {
                            StyleChoice::Custom(
                                entry
                                    .path()
                                    .file_stem()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_owned(),
                            )
                        })
                        .collect(),
                );
                self.style_choice = Some(0);

                return window::resize(
                    window::Id::MAIN,
                    iced::Size {
                        width: self.config.width,
                        height: self.config.height,
                    },
                );
            }
            Message::LoadStyle(style) => {
                self.style_choice = Some(style);

                if self.style_options[style] == StyleChoice::Default {
                    self.style = Style::default();
                    return Command::none();
                }
                let style = match &self.style_options[style] {
                    StyleChoice::Custom(style) => style,
                    _ => unreachable!(),
                };
                let mut path = home::home_dir().unwrap();
                path.push(".local/share/NuhxBoard/keyboards");
                path.push(self.keyboard_category.as_ref().unwrap());
                path.push(self.keyboard_options[self.keyboard.unwrap()].clone());
                path.push(format!("{}.style", style));

                let mut style_file = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => {
                        return self.error(Error::StyleOpen(e));
                    }
                };
                let mut style_string = "".into();
                style_file.read_to_string(&mut style_string).unwrap();
                self.style = match serde_json::from_str(style_string.as_str()) {
                    Ok(style) => style,
                    Err(e) => return self.error(Error::StyleParse(e)),
                };
            }
            Message::WindowClosed(id) => {
                let mut commands = vec![];
                if let Some(load_keyboard_window_id) = self.load_keyboard_window_id
                    && id == load_keyboard_window_id
                {
                    self.load_keyboard_window_id = None;
                } else if id == window::Id::MAIN {
                    if let Some(load_keyboard_window_id) = self.load_keyboard_window_id {
                        commands.push(window::close(load_keyboard_window_id));
                    }
                    for error_window in &self.error_windows {
                        commands.push(window::close(*error_window.0));
                    }
                }
                if self.error_windows.contains_key(&id) {
                    self.error_windows.remove(&id);
                }
                return Command::batch(commands);
            }
            _ => {}
        }
        self.canvas.clear();
        Command::none()
    }

    fn view(
        &self,
        window: window::Id,
    ) -> iced::Element<'_, Self::Message, Self::Theme, crate::Renderer> {
        if window == window::Id::MAIN {
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
        } else if let Some(load_keyboard_window) = self.load_keyboard_window_id
            && load_keyboard_window == window
        {
            column([
                text("Category:").into(),
                pick_list(
                    self.keyboard_category_options.clone(),
                    self.keyboard_category.clone(),
                    Message::ChangeKeyboardCategory,
                )
                .into(),
                row([
                    SelectionList::new_with(
                        self.keyboard_options.clone().leak(),
                        |i, _| Message::LoadKeyboard(i),
                        12.0,
                        5.0,
                        <Theme as iced_aw::style::selection_list::StyleSheet>::Style::default(),
                        self.keyboard,
                        iced::Font::default(),
                    )
                    .into(),
                    SelectionList::new_with(
                        self.style_options.clone().leak(),
                        |i, _| Message::LoadStyle(i),
                        12.0,
                        5.0,
                        <Theme as iced_aw::style::selection_list::StyleSheet>::Style::default(),
                        self.style_choice,
                        iced::Font::default(),
                    )
                    .into(),
                ])
                .into(),
            ])
            .into()
        } else if self.error_windows.contains_key(&window) {
            let error = self.error_windows.get(&window).unwrap();
            let kind = match error {
                Error::ConfigOpen(_) => "Keyboard file could not be opened.",
                Error::ConfigParse(_) => "Keyboard file could not be parsed.",
                Error::StyleOpen(_) => "Style file could not be opened.",
                Error::StyleParse(_) => "Style file could not be parsed.",
                Error::UnknownKey(_) => "Unknown Key.",
                Error::UnknownButton(_) => "Unknown Mouse Button.",
            };
            let info = match error {
                Error::ConfigParse(e) => {
                    if e.is_eof() {
                        format!("Unexpected EOF (End of file) at line {}", e.line())
                    } else {
                        format!("{}", e)
                    }
                }
                Error::ConfigOpen(e) => format!("{}", e),
                Error::StyleParse(e) => {
                    if e.is_eof() {
                        format!("Unexpeted EOF (End of file) at line {}", e.line())
                    } else {
                        format!("{}", e)
                    }
                }
                Error::StyleOpen(e) => format!("{}", e),
                Error::UnknownKey(key) => format!("Key: {:?}", key),
                Error::UnknownButton(button) => format!("Button: {:?}", button),
            };
            container(
                column([
                    text("Error:").into(),
                    text(kind).into(),
                    text("More info:").into(),
                    text(info).into(),
                ])
                .align_items(iced::Alignment::Center),
            )
            .height(iced::Length::Fill)
            .width(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into()
        } else {
            unreachable!()
        }
    }

    fn theme(&self, window: window::Id) -> Self::Theme {
        if window == window::Id::MAIN {
            let red = self.style.background_color.red / 255.0;
            let green = self.style.background_color.green / 255.0;
            let blue = self.style.background_color.blue / 255.0;
            let palette = iced::theme::Palette {
                background: Color::from_rgb(red, green, blue),
                ..iced::theme::Palette::DARK
            };
            Theme::Custom(Arc::new(iced::theme::Custom::new("Custom".into(), palette)))
        } else if let Some(load_keyboard_window) = self.load_keyboard_window_id
            && load_keyboard_window == window
        {
            Theme::Light
        } else {
            Theme::Dark
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            listener::bind().map(Message::Listener),
            iced::event::listen_with(|event, _| match event {
                iced::Event::Window(id, window::Event::Closed) => Some(Message::WindowClosed(id)),
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

impl NuhxBoard {
    fn error<T>(&mut self, error: Error) -> iced::Command<T> {
        let (id, command) = window::spawn(window::Settings {
            size: iced::Size {
                width: 400.0,
                height: 150.0,
            },
            resizable: false,
            ..Default::default()
        });
        self.error_windows.insert(id, error);
        command
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

    /// Install the app to your system; Create a desktop entry and install the icon.
    #[arg(long)]
    install: bool,
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

    if args.install {
        match std::env::consts::OS {
            #[cfg(target_os = "linux")]
            "linux" => {
                let mut path = home::home_dir().unwrap();
                path.push(".local/share/");

                let res = reqwest::blocking::get("https://raw.githubusercontent.com/justDeeevin/NuhxBoard/main/nuhxboard.desktop")?;
                let desktop_entry = res.bytes()?;
                fs::File::create(path.clone().join("applications/nuhxboard.desktop"))?
                    .write_all(&desktop_entry)?;

                fs::File::create(path.join("NuhxBoard/NuhxBoard.png"))?.write_all(IMAGE)?;
            }
            #[cfg(target_os = "windows")]
            "windows" => {
                let mut lnk_path = home::home_dir().unwrap();
                lnk_path
                    .push("AppData/Roaming/Microsoft/Windows/Start Menu/Programs/NuhxBoard.lnk");

                let lnk = lnk_path.to_str().unwrap();

                let mut target_path = home::home_dir().unwrap();
                target_path.push(".cargo/bin/nuhxboard.exe");

                let target = target_path.to_str().unwrap();

                let sl = mslnk::ShellLink::new(target)?;
                sl.create_lnk(lnk)?;
            }
            "macos" => {
                eprintln!("Sorry, the install command isn't implemented for MacOS yet.");
                std::process::exit(1);
            }
            _ => {
                eprintln!("Sorry, the install command isn't implemented for your OS yet. If there isn't a GitHub issue open for your OS, open one!");
                std::process::exit(1);
            }
        }

        println!("NuhxBoard installed successfully!");

        return Ok(());
    }

    let icon = window::icon::from_rgba(icon_image.to_rgba8().to_vec(), 256, 256)?;
    let flags = Flags {
        verbose: args.verbose,
    };

    let settings = iced::Settings {
        window: window::Settings {
            size: DEFAULT_WINDOW_SIZE,
            resizable: false,
            icon: Some(icon),
            ..window::Settings::default()
        },
        flags,
        ..iced::Settings::default()
    };
    NuhxBoard::run(settings)?;

    Ok(())
}
