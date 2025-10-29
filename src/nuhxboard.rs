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
use nalgebra::Vector2;
use nuhxboard_logic::{listener::RdevinSubscriber, mouse_button_code_convert};
use nuhxboard_types::{
    layout::*,
    settings::*,
    style::{self, *},
};
use rdevin::keycodes::windows::code_from_key as win_keycode_from_key;
use smol::Timer;
use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, File},
    path::PathBuf,
    rc::Rc,
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};
use tracing::{debug, info, info_span, instrument, trace};

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

pub static KEYBOARDS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    confy::get_configuration_file_path("NuhxBoard", None)
        .unwrap()
        .parent()
        .unwrap()
        .join("keyboards")
});

// TODO: selectively clear fg or bg depending on the context
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

    pub fn new_rc() -> Rc<Self> {
        Rc::new(Self {
            fg: Cache::new(),
            bg: Cache::new(),
        })
    }
}

pub struct NuhxBoard {
    pub windows: WindowManager<Self, Theme, Message>,
    pub main_window: window::Id,
    pub caches: Vec<Rc<ElementCache>>,
    pub caches_by_keycode: HashMap<u32, Rc<ElementCache>>,
    pub caches_by_mouse_button: HashMap<u32, Rc<ElementCache>>,
    pub caches_by_scroll_button: HashMap<u32, Rc<ElementCache>>,
    pub caches_by_id: HashMap<u32, Rc<ElementCache>>,
    pub mouse_speed_indicator_caches: HashMap<u32, Rc<ElementCache>>,
    pub layout: Layout,
    pub style: Style,
    /// `{[keycode]: [time_pressed]}`
    pub pressed_keys: HashMap<u32, Instant>,
    /// `{[keycode]: [time_pressed]}`
    pub pressed_mouse_buttons: HashMap<u32, Instant>,
    /// `{[axis]: [releases_queued]}`
    pub pressed_scroll_buttons: HashMap<u32, u32>,
    pub mouse_velocity: Vector2<f32>,
    pub previous_mouse_position: Coord<f32>,
    pub previous_mouse_time: std::time::SystemTime,
    pub caps: bool,
    pub true_caps: bool,
    pub layout_choice: Option<usize>,
    pub style_choice: usize,
    pub layout_options: Vec<String>,
    pub keyboard_category_options: Vec<String>,
    pub style_options: Vec<StyleChoice>,
    pub startup: bool,
    pub settings: Settings,
    pub display_options: Vec<DisplayInfo>,
    pub edit_mode: bool,
    pub edit_history: Vec<Change>,
    pub history_depth: usize,
    pub save_keyboard_as_category: String,
    pub save_layout_as_name: String,
    pub save_style_as_name: String,
    pub save_style_as_global: bool,
    pub color_pickers: ColorPickers,
    pub text_input: TextInput,
    pub number_input: NumberInput,
    pub selections: SelectionLists,
    pub hovered_element: Option<usize>,
    pub detecting: Vec<usize>,
    pub right_click_pos: iced::Point,
    pub mouse_pos: iced::Point,
    pub layout_commited: bool,
    pub style_commited: bool,
}

const DEFAULT_KEY_SIZE: f32 = 43.0;

pub const DEFAULT_WINDOW_SIZE: iced::Size = iced::Size {
    width: 200.0,
    height: 200.0,
};

impl NuhxBoard {
    pub fn new() -> (Self, Task<Message>) {
        let span = info_span!("startup");
        let _guard = span.enter();
        let mut settings_error = None;

        info!("Loading settings");
        let settings: Settings = confy::load("NuhxBoard", None).unwrap_or_else(|e| {
            settings_error = Some(NuhxBoardError::SettingsParse(Arc::new(e)));
            Settings::default()
        });

        let layout = Layout {
            version: None,
            width: DEFAULT_WINDOW_SIZE.width,
            height: DEFAULT_WINDOW_SIZE.height,
            elements: Vec::new(),
        };

        let category = settings.category.clone();
        let keyboard = settings.layout_index;
        let style = settings.style;

        let caps = match settings.capitalization {
            Capitalization::Upper => true,
            Capitalization::Lower => false,
            Capitalization::Follow => false,
        };

        let mut windows = WindowManager::default();

        // The app will open the main window on startup. The WindowManager automatically tracks IDs
        // and corresponding window types and runs the correct view, theme, and title logic when
        // necessary.
        let (main_window, window_open_task) = windows.open(Box::new(Main));

        let mut app = Self {
            windows,
            main_window,
            caches: Vec::new(),
            caches_by_keycode: HashMap::new(),
            caches_by_mouse_button: HashMap::new(),
            caches_by_scroll_button: HashMap::new(),
            caches_by_id: HashMap::new(),
            mouse_speed_indicator_caches: HashMap::new(),
            layout,
            style: Style::default(),
            pressed_keys: HashMap::new(),
            pressed_mouse_buttons: HashMap::new(),
            caps,
            true_caps: false,
            mouse_velocity: Vector2::zeros(),
            pressed_scroll_buttons: HashMap::new(),
            previous_mouse_position: Coord::zero(),
            previous_mouse_time: std::time::SystemTime::now(),
            layout_choice: Some(settings.layout_index),
            style_choice: settings.style,
            layout_options: Vec::new(),
            keyboard_category_options: Vec::new(),
            style_options: Vec::new(),
            startup: false,
            settings,
            display_options: DisplayInfo::all().unwrap(),
            edit_mode: false,
            edit_history: Vec::new(),
            history_depth: 0,
            save_keyboard_as_category: String::new(),
            save_layout_as_name: String::new(),
            save_style_as_name: String::new(),
            save_style_as_global: false,
            color_pickers: ColorPickers::default(),
            text_input: TextInput::default(),
            hovered_element: None,
            number_input: NumberInput::default(),
            selections: SelectionLists::default(),
            detecting: Vec::new(),
            right_click_pos: iced::Point::default(),
            mouse_pos: iced::Point::default(),
            layout_commited: true,
            style_commited: true,
        };

        let mut tasks = Vec::with_capacity(5);
        tasks.push(window_open_task.map(|_| Message::None));
        if !category.is_empty() {
            tasks.extend([
                app.update(Message::ChangeKeyboardCategory(category)),
                app.update(Message::LoadLayout(keyboard)),
                app.update(Message::LoadStyle(style)),
            ]);
        }
        if let Some(error) = settings_error {
            tasks.push(app.error(error));
        }

        app.startup = false;

        (app, Task::batch(tasks))
    }

    #[instrument(skip_all)]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Listener(event) => {
                return self.input_event(event);
            }
            Message::None => {}
            Message::ReleaseScroll(button) => {
                debug!(button, "Scroll release");
                match self.pressed_scroll_buttons.get_mut(&button) {
                    None => {}
                    Some(1) => {
                        debug!("Disabling scroll highlight");
                        self.pressed_scroll_buttons.remove(&button);
                        self.caches_by_scroll_button
                            .entry(button)
                            .and_modify(|c| c.clear());
                    }
                    Some(n) => {
                        *n -= 1;
                    }
                }
            }
            Message::ChangeKeyboardCategory(category) => {
                info!(category, "Keyboard category changed");
                assert!(!category.is_empty());
                self.settings.category = category.clone();

                self.save_keyboard_as_category = category;

                if !self.startup {
                    self.layout_choice = None;
                    self.settings.layout_index = 0;
                    self.style_choice = 0;
                    self.settings.style = 0;
                    self.style_options = Vec::new();
                }

                self.layout_options = fs::read_dir(KEYBOARDS_PATH.join(&self.settings.category))
                    .unwrap()
                    .map(|r| r.unwrap())
                    .filter_map(|entry| {
                        if entry.file_type().unwrap().is_dir() && entry.file_name() != "images" {
                            Some(entry.file_name().to_str().unwrap().to_owned())
                        } else {
                            None
                        }
                    })
                    .collect();
                self.layout_options.sort();
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
                    self.layout_options[self.layout_choice.unwrap()]
                )));
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                let mut file = File::create(path).unwrap();
                serde_json::to_writer_pretty(&mut file, &self.layout).unwrap();
                self.layout_commited = true;
            }
            Message::SaveStyle(file) => {
                info!(?file, "Saving style");
                let path = file.unwrap_or(KEYBOARDS_PATH.join(format!(
                    "{}/{}/{}.style",
                    self.settings.category,
                    self.layout_options[self.layout_choice.unwrap()],
                    self.style_options[self.style_choice]
                )));
                let mut file = File::create(path).unwrap();
                serde_json::to_writer_pretty(&mut file, &self.style).unwrap();
                self.style_commited = true;
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
                self.layout_commited = false;
            }
            Message::Undo => {
                debug!("Undo");
                if self.history_depth < self.edit_history.len() {
                    self.history_depth += 1;
                    self.apply_change(
                        self.edit_history[self.edit_history.len() - self.history_depth].clone(),
                        true,
                    );
                }
            }
            Message::Redo => {
                debug!("Redo");
                if self.history_depth > 0 {
                    self.history_depth -= 1;
                    self.apply_change(
                        self.edit_history[self.edit_history.len() - self.history_depth - 1].clone(),
                        false,
                    );
                }
            }
            Message::ChangeTextInput(input, value) => {
                debug!(?input, value, "Changing text input");
                match input {
                    TextInputType::SaveStyleAsName => self.save_style_as_name = value,
                    TextInputType::SaveKeyboardAsName => {
                        self.save_layout_as_name = value;
                    }
                    TextInputType::SaveKeyboardAsCategory => {
                        self.save_keyboard_as_category = value;
                    }
                    TextInputType::KeyboardBackgroundImage => {
                        self.text_input.keyboard_background_image = value;
                    }
                    TextInputType::DefaultLooseKeyBackgroundImage => {
                        self.text_input.default_loose_key_background_image = value;
                    }
                    TextInputType::DefaultLooseKeyFontFamily => {
                        self.text_input.default_loose_key_font_family = value;
                    }
                    TextInputType::DefaultPressedKeyBackgroundImage => {
                        self.text_input.default_pressed_key_background_image = value;
                    }
                    TextInputType::DefaultPressedKeyFontFamily => {
                        self.text_input.default_pressed_key_font_family = value;
                    }
                    TextInputType::LooseBackgroundImage(id) => {
                        self.text_input.loose_background_image.insert(id, value);
                    }
                    TextInputType::LooseFontFamily(id) => {
                        self.text_input.loose_font_family.insert(id, value);
                    }
                    TextInputType::PressedBackgroundImage(id) => {
                        self.text_input.pressed_background_image.insert(id, value);
                    }
                    TextInputType::PressedFontFamily(id) => {
                        self.text_input.pressed_font_family.insert(id, value);
                    }
                }
            }
            Message::ChangeStyle(style) => {
                debug!(?style, "Changing style");
                self.change_style(style);
            }
            Message::ToggleSaveStyleAsGlobal => {
                debug!(
                    save_style_as_global = !self.save_style_as_global,
                    "Toggling save style as global"
                );
                self.save_style_as_global = !self.save_style_as_global;
            }
            Message::Open(window) => {
                info!(id = window.id(), "Opening new window");
                if window == LoadKeyboard {
                    if (!self.layout_commited || !self.style_commited)
                        && !self
                            .windows
                            .any_of(&UnsavedChangesPopup(Action::LoadKeyboard))
                    {
                        return self
                            .windows
                            .open(Box::new(UnsavedChangesPopup(Action::LoadKeyboard)))
                            .1
                            .map(|_| Message::None);
                    }
                    self.keyboard_category_options = fs::read_dir(&*KEYBOARDS_PATH)
                        .unwrap()
                        .filter_map(|r| {
                            let entry = r.unwrap();
                            if entry.file_type().unwrap().is_dir() && entry.file_name() != "global"
                            {
                                Some(entry.file_name().to_str().unwrap().to_owned())
                            } else {
                                None
                            }
                        })
                        .collect();
                    self.keyboard_category_options.sort();
                } else if window == SaveStyleAs {
                    self.save_style_as_global = self.style_options[self.style_choice].is_global();
                }
                return self.windows.open(window).1.map(|_| Message::None);
            }
            Message::CloseAllOf(window) => {
                info!(id = window.id(), "Closing all windows");
                return self.windows.close_all_of(window).map(|_| Message::None);
            }
            Message::Exit => {
                info!("Exiting");
                return immediate_task(Message::CloseRequested);
            }
            Message::Closed(window) => {
                info!(%window, "Window closed");
                self.windows.was_closed(window);

                if self.windows.empty() {
                    return iced::exit();
                }
            }
            Message::CloseRequested => {
                if (!self.layout_commited || !self.style_commited)
                    && !self.windows.any_of(&UnsavedChangesPopup(Action::Exit))
                {
                    return self
                        .windows
                        .open(Box::new(UnsavedChangesPopup(Action::Exit)))
                        .1
                        .map(|_| Message::None);
                }
                confy::store("nuhxboard", None, self.settings.clone()).unwrap();
                if !self.windows.empty() {
                    return self.windows.close_all().map(|_| Message::None);
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
                self.style_commited = false;
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
                let mut handled = true;
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
                            let mut set = BTreeSet::from_iter(def.key_codes.clone());
                            if let Some(v) = v {
                                if mouse_key {
                                    set.clear();
                                }
                                set.insert(v);
                            } else {
                                set.remove(&def.key_codes[i]);
                                self.selections.keycode.remove(&element_i);
                            }
                            *def.key_codes = set.into_iter().collect();
                        }
                        _ => handled = false,
                    }
                } else {
                    handled = false;
                }
                if !handled {
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
                self.layout_commited = false;
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
                self.layout_commited = false;
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
                self.layout_commited = false;
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
                debug!(index = i, "Clearing cache");
                self.caches[i].clear();
            }
            Message::ClearAllCaches => {
                debug!("Clearing all caches");
                self.clear_all_caches();
            }
            Message::AddKeyboardKey => {
                debug!("Adding keyboard key");
                let common = self.new_def();
                let cache = self.new_cache(common.id);
                self.caches_by_keycode
                    .extend(common.key_codes.iter().map(|c| (*c, cache.clone())));

                self.layout
                    .elements
                    .push(BoardElement::KeyboardKey(KeyboardKeyDefinition {
                        id: common.id,
                        boundaries: common.boundaries,
                        text_position: common.text_position,
                        key_codes: common.key_codes,
                        text: common.text,
                        shift_text: String::new(),
                        change_on_caps: false,
                    }));
                self.layout_commited = false;
            }
            Message::AddMouseKey => {
                debug!("Adding mouse key");
                let mut common = self.new_def();
                common.key_codes.push(0);
                let cache = self.new_cache(common.id);
                self.caches_by_mouse_button
                    .extend(common.key_codes.iter().map(|c| (*c, cache.clone())));

                self.layout.elements.push(BoardElement::MouseKey(common));
                self.layout_commited = false;
            }
            Message::AddMouseScroll => {
                debug!("Adding mouse scroll");
                let common = self.new_def();
                let cache = self.new_cache(common.id);
                self.caches_by_scroll_button
                    .extend(common.key_codes.iter().map(|c| (*c, cache.clone())));
                self.layout.elements.push(BoardElement::MouseScroll(common));
                self.layout_commited = false;
            }
            Message::AddMouseSpeedIndicator => {
                debug!("Adding mouse speed indicator");
                let def = MouseSpeedIndicatorDefinition {
                    id: self.new_id(),
                    location: self.right_click_pos.into(),
                    radius: 20.0,
                };
                let cache = self.new_cache(def.id);
                self.mouse_speed_indicator_caches.insert(def.id, cache);
                self.layout
                    .elements
                    .push(BoardElement::MouseSpeedIndicator(def));
                self.layout_commited = false;
            }
            Message::RightClick(window) => {
                debug!(%window, "Right click");
                if window == self.main_window {
                    self.right_click_pos = self.mouse_pos;
                }
            }
            Message::MouseMoved {
                position,
                window_id,
            } => {
                trace!(?window_id, ?position, "Mouse moved");
                if window_id == self.main_window {
                    self.mouse_pos = position;
                }
            }
            Message::RemoveElement => {
                debug!("Removing element");
                let Some(i) = self.hovered_element.take() else {
                    return Task::none();
                };
                let element = &self.layout.elements[i];
                match element {
                    BoardElement::KeyboardKey(def) => {
                        for code in &def.key_codes {
                            self.caches_by_keycode.remove(code);
                        }
                    }
                    BoardElement::MouseKey(def) => {
                        for code in &def.key_codes {
                            self.caches_by_mouse_button.remove(code);
                        }
                    }
                    BoardElement::MouseScroll(def) => {
                        for code in &def.key_codes {
                            self.caches_by_scroll_button.remove(code);
                        }
                    }
                    BoardElement::MouseSpeedIndicator(def) => {
                        self.mouse_speed_indicator_caches.remove(&def.id);
                    }
                }
                self.caches_by_id.remove(&element.id());
                self.caches.remove(i);
                self.layout.elements.remove(i);
                self.layout_commited = false;
            }
            Message::Commit(action) => {
                self.layout_commited = true;
                self.style_commited = true;
                match action {
                    Action::LoadKeyboard => {
                        return Task::batch([
                            immediate_task(Message::Open(Box::new(LoadKeyboard))),
                            self.windows
                                .close_all_of(Box::new(UnsavedChangesPopup(action)))
                                .map(|_| Message::None),
                        ]);
                    }
                    Action::Exit => {
                        return immediate_task(Message::Exit);
                    }
                }
            }
            Message::CancelDiscard(action) => {
                return self
                    .windows
                    .close_all_of(Box::new(UnsavedChangesPopup(action)))
                    .map(|_| Message::None);
            }
        }
        Task::none()
    }

    fn new_id(&self) -> u32 {
        self.layout
            .elements
            .iter()
            .map(|e| e.id())
            .max()
            .unwrap_or_default()
            + 1
    }

    fn new_def(&self) -> CommonDefinition {
        CommonDefinition {
            id: self.new_id(),
            text: String::new(),
            boundaries: vec![
                SerializablePoint {
                    x: (self.right_click_pos.x - DEFAULT_KEY_SIZE / 2.0).into(),
                    y: (self.right_click_pos.y - DEFAULT_KEY_SIZE / 2.0).into(),
                },
                SerializablePoint {
                    x: (self.right_click_pos.x + DEFAULT_KEY_SIZE / 2.0).into(),
                    y: (self.right_click_pos.y - DEFAULT_KEY_SIZE / 2.0).into(),
                },
                SerializablePoint {
                    x: (self.right_click_pos.x + DEFAULT_KEY_SIZE / 2.0).into(),
                    y: (self.right_click_pos.y + DEFAULT_KEY_SIZE / 2.0).into(),
                },
                SerializablePoint {
                    x: (self.right_click_pos.x - DEFAULT_KEY_SIZE / 2.0).into(),
                    y: (self.right_click_pos.y + DEFAULT_KEY_SIZE / 2.0).into(),
                },
            ],
            text_position: self.right_click_pos.into(),
            key_codes: Vec::new(),
        }
    }

    fn new_cache(&mut self, id: u32) -> Rc<ElementCache> {
        let cache = ElementCache::new_rc();
        self.caches.push(cache.clone());
        self.caches_by_id.insert(id, cache.clone());
        cache
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
            iced::window::close_requests().map(|_| Message::CloseRequested),
            iced::event::listen_with(|e, _, id| match e {
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Right,
                )) => Some(Message::RightClick(id)),
                iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                    Some(Message::MouseMoved {
                        position,
                        window_id: id,
                    })
                }
                _ => None,
            }),
        ])
    }

    fn error(&mut self, error: NuhxBoardError) -> iced::Task<Message> {
        let (_, command) = self.windows.open(Box::new(ErrorPopup { error }));
        command.map(|_| Message::None)
    }

    fn load_layout(&mut self, index: usize) -> Task<Message> {
        if index >= self.layout_options.len() {
            return self.error(NuhxBoardError::LayoutOpen(Arc::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Provided layout index is out of bounds",
            ))));
        }

        self.edit_mode = false;
        self.settings.layout_index = index;

        self.layout_choice = Some(index);
        self.style = Style::default();

        self.save_layout_as_name = self.layout_options[index].clone();

        let layout_file = match File::open(
            KEYBOARDS_PATH
                .join(&self.settings.category)
                .join(&self.layout_options[index])
                .join("keyboard.json"),
        ) {
            Ok(file) => file,
            Err(e) => {
                return self.error(NuhxBoardError::LayoutOpen(Arc::new(e)));
            }
        };

        self.layout = match serde_json::from_reader(layout_file) {
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
        for e in &self.layout.elements {
            let cache = ElementCache::new_rc();
            self.caches.push(cache.clone());
            self.caches_by_id.insert(e.id(), cache.clone());

            match e {
                BoardElement::KeyboardKey(def) => {
                    self.caches_by_keycode
                        .extend(def.key_codes.iter().map(|c| (*c, cache.clone())));
                }
                BoardElement::MouseKey(def) => {
                    self.caches_by_mouse_button
                        .extend(def.key_codes.iter().map(|c| (*c, cache.clone())));
                }
                BoardElement::MouseScroll(def) => {
                    self.caches_by_scroll_button
                        .extend(def.key_codes.iter().map(|c| (*c, cache.clone())));
                }
                BoardElement::MouseSpeedIndicator(def) => {
                    self.mouse_speed_indicator_caches.insert(def.id, cache);
                }
            }
        }

        self.style_options = vec![StyleChoice::Default];
        self.style_options.append(
            &mut fs::read_dir(
                KEYBOARDS_PATH
                    .join(&self.settings.category)
                    .join(&self.layout_options[index]),
            )
            .unwrap()
            .filter_map(|r| {
                let entry = r.unwrap();
                if entry.file_type().unwrap().is_file()
                    && entry.path().extension() == Some(std::ffi::OsStr::new("style"))
                {
                    Some(StyleChoice::Custom(
                        entry
                            .path()
                            .file_stem()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned(),
                    ))
                } else {
                    None
                }
            })
            .collect(),
        );
        self.style_options.append(
            &mut fs::read_dir(KEYBOARDS_PATH.join("global"))
                .unwrap()
                .filter_map(|r| {
                    let entry = r.unwrap();
                    if entry.file_type().unwrap().is_file()
                        && entry.path().extension() == Some(std::ffi::OsStr::new("style"))
                    {
                        Some(StyleChoice::Custom(
                            entry
                                .path()
                                .file_stem()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_owned(),
                        ))
                    } else {
                        None
                    }
                })
                .collect(),
        );
        self.style_choice = 0;

        window::resize(
            self.main_window,
            iced::Size {
                width: self.layout.width,
                height: self.layout.height,
            },
        )
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
        if style >= self.style_options.len() {
            return self.error(NuhxBoardError::StyleOpen(Arc::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Provided style index is out of bounds",
            ))));
        }

        self.settings.style = style;

        self.style_choice = style;

        if self.style_options[style] == StyleChoice::Default {
            self.style = Style::default();
        } else {
            let path = KEYBOARDS_PATH.join(match &self.style_options[style] {
                StyleChoice::Default => unreachable!(),
                StyleChoice::Global(style_name) => {
                    format!("global/{style_name}.style")
                }
                StyleChoice::Custom(style_name) => format!(
                    "{}/{}/{}.style",
                    self.settings.category,
                    self.layout_options[self.layout_choice.unwrap()],
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

        // Refreshes the background image
        self.change_background_image(None);

        self.save_style_as_name = self.style_options[style].name();

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

    #[instrument(level = "trace", skip_all, fields(event = ?event.event_type))]
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
                let Some(keycode) = win_keycode_from_key(key) else {
                    return self.error(NuhxBoardError::UnknownKey(key));
                };
                self.pressed_keys.insert(keycode, Instant::now());
                if let Some(cache) = self.caches_by_keycode.get(&keycode) {
                    cache.clear();
                }
                if !self.detecting.is_empty() {
                    captured_key = Some(keycode);
                }
            }
            rdevin::EventType::KeyRelease(key) => {
                debug!(?key, "Key released");
                let Some(keycode) = win_keycode_from_key(key) else {
                    return self.error(NuhxBoardError::UnknownKey(key));
                };
                let Some(elapsed) = self.pressed_keys.get(&keycode).map(Instant::elapsed) else {
                    return Task::none();
                };
                if elapsed.as_millis() < self.settings.min_press_time.into() {
                    return Task::perform(
                        Timer::after(Duration::from_millis(self.settings.min_press_time) - elapsed),
                        move |_| Message::key_release(key),
                    );
                }
                debug!("Disabling key highlight");
                self.pressed_keys.remove(&keycode);
                if let Some(cache) = self.caches_by_keycode.get(&keycode) {
                    cache.clear();
                }
            }
            rdevin::EventType::ButtonPress(button) => {
                debug!(?button, "Button pressed");
                // Scroll wheel
                if button == rdevin::Button::Unknown(6) || button == rdevin::Button::Unknown(7) {
                    return Task::none();
                }
                let Ok(button_code) = mouse_button_code_convert(button) else {
                    return self.error(NuhxBoardError::UnknownButton(button));
                };

                self.pressed_mouse_buttons
                    .insert(button_code, Instant::now());
                if let Some(cache) = self.caches_by_mouse_button.get(&button_code) {
                    cache.clear();
                }
                if !self.detecting.is_empty() {
                    captured_key = Some(button_code);
                }
            }
            rdevin::EventType::ButtonRelease(button) => {
                debug!(?button, "Button released");
                let Ok(button_code) = mouse_button_code_convert(button) else {
                    return self.error(NuhxBoardError::UnknownButton(button));
                };
                // Scroll wheel
                if button == rdevin::Button::Unknown(6) || button == rdevin::Button::Unknown(7) {
                    return Task::none();
                }
                let Some(elapsed) = self
                    .pressed_mouse_buttons
                    .get(&button_code)
                    .map(Instant::elapsed)
                else {
                    return Task::none();
                };
                if elapsed.as_millis() < self.settings.min_press_time.into() {
                    return Task::perform(
                        Timer::after(Duration::from_millis(self.settings.min_press_time) - elapsed),
                        move |_| Message::button_release(button),
                    );
                }
                debug!("Disabling button highlight");
                self.pressed_mouse_buttons.remove(&button_code);
                if let Some(cache) = self.caches_by_mouse_button.get(&button_code) {
                    cache.clear();
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
                if !self.detecting.is_empty() {
                    captured_key = Some(button);
                }

                out = Task::perform(
                    Timer::after(std::time::Duration::from_millis(
                        self.settings.scroll_hold_time,
                    )),
                    move |_| Message::ReleaseScroll(button),
                );
                self.caches_by_scroll_button
                    .entry(button)
                    .and_modify(|c| c.clear());
            }
            rdevin::EventType::MouseMove { x, y } => {
                trace!("Mouse moved");
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
                        let mut center = None;

                        for display in &self.display_options {
                            if display.id == self.settings.display_choice.id {
                                center = Some(Coord {
                                    x: display.x as f32 + (display.width as f32 / 2.0),
                                    y: display.y as f32 + (display.height as f32 / 2.0),
                                });
                                break;
                            }
                        }
                        center.expect("No display found with selected ID")
                    }
                    false => self.previous_mouse_position,
                };

                let position_diff = (x - previous_pos.x, y - previous_pos.y);
                trace!(?position_diff);

                self.mouse_velocity = Vector2::new(
                    position_diff.0 / time_diff.as_secs_f32(),
                    position_diff.1 / time_diff.as_secs_f32(),
                );
                self.previous_mouse_position = Coord { x, y };
                self.previous_mouse_time = current_time;
                trace!("Clearing mouse speed indicator caches");
                for cache in self.mouse_speed_indicator_caches.values() {
                    cache.clear();
                }
            }
        }

        if let Some(key) = captured_key {
            debug!(?key, "Key captured, updating layout def");
            for i in &self.detecting {
                self.number_input.keycode.insert(*i, key);
            }
            self.detecting.clear();
        }

        out
    }

    fn change_style(&mut self, style: StyleSetting) {
        match style {
            StyleSetting::DefaultMouseSpeedIndicatorOutlineWidth(width) => {
                self.style.default_mouse_speed_indicator_style.outline_width = width;
                self.mouse_speed_indicator_caches
                    .values()
                    .for_each(|c| c.clear());
            }
            StyleSetting::DefaultLooseKeyFontFamily => {
                let new_font = self.text_input.default_loose_key_font_family.clone();
                self.style.default_key_style.loose.font.font_family = new_font;
                self.clear_all_caches();
            }
            StyleSetting::DefaultLooseKeyShowOutline => {
                self.style.default_key_style.loose.show_outline =
                    !self.style.default_key_style.loose.show_outline;
                self.clear_all_caches();
            }
            StyleSetting::DefaultLooseKeyOutlineWidth(width) => {
                self.style.default_key_style.loose.outline_width = width;
                self.clear_all_caches();
            }
            StyleSetting::DefaultLooseKeyBackgroundImage => {
                let image = self.text_input.default_loose_key_background_image.clone();
                self.style
                    .default_key_style
                    .loose
                    .background_image_file_name = if image.is_empty() { None } else { Some(image) };
                self.clear_all_caches();
            }
            StyleSetting::DefaultPressedKeyFontFamily => {
                let new_font = self.text_input.default_pressed_key_font_family.clone();
                self.style.default_key_style.pressed.font.font_family = new_font;
                self.clear_all_caches();
            }
            StyleSetting::DefaultPressedKeyShowOutline => {
                self.style.default_key_style.pressed.show_outline =
                    !self.style.default_key_style.pressed.show_outline;
                self.clear_all_caches();
            }
            StyleSetting::DefaultPressedKeyOutlineWidth(width) => {
                self.style.default_key_style.pressed.outline_width = width;
                self.clear_all_caches();
            }
            StyleSetting::DefaultPressedKeyBackgroundImage => {
                let image = self.text_input.default_pressed_key_background_image.clone();
                self.style
                    .default_key_style
                    .pressed
                    .background_image_file_name = if image.is_empty() { None } else { Some(image) };
                self.clear_all_caches();
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
                key_style_change!(
                    self,
                    loose,
                    {
                        loose.font.font_family = new_font.clone();
                    },
                    id
                );
                self.clear_cache_by_id(id);
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
                self.clear_cache_by_id(id);
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
                self.clear_cache_by_id(id);
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
                self.clear_cache_by_id(id);
            }
            StyleSetting::PressedKeyFontFamily(id) => {
                let new_font = self
                    .text_input
                    .pressed_font_family
                    .get(&id)
                    .cloned()
                    .unwrap_or_default();
                key_style_change!(
                    self,
                    pressed,
                    {
                        pressed.font.font_family = new_font.clone();
                    },
                    id
                );
                self.clear_cache_by_id(id);
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
                self.clear_cache_by_id(id);
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
                self.clear_cache_by_id(id);
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
                self.clear_cache_by_id(id);
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
                self.clear_cache_by_id(id);
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
                self.clear_cache_by_id(id);
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
                self.clear_cache_by_id(id);
            }
        }
        self.style_commited = false;
    }

    fn clear_all_caches(&self) {
        for c in &self.caches {
            c.clear();
        }
    }

    fn clear_cache_by_id(&self, id: u32) {
        if let Some(cache) = self.caches_by_id.get(&id) {
            cache.clear();
        }
    }

    fn apply_change(&mut self, change: Change, undo: bool) {
        let signum = if undo { -1.0 } else { 1.0 };
        let clear_index = match change {
            Change::MoveElement { index, delta } => {
                self.layout.elements[index]
                    .translate(delta * signum, self.settings.update_text_position);
                index
            }
            Change::MoveFace { index, face, delta } => {
                match CommonDefinitionMut::try_from(&mut self.layout.elements[index]) {
                    Ok(mut def) => {
                        def.translate_face(face, delta * signum);
                    }
                    Err(def) => {
                        def.radius += delta.x * signum;
                    }
                }
                self.caches[index].clear();
                index
            }
            Change::MoveVertex {
                index,
                vertex,
                delta,
            } => {
                let def = CommonDefinitionMut::try_from(&mut self.layout.elements[index]).unwrap();
                def.boundaries[vertex] += delta * signum;
                self.caches[index].clear();
                index
            }
        };
        self.caches[clear_index].clear();
    }
}

fn immediate_task(message: Message) -> Task<Message> {
    Task::perform(std::future::ready(message), |m| m)
}
