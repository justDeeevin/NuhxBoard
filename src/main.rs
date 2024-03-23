#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logic;
mod nuhxboard;
mod types;
mod ui;

use color_eyre::eyre::Result;
use iced::{multi_window::Application, window};
use nuhxboard::*;
use std::{fs::File, io::prelude::*};
use types::settings::Settings;

static IMAGE: &[u8] = include_bytes!("../NuhxBoard.png");

fn main() -> Result<()> {
    color_eyre::install()?;

    let icon_image = image::load_from_memory(IMAGE)?;

    let nuhxboard_path = home::home_dir().unwrap().join(".local/share/NuhxBoard");

    if !nuhxboard_path.exists() {
        std::fs::create_dir_all(nuhxboard_path.clone())?;
    }

    match std::env::consts::OS {
        "linux" => {
            let apps_path = home::home_dir().unwrap().join(".local/share/applications");

            if !apps_path.join("nuhxboard.desktop").exists() {
                let res = reqwest::blocking::get(
                    "https://raw.githubusercontent.com/justDeeevin/NuhxBoard/main/nuhxboard.desktop",
                )?;
                let desktop_entry = res.bytes()?;
                File::create(apps_path.clone().join("applications/nuhxboard.desktop"))?
                    .write_all(&desktop_entry)?;

                File::create(nuhxboard_path.join("NuhxBoard.png"))?.write_all(IMAGE)?;
            }
        }
        // cfg necessary b/c lnk uses windows-only code
        #[cfg(target_os = "windows")]
        "windows" => {
            let lnk_path = home::home_dir()
                .unwrap()
                .join("AppData/Roaming/Microsoft/Windows/Start Menu/Programs/NuhxBoard.lnk");

            if !lnk_path.exists() {
                let lnk = lnk_path.to_str().unwrap();

                let target_path = std::env::current_exe()?;

                let target = target_path.to_str().unwrap();

                let sl = mslnk::ShellLink::new(target)?;
                sl.create_lnk(lnk)?;
            }
        }
        _ => {}
    }
    if !nuhxboard_path.join("NuhxBoard.json").exists() {
        let mut settings = File::create(nuhxboard_path.join("NuhxBoard.json"))?;
        settings.write_all(serde_json::to_string_pretty(&Settings::default())?.as_bytes())?;
    }

    let settings_file = File::open(nuhxboard_path)?;

    let settings: Settings = serde_json::from_reader(settings_file)?;

    let icon = window::icon::from_rgba(icon_image.to_rgba8().to_vec(), 256, 256)?;
    let flags = Flags { settings };

    let window_settings = iced::Settings {
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
    NuhxBoard::run(window_settings)?;

    Ok(())
}
