use crate::{
    logic::listener,
    types::{config::*, settings::*, style::*},
    ui::app::*,
    WindowUnion,
};
use display_info::DisplayInfo;
use iced::{
    advanced::graphics::core::SmolStr, multi_window::Application, widget::canvas::Cache, window,
    Color, Command, Renderer, Subscription, Theme,
};
use iced_multi_window::{window, WindowManager};
use std::{
    collections::HashMap,
    fs::{self, File},
    time::Instant,
};

#[derive(Debug)]
pub struct NuhxBoard {
    pub windows: WindowManager<NuhxBoard, WindowUnion>,
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

#[derive(Debug, Clone)]
pub enum Message {
    Open(WindowUnion),
    Closed(window::Id),
    Listener(listener::Event),
    ReleaseScroll(u32),
    LoadStyle(usize),
    ClosingMain,
    ChangeKeyboardCategory(String),
    LoadLayout(usize),
    ChangeSetting(Setting),
    ClearPressedKeys,
    ToggleEditMode,
    MoveElement { index: usize, delta: geo::Coord },
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
}

#[derive(Debug, Clone)]
pub enum ColorPicker {
    KeyboardBackground,
}

// TODO: Are window resized undoable?
#[derive(Debug, Clone)]
pub enum Change {
    MoveElement {
        index: usize,
        delta: geo::Coord,
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

impl Application for NuhxBoard {
    type Flags = Settings;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new(flags: Settings) -> (Self, Command<Self::Message>) {
        let keyboards_path = home::home_dir()
            .unwrap()
            .join(".local/share/NuhxBoard/keyboards");
        let layout = Layout {
            version: None,
            width: DEFAULT_WINDOW_SIZE.width,
            height: DEFAULT_WINDOW_SIZE.height,
            elements: Vec::new(),
        };

        let category = flags.category.clone();

        let caps = match flags.capitalization {
            Capitalization::Upper => true,
            Capitalization::Lower => false,
            Capitalization::Follow => false,
        };

        (
            Self {
                windows: WindowManager::new(window!(Main {})),
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
                keyboard_choice: Some(flags.keyboard),
                style_choice: Some(flags.style),
                keyboard_options: Vec::new(),
                keyboard_category_options: Vec::new(),
                style_options: Vec::new(),
                keyboards_path,
                startup: true,
                settings: flags,
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
            Command::batch([
                Command::perform(async {}, move |_| Message::ChangeKeyboardCategory(category)),
                iced::font::load(iced_aw::core::icons::BOOTSTRAP_FONT_BYTES)
                    .map(|_| Message::none()),
            ]),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        let mut clear_canvas = true;
        match message {
            Message::Listener(listener::Event::KeyReceived(event)) => {
                self.canvas.clear();
                return self.input_event(event);
            }
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
                // TODO: why is this here?
                if category.is_empty() {
                    return Command::none();
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
            Message::ClosingMain => {
                let mut settings_file = File::create(
                    home::home_dir()
                        .unwrap()
                        .join(".local/share/NuhxBoard/NuhxBoard.json"),
                )
                .unwrap();
                serde_json::to_writer_pretty(&mut settings_file, &self.settings).unwrap();
                return self.windows.close_all();
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
            Message::Listener(_) => clear_canvas = false,
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
                    window::Id::MAIN,
                    iced::Size {
                        width: self.layout.width,
                        height: self.layout.height,
                    },
                );
            }
            Message::SetWidth(width) => {
                self.layout.width = width;
                return window::resize(
                    window::Id::MAIN,
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
                clear_canvas = false;
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
                match window {
                    window!(LoadKeyboard {}) => {
                        self.keyboard_category_options = fs::read_dir(&self.keyboards_path)
                            .unwrap()
                            .map(|r| r.unwrap())
                            .filter(|entry| {
                                entry.file_type().unwrap().is_dir() && entry.file_name() != "global"
                            })
                            .map(|entry| entry.file_name().to_str().unwrap().to_owned())
                            .collect::<Vec<_>>();
                    }
                    window!(SaveDefinitionAs {}) => {
                        self.save_keyboard_as_category
                            .clone_from(&self.settings.category);
                        self.save_keyboard_as_name
                            .clone_from(&self.keyboard_options[self.keyboard_choice.unwrap()]);
                    }
                    window!(SaveStyleAs {}) => {
                        self.save_style_as_name =
                            self.style_options[self.style_choice.unwrap()].name();
                        self.save_style_as_global =
                            self.style_options[self.style_choice.unwrap()].is_global();
                    }
                    _ => {}
                }
                return self.windows.spawn(window).1;
            }
            Message::Closed(window) => {
                self.windows.closed(window);
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
        }
        if clear_canvas {
            self.canvas.clear();
        }
        Command::none()
    }

    fn view(&self, window: window::Id) -> iced::Element<'_, Self::Message, Self::Theme, Renderer> {
        self.windows.view(self, window)
    }

    fn theme(&self, window: window::Id) -> Self::Theme {
        self.windows.theme(self, window)
    }

    fn title(&self, window: window::Id) -> String {
        self.windows.title(self, window)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            listener::bind().map(Message::Listener),
            iced::event::listen_with(|event, _| match event {
                iced::Event::Window(id, window::Event::Closed) => Some(Message::Closed(id)),
                iced::Event::Window(window::Id::MAIN, window::Event::CloseRequested) => {
                    Some(Message::ClosingMain)
                }
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key,
                    location: _,
                    modifiers,
                    text: _,
                }) => {
                    if modifiers.command()
                        && key == iced::keyboard::Key::Character(SmolStr::new("z"))
                    {
                        if modifiers.shift() {
                            Some(Message::Redo)
                        } else {
                            Some(Message::Undo)
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
    pub fn error(&mut self, error: Error) -> iced::Command<Message> {
        let (_, command) = self.windows.spawn(window!(ErrorPopup { error }));
        command
    }
}
