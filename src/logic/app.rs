use crate::{nuhxboard::*, types::style::*};
use iced::{window, Command};
use std::fs::{self, File};

impl NuhxBoard {
    pub fn load_keyboard(&mut self, keyboard: usize) -> Command<Message> {
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
            &mut fs::read_dir(self.keyboards_path.clone().join("global"))
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
            window::Id::MAIN,
            iced::Size {
                width: self.config.width,
                height: self.config.height,
            },
        )
    }
}
