#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod canvas;
mod code_convert;
mod listener;
mod nuhxboard;
mod types;

use clap::Parser;
use color_eyre::eyre::Result;
use iced::{multi_window::Application, window};
use nuhxboard::*;
use std::{fs::File, io::prelude::*};
use types::settings::Settings;

/// NuhxBoard - The cross-platform alternative to NohBoard
#[derive(Parser, Debug)]
#[command(
    version,
    after_help = "Add keyboard categorys to ~/.local/share/NuhxBoard/keyboards/"
)]
struct Args {
    /// Display debug info
    #[arg(short, long)]
    verbose: bool,

    /// Install the app to your system; Create a desktop entry and install the icon.
    #[arg(long)]
    install: bool,
}

static IMAGE: &[u8] = include_bytes!("../NuhxBoard.png");

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let icon_image = image::load_from_memory(IMAGE)?;

    if args.install {
        std::fs::create_dir_all(
            home::home_dir()
                .unwrap()
                .join(".local/share/NuhxBoard/keyboards"),
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

                let mut target_path = home::home_dir().unwrap();
                target_path.push(".cargo/bin/nuhxboard.exe");

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

        println!("NuhxBoard installed successfully!");

        return Ok(());
    }

    let settings_file = File::open(
        home::home_dir()
            .unwrap()
            .join(".local/share/NuhxBoard/NuhxBoard.json"),
    )?;

    let settings: Settings = serde_json::from_reader(settings_file)?;

    let icon = window::icon::from_rgba(icon_image.to_rgba8().to_vec(), 256, 256)?;
    let flags = Flags {
        verbose: args.verbose,
        settings,
    };

    let settings = iced::Settings {
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
    NuhxBoard::run(settings)?;

    Ok(())
}
