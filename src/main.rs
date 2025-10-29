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
use color_eyre::eyre::{Context, eyre};
use nuhxboard::*;
use tracing::{Level, debug, debug_span, info};
use tracing_subscriber::{filter, prelude::*};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    iced_tracing: bool,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    if !args.iced_tracing {
        let registry = tracing_subscriber::registry().with(tracing_subscriber::fmt::layer());
        let level = std::env::var("RUST_LOG").unwrap_or_default();
        let filter =
            filter::Targets::new().with_target("nuhxboard", level.parse().unwrap_or(Level::INFO));
        registry.with(filter).init();
    } else {
        tracing_subscriber::fmt::init();
    }

    let config_path = KEYBOARDS_PATH
        .parent()
        .ok_or_else(|| eyre!("Config lives at root?"))?;

    if !KEYBOARDS_PATH.exists() {
        fs::create_dir_all(&*KEYBOARDS_PATH).context("Failed to create config directory")?;
    } else if !KEYBOARDS_PATH.is_dir() {
        info!("Config directory exists but is not a directory. Removing and recreating");
        fs::remove_file(&*KEYBOARDS_PATH).context("Failed to remove file at config path")?;
        fs::create_dir_all(&*KEYBOARDS_PATH).context("Failed to create config directory")?;
    }

    if fs::read_dir(&*KEYBOARDS_PATH)?.count() == 0 {
        info!("Downloading sample keyboards");
        let res = reqwest::blocking::get(
            "https://raw.githubusercontent.com/justdeeevin/nuhxboard/main/keyboards.zip",
        )
        .context("Failed to download sample keyboards")?;

        let mut keyboards_file = File::create_new(config_path.join("keyboards.zip"))
            .context("Failed to create keyboards.zip")?;

        keyboards_file
            .write_all(
                &res.bytes()
                    .context("Failed to get bytes for keyboards.zip")?,
            )
            .context("Failed to write keyboards.zip")?;

        let mut keyboards_archive =
            zip::ZipArchive::new(keyboards_file).context("Failed to load keyboards.zip")?;

        info!("Extracting sample keyboards");
        let span = debug_span!("unzip");
        let _guard = span.enter();
        let len = keyboards_archive.len();
        for i in 1..=len {
            let mut file = keyboards_archive
                .by_index(i - 1)
                .with_context(|| format!("Failed to get file #{i} from zip"))?;
            let outpath = match file.enclosed_name() {
                Some(path) => KEYBOARDS_PATH.join(path),
                None => continue,
            };
            debug!("{} ({i}/{len})", outpath.display());

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath)
                    .with_context(|| format!("Failed to create directory {outpath:?}"))?;
            } else {
                if let Some(p) = outpath.parent()
                    && !p.exists()
                {
                    fs::create_dir_all(p)
                        .with_context(|| format!("Failed to create directory {p:?}"))?;
                }
                let mut outfile = File::create(&outpath)
                    .with_context(|| format!("Failed to create file {outpath:?}"))?;
                io::copy(&mut file, &mut outfile)
                    .with_context(|| format!("Failed to populate file {outpath:?} from zip"))?;
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                        .with_context(|| format!("Failed to set permissions on {outpath:?}"))?;
                }
            }
        }

        fs::remove_file(config_path.join("keyboards.zip"))
            .context("Failed to remove keyboards.zip")?;
    }

    let global_path = KEYBOARDS_PATH.join("global");

    if !global_path.exists() {
        fs::create_dir_all(&global_path).context("Failed to create global theme directory")?;
    }

    // Runs the app, initializing state using NuhxBoard::new
    iced::daemon(NuhxBoard::new, NuhxBoard::update, NuhxBoard::view)
        .title(NuhxBoard::title)
        .theme(NuhxBoard::theme)
        .subscription(NuhxBoard::subscription)
        .font(iced_aw::ICED_AW_FONT_BYTES)
        .run()?;

    Ok(())
}
