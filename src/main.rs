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

    let settings_path = home::home_dir()
        .unwrap()
        .join(".local/share/NuhxBoard/NuhxBoard.json");

    if !settings_path.exists() {
        std::fs::create_dir_all(
            home::home_dir()
                .unwrap()
                .join(".local/share/NuhxBoard/keyboards/global"),
        )?;
        let mut settings = File::create(
            home::home_dir()
                .unwrap()
                .join(".local/share/NuhxBoard/NuhxBoard.json"),
        )?;

        settings.write_all(serde_json::to_string_pretty(&Settings::default())?.as_bytes())?;
        match std::env::consts::OS {
            #[cfg(target_os = "linux")]
            "linux" => {
                let mut path = home::home_dir().unwrap();
                path.push(".local/share/");

                let res = reqwest::blocking::get("https://raw.githubusercontent.com/justDeeevin/NuhxBoard/main/nuhxboard.desktop")?;
                let desktop_entry = res.bytes()?;
                File::create(path.clone().join("applications/nuhxboard.desktop"))?
                    .write_all(&desktop_entry)?;

                File::create(path.join("NuhxBoard/NuhxBoard.png"))?.write_all(IMAGE)?;
            }
            #[cfg(target_os = "windows")]
            "windows" => {
                let mut lnk_path = home::home_dir().unwrap();
                lnk_path
                    .push("AppData/Roaming/Microsoft/Windows/Start Menu/Programs/NuhxBoard.lnk");

                let lnk = lnk_path.to_str().unwrap();

                let target_path = std::env::current_exe()?;

                let target = target_path.to_str().unwrap();

                let sl = mslnk::ShellLink::new(target)?;
                sl.create_lnk(lnk)?;
            }
            "macos" => {
                eprintln!("Sorry, the install command isn't implemented for MacOS yet.");
                std::process::exit(1);
            }
            _ => {
                eprintln!("Sorry, the install command isn't implemented for your OS yet. If there isn't a GitHub issue open for your OS, open one!");
                std::process::exit(1);
            }
        }
    }

    let settings_file = File::open(settings_path)?;

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
