#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logic;
mod nuhxboard;
mod types;
mod ui;

use iced::{multi_window::Application, window};
use iced_multi_window::multi_window;
use nuhxboard::*;
use std::{
    fs::{self, File},
    io::{self, prelude::*},
};
use types::settings::Settings;
use ui::app::*;

multi_window! {
    NuhxBoard,
    Main,
    SettingsWindow,
    LoadKeyboard,
    ErrorPopup,
    KeyboardProperties,
    SaveDefinitionAs,
    SaveStyleAs,
    KeyboardStyle,
}

static IMAGE: &[u8] = include_bytes!("../NuhxBoard.png");

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let nuhxboard_path = home::home_dir().unwrap().join(".local/share/NuhxBoard");

    if !nuhxboard_path.exists() {
        fs::create_dir_all(&nuhxboard_path)?;
    }

    let settings_path = nuhxboard_path.join("NuhxBoard.json");

    if !settings_path.exists() {
        let mut settings = File::create(&settings_path)?;
        settings.write_all(serde_json::to_string_pretty(&Settings::default())?.as_bytes())?;
    }

    let keyboards_path = nuhxboard_path.join("keyboards");

    if !keyboards_path.exists() {
        fs::create_dir_all(&keyboards_path)?;
    } else if !keyboards_path.is_dir() {
        fs::remove_file(&keyboards_path)?;
        fs::create_dir_all(&keyboards_path)?;
    }

    if fs::read_dir(&keyboards_path)?.count() == 0 {
        let res = reqwest::blocking::get(
            "https://raw.githubusercontent.com/justdeeevin/nuhxboard/main/keyboards.zip",
        )?;

        let mut keyboards_file = File::create_new(nuhxboard_path.join("keyboards.zip"))?;

        keyboards_file.write_all(&res.bytes()?)?;

        let mut keyboards_archive = zip::ZipArchive::new(keyboards_file).unwrap();

        for i in 0..keyboards_archive.len() {
            let mut file = keyboards_archive.by_index(i).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => keyboards_path.join(path),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath).unwrap();
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }

        fs::remove_file(nuhxboard_path.join("keyboards.zip"))?;
    }

    let global_path = keyboards_path.join("global");

    if !global_path.exists() {
        fs::create_dir_all(&global_path)?;
    }

    let settings_file = File::open(settings_path)?;

    let settings: Settings = serde_json::from_reader(settings_file)?;

    let icon_image = image::load_from_memory(IMAGE)?;
    let icon = window::icon::from_rgba(icon_image.to_rgba8().to_vec(), 256, 256)?;

    let window_settings = iced::Settings {
        window: window::Settings {
            size: DEFAULT_WINDOW_SIZE,
            resizable: false,
            icon: Some(icon),
            exit_on_close_request: false,
            ..window::Settings::default()
        },
        flags: settings,
        ..iced::Settings::default()
    };
    NuhxBoard::run(window_settings)?;

    Ok(())
}
