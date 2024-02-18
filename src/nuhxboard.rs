use crate::{
    code_convert::*,
    listener,
    types::{config::*, settings::*, style::*, stylesheets::*},
};
use async_std::task::sleep;
use display_info::DisplayInfo;
use iced::{
    multi_window::Application,
    widget::{button, canvas, canvas::Cache, column, container, pick_list, row, text},
    window, Color, Command, Length, Renderer, Subscription, Theme,
};
use iced_aw::{ContextMenu, SelectionList};
use std::sync::Arc;
use std::{
    collections::HashMap,
    fs::{self, File},
    time::Instant,
};

pub struct NuhxBoard {
    pub config: Config,
    pub style: Style,
    pub canvas: Cache,
    /// `[keycode: (press_time, releases_queued)]`
    pub pressed_keys: HashMap<u32, (Instant, u32)>,
    /// `[keycode: (press_time, releases_queued)]`
    pub pressed_mouse_buttons: HashMap<u32, (Instant, u32)>,
    /// `[axis: releases_queued]`
    pub pressed_scroll_buttons: HashMap<u32, u32>,
    /// `(x, y)`
    pub mouse_velocity: (f32, f32),
    /// `(x, y)`
    pub previous_mouse_position: (f32, f32),
    pub previous_mouse_time: std::time::SystemTime,
    pub caps: bool,
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
    pub settings: Settings,
    /// `(x, y)`
    center: (f32, f32),
}

#[derive(Default)]
pub struct Flags {
    pub verbose: bool,
    pub settings: Settings,
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
pub enum Message {
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

pub const DEFAULT_WINDOW_SIZE: iced::Size = iced::Size {
    width: 200.0,
    height: 200.0,
};

pub const LOAD_KEYBOARD_WINDOW_SIZE: iced::Size = iced::Size {
    width: 300.0,
    height: 250.0,
};

pub const ERROR_WINDOW_SIZE: iced::Size = iced::Size {
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

    fn view(&self, window: window::Id) -> iced::Element<'_, Self::Message, Self::Theme, Renderer> {
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
