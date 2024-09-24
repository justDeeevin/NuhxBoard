use crate::{
    logic::code_convert::*,
    nuhxboard::*,
    types::{settings::*, style::*},
};
use async_std::task::sleep;
use iced::{window, Task};
use image::ImageReader;
use std::{
    fs::{self, File},
    time::Instant,
};

impl NuhxBoard {
    pub fn load_layout(&mut self, keyboard: usize) -> Task<Message> {
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

    pub fn load_style(&mut self, style: usize) -> Task<Message> {
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

    pub fn input_event(&mut self, event: rdev::Event) -> Task<Message> {
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
