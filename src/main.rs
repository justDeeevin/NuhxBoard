#![allow(non_snake_case)]
#![windows_subsystem = "windows"]
#![feature(let_chains)]

mod code_convert;
mod config;
mod listener;
mod settings;
mod style;
mod stylesheets;
use async_std::task::sleep;
use clap::Parser;
use code_convert::*;
use color_eyre::eyre::Result;
use config::*;
use display_info::DisplayInfo;
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
use settings::*;
use std::sync::Arc;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::prelude::*,
    time::Instant,
};
use style::*;
use stylesheets::*;

struct NuhxBoard {
    config: Config,
    style: Style,
    canvas: Cache,
    pressed_keys: HashMap<u32, (Instant, u32)>,
    pressed_mouse_buttons: HashMap<u32, (Instant, u32)>,
    pressed_scroll_buttons: HashMap<u32, u32>,
    mouse_velocity: (f32, f32),
    previous_mouse_position: (f32, f32),
    previous_mouse_time: std::time::SystemTime,
    caps: bool,
    verbose: bool,
    load_keyboard_window_id: Option<window::Id>,
    keyboard: Option<usize>,
    style_choice: Option<usize>,
    error_windows: HashMap<window::Id, Error>,
    keyboard_options: Vec<String>,
    keyboard_category_options: Vec<String>,
    style_options: Vec<StyleChoice>,
    keyboards_path: std::path::PathBuf,
    startup: bool,
    settings: Settings,
    /// `(width, height)`
    center: (f32, f32),
}

#[derive(Default)]
struct Flags {
    verbose: bool,
    settings: Settings,
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
    Quitting,
}

impl Message {
    pub fn key_release(key: rdev::Key) -> Self {
        Message::Listener(listener::Event::KeyReceived(rdev::Event {
            event_type: rdev::EventType::KeyRelease(key),
            time: std::time::SystemTime::now(),
            name: None,
        }))
    }

    pub fn button_release(button: rdev::Button) -> Self {
        Message::Listener(listener::Event::KeyReceived(rdev::Event {
            event_type: rdev::EventType::ButtonRelease(button),
            time: std::time::SystemTime::now(),
            name: None,
        }))
    }
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

const ERROR_WINDOW_SIZE: iced::Size = iced::Size {
    width: 400.0,
    height: 150.0,
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

        let path = home::home_dir()
            .unwrap()
            .join(".local/share/NuhxBoard/keyboards");
        let config = Config {
            version: 2,
            width: DEFAULT_WINDOW_SIZE.width,
            height: DEFAULT_WINDOW_SIZE.height,
            elements: vec![],
        };

        let category = flags.settings.category.clone();

        let mut center = (0.0, 0.0);

        let displays = DisplayInfo::all().unwrap();

        for display in displays {
            if display.id == flags.settings.display_id {
                center = (
                    display.x as f32 + (display.width as f32 / 2.0),
                    display.height as f32 / 2.0,
                )
            }
        }

        (
            Self {
                config,
                style: Style::default(),
                canvas: Cache::default(),
                pressed_keys: HashMap::new(),
                pressed_mouse_buttons: HashMap::new(),
                caps: match flags.settings.capitalization {
                    Capitalization::Upper => true,
                    Capitalization::Lower => false,
                    Capitalization::Follow => false,
                },
                mouse_velocity: (0.0, 0.0),
                pressed_scroll_buttons: HashMap::new(),
                previous_mouse_position: (0.0, 0.0),
                previous_mouse_time: std::time::SystemTime::now(),
                verbose: flags.verbose,
                load_keyboard_window_id: None,
                keyboard: Some(flags.settings.keyboard),
                style_choice: Some(flags.settings.style),
                error_windows: HashMap::new(),
                keyboard_options: vec![],
                keyboard_category_options: vec![],
                style_options: vec![],
                keyboards_path: path,
                startup: true,
                settings: flags.settings,
                center,
            },
            Command::perform(noop(), move |_| Message::ChangeKeyboardCategory(category)),
        )
    }

    fn title(&self, window: window::Id) -> String {
        if window == window::Id::MAIN {
            self.settings.window_title.clone()
        } else if let Some(load_keyboard_window_id) = self.load_keyboard_window_id
            && window == load_keyboard_window_id
        {
            "Load Keyboard".to_owned()
        } else if self.error_windows.contains_key(&window) {
            "Error".to_owned()
        } else {
            unreachable!()
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        if self.verbose
            && let Message::Listener(listener::Event::KeyReceived(event)) = &message
        {
            match event.event_type {
                rdev::EventType::MouseMove { x: _, y: _ } => {}
                _ => {
                    dbg!(&event);
                }
            }
        }

        match message {
            Message::Listener(listener::Event::KeyReceived(event)) => match event.event_type {
                rdev::EventType::KeyPress(key) => {
                    if let Err(bad_key) = keycode_convert(key) {
                        return self.error(Error::UnknownKey(bad_key));
                    }
                    if key == rdev::Key::CapsLock
                        && self.settings.capitalization == Capitalization::Follow
                    {
                        self.caps = !self.caps;
                    }
                    let key = keycode_convert(key).unwrap();
                    self.pressed_keys
                        .entry(key)
                        .and_modify(|(time, count)| {
                            *time = Instant::now();
                            *count += 1;
                        })
                        .or_insert((Instant::now(), 1));
                }
                rdev::EventType::KeyRelease(key) => {
                    if let Err(bad_key) = keycode_convert(key) {
                        return self.error(Error::UnknownKey(bad_key));
                    }
                    let key_num = keycode_convert(key).unwrap();
                    if self
                        .pressed_keys
                        .get(&key_num)
                        .unwrap()
                        .0
                        .elapsed()
                        .as_millis()
                        < self.settings.min_press_time
                    {
                        return Command::perform(
                            sleep(std::time::Duration::from_millis(
                                (self.settings.min_press_time
                                    - self
                                        .pressed_keys
                                        .get(&key_num)
                                        .unwrap()
                                        .0
                                        .elapsed()
                                        .as_millis())
                                .try_into()
                                .unwrap(),
                            )),
                            move |_| Message::key_release(key),
                        );
                    } else {
                        match &mut self.pressed_keys.get_mut(&key_num).unwrap().1 {
                            1 => {
                                self.pressed_keys.remove(&key_num);
                            }
                            n => {
                                *n -= 1;
                            }
                        }
                    }
                }
                rdev::EventType::ButtonPress(button) => {
                    if let Err(bad_button) = mouse_button_code_convert(button) {
                        return self.error(Error::UnknownButton(bad_button));
                    }

                    if button == rdev::Button::Unknown(6) || button == rdev::Button::Unknown(7) {
                        return Command::none();
                    }

                    let button = mouse_button_code_convert(button).unwrap();
                    self.pressed_mouse_buttons
                        .entry(button)
                        .and_modify(|(time, count)| {
                            *time = Instant::now();
                            *count += 1;
                        })
                        .or_insert((Instant::now(), 1));
                }
                rdev::EventType::ButtonRelease(button) => {
                    if let Err(bad_button) = mouse_button_code_convert(button) {
                        return self.error(Error::UnknownButton(bad_button));
                    }

                    if button == rdev::Button::Unknown(6) || button == rdev::Button::Unknown(7) {
                        return Command::none();
                    }

                    let button_num = mouse_button_code_convert(button).unwrap();
                    if self
                        .pressed_mouse_buttons
                        .get(&button_num)
                        .unwrap()
                        .0
                        .elapsed()
                        .as_millis()
                        < self.settings.min_press_time
                    {
                        return Command::perform(
                            sleep(std::time::Duration::from_millis(
                                (self.settings.min_press_time
                                    - self
                                        .pressed_mouse_buttons
                                        .get(&button_num)
                                        .unwrap()
                                        .0
                                        .elapsed()
                                        .as_millis())
                                .try_into()
                                .unwrap(),
                            )),
                            move |_| Message::button_release(button),
                        );
                    } else {
                        match &mut self.pressed_mouse_buttons.get_mut(&button_num).unwrap().1 {
                            1 => {
                                self.pressed_mouse_buttons.remove(&button_num);
                            }
                            n => {
                                *n -= 1;
                            }
                        }
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

                    self.pressed_scroll_buttons
                        .entry(button)
                        .and_modify(|v| *v += 1)
                        .or_insert(1);

                    self.canvas.clear();

                    return Command::perform(
                        sleep(std::time::Duration::from_millis(
                            self.settings.scroll_hold_time,
                        )),
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

                    let previous_pos = match self.settings.mouse_from_center {
                        true => (self.center.0, self.center.1),
                        false => self.previous_mouse_position,
                    };
                    let position_diff = (x - previous_pos.0, y - previous_pos.1);
                    self.mouse_velocity = (
                        position_diff.0 / time_diff.as_secs_f32(),
                        position_diff.1 / time_diff.as_secs_f32(),
                    );
                    self.previous_mouse_position = (x, y);
                    self.previous_mouse_time = current_time;
                }
            },
            Message::ReleaseScroll(button) => {
                match self.pressed_scroll_buttons.get_mut(&button).unwrap() {
                    1 => {
                        self.pressed_scroll_buttons.remove(&button);
                    }
                    n => {
                        *n -= 1;
                    }
                }
            }
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
                self.settings.category = category.clone();

                if !self.startup {
                    self.keyboard = None;
                    self.style_choice = None;
                    self.style_options = vec![];
                }
                self.keyboard_options = {
                    path.push(&self.settings.category);
                    fs::read_dir(&path)
                        .unwrap()
                        .map(|r| r.unwrap())
                        .filter(|entry| {
                            entry.file_type().unwrap().is_dir() && entry.file_name() != "images"
                        })
                        .map(|entry| entry.file_name().to_str().unwrap().to_owned())
                        .collect()
                };

                if self.startup {
                    self.startup = false;
                    let keyboard = self.keyboard.unwrap();
                    return Command::perform(noop(), move |_| Message::LoadKeyboard(keyboard));
                }
            }
            Message::LoadKeyboard(keyboard) => {
                self.settings.keyboard = keyboard;

                self.keyboard = Some(keyboard);
                self.style = Style::default();

                let mut path = self.keyboards_path.clone();
                path.push(&self.settings.category);
                path.push(self.keyboard_options[keyboard].clone());
                path.push("keyboard.json");
                let config_file = match File::open(path) {
                    Ok(file) => file,
                    Err(e) => {
                        return self.error(Error::ConfigOpen(e));
                    }
                };

                self.config = match serde_json::from_reader(config_file) {
                    Ok(config) => config,
                    Err(e) => return self.error(Error::ConfigParse(e)),
                };

                let mut path = self.keyboards_path.clone();
                path.push(&self.settings.category);
                path.push(self.keyboard_options[keyboard].clone());

                self.style_options = vec![StyleChoice::Default];
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
                self.settings.style = style;

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
                path.push(&self.settings.category);
                path.push(self.keyboard_options[self.keyboard.unwrap()].clone());
                path.push(format!("{}.style", style));

                let style_file = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => {
                        return self.error(Error::StyleOpen(e));
                    }
                };
                self.style = match serde_json::from_reader(style_file) {
                    Ok(style) => style,
                    Err(e) => return self.error(Error::StyleParse(e)),
                };
            }
            Message::WindowClosed(id) => {
                if let Some(load_keyboard_window_id) = self.load_keyboard_window_id
                    && id == load_keyboard_window_id
                {
                    self.load_keyboard_window_id = None;
                }
                if self.error_windows.contains_key(&id) {
                    self.error_windows.remove(&id);
                }
            }
            Message::Quitting => {
                let mut settings_file = File::create(
                    home::home_dir()
                        .unwrap()
                        .join(".local/share/NuhxBoard/NuhxBoard.json"),
                )
                .unwrap();
                serde_json::to_writer_pretty(&mut settings_file, &self.settings).unwrap();
                let mut commands = vec![];
                if let Some(load_keyboard_window_id) = self.load_keyboard_window_id {
                    commands.push(window::close(load_keyboard_window_id));
                }
                for error_window in &self.error_windows {
                    commands.push(window::close(*error_window.0));
                }

                commands.push(window::close(window::Id::MAIN));
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
                    Some(self.settings.category.clone()),
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
                iced::Event::Window(window::Id::MAIN, window::Event::CloseRequested) => {
                    Some(Message::Quitting)
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

impl NuhxBoard {
    fn error<T>(&mut self, error: Error) -> iced::Command<T> {
        let (id, command) = window::spawn(window::Settings {
            size: ERROR_WINDOW_SIZE,
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
            let mut settings = File::create(
                home::home_dir()
                    .unwrap()
                    .join(".local/share/NuhxBoard/NuhxBoard.json"),
            )?;

            settings.write_all(serde_json::to_string_pretty(&Settings::default())?.as_bytes())?;
        } else {
            std::process::exit(0);
        }
    }

    let args = Args::parse();

    let icon_image = image::load_from_memory(IMAGE)?;

    let settings_file = File::open(
        home::home_dir()
            .unwrap()
            .join(".local/share/NuhxBoard/NuhxBoard.json"),
    )?;

    let settings: Settings = serde_json::from_reader(settings_file)?;

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
        settings,
    };

    let settings = iced::Settings {
        window: window::Settings {
            size: DEFAULT_WINDOW_SIZE,
            resizable: false,
            icon: Some(icon),
            exit_on_close_request: false,
            ..window::Settings::default()
        },
        flags,
        ..iced::Settings::default()
    };
    NuhxBoard::run(settings)?;

    Ok(())
}
