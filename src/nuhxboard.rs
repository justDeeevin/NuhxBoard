use crate::{
    message::*,
    types::*,
    ui::{app::*, popups::*},
};
use display_info::DisplayInfo;
use geo::{Centroid, Coord, CoordsIter, LineString, Polygon, Rect};
use iced::{
    advanced::{graphics::core::SmolStr, subscription},
    widget::canvas::Cache,
    window, Renderer, Subscription, Task, Theme,
};
use iced_multi_window::WindowManager;
use image::ImageReader;
use nuhxboard_logic::{listener::RdevinSubscriber, mouse_button_code_convert};
use nuhxboard_types::{
    layout::*,
    settings::*,
    style::{self, *},
};
use rdevin::keycodes::windows::code_from_key as win_keycode_from_key;
use smol::Timer;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
    time::{Duration, Instant},
};
use tracing::{debug, info, trace};

macro_rules! key_style_change {
    ($self:expr, $state:ident, $block:block, $id:ident) => {
        $self.style
            .element_styles
            .entry($id)
            .and_modify(|style| {
                let style::ElementStyle::KeyStyle(ref mut style) = style else {
                    panic!()
                };
                if let Some($state) = style.$state.as_mut() {
                    $block
                } else {
                    let mut $state = $self.style.default_key_style.$state.clone();
                    $block
                    style.$state = Some($state);
                }
            })
            .or_insert_with(|| {
                let mut style = $self.style.default_key_style.clone();
                let $state = &mut style.$state;
                $block
                style::ElementStyle::KeyStyle(style.into())
            });

        $self.clear_cache_by_id($id);
    }
}

// Iced requires font family names to have a static lifetime. This means that `String`s read from config
// must be leaked to be used. This data structure acts as a cache of fonts that have been used, so
// the names only need to be leaked once.
pub static FONTS: LazyLock<RwLock<HashSet<&'static str>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

pub static KEYBOARDS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    confy::get_configuration_file_path("NuhxBoard", None)
        .unwrap()
        .parent()
        .unwrap()
        .join("keyboards")
});

#[derive(Default)]
pub struct ElementCache {
    pub fg: Cache,
    pub bg: Cache,
}

impl ElementCache {
    pub fn clear(&self) {
        self.fg.clear();
        self.bg.clear();
    }
}

pub struct NuhxBoard {
    pub windows: WindowManager<Self, Theme, Message>,
    pub main_window: window::Id,
    pub caches: Vec<ElementCache>,
    pub caches_by_keycode: HashMap<u32, usize>,
    pub caches_by_mouse_button: HashMap<u32, usize>,
    pub caches_by_scroll_button: HashMap<u32, usize>,
    pub caches_by_id: HashMap<u32, usize>,
    pub mouse_speed_indicator_caches: Vec<usize>,
    pub layout: Layout,
    pub style: Style,
    /// `{[keycode]: [time_pressed]}`
    pub pressed_keys: HashMap<u32, Instant>,
    /// `{[keycode]: [time_pressed]}`
    pub pressed_mouse_buttons: HashMap<u32, Instant>,
    /// `{[axis]: [releases_queued]}`
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
    pub text_input: TextInput,
    pub number_input: NumberInput,
    pub selections: SelectionLists,
    pub hovered_element: Option<usize>,
    pub detecting: Vec<usize>,
}

pub const DEFAULT_WINDOW_SIZE: iced::Size = iced::Size {
    width: 200.0,
    height: 200.0,
};

impl NuhxBoard {
    pub fn new() -> (Self, Task<Message>) {
        info!("Startup");
        let mut errors = Vec::new();

        info!("Loading settings");
        let settings: Settings = confy::load("NuhxBoard", None).unwrap_or_else(|e| {
            errors.push(NuhxBoardError::SettingsParse(Arc::new(e)));
            Settings::default()
        });

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

        debug!("Main window");
        // The app will open the main window on startup. The WindowManager automatically tracks IDs
        // and corresponding window types and runs the correct view, theme, and title logic when
        // necessary.
        let (main_window, window_open_task) = windows.open(Box::new(Main));

        (
            Self {
                windows,
                main_window,
                caches: Vec::new(),
                caches_by_keycode: HashMap::new(),
                caches_by_mouse_button: HashMap::new(),
                caches_by_scroll_button: HashMap::new(),
                caches_by_id: HashMap::new(),
                mouse_speed_indicator_caches: Vec::new(),
                layout,
                style: Style::default(),
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
                text_input: TextInput::default(),
                hovered_element: None,
                number_input: NumberInput::default(),
                selections: SelectionLists::default(),
                detecting: Vec::new(),
            },
            Task::batch([
                Task::perform(async {}, move |_| {
                    Message::ChangeKeyboardCategory(category.clone())
                }),
                window_open_task.map(|_| Message::None),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Listener(event) => {
                debug!("Input event");
                trace!(?event);
                return self.input_event(event);
            }
            Message::None => {}
            Message::ReleaseScroll(button) => {
                debug!(button, "Scroll release");
                match self.pressed_scroll_buttons.get_mut(&button).unwrap() {
                    1 => {
                        debug!("Disabling scroll highlight");
                        self.pressed_scroll_buttons.remove(&button);
                    }
                    n => {
                        *n -= 1;
                    }
                }
            }
            Message::ChangeKeyboardCategory(category) => {
                info!(category, "Keyboard category changed");
                if category.is_empty() {
                    return Task::none();
                }
                self.settings.category = category.clone();

                self.text_input.save_keyboard_as_category = category;

                if !self.startup {
                    self.keyboard_choice = None;
                    self.settings.keyboard = 0;
                    self.style_choice = None;
                    self.settings.style = 0;
                    self.style_options = Vec::new();
                }

                self.keyboard_options = fs::read_dir(KEYBOARDS_PATH.join(&self.settings.category))
                    .unwrap()
                    .map(|r| r.unwrap())
                    .filter(|entry| {
                        entry.file_type().unwrap().is_dir() && entry.file_name() != "images"
                    })
                    .map(|entry| entry.file_name().to_str().unwrap().to_owned())
                    .collect();
                self.keyboard_options.sort();

                if self.startup {
                    return self.update(Message::LoadLayout(self.keyboard_choice.unwrap()));
                }
            }
            Message::LoadLayout(layout) => {
                info!(layout, "Layout changed");
                return self.load_layout(layout);
            }
            Message::LoadStyle(style) => {
                info!(style, "Style changed");
                return self.load_style(style);
            }
            Message::ChangeSetting(setting) => {
                info!(?setting, "Setting changed");
                match setting {
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
                    Setting::UpdateTextPosition => {
                        self.settings.update_text_position = !self.settings.update_text_position;
                    }
                }
            }
            Message::ClearPressedKeys => {
                info!("Clearing pressed keys");
                self.pressed_keys.clear();
                self.clear_all_caches();
            }
            Message::ToggleEditMode => {
                if self.edit_mode {
                    info!("Exiting edit mode");
                } else {
                    info!("Entering edit mode");
                }
                self.edit_mode = !self.edit_mode;
            }
            Message::MoveElement { index, delta } => {
                debug!(delta = ?(delta.x, delta.y), index, "Moving element");
                self.layout.elements[index].translate(delta, self.settings.update_text_position);
                self.caches[index].clear();
            }
            Message::MoveFace { index, face, delta } => {
                debug!(index, face, delta = ?(delta.x, delta.y), "Moving face");
                match CommonDefinitionMut::try_from(&mut self.layout.elements[index]) {
                    Ok(mut def) => {
                        def.translate_face(face, delta);
                    }
                    Err(def) => {
                        def.radius += delta.x;
                    }
                }
                self.caches[index].clear();
            }
            Message::MoveVertex {
                index,
                vertex,
                delta,
            } => {
                debug!(index, vertex, delta = ?(delta.x, delta.y), "Moving vertex");
                let def = CommonDefinitionMut::try_from(&mut self.layout.elements[index]).unwrap();
                def.boundaries[vertex] += delta;
                self.caches[index].clear();
            }
            Message::SaveLayout(file) => {
                info!(?file, "Saving layout");
                let path = file.unwrap_or(KEYBOARDS_PATH.join(format!(
                    "{}/{}/keyboard.json",
                    self.settings.category,
                    self.keyboard_options[self.keyboard_choice.unwrap()]
                )));
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                let mut file = File::create(path).unwrap();
                serde_json::to_writer_pretty(&mut file, &self.layout).unwrap();
            }
            Message::SaveStyle(file) => {
                info!(?file, "Saving style");
                let path = file.unwrap_or(KEYBOARDS_PATH.join(format!(
                    "{}/{}/{}.style",
                    self.settings.category,
                    self.keyboard_options[self.keyboard_choice.unwrap()],
                    self.style_options[self.style_choice.unwrap()]
                )));
                let mut file = File::create(path).unwrap();
                serde_json::to_writer_pretty(&mut file, &self.style).unwrap();
            }
            Message::SetHeight(height) => {
                debug!(height, "Setting height");
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
                debug!(width, "Setting width");
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
                debug!(?change, "Pushing change");
                if self.history_depth > 0 {
                    self.edit_history
                        .truncate(self.edit_history.len() - self.history_depth);
                    self.history_depth = 0;
                }
                self.edit_history.push(change);
            }
            Message::Undo => {
                debug!("Undo");
                if self.history_depth < self.edit_history.len() {
                    self.history_depth += 1;
                    match self.edit_history[self.edit_history.len() - self.history_depth] {
                        Change::MoveElement { index, delta } => {
                            self.layout.elements[index]
                                .translate(-delta, self.settings.update_text_position);
                            self.caches[index].clear();
                        }
                        Change::MoveFace { index, face, delta } => {
                            match CommonDefinitionMut::try_from(&mut self.layout.elements[index]) {
                                Ok(mut def) => {
                                    def.translate_face(face, -delta);
                                }
                                Err(def) => {
                                    def.radius -= delta.x;
                                }
                            }
                            self.caches[index].clear();
                        }
                        Change::MoveVertex {
                            index,
                            vertex,
                            delta,
                        } => {
                            let def =
                                CommonDefinitionMut::try_from(&mut self.layout.elements[index])
                                    .unwrap();
                            def.boundaries[vertex] -= delta;
                            self.caches[index].clear();
                        }
                    }
                }
            }
            Message::Redo => {
                debug!("Redo");
                if self.history_depth > 0 {
                    self.history_depth -= 1;
                    match self.edit_history[self.edit_history.len() - self.history_depth - 1] {
                        Change::MoveElement { index, delta } => {
                            self.layout.elements[index]
                                .translate(delta, self.settings.update_text_position);
                        }
                        Change::MoveFace { index, face, delta } => {
                            match CommonDefinitionMut::try_from(&mut self.layout.elements[index]) {
                                Ok(mut def) => {
                                    def.translate_face(face, delta);
                                }
                                Err(def) => {
                                    def.radius += delta.x;
                                }
                            }
                            self.caches[index].clear();
                        }
                        Change::MoveVertex {
                            index,
                            vertex,
                            delta,
                        } => {
                            let def =
                                CommonDefinitionMut::try_from(&mut self.layout.elements[index])
                                    .unwrap();
                            def.boundaries[vertex] += delta;
                            self.caches[index].clear();
                        }
                    }
                }
            }
            Message::ChangeTextInput(input, value) => {
                debug!(?input, value, "Changing text input");
                match input {
                    TextInputType::SaveStyleAsName => self.text_input.save_style_as_name = value,
                    TextInputType::SaveKeyboardAsName => {
                        self.text_input.save_keyboard_as_name = value;
                    }
                    TextInputType::SaveKeyboardAsCategory => {
                        self.text_input.save_keyboard_as_category = value;
                    }
                    TextInputType::KeyboardBackgroundImage => {
                        self.text_input.keyboard_background_image = value;
                    }
                    TextInputType::DefaultLooseKeyBackgroundImage => {
                        self.text_input.default_loose_key_background_image = value;
                        self.clear_all_caches();
                    }
                    TextInputType::DefaultLooseKeyFontFamily => {
                        self.text_input.default_loose_key_font_family = value;
                        self.clear_all_caches();
                    }
                    TextInputType::DefaultPressedKeyBackgroundImage => {
                        self.text_input.default_pressed_key_background_image = value;
                        self.clear_all_caches();
                    }
                    TextInputType::DefaultPressedKeyFontFamily => {
                        self.text_input.default_pressed_key_font_family = value;
                        self.clear_all_caches();
                    }
                    TextInputType::LooseBackgroundImage(id) => {
                        self.text_input.loose_background_image.insert(id, value);
                        self.clear_cache_by_id(id);
                    }
                    TextInputType::LooseFontFamily(id) => {
                        self.text_input.loose_font_family.insert(id, value);
                        self.clear_cache_by_id(id);
                    }
                    TextInputType::PressedBackgroundImage(id) => {
                        self.text_input.pressed_background_image.insert(id, value);
                        self.clear_cache_by_id(id);
                    }
                    TextInputType::PressedFontFamily(id) => {
                        self.text_input.pressed_font_family.insert(id, value);
                        self.clear_cache_by_id(id);
                    }
                }
            }
            Message::ChangeStyle(style) => {
                debug!(?style, "Changing style");
                self.change_style(style);
            }
            Message::ToggleSaveStyleAsGlobal => {
                self.save_style_as_global = !self.save_style_as_global;
            }
            Message::Open(window) => {
                info!(id = window.id(), "Opening new window");
                if window == LoadKeyboard {
                    self.keyboard_category_options = fs::read_dir(&*KEYBOARDS_PATH)
                        .unwrap()
                        .map(|r| r.unwrap())
                        .filter(|entry| {
                            entry.file_type().unwrap().is_dir() && entry.file_name() != "global"
                        })
                        .map(|entry| entry.file_name().to_str().unwrap().to_owned())
                        .collect::<Vec<_>>();
                    self.keyboard_category_options.sort();
                } else if window == SaveStyleAs {
                    self.save_style_as_global =
                        self.style_options[self.style_choice.unwrap()].is_global();
                }
                return self.windows.open(window).1.map(|_| Message::None);
            }
            Message::CloseAllOf(window) => {
                info!(id = window.id(), "Closing all windows");
                return self.windows.close_all_of(window).map(|_| Message::None);
            }
            Message::Exit => {
                info!("Exiting");
                return window::close(self.main_window);
            }
            Message::Closed(window) => {
                info!(%window, "Window closed");
                self.windows.was_closed(window);

                if window == self.main_window {
                    confy::store("nuhxboard", None, self.settings.clone()).unwrap();
                    if self.windows.empty() {
                        return iced::exit();
                    } else {
                        return self.windows.close_all().map(|_| Message::None);
                    }
                }

                if self.windows.empty() {
                    return iced::exit();
                }
            }
            Message::ChangeColor(picker, color) => {
                debug!(?picker, ?color, "Changing color picker");
                // I love macros!
                macro_rules! mouse_speed_indicator_style_change {
                    ($name:ident, $block:block, $id:ident) => {
                        if let Some($name) = self
                            .style
                            .element_styles
                            .get_mut(&$id)
                            .map(|v| {
                                let style::ElementStyle::MouseSpeedIndicatorStyle(ref mut key) =
                                    v else {
                                        panic!()
                                    };
                                key
                            })
                        {
                            $block
                        } else {
                            let mut $name = self.style.default_mouse_speed_indicator_style.clone();
                            $block
                            self.style.element_styles.insert(
                                $id,
                                style::ElementStyle::MouseSpeedIndicatorStyle($name),
                            );
                        }
                        self.clear_cache_by_id($id);
                    }
                }
                let loose = &mut self.style.default_key_style.loose;
                let pressed = &mut self.style.default_key_style.pressed;
                self.color_pickers.toggle(picker);
                match picker {
                    ColorPicker::KeyboardBackground => {
                        self.style.background_color = color.into();
                    }
                    ColorPicker::DefaultLooseBackground => {
                        loose.background = color.into();
                        self.clear_all_caches();
                    }
                    ColorPicker::DefaultLooseText => {
                        loose.text = color.into();
                        self.clear_all_caches();
                    }
                    ColorPicker::DefaultLooseOutline => {
                        loose.outline = color.into();
                        self.clear_all_caches();
                    }
                    ColorPicker::DefaultPressedBackground => {
                        pressed.background = color.into();
                        self.clear_all_caches();
                    }
                    ColorPicker::DefaultPressedText => {
                        pressed.text = color.into();
                        self.clear_all_caches();
                    }
                    ColorPicker::DefaultPressedOutline => {
                        pressed.outline = color.into();
                        self.clear_all_caches();
                    }
                    ColorPicker::DefaultMouseSpeedIndicator1 => {
                        self.style.default_mouse_speed_indicator_style.inner_color = color.into();
                        self.clear_all_caches();
                    }
                    ColorPicker::DefaultMouseSpeedIndicator2 => {
                        self.style.default_mouse_speed_indicator_style.outer_color = color.into();
                        self.clear_all_caches();
                    }
                    ColorPicker::LooseBackground(id) => {
                        key_style_change!(self, loose, { loose.background = color.into() }, id);
                    }
                    ColorPicker::LooseText(id) => {
                        key_style_change!(self, loose, { loose.text = color.into() }, id);
                    }
                    ColorPicker::LooseOutline(id) => {
                        key_style_change!(self, loose, { loose.outline = color.into() }, id);
                    }
                    ColorPicker::PressedBackground(id) => {
                        key_style_change!(self, pressed, { pressed.background = color.into() }, id);
                    }
                    ColorPicker::PressedText(id) => {
                        key_style_change!(self, pressed, { pressed.text = color.into() }, id);
                    }
                    ColorPicker::PressedOutline(id) => {
                        key_style_change!(self, pressed, { pressed.outline = color.into() }, id);
                    }
                    ColorPicker::MouseSpeedIndicator1(id) => {
                        mouse_speed_indicator_style_change!(
                            style,
                            { style.inner_color = color.into() },
                            id
                        );
                    }
                    ColorPicker::MouseSpeedIndicator2(id) => {
                        mouse_speed_indicator_style_change!(
                            style,
                            { style.outer_color = color.into() },
                            id
                        );
                    }
                }
            }
            Message::ToggleColorPicker(picker) => {
                debug!(?picker, "Toggling color picker");
                self.color_pickers.toggle(picker);
            }
            Message::UpdateHoveredElement(hovered_element) => {
                debug!(?hovered_element, "Updating hovered element");
                if let Some(hovered_element) = self.hovered_element {
                    self.caches[hovered_element].clear();
                }
                if let Some(hovered_element) = hovered_element {
                    self.caches[hovered_element].clear();
                }
                self.hovered_element = hovered_element;
            }
            Message::ChangeElement(element_i, property) => {
                debug!(element_i, ?property, "Changing element");
                let element = &mut self.layout.elements[element_i];
                let mouse_key = matches!(
                    element,
                    BoardElement::MouseKey(_) | BoardElement::MouseScroll(_)
                );
                let mut handeled = true;
                if let Ok(def) = CommonDefinitionMut::try_from(&mut *element) {
                    match property {
                        ElementProperty::Text(ref v) => *def.text = v.clone(),
                        ElementProperty::TextPositionX(v) => *def.text_position.x = v,
                        ElementProperty::TextPositionY(v) => *def.text_position.y = v,
                        ElementProperty::Boundary(i, ref v) => {
                            if let Some(v) = v {
                                if i >= def.boundaries.len() {
                                    def.boundaries.push(v.clone());
                                } else {
                                    def.boundaries[i] = v.clone();
                                }
                            } else {
                                def.boundaries.remove(i);
                                self.selections.boundary.remove(&element_i);
                            }
                        }
                        ElementProperty::Keycode(i, v) => {
                            if let Some(v) = v {
                                if mouse_key {
                                    def.key_codes[0] = v;
                                } else {
                                    def.key_codes.push(v);
                                }
                            } else {
                                def.key_codes.remove(i);
                                self.selections.keycode.remove(&element_i);
                            }
                        }
                        _ => handeled = false,
                    }
                } else {
                    handeled = false;
                }
                if !handeled {
                    match element {
                        BoardElement::KeyboardKey(def) => match property {
                            ElementProperty::ShiftText(v) => def.shift_text = v,
                            ElementProperty::FollowCaps => def.change_on_caps = !def.change_on_caps,
                            _ => panic!("Invalid property for selected element"),
                        },
                        BoardElement::MouseKey(_) | BoardElement::MouseScroll(_) => {
                            panic!("Invalid property for selected element")
                        }
                        BoardElement::MouseSpeedIndicator(def) => match property {
                            ElementProperty::MouseSpeedIndicatorPositionX(v) => *def.location.x = v,
                            ElementProperty::MouseSpeedIndicatorPositionY(v) => *def.location.y = v,
                            ElementProperty::MouseSpeedIndicatorRadius(v) => def.radius = v,
                            _ => panic!("Invalid property for selected element"),
                        },
                    }
                }
                self.caches[element_i].clear();
            }
            Message::CenterTextPosition(i) => {
                debug!(element = i, "Centering text position");
                let element = &mut self.layout.elements[i];
                let Ok(def) = CommonDefinitionMut::try_from(element) else {
                    panic!("Cannot center text position of mouse speed indicator");
                };
                let bounds = Polygon::new(
                    LineString::new(
                        def.boundaries
                            .iter()
                            .map(|p| Coord::from(p.clone()))
                            .collect::<Vec<_>>(),
                    ),
                    vec![],
                );
                let centroid = bounds.centroid().unwrap();

                def.text_position.x = centroid.x().trunc().into();
                def.text_position.y = centroid.y().trunc().into();
                self.caches[i].clear();
            }
            Message::ChangeNumberInput(input_type) => {
                debug!(?input_type, "Changing number input");
                match input_type {
                    NumberInputType::BoundaryX(element, v) => {
                        self.number_input.boundary_x.insert(element, v);
                    }
                    NumberInputType::BoundaryY(element, v) => {
                        self.number_input.boundary_y.insert(element, v);
                    }
                    NumberInputType::Keycode(element, v) => {
                        self.number_input.keycode.insert(element, v);
                    }
                    NumberInputType::RectanglePositionX(element, v) => {
                        self.number_input.rectangle_position_x.insert(element, v);
                    }
                    NumberInputType::RectanglePositionY(element, v) => {
                        self.number_input.rectangle_position_y.insert(element, v);
                    }
                    NumberInputType::RectangleSizeX(element, v) => {
                        self.number_input.rectangle_size_x.insert(element, v);
                    }
                    NumberInputType::RectangleSizeY(element, v) => {
                        self.number_input.rectangle_size_y.insert(element, v);
                    }
                }
            }
            Message::ChangeSelection(element, selection_type, selection) => {
                debug!(?element, ?selection_type, ?selection, "Changing selection");
                match selection_type {
                    SelectionType::Boundary => {
                        self.selections.boundary.insert(element, selection);
                    }
                    SelectionType::Keycode => {
                        self.selections.keycode.insert(element, selection);
                    }
                }
            }
            Message::SwapBoundaries(element_i, left, right) => {
                debug!(element_i, left, right, "Swapping boundaries");
                let element = &mut self.layout.elements[element_i];
                let Ok(def) = CommonDefinitionMut::try_from(element) else {
                    panic!("Cannot swap boundaries of mouse speed indicator");
                };
                def.boundaries.swap(left, right);
                self.selections.boundary.insert(element_i, right);
                self.caches[element_i].clear();
            }
            Message::MakeRectangle(element_i) => {
                debug!(element_i, "Making rectangle");
                let element = &mut self.layout.elements[element_i];
                let Ok(def) = CommonDefinitionMut::try_from(element) else {
                    panic!("Cannot make rectangle of mouse speed indicator");
                };
                def.boundaries.clear();
                let top_left = Coord {
                    x: self
                        .number_input
                        .rectangle_position_x
                        .get(&element_i)
                        .copied()
                        .unwrap_or_default(),
                    y: self
                        .number_input
                        .rectangle_position_y
                        .get(&element_i)
                        .copied()
                        .unwrap_or_default(),
                };
                let rect = Rect::new(
                    top_left,
                    top_left
                        + Coord {
                            x: self
                                .number_input
                                .rectangle_size_x
                                .get(&element_i)
                                .copied()
                                .unwrap_or_default(),
                            y: self
                                .number_input
                                .rectangle_size_y
                                .get(&element_i)
                                .copied()
                                .unwrap_or_default(),
                        },
                );
                rect.exterior_coords_iter().for_each(|point| {
                    def.boundaries.push(point.into());
                });

                self.caches[element_i].clear();

                return self
                    .windows
                    .close_all_of(Box::new(RectangleDialog { index: element_i }))
                    .map(|_| Message::None);
            }
            Message::StartDetecting(element) => {
                debug!(element, "Detection begun for element");
                self.detecting.push(element);
            }
            Message::ClearCache(i) => {
                self.caches[i].clear();
            }
            Message::ClearAllCaches => {
                self.clear_all_caches();
            }
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
            subscription::from_recipe(RdevinSubscriber).map(Message::Listener),
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

    fn error(&mut self, error: NuhxBoardError) -> iced::Task<Message> {
        let (_, command) = self.windows.open(Box::new(ErrorPopup { error }));
        command.map(|_| Message::None)
    }

    fn load_layout(&mut self, keyboard: usize) -> Task<Message> {
        self.edit_mode = false;
        self.settings.keyboard = keyboard;

        self.keyboard_choice = Some(keyboard);
        self.style = Style::default();
        self.update_fonts();

        self.text_input.save_keyboard_as_name = self.keyboard_options[keyboard].clone();

        let config_file = match File::open(
            KEYBOARDS_PATH
                .join(&self.settings.category)
                .join(&self.keyboard_options[keyboard])
                .join("keyboard.json"),
        ) {
            Ok(file) => file,
            Err(e) => {
                return self.error(NuhxBoardError::LayoutOpen(Arc::new(e)));
            }
        };

        self.layout = match serde_json::from_reader(config_file) {
            Ok(config) => config,
            Err(e) => {
                return self.error(NuhxBoardError::LayoutParse(Arc::new(e)));
            }
        };

        self.caches.clear();
        self.caches_by_keycode.clear();
        self.caches_by_mouse_button.clear();
        self.caches_by_scroll_button.clear();
        self.mouse_speed_indicator_caches.clear();
        for (i, e) in self.layout.elements.iter().enumerate() {
            self.caches_by_id.insert(e.id(), i);
            let (fg, bg) = std::array::from_fn(|_| Cache::new()).into();
            self.caches.push(ElementCache { fg, bg });
            match e {
                BoardElement::KeyboardKey(def) => {
                    self.caches_by_keycode
                        .extend(def.key_codes.iter().map(|c| (*c, i)));
                }
                BoardElement::MouseKey(def) => {
                    self.caches_by_mouse_button
                        .extend(def.key_codes.iter().map(|c| (*c, i)));
                }
                BoardElement::MouseScroll(def) => {
                    self.caches_by_scroll_button
                        .extend(def.key_codes.iter().map(|c| (*c, i)));
                }
                _ => {}
            }
        }

        self.style_options = vec![StyleChoice::Default];
        self.style_options.append(
            &mut fs::read_dir(
                KEYBOARDS_PATH
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
            &mut fs::read_dir(KEYBOARDS_PATH.join("global"))
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
        self.style_options.sort();
        self.style_choice = Some(0);

        let resize_task = window::resize(
            self.main_window,
            iced::Size {
                width: self.layout.width,
                height: self.layout.height,
            },
        );

        if self.startup {
            self.startup = false;
            Task::batch([
                resize_task,
                self.update(Message::LoadStyle(self.settings.style)),
            ])
        } else {
            resize_task
        }
    }

    fn change_background_image(&mut self, new_image: Option<Option<String>>) {
        if let Some(new_image) = new_image {
            self.style.background_image_file_name = new_image;
        }
        if let Some(name) = &self.style.background_image_file_name {
            let path = KEYBOARDS_PATH
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
                    .save(KEYBOARDS_PATH.parent().unwrap().join("background.png"))
                    .unwrap();
            } else {
                let _ = fs::remove_file(KEYBOARDS_PATH.parent().unwrap().join("background.png"));
            }
        } else {
            let _ = fs::remove_file(KEYBOARDS_PATH.parent().unwrap().join("background.png"));
        }
    }

    fn load_style(&mut self, style: usize) -> Task<Message> {
        self.settings.style = style;

        self.style_choice = Some(style);

        if self.style_options[style] == StyleChoice::Default {
            self.style = Style::default();
        } else {
            let path = KEYBOARDS_PATH.join(match &self.style_options[style] {
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
                    return self.error(NuhxBoardError::StyleOpen(Arc::new(e)));
                }
            };
            self.style = match serde_json::from_reader(style_file) {
                Ok(style) => style,
                Err(e) => {
                    return self.error(NuhxBoardError::StyleParse(Arc::new(e)));
                }
            };
        }

        self.change_background_image(None);

        self.update_fonts();

        self.text_input.save_style_as_name = self.style_options[style].name();

        self.text_input.keyboard_background_image = self
            .style
            .background_image_file_name
            .clone()
            .unwrap_or_default();
        self.text_input.default_loose_key_background_image = self
            .style
            .default_key_style
            .loose
            .background_image_file_name
            .clone()
            .unwrap_or_default();
        self.text_input.default_pressed_key_background_image = self
            .style
            .default_key_style
            .pressed
            .background_image_file_name
            .clone()
            .unwrap_or_default();

        self.clear_all_caches();
        Task::none()
    }

    fn update_fonts(&self) {
        let mut new_fonts = HashSet::new();
        new_fonts.insert(self.style.default_key_style.loose.font.font_family.clone());
        new_fonts.insert(
            self.style
                .default_key_style
                .pressed
                .font
                .font_family
                .clone(),
        );
        new_fonts.extend(
            self.style
                .element_styles
                .iter()
                .filter_map(|(_, style)| match style {
                    style::ElementStyle::KeyStyle(key_style) => Some(
                        [
                            if let Some(loose) = &key_style.loose {
                                loose.font.font_family.clone()
                            } else {
                                Font::default().font_family
                            },
                            if let Some(pressed) = &key_style.pressed {
                                pressed.font.font_family.clone()
                            } else {
                                Font::default().font_family
                            },
                        ]
                        .into_iter(),
                    ),
                    style::ElementStyle::MouseSpeedIndicatorStyle(_) => None,
                })
                .flatten(),
        );

        for font in new_fonts {
            if !FONTS.read().unwrap().contains(font.as_str()) {
                FONTS.write().unwrap().insert(font.leak());
            }
        }
    }

    fn input_event(&mut self, event: rdevin::Event) -> Task<Message> {
        let mut captured_key = None;
        let mut out = Task::none();
        match event.event_type {
            rdevin::EventType::KeyPress(key) => {
                debug!(?key, "Key pressed");
                if key == rdevin::Key::CapsLock {
                    self.true_caps = !self.true_caps;
                    if self.settings.capitalization == Capitalization::Follow {
                        self.caps = !self.caps;
                    }
                }
                let Some(key) = win_keycode_from_key(key) else {
                    return self.error(NuhxBoardError::UnknownKey(key));
                };
                self.pressed_keys.insert(key, Instant::now());
                if let Some(i) = self.caches_by_keycode.get(&key) {
                    self.caches[*i].clear();
                }
                captured_key = Some(key);
            }
            rdevin::EventType::KeyRelease(key) => {
                debug!(?key, "Key released");
                let Some(key_num) = win_keycode_from_key(key) else {
                    return self.error(NuhxBoardError::UnknownKey(key));
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
                    < self.settings.min_press_time.into()
                {
                    return Task::perform(
                        Timer::after(
                            Duration::from_millis(self.settings.min_press_time)
                                - self.pressed_keys.get(&key_num).unwrap().elapsed(),
                        ),
                        move |_| Message::key_release(key),
                    );
                }
                debug!("Disabling key highlight");
                self.pressed_keys.remove(&key_num);
                if let Some(i) = self.caches_by_keycode.get(&key_num) {
                    self.caches[*i].clear();
                }
            }
            rdevin::EventType::ButtonPress(button) => {
                debug!(?button, "Button pressed");
                if button == rdevin::Button::Unknown(6) || button == rdevin::Button::Unknown(7) {
                    return Task::none();
                }
                let Ok(button) = mouse_button_code_convert(button) else {
                    return self.error(NuhxBoardError::UnknownButton(button));
                };

                self.pressed_mouse_buttons.insert(button, Instant::now());
                if let Some(i) = self.caches_by_mouse_button.get(&button) {
                    self.caches[*i].clear();
                }
                captured_key = Some(button);
            }
            rdevin::EventType::ButtonRelease(button) => {
                debug!(?button, "Button released");
                let Ok(button_num) = mouse_button_code_convert(button) else {
                    return self.error(NuhxBoardError::UnknownButton(button));
                };
                if button == rdevin::Button::Unknown(6) || button == rdevin::Button::Unknown(7) {
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
                    < self.settings.min_press_time.into()
                {
                    return Task::perform(
                        Timer::after(
                            Duration::from_millis(self.settings.min_press_time)
                                - self
                                    .pressed_mouse_buttons
                                    .get(&button_num)
                                    .unwrap()
                                    .elapsed(),
                        ),
                        move |_| Message::button_release(button),
                    );
                }
                debug!("Disabling button highlight");
                self.pressed_mouse_buttons.remove(&button_num);
                if let Some(i) = self.caches_by_mouse_button.get(&button_num) {
                    self.caches[*i].clear();
                }
            }
            rdevin::EventType::Wheel { delta_x, delta_y } => {
                debug!("Wheel moved: ({delta_x}, {delta_y})");
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
                captured_key = Some(button);

                out = Task::perform(
                    Timer::after(std::time::Duration::from_millis(
                        self.settings.scroll_hold_time,
                    )),
                    move |_| Message::ReleaseScroll(button),
                );
                if let Some(i) = self.caches_by_scroll_button.get(&button) {
                    self.caches[*i].clear();
                }
            }
            rdevin::EventType::MouseMove { x, y } => {
                debug!("Mouse moved: ({x}, {y})");
                let (x, y) = (x as f32, y as f32);

                let current_time = event.time;
                let time_diff = match current_time.duration_since(self.previous_mouse_time) {
                    Ok(diff) => diff,
                    Err(_) => return Task::none(),
                };
                if time_diff.as_millis() < 10 {
                    trace!("Mouse move event ignored due to time diff");
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
                trace!(?position_diff);

                self.mouse_velocity = (
                    position_diff.0 / time_diff.as_secs_f32(),
                    position_diff.1 / time_diff.as_secs_f32(),
                );
                self.previous_mouse_position = (x, y);
                self.previous_mouse_time = current_time;
                for i in &self.mouse_speed_indicator_caches {
                    self.caches[*i].clear();
                }
            }
        }

        if let Some(key) = captured_key {
            debug!(?key, "Key captured, updating layout def");
            for i in &self.detecting {
                let BoardElement::KeyboardKey(def) = &mut self.layout.elements[*i] else {
                    continue;
                };
                def.key_codes.push(key);
            }
            self.detecting.clear();
        }

        out
    }

    fn change_style(&mut self, style: StyleSetting) {
        match style {
            StyleSetting::DefaultMouseSpeedIndicatorOutlineWidth(width) => {
                self.style.default_mouse_speed_indicator_style.outline_width = width;
            }
            StyleSetting::DefaultLooseKeyFontFamily => {
                let new_font = self.text_input.default_loose_key_font_family.clone();
                if !FONTS.read().unwrap().contains(new_font.as_str()) {
                    FONTS.write().unwrap().insert(new_font.clone().leak());
                }
                self.style.default_key_style.loose.font.font_family = new_font;
            }
            StyleSetting::DefaultLooseKeyShowOutline => {
                self.style.default_key_style.loose.show_outline =
                    !self.style.default_key_style.loose.show_outline;
            }
            StyleSetting::DefaultLooseKeyOutlineWidth(width) => {
                self.style.default_key_style.loose.outline_width = width;
            }
            StyleSetting::DefaultLooseKeyBackgroundImage => {
                let image = self.text_input.default_loose_key_background_image.clone();
                self.style
                    .default_key_style
                    .loose
                    .background_image_file_name = if image.is_empty() { None } else { Some(image) };
            }
            StyleSetting::DefaultPressedKeyFontFamily => {
                let new_font = self.text_input.default_pressed_key_font_family.clone();
                if !FONTS.read().unwrap().contains(new_font.as_str()) {
                    FONTS.write().unwrap().insert(new_font.clone().leak());
                }
                self.style.default_key_style.pressed.font.font_family = new_font;
            }
            StyleSetting::DefaultPressedKeyShowOutline => {
                self.style.default_key_style.pressed.show_outline =
                    !self.style.default_key_style.pressed.show_outline;
            }
            StyleSetting::DefaultPressedKeyOutlineWidth(width) => {
                self.style.default_key_style.pressed.outline_width = width;
            }
            StyleSetting::DefaultPressedKeyBackgroundImage => {
                let image = self.text_input.default_pressed_key_background_image.clone();
                self.style
                    .default_key_style
                    .pressed
                    .background_image_file_name = if image.is_empty() { None } else { Some(image) };
            }
            StyleSetting::KeyboardBackgroundImage => {
                let image = self.text_input.keyboard_background_image.clone();
                self.change_background_image(Some(if image.is_empty() {
                    None
                } else {
                    Some(image)
                }));
            }
            StyleSetting::LooseKeyFontFamily(id) => {
                let new_font = self
                    .text_input
                    .loose_font_family
                    .get(&id)
                    .cloned()
                    .unwrap_or_default();
                if !FONTS.read().unwrap().contains(new_font.as_str()) {
                    FONTS.write().unwrap().insert(new_font.clone().leak());
                }
                key_style_change!(
                    self,
                    loose,
                    {
                        loose.font.font_family = new_font.clone();
                    },
                    id
                );
            }
            StyleSetting::LooseKeyShowOutline(id) => {
                key_style_change!(
                    self,
                    loose,
                    {
                        loose.show_outline = !loose.show_outline;
                    },
                    id
                );
            }
            StyleSetting::LooseKeyOutlineWidth { id, width } => {
                key_style_change!(
                    self,
                    loose,
                    {
                        loose.outline_width = width;
                    },
                    id
                );
            }
            StyleSetting::LooseKeyBackgroundImage(id) => {
                let image = self
                    .text_input
                    .loose_background_image
                    .get(&id)
                    .cloned()
                    .unwrap_or_default();
                key_style_change!(
                    self,
                    loose,
                    {
                        loose.background_image_file_name = if image.is_empty() {
                            None
                        } else {
                            Some(image.clone())
                        };
                    },
                    id
                );
            }
            StyleSetting::PressedKeyFontFamily(id) => {
                let new_font = self
                    .text_input
                    .pressed_font_family
                    .get(&id)
                    .cloned()
                    .unwrap_or_default();
                if !FONTS.read().unwrap().contains(new_font.as_str()) {
                    FONTS.write().unwrap().insert(new_font.clone().leak());
                }
                key_style_change!(
                    self,
                    pressed,
                    {
                        pressed.font.font_family = new_font.clone();
                    },
                    id
                );
            }
            StyleSetting::PressedKeyShowOutline(id) => {
                key_style_change!(
                    self,
                    pressed,
                    {
                        pressed.show_outline = !pressed.show_outline;
                    },
                    id
                );
            }
            StyleSetting::PressedKeyOutlineWidth { id, width } => {
                key_style_change!(
                    self,
                    pressed,
                    {
                        pressed.outline_width = width;
                    },
                    id
                );
            }
            StyleSetting::PressedKeyBackgroundImage(id) => {
                let image = self
                    .text_input
                    .pressed_background_image
                    .get(&id)
                    .cloned()
                    .unwrap_or_default();
                key_style_change!(
                    self,
                    pressed,
                    {
                        pressed.background_image_file_name = if image.is_empty() {
                            None
                        } else {
                            Some(image.clone())
                        };
                    },
                    id
                );
            }
            StyleSetting::LooseKeyFontStyle {
                id,
                style: font_style,
            } => {
                key_style_change!(
                    self,
                    loose,
                    {
                        loose.font.style.toggle(font_style);
                    },
                    id
                );
            }
            StyleSetting::PressedKeyFontStyle {
                id,
                style: font_style,
            } => {
                key_style_change!(
                    self,
                    pressed,
                    {
                        pressed.font.style.toggle(font_style);
                    },
                    id
                );
            }
            StyleSetting::MouseSpeedIndicatorOutlineWidth { id, width } => {
                let mut style = self.style.default_mouse_speed_indicator_style.clone();
                style.outline_width = width;
                self.style
                    .element_styles
                    .entry(id)
                    .and_modify(|v| {
                        let style::ElementStyle::MouseSpeedIndicatorStyle(ref mut key) = v else {
                            panic!()
                        };
                        key.outline_width = width;
                    })
                    .or_insert(style::ElementStyle::MouseSpeedIndicatorStyle(style));
            }
        }
    }

    fn clear_all_caches(&self) {
        for c in &self.caches {
            c.clear();
        }
    }

    fn clear_cache_by_id(&self, id: u32) {
        if let Some(i) = self.caches_by_id.get(&id) {
            self.caches[*i].clear();
        }
    }
}
