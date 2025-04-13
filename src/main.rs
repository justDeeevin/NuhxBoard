#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod message;
mod nuhxboard;
mod types;
mod ui;

use color_eyre::eyre::ContextCompat;
use nuhxboard::*;
use std::{
    fs::{self, File},
    io::{self, prelude::*},
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let config_path = KEYBOARDS_PATH.parent().wrap_err("Config exists in root?")?;

    if !KEYBOARDS_PATH.exists() {
        fs::create_dir_all(&*KEYBOARDS_PATH)?;
    } else if !KEYBOARDS_PATH.is_dir() {
        fs::remove_file(&*KEYBOARDS_PATH)?;
        fs::create_dir_all(&*KEYBOARDS_PATH)?;
    }

    if fs::read_dir(&*KEYBOARDS_PATH)?.count() == 0 {
        let res = reqwest::blocking::get(
            "https://raw.githubusercontent.com/justdeeevin/nuhxboard/main/keyboards.zip",
        )?;

        let mut keyboards_file = File::create_new(config_path.join("keyboards.zip"))?;

        keyboards_file.write_all(&res.bytes()?)?;

        let mut keyboards_archive = zip::ZipArchive::new(keyboards_file).unwrap();

        for i in 0..keyboards_archive.len() {
            let mut file = keyboards_archive.by_index(i).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => KEYBOARDS_PATH.join(path),
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

        fs::remove_file(config_path.join("keyboards.zip"))?;
    }

    let global_path = KEYBOARDS_PATH.join("global");

    if !global_path.exists() {
        fs::create_dir_all(&global_path)?;
    }

    // Runs the app, initializing state using NuhxBoard::new
    iced::daemon(NuhxBoard::title, NuhxBoard::update, NuhxBoard::view)
        .theme(NuhxBoard::theme)
        .subscription(NuhxBoard::subscription)
        .font(iced_fonts::REQUIRED_FONT_BYTES)
        .run_with(NuhxBoard::new)?;

    Ok(())
}
