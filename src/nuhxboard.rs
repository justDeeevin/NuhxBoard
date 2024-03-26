use crate::{
    logic::{code_convert::*, listener},
    types::{config::*, settings::*, style::*}, ui::app::DisplayChoice,
};
use async_std::task::sleep;
use display_info::DisplayInfo;
use iced::{
    advanced::graphics::core::SmolStr, multi_window::Application, widget::canvas::Cache, window, Color, Command, Renderer, Subscription, Theme
};
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
    /// `[keycode: press_time]`
    pub pressed_keys: HashMap<u32, Instant>,
    /// `[keycode: press_time]`
    pub pressed_mouse_buttons: HashMap<u32, Instant>,
    /// `[axis: releases_queued]`
    pub pressed_scroll_buttons: HashMap<u32, u32>,
    /// `(x, y)`
    pub mouse_velocity: (f32, f32),
    /// `(x, y)`
    pub previous_mouse_position: (f32, f32),
    pub previous_mouse_time: std::time::SystemTime,
    pub caps: bool,
    pub true_caps: bool,
    pub load_keyboard_window_id: Option<window::Id>,
    pub settings_window_id: Option<window::Id>,
    pub keyboard: Option<usize>,
    pub style_choice: Option<usize>,
    pub error_windows: HashMap<window::Id, Error>,
    pub keyboard_options: Vec<String>,
    pub keyboard_category_options: Vec<String>,
    pub style_options: Vec<StyleChoice>,
    pub keyboards_path: std::path::PathBuf,
    pub startup: bool,
    pub settings: Settings,
    pub display_options: Vec<DisplayInfo>,
    pub edit_mode: bool,
    pub keyboard_properties_window_id: Option<window::Id>,
    pub edit_history: Vec<Change>,
    pub history_depth: usize,
}

#[derive(Default)]
pub struct Flags {
    pub settings: Settings,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StyleChoice {
    Default,
    Global(String),
    Custom(String),
}

impl std::fmt::Display for StyleChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StyleChoice::Default => write!(f, "Global Default"),
            StyleChoice::Custom(s) => write!(f, "{}", s),
            StyleChoice::Global(s) => write!(f, "Global: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Listener(listener::Event),
    ReleaseScroll(u32),
    LoadStyle(usize),
    OpenLoadKeyboardWindow,
    OpenSettingsWindow,
    WindowClosed(window::Id),
    ChangeKeyboardCategory(String),
    LoadKeyboard(usize),
    Quitting,
    ChangeSetting(Setting),
    ClearPressedKeys,
    ToggleEditMode,
    MoveElement {
        index: usize,
        delta: geo::Coord,
    },
    SaveKeyboard(Option<std::path::PathBuf>),
    SaveStyle(Option<std::path::PathBuf>),
    SetHeight(f32),
    SetWidth(f32),
    OpenKeyboardProperties,
    PushChange(Change),
    Undo,
    Redo
}

#[derive(Debug, Clone)]
pub enum Change {
    MoveElement{
        index: usize,
        delta: geo::Coord,
    },
}

#[derive(Debug, Clone)]
pub enum Setting {
    MouseSensitivity(f32),
    ScrollHoldTime(u64),
    CenterMouse,
    DisplayChoice(DisplayChoice),
    MinPressTime(u128),
    WindowTitle(String),
    Capitalization(Capitalization),
    FollowForCapsSensitive,
    FollowForCapsInsensitive,
    AutoDesktopEntry,
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

    pub fn none() -> Self {
        Message::Listener(listener::Event::None)
    }
}

#[derive(Debug)]
pub enum Error {
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

const LOAD_KEYBOARD_WINDOW_SIZE: iced::Size = iced::Size {
    width: 300.0,
    height: 250.0,
};

const ERROR_WINDOW_SIZE: iced::Size = iced::Size {
    width: 400.0,
    height: 150.0,
};

const KEYBOARD_PROPERTIES_WINDOW_SIZE: iced::Size = iced::Size {
    width: 200.0,
    height: 100.0,
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
            version: None,
            width: DEFAULT_WINDOW_SIZE.width,
            height: DEFAULT_WINDOW_SIZE.height,
            elements: vec![],
        };

        let category = flags.settings.category.clone();

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
                true_caps: false,
                mouse_velocity: (0.0, 0.0),
                pressed_scroll_buttons: HashMap::new(),
                previous_mouse_position: (0.0, 0.0),
                previous_mouse_time: std::time::SystemTime::now(),
                load_keyboard_window_id: None,
                settings_window_id: None,
                keyboard: Some(flags.settings.keyboard),
                style_choice: Some(flags.settings.style),
                error_windows: HashMap::new(),
                keyboard_options: vec![],
                keyboard_category_options: vec![],
                style_options: vec![],
                keyboards_path: path,
                startup: true,
                settings: flags.settings,
                display_options: DisplayInfo::all().unwrap(),
                edit_mode: false,
                keyboard_properties_window_id: None,
                edit_history: vec![],
                history_depth: 0,
            },
            Command::batch([
                Command::perform(noop(), move |_| Message::ChangeKeyboardCategory(category)),
                iced::font::load(iced_aw::graphics::icons::BOOTSTRAP_FONT_BYTES)
                    .map(|_| Message::none()),
            ]),
        )
    }

    fn title(&self, window: window::Id) -> String {
        if window == window::Id::MAIN {
            self.settings.window_title.clone()
        } else if Some(window) == self.load_keyboard_window_id {
            "Load Keyboard".to_owned()
        } else if self.error_windows.contains_key(&window) {
            "Error".to_owned()
        } else if Some(window) == self.settings_window_id {
            "Settings".to_owned()
        } else if Some(window) == self.keyboard_properties_window_id {
            "Keyboard Properties".to_owned()
        } else {
            unreachable!()
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Listener(listener::Event::KeyReceived(event)) => match event.event_type {
                rdev::EventType::KeyPress(key) => {
                    if keycode_convert(key).is_err() {
                        return self.error(Error::UnknownKey(key));
                    }
                    if key == rdev::Key::CapsLock
                        && self.settings.capitalization == Capitalization::Follow
                    {
                        self.caps = !self.caps;
                    }
                    self.true_caps = !self.true_caps;
                    let key = keycode_convert(key).unwrap();
                    self.pressed_keys.insert(key, Instant::now());
                }
                rdev::EventType::KeyRelease(key) => {
                    if keycode_convert(key).is_err() {
                        return self.error(Error::UnknownKey(key));
                    }
                    let key_num = keycode_convert(key).unwrap();
                    if !self.pressed_keys.contains_key(&key_num) {
                        return Command::none();
                    }
                    if self
                        .pressed_keys
                        .get(&key_num)
                        .unwrap()
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
                                        .elapsed()
                                        .as_millis())
                                .try_into()
                                .unwrap(),
                            )),
                            move |_| Message::key_release(key),
                        );
                    }
                    self.pressed_keys.remove(&key_num);
                }
                rdev::EventType::ButtonPress(button) => {
                    if mouse_button_code_convert(button).is_err() {
                        return self.error(Error::UnknownButton(button));
                    }

                    if button == rdev::Button::Unknown(6) || button == rdev::Button::Unknown(7) {
                        return Command::none();
                    }

                    let button = mouse_button_code_convert(button).unwrap();
                    self.pressed_mouse_buttons.insert(button, Instant::now());
                }
                rdev::EventType::ButtonRelease(button) => {
                    if mouse_button_code_convert(button).is_err() {
                        return self.error(Error::UnknownButton(button));
                    }
                    if button == rdev::Button::Unknown(6) || button == rdev::Button::Unknown(7) {
                        return Command::none();
                    }
                    let button_num = mouse_button_code_convert(button).unwrap();
                    if !self.pressed_mouse_buttons.contains_key(&button_num) {
                        return Command::none();
                    }
                    if self
                        .pressed_mouse_buttons
                        .get(&button_num)
                        .unwrap()
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
                                        .elapsed()
                                        .as_millis())
                                .try_into()
                                .unwrap(),
                            )),
                            move |_| Message::button_release(button),
                        );
                    }
                    self.pressed_mouse_buttons.remove(&button_num);
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

                    let mut center = (0.0, 0.0);

                    for display in &self.display_options {
                        if display.id == self.settings.display_choice.id {
                            center = (
                                display.x as f32 + (display.width as f32 / 2.0),
                                display.height as f32 / 2.0,
                            )
                        }
                    }

                    let previous_pos = match self.settings.mouse_from_center {
                        true => (center.0, center.1),
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
            Message::OpenSettingsWindow => {
                let (id, command) = window::spawn(window::Settings {
                    resizable: false,
                    size: iced::Size {
                        width: 420.0,
                        height: 255.0,
                    },
                    ..Default::default()
                });
                self.settings_window_id = Some(id);
                return command;
            }
            Message::OpenLoadKeyboardWindow => {
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
                    .filter(|entry| entry.file_type().unwrap().is_dir() && entry.file_name() != "global")
                    .map(|entry| entry.file_name().to_str().unwrap().to_owned())
                    .collect::<Vec<_>>();

                return command;
            }
            Message::ChangeKeyboardCategory(category) => {
                if category.is_empty() {
                    return Command::none();
                }
                let mut path = self.keyboards_path.clone();
                self.settings.category = category;

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
                    return self.update(Message::LoadKeyboard(keyboard));
                }
            }
            Message::LoadKeyboard(keyboard) => {
                return self.load_keyboard(keyboard);
            }
            Message::LoadStyle(style) => {
                self.settings.style = style;

                self.style_choice = Some(style);

                if self.style_options[style] == StyleChoice::Default {
                    self.style = Style::default();
                    return Command::none();
                }

                let path = self
                    .keyboards_path
                    .clone()
                    .join(match &self.style_options[style] {
                        StyleChoice::Default => unreachable!(),
                        StyleChoice::Global(style_name) => format!("global/{}.style", style_name),
                        StyleChoice::Custom(style_name) => format!(
                            "{}/{}/{}.style",
                            self.settings.category,
                            self.keyboard_options[self.keyboard.unwrap()],
                            style_name
                        ),
                    });

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
                if Some(id) == self.load_keyboard_window_id {
                    self.load_keyboard_window_id = None;
                }
                self.error_windows.remove(&id);
                if Some(id) == self.settings_window_id {
                    self.settings_window_id = None;
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
                if let Some(settings_window_id) = self.settings_window_id {
                    commands.push(window::close(settings_window_id));
                }

                commands.push(window::close(window::Id::MAIN));
                return Command::batch(commands);
            }
            Message::ChangeSetting(setting) => match setting {
                Setting::MouseSensitivity(sens) => {
                    self.settings.mouse_sensitivity = sens;
                }
                Setting::ScrollHoldTime(time) => {
                    self.settings.scroll_hold_time = time;
                }
                Setting::CenterMouse => {
                    self.settings.mouse_from_center = !self.settings.mouse_from_center;
                }
                Setting::DisplayChoice(choice) => {
                    self.settings.display_choice = choice;
                }
                Setting::MinPressTime(time) => {
                    self.settings.min_press_time = time;
                }
                Setting::WindowTitle(title) => {
                    self.settings.window_title = title;
                }
                Setting::Capitalization(cap) => {
                    match cap {
                        Capitalization::Lower => {
                            self.caps = false;
                        }
                        Capitalization::Upper => {
                            self.caps = true;
                        }
                        Capitalization::Follow => {
                            self.caps = self.true_caps;
                        }
                    }
                    self.settings.capitalization = cap;
                }
                Setting::FollowForCapsSensitive => {
                    self.settings.follow_for_caps_sensitive =
                        !self.settings.follow_for_caps_sensitive;
                }
                Setting::FollowForCapsInsensitive => {
                    self.settings.follow_for_caps_insensitive =
                        !self.settings.follow_for_caps_insensitive;
                }
                Setting::AutoDesktopEntry => {
                    self.settings.auto_desktop_entry = !self.settings.auto_desktop_entry;
                }
            },
            Message::ClearPressedKeys => {
                self.pressed_keys.clear();
            }
            Message::Listener(_) => {}
            Message::ToggleEditMode => {
                self.edit_mode = !self.edit_mode;
            }
            Message::MoveElement { index, delta } => {
                self.config.elements[index].translate(delta);
            }
            Message::SaveKeyboard(file) => {
                let path = file.unwrap_or(self.keyboards_path.clone().join(format!(
                    "{}/{}/keyboard.json",
                    self.settings.category,
                    self.keyboard_options[self.keyboard.unwrap()]
                )));
                let mut file = File::create(path).unwrap();
                serde_json::to_writer_pretty(&mut file, &self.config).unwrap();
            }
            Message::SaveStyle(file) => {
                let path = file.unwrap_or(self.keyboards_path.clone().join(format!(
                    "{}/{}/{}.style",
                    self.settings.category,
                    self.keyboard_options[self.keyboard.unwrap()],
                    self.style_options[self.style_choice.unwrap()]
                )));
                let mut file = File::create(path).unwrap();
                serde_json::to_writer_pretty(&mut file, &self.style).unwrap();
            }
            Message::SetHeight(height) => {
                self.config.height = height;
                return window::resize(
                    window::Id::MAIN,
                    iced::Size {
                        width: self.config.width,
                        height: self.config.height,
                    },
                );
            }
            Message::SetWidth(width) => {
                self.config.width = width;
                return window::resize(
                    window::Id::MAIN,
                    iced::Size {
                        width: self.config.width,
                        height: self.config.height,
                    },
                );
            }
            Message::OpenKeyboardProperties => {
                let (id, command) = window::spawn(window::Settings {
                    size: KEYBOARD_PROPERTIES_WINDOW_SIZE,
                    resizable: false,
                    ..Default::default()
                });
                self.keyboard_properties_window_id = Some(id);
                return command;
            }
            Message::PushChange(change) => {
                if self.history_depth > 0 {
                    self.edit_history.truncate(self.edit_history.len() - self.history_depth);
                    self.history_depth = 0;
                }
                self.edit_history.push(change);
            }
            Message::Undo => {
                if self.history_depth < self.edit_history.len() {
                    self.history_depth += 1;
                    match self.edit_history[self.edit_history.len() - self.history_depth] {
                        Change::MoveElement { index, delta } => {
                            self.config.elements[index].translate(-delta);
                        }
                    }
                }
            }
            Message::Redo => {
                if self.history_depth > 0 {
                    self.history_depth -= 1;
                    match self.edit_history[self.edit_history.len() - self.history_depth - 1] {
                        Change::MoveElement { index, delta } => {
                            self.config.elements[index].translate(delta);
                        }
                    }
                }
            }
        }
        self.canvas.clear();
        Command::none()
    }

    fn view(&self, window: window::Id) -> iced::Element<'_, Self::Message, Self::Theme, Renderer> {
        if window == window::Id::MAIN {
            self.draw_main_window()
        } else if Some(window) == self.load_keyboard_window_id {
            self.draw_load_keyboard_window()
        } else if self.error_windows.contains_key(&window) {
            self.draw_error_window(&window)
        } else if Some(window) == self.settings_window_id {
            self.draw_settings_window()
        } else if Some(window) == self.keyboard_properties_window_id {
            self.draw_keyboard_properties_window()
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
            return Theme::Custom(Arc::new(iced::theme::Custom::new("Custom".into(), palette)));
        }
        Theme::Light
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            listener::bind().map(Message::Listener),
            iced::event::listen_with(|event, _| match event {
                iced::Event::Window(id, window::Event::Closed) => Some(Message::WindowClosed(id)),
                iced::Event::Window(window::Id::MAIN, window::Event::CloseRequested) => {
                    Some(Message::Quitting)
                }
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, location: _, modifiers, text: _ }) => {
                    if modifiers.command() {
                        if key == iced::keyboard::Key::Character(SmolStr::new("z")) {
                            Some(Message::Undo)
                        } else if key == iced::keyboard::Key::Character(SmolStr::new("Z")) {
                            Some(Message::Redo)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }),
        ])
    }
}

impl NuhxBoard {
    pub fn error<T>(&mut self, error: Error) -> iced::Command<T> {
        let (id, command) = window::spawn(window::Settings {
            size: ERROR_WINDOW_SIZE,
            resizable: false,
            ..Default::default()
        });
        self.error_windows.insert(id, error);
        command
    }
}
