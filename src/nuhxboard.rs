use crate::ui::app::*;
use async_std::task::sleep;
use display_info::DisplayInfo;
use iced::{
    advanced::graphics::core::SmolStr, widget::canvas::Cache, window, Color, Renderer,
    Subscription, Task, Theme,
};
use iced_multi_window::{Window, WindowManager};
use image::ImageReader;
use logic::{code_convert::*, listener};
use std::{
    collections::HashMap,
    fs::{self, File},
    time::Instant,
};
use types::{config::*, settings::*, style::*};

pub struct NuhxBoard {
    pub windows: WindowManager<NuhxBoard, Theme, Message>,
    pub main_window: window::Id,
    pub layout: Layout,
    pub style: Style,
    pub canvas: Cache,
    /// `[keycode: time_pressed]`
    pub pressed_keys: HashMap<u32, Instant>,
    /// `[keycode: time_pressed]`
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
    pub keyboard_choice: Option<usize>,
    pub style_choice: Option<usize>,
    pub keyboard_options: Vec<String>,
    pub keyboard_category_options: Vec<String>,
    pub style_options: Vec<StyleChoice>,
    pub keyboards_path: std::path::PathBuf,
    pub startup: bool,
    pub settings: Settings,
    pub display_options: Vec<DisplayInfo>,
    pub edit_mode: bool,
    pub edit_history: Vec<Change>,
    pub history_depth: usize,
    pub save_keyboard_as_category: String,
    pub save_keyboard_as_name: String,
    pub save_style_as_name: String,
    pub save_style_as_global: bool,
    pub color_pickers: ColorPickers,
}

#[derive(Debug, Default)]
pub struct ColorPickers {
    pub keyboard_background: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StyleChoice {
    Default,
    Global(String),
    Custom(String),
}

impl StyleChoice {
    pub fn is_global(&self) -> bool {
        matches!(self, StyleChoice::Global(_))
    }

    pub fn name(&self) -> String {
        match self {
            Self::Global(name) => name.clone(),
            _ => self.to_string(),
        }
    }
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

#[derive(Clone, Debug)]
pub enum Message {
    Open(Box<dyn Window<NuhxBoard, Theme, Message>>),
    Exit,
    Closed(window::Id),
    Listener(listener::Event),
    ReleaseScroll(u32),
    LoadStyle(usize),
    ChangeKeyboardCategory(String),
    LoadLayout(usize),
    ChangeSetting(Setting),
    ClearPressedKeys,
    ToggleEditMode,
    MoveElement {
        index: usize,
        delta: geo::Coord<f32>,
    },
    SaveKeyboard(Option<std::path::PathBuf>),
    SaveStyle(Option<std::path::PathBuf>),
    SetHeight(f32),
    SetWidth(f32),
    PushChange(Change),
    Undo,
    Redo,
    ChangeSaveLayoutAsCategory(String),
    ChangeSaveLayoutAsName(String),
    ChangeSaveStyleAsName(String),
    ToggleSaveStyleAsGlobal,
    ChangeBackground(Color),
    ToggleColorPicker(ColorPicker),
    UpdateCanvas,
}

#[derive(Debug, Clone)]
pub enum ColorPicker {
    KeyboardBackground,
}

// TODO: Are window resized undoable in NohBoard?
#[derive(Debug, Clone)]
pub enum Change {
    MoveElement {
        index: usize,
        delta: geo::Coord<f32>,
        move_text: bool,
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
    UpdateTextPosition,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ConfigOpen(String),
    ConfigParse(String),
    StyleOpen(String),
    StyleParse(String),
    UnknownKey(rdev::Key),
    UnknownButton(rdev::Button),
}

pub const DEFAULT_WINDOW_SIZE: iced::Size = iced::Size {
    width: 200.0,
    height: 200.0,
};

impl NuhxBoard {
    pub fn new() -> (Self, Task<Message>) {
        let nuhxboard_path = home::home_dir().unwrap().join(".local/share/NuhxBoard");

        let settings: Settings =
            serde_json::from_reader(File::open(nuhxboard_path.join("NuhxBoard.json")).unwrap())
                .unwrap();

        let keyboards_path = nuhxboard_path.join("keyboards");
        let layout = Layout {
            version: None,
            width: DEFAULT_WINDOW_SIZE.width,
            height: DEFAULT_WINDOW_SIZE.height,
            elements: Vec::new(),
        };

        let category = settings.category.clone();

        let caps = match settings.capitalization {
            Capitalization::Upper => true,
            Capitalization::Lower => false,
            Capitalization::Follow => false,
        };

        let mut windows = WindowManager::default();

        let (main_window, task) = windows.open(Box::new(Main));

        (
            Self {
                windows,
                main_window,
                layout,
                style: Style::default(),
                canvas: Cache::default(),
                pressed_keys: HashMap::new(),
                pressed_mouse_buttons: HashMap::new(),
                caps,
                true_caps: false,
                mouse_velocity: (0.0, 0.0),
                pressed_scroll_buttons: HashMap::new(),
                previous_mouse_position: (0.0, 0.0),
                previous_mouse_time: std::time::SystemTime::now(),
                keyboard_choice: Some(settings.keyboard),
                style_choice: Some(settings.style),
                keyboard_options: Vec::new(),
                keyboard_category_options: Vec::new(),
                style_options: Vec::new(),
                keyboards_path,
                startup: true,
                settings,
                display_options: DisplayInfo::all().unwrap(),
                edit_mode: false,
                edit_history: Vec::new(),
                history_depth: 0,
                save_keyboard_as_category: String::new(),
                save_keyboard_as_name: String::new(),
                save_style_as_name: String::new(),
                save_style_as_global: false,
                color_pickers: ColorPickers::default(),
            },
            Task::batch([
                Task::perform(async {}, move |_| {
                    Message::ChangeKeyboardCategory(category.clone())
                }),
                task.map(|_| Message::none()),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let mut clear_canvas = true;
        match message {
            Message::Listener(listener::Event::KeyReceived(event)) => {
                self.canvas.clear();
                return self.input_event(event);
            }
            Message::Listener(_) => clear_canvas = false,
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
            Message::ChangeKeyboardCategory(category) => {
                if category.is_empty() {
                    return Task::none();
                }
                self.settings.category = category;

                if !self.startup {
                    self.keyboard_choice = None;
                    self.settings.keyboard = 0;
                    self.style_choice = None;
                    self.settings.style = 0;
                    self.style_options = Vec::new();
                }
                self.keyboard_options = {
                    fs::read_dir(self.keyboards_path.join(&self.settings.category))
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
                    let keyboard = self.keyboard_choice.unwrap();
                    return self.update(Message::LoadLayout(keyboard));
                }
                clear_canvas = false;
            }
            Message::LoadLayout(layout) => {
                self.canvas.clear();
                return self.load_layout(layout);
            }
            Message::LoadStyle(style) => {
                self.canvas.clear();
                return self.load_style(style);
            }
            Message::ChangeSetting(setting) => match setting {
                Setting::MouseSensitivity(sens) => {
                    self.settings.mouse_sensitivity = sens;
                }
                Setting::ScrollHoldTime(time) => {
                    self.settings.scroll_hold_time = time;
                    clear_canvas = false;
                }
                Setting::CenterMouse => {
                    self.settings.mouse_from_center = !self.settings.mouse_from_center;
                }
                Setting::DisplayChoice(choice) => {
                    self.settings.display_choice = choice;
                }
                Setting::MinPressTime(time) => {
                    self.settings.min_press_time = time;
                    clear_canvas = false;
                }
                Setting::WindowTitle(title) => {
                    self.settings.window_title = title;
                    clear_canvas = false;
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
                Setting::UpdateTextPosition => {
                    self.settings.update_text_position = !self.settings.update_text_position;
                    clear_canvas = false;
                }
            },
            Message::ClearPressedKeys => {
                self.pressed_keys.clear();
            }
            Message::ToggleEditMode => {
                self.edit_mode = !self.edit_mode;
            }
            Message::MoveElement { index, delta } => {
                self.layout.elements[index].translate(delta, self.settings.update_text_position);
            }
            Message::SaveKeyboard(file) => {
                let path = file.unwrap_or(self.keyboards_path.join(format!(
                    "{}/{}/keyboard.json",
                    self.settings.category,
                    self.keyboard_options[self.keyboard_choice.unwrap()]
                )));
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                let mut file = File::create(path).unwrap();
                serde_json::to_writer_pretty(&mut file, &self.layout).unwrap();
                clear_canvas = false;
            }
            Message::SaveStyle(file) => {
                let path = file.unwrap_or(self.keyboards_path.join(format!(
                    "{}/{}/{}.style",
                    self.settings.category,
                    self.keyboard_options[self.keyboard_choice.unwrap()],
                    self.style_options[self.style_choice.unwrap()]
                )));
                let mut file = File::create(path).unwrap();
                serde_json::to_writer_pretty(&mut file, &self.style).unwrap();
                clear_canvas = false;
            }
            Message::SetHeight(height) => {
                self.layout.height = height;
                return window::resize(
                    self.main_window,
                    iced::Size {
                        width: self.layout.width,
                        height: self.layout.height,
                    },
                );
            }
            Message::SetWidth(width) => {
                self.layout.width = width;
                return window::resize(
                    self.main_window,
                    iced::Size {
                        width: self.layout.width,
                        height: self.layout.height,
                    },
                );
            }
            Message::PushChange(change) => {
                if self.history_depth > 0 {
                    self.edit_history
                        .truncate(self.edit_history.len() - self.history_depth);
                    self.history_depth = 0;
                }
                self.edit_history.push(change);
            }
            Message::Undo => {
                if self.history_depth < self.edit_history.len() {
                    self.history_depth += 1;
                    match self.edit_history[self.edit_history.len() - self.history_depth] {
                        Change::MoveElement {
                            index,
                            delta,
                            move_text,
                        } => {
                            self.layout.elements[index].translate(-delta, move_text);
                        }
                    }
                }
            }
            Message::Redo => {
                if self.history_depth > 0 {
                    self.history_depth -= 1;
                    match self.edit_history[self.edit_history.len() - self.history_depth - 1] {
                        Change::MoveElement {
                            index,
                            delta,
                            move_text,
                        } => {
                            self.layout.elements[index].translate(delta, move_text);
                        }
                    }
                }
            }
            Message::ChangeSaveLayoutAsCategory(category) => {
                self.save_keyboard_as_category = category;
                clear_canvas = false;
            }
            Message::ChangeSaveLayoutAsName(name) => {
                self.save_keyboard_as_name = name;
                clear_canvas = false;
            }
            Message::ChangeSaveStyleAsName(name) => {
                self.save_style_as_name = name;
                clear_canvas = false;
            }
            Message::ToggleSaveStyleAsGlobal => {
                self.save_style_as_global = !self.save_style_as_global;
                clear_canvas = false;
            }
            Message::Open(window) => {
                if window.eq(&LoadKeyboard) {
                    self.keyboard_category_options = fs::read_dir(&self.keyboards_path)
                        .unwrap()
                        .map(|r| r.unwrap())
                        .filter(|entry| {
                            entry.file_type().unwrap().is_dir() && entry.file_name() != "global"
                        })
                        .map(|entry| entry.file_name().to_str().unwrap().to_owned())
                        .collect::<Vec<_>>();
                } else if window.eq(&SaveDefinitionAs) {
                    self.save_keyboard_as_category
                        .clone_from(&self.settings.category);
                    self.save_keyboard_as_name
                        .clone_from(&self.keyboard_options[self.keyboard_choice.unwrap()]);
                } else if window.eq(&SaveStyleAs) {
                    self.save_style_as_name = self.style_options[self.style_choice.unwrap()].name();
                    self.save_style_as_global =
                        self.style_options[self.style_choice.unwrap()].is_global();
                }
                return self.windows.open(window).1.map(|_| Message::none());
            }
            Message::Exit => return window::close(self.main_window),
            Message::Closed(window) => {
                self.windows.was_closed(window);
                if window == self.main_window {
                    let mut settings_file = File::create(
                        home::home_dir()
                            .unwrap()
                            .join(".local/share/NuhxBoard/NuhxBoard.json"),
                    )
                    .unwrap();
                    serde_json::to_writer_pretty(&mut settings_file, &self.settings).unwrap();
                    if self.windows.empty() {
                        return iced::exit();
                    } else {
                        return self.windows.close_all().map(|_| Message::none());
                    }
                }
                if self.windows.empty() {
                    return iced::exit();
                }
                clear_canvas = false;
            }
            Message::ChangeBackground(color) => {
                self.style.background_color = color.into();
                self.color_pickers.keyboard_background = false;
            }
            Message::ToggleColorPicker(picker) => match picker {
                ColorPicker::KeyboardBackground => {
                    self.color_pickers.keyboard_background =
                        !self.color_pickers.keyboard_background;
                    clear_canvas = false;
                }
            },
            Message::UpdateCanvas => {}
        }
        if clear_canvas {
            self.canvas.clear();
        }
        Task::none()
    }

    pub fn view(&self, window: window::Id) -> iced::Element<'_, Message, Theme, Renderer> {
        self.windows.view(self, window)
    }

    pub fn theme(&self, window: window::Id) -> Theme {
        self.windows.theme(self, window)
    }

    pub fn title(&self, window: window::Id) -> String {
        self.windows.title(self, window)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            Subscription::run(listener::bind).map(Message::Listener),
            iced::keyboard::on_key_press(|key, modifiers| {
                if key == iced::keyboard::Key::Character(SmolStr::new("z"))
                    && ((std::env::consts::OS == "macos" && modifiers.command())
                        || modifiers.control())
                {
                    if modifiers.shift() {
                        Some(Message::Redo)
                    } else {
                        Some(Message::Undo)
                    }
                } else {
                    None
                }
            }),
            iced::window::close_events().map(Message::Closed),
        ])
    }

    fn error(&mut self, error: Error) -> iced::Task<Message> {
        let (_, command) = self.windows.open(Box::new(ErrorPopup { error }));
        command.map(|_| Message::none())
    }

    fn load_layout(&mut self, keyboard: usize) -> Task<Message> {
        self.settings.keyboard = keyboard;

        self.keyboard_choice = Some(keyboard);
        self.style = Style::default();

        let config_file = match File::open(
            self.keyboards_path
                .join(&self.settings.category)
                .join(&self.keyboard_options[keyboard])
                .join("keyboard.json"),
        ) {
            Ok(file) => file,
            Err(e) => {
                return self.error(Error::ConfigOpen(e.to_string()));
            }
        };

        self.layout = match serde_json::from_reader(config_file) {
            Ok(config) => config,
            Err(e) => {
                return self.error(Error::ConfigParse(if e.is_eof() {
                    format!("Unexpected EOF (End of file) at line {}", e.line())
                } else {
                    e.to_string()
                }))
            }
        };

        self.style_options = vec![StyleChoice::Default];
        self.style_options.append(
            &mut fs::read_dir(
                self.keyboards_path
                    .join(&self.settings.category)
                    .join(&self.keyboard_options[keyboard]),
            )
            .unwrap()
            .map(|r| r.unwrap())
            .filter(|entry| entry.file_type().unwrap().is_file())
            .filter(|entry| entry.path().extension() == Some(std::ffi::OsStr::new("style")))
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
        self.style_options.append(
            &mut fs::read_dir(self.keyboards_path.join("global"))
                .unwrap()
                .map(|r| r.unwrap())
                .filter(|entry| entry.file_type().unwrap().is_file())
                .filter(|entry| entry.path().extension() == Some(std::ffi::OsStr::new("style")))
                .map(|entry| {
                    StyleChoice::Global(
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

        window::resize(
            self.main_window,
            iced::Size {
                width: self.layout.width,
                height: self.layout.height,
            },
        )
    }

    fn load_style(&mut self, style: usize) -> Task<Message> {
        self.settings.style = style;

        self.style_choice = Some(style);

        if self.style_options[style] == StyleChoice::Default {
            self.style = Style::default();
        } else {
            let path = self.keyboards_path.join(match &self.style_options[style] {
                StyleChoice::Default => unreachable!(),
                StyleChoice::Global(style_name) => {
                    format!("global/{}.style", style_name)
                }
                StyleChoice::Custom(style_name) => format!(
                    "{}/{}/{}.style",
                    self.settings.category,
                    self.keyboard_options[self.keyboard_choice.unwrap()],
                    style_name
                ),
            });

            let style_file = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    return self.error(Error::StyleOpen(e.to_string()));
                }
            };
            self.style = match serde_json::from_reader(style_file) {
                Ok(style) => style,
                Err(e) => {
                    return self.error(Error::StyleParse(if e.is_eof() {
                        format!("Unexpeted EOF (End of file) at line {}", e.line())
                    } else {
                        e.to_string()
                    }))
                }
            };
        }

        if let Some(name) = &self.style.background_image_file_name {
            let path = self
                .keyboards_path
                .join(&self.settings.category)
                .join("images")
                .join(name);
            if !name.is_empty() && path.exists() {
                ImageReader::open(path)
                    .unwrap()
                    .decode()
                    .unwrap()
                    .resize_exact(
                        self.layout.width as u32,
                        self.layout.height as u32,
                        image::imageops::FilterType::Nearest,
                    )
                    .save(self.keyboards_path.parent().unwrap().join("background.png"))
                    .unwrap();
            } else {
                let _ =
                    fs::remove_file(self.keyboards_path.parent().unwrap().join("background.png"));
            }
        } else {
            let _ = fs::remove_file(self.keyboards_path.parent().unwrap().join("background.png"));
        }

        Task::none()
    }

    fn input_event(&mut self, event: rdev::Event) -> Task<Message> {
        match event.event_type {
            rdev::EventType::KeyPress(key) => {
                if key == rdev::Key::CapsLock {
                    self.true_caps = !self.true_caps;
                    if self.settings.capitalization == Capitalization::Follow {
                        self.caps = !self.caps;
                    }
                }
                let Ok(key) = keycode_convert(key) else {
                    return self.error(Error::UnknownKey(key));
                };
                self.pressed_keys.insert(key, Instant::now());
            }
            rdev::EventType::KeyRelease(key) => {
                let Ok(key_num) = keycode_convert(key) else {
                    return self.error(Error::UnknownKey(key));
                };
                if !self.pressed_keys.contains_key(&key_num) {
                    return Task::none();
                }
                if self
                    .pressed_keys
                    .get(&key_num)
                    .unwrap()
                    .elapsed()
                    .as_millis()
                    < self.settings.min_press_time
                {
                    return Task::perform(
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
                if button == rdev::Button::Unknown(6) || button == rdev::Button::Unknown(7) {
                    return Task::none();
                }
                let Ok(button) = mouse_button_code_convert(button) else {
                    return self.error(Error::UnknownButton(button));
                };

                self.pressed_mouse_buttons.insert(button, Instant::now());
            }
            rdev::EventType::ButtonRelease(button) => {
                let Ok(button_num) = mouse_button_code_convert(button) else {
                    return self.error(Error::UnknownButton(button));
                };
                if button == rdev::Button::Unknown(6) || button == rdev::Button::Unknown(7) {
                    return Task::none();
                }
                if !self.pressed_mouse_buttons.contains_key(&button_num) {
                    return Task::none();
                }
                if self
                    .pressed_mouse_buttons
                    .get(&button_num)
                    .unwrap()
                    .elapsed()
                    .as_millis()
                    < self.settings.min_press_time
                {
                    return Task::perform(
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

                return Task::perform(
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
                    Err(_) => return Task::none(),
                };
                if time_diff.as_millis() < 10 {
                    return Task::none();
                }

                let previous_pos = match self.settings.mouse_from_center {
                    true => {
                        let mut center = (0.0, 0.0);

                        for display in &self.display_options {
                            if display.id == self.settings.display_choice.id {
                                center = (
                                    display.x as f32 + (display.width as f32 / 2.0),
                                    display.y as f32 + (display.height as f32 / 2.0),
                                )
                            }
                        }
                        center
                    }
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
        }

        Task::none()
    }
}
