#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod message;
mod nuhxboard;
mod types;
mod ui;

use std::{
    fs::{self, File},
    io::{self, prelude::*},
};

use clap::Parser;
use color_eyre::eyre::ContextCompat;
use nuhxboard::*;
use tracing::{debug, info, Level};
use tracing_subscriber::{filter, prelude::*};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    iced_tracing: bool,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let registry = tracing_subscriber::registry().with(tracing_subscriber::fmt::layer());
    if !args.iced_tracing {
        let level = std::env::var("RUST_LOG").unwrap_or("INFO".to_owned());
        let filter =
            filter::Targets::new().with_target("nuhxboard", level.parse().unwrap_or(Level::INFO));
        registry.with(filter).init();
    } else {
        registry.init();
    }

    let config_path = KEYBOARDS_PATH.parent().wrap_err("Config exists in root?")?;

    if !KEYBOARDS_PATH.exists() {
        fs::create_dir_all(&*KEYBOARDS_PATH)?;
    } else if !KEYBOARDS_PATH.is_dir() {
        fs::remove_file(&*KEYBOARDS_PATH)?;
        fs::create_dir_all(&*KEYBOARDS_PATH)?;
    }

    if fs::read_dir(&*KEYBOARDS_PATH)?.count() == 0 {
        info!("Downloading sample keyboards");
        let res = reqwest::blocking::get(
            "https://raw.githubusercontent.com/justdeeevin/nuhxboard/main/keyboards.zip",
        )?;

        let mut keyboards_file = File::create_new(config_path.join("keyboards.zip"))?;

        keyboards_file.write_all(&res.bytes()?)?;

        let mut keyboards_archive = zip::ZipArchive::new(keyboards_file).unwrap();

        info!("Extracting sample keyboards");
        let len = keyboards_archive.len();
        for i in 1..=len {
            let mut file = keyboards_archive.by_index(i - 1).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => KEYBOARDS_PATH.join(path),
                None => continue,
            };
            debug!("{} ({i}/{len})", outpath.display());

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
