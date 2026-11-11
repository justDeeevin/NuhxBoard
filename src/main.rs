#![allow(clippy::await_holding_refcell_ref)]

use anyhow::{Result, ensure};
use async_compression::futures::bufread::GzipDecoder;
use async_net::TcpStream;
use async_tar::{Archive, EntryType};
use futures_rustls::{TlsConnector, pki_types::ServerName, rustls::ClientConfig};
use futures_util::{AsyncReadExt, StreamExt, TryStreamExt, io::copy};
use http_body_util::{BodyDataStream, BodyExt, Empty};
use hyper::{Request, Uri, body::Bytes, client::conn::http1};
use json::{PathSegment, layout::Layout, settings::Settings, style::Style};
use rustls_platform_verifier::BuilderVerifierExt;
use slint::{Image, SharedString, ToSharedString, Weak};
use smol_hyper::rt::FuturesIo;
use std::{
    cell::RefCell,
    fmt::Display,
    io::ErrorKind,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, LazyLock},
};

slint::include_modules!();

mod json;

static NUHXBOARD_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    confy::get_configuration_file_path("NuhxBoard", None)
        .expect("Failed to get config path")
        .parent()
        .expect("Config file has no parent?")
        .into()
});

fn main() {
    let app = Main::new().unwrap();
    app.on_open_settings(|| {
        let _ = SettingsPopup::new().and_then(|popup| popup.show());
    });
    let settings = Rc::new(RefCell::new(Settings::default()));

    let weak = app.as_weak();
    let settings_clone = settings.clone();
    slint::spawn_local(async move {
        setup(weak.clone(), settings_clone).await;
        weak.unwrap().set_loading(false);
    })
    .unwrap();

    app.run().unwrap();
}

async fn setup(app: Weak<Main>, settings: Rc<RefCell<Settings>>) {
    let res = get_keyboards().await;
    if let Err(e) = res {
        ErrorPopup::pop(e);
    }
    match load_settings().await {
        Ok(s) => *settings.borrow_mut() = s,
        Err(e) => ErrorPopup::pop(e),
    }
    app.unwrap()
        .set_t(settings.borrow().window_title.to_shared_string());
    app.unwrap().global::<Loaded>().set_category(
        settings
            .borrow()
            .category
            .as_deref()
            .cloned()
            .map(Into::into)
            .unwrap_or_default(),
    );
    let Some(category) = &settings.borrow().category else {
        return;
    };
    match load_category(category).await {
        Ok(options) => app
            .unwrap()
            .global::<Loaded>()
            .set_layouts(options.as_slice().into()),
        Err(e) => {
            ErrorPopup::pop(e);
            return;
        }
    }
    let Some(layout) = &settings.borrow().layout else {
        return;
    };
    match load_layout(category, layout).await {
        Ok(layout) => app.unwrap().apply_layout(&layout),
        Err(e) => ErrorPopup::pop(e),
    }
    let Some(style_name) = &settings.borrow().style else {
        return;
    };
    let source = if settings.borrow().global_style {
        StyleSource::Global
    } else {
        StyleSource::Local { category, layout }
    };
    match load_style(source, style_name).await {
        Ok(style) => app.unwrap().apply_style(source, style_name, &style),
        Err(e) => ErrorPopup::pop(e),
    }
}

impl Main {
    fn apply_layout(&self, layout: &Layout) {
        self.set_w(layout.width);
        self.set_h(layout.height);
    }

    fn apply_style(&self, source: StyleSource<'_, '_>, style_name: &PathSegment, style: &Style) {
        self.set_background_color(style.background_color.into());
        let images_path = match source {
            StyleSource::Global => format!("global/{style_name}"),
            StyleSource::Local { category, layout } => format!("{category}/{layout}/images"),
        };
        if let Some(image) = &style.background_image_file_name {
            if let Ok(i) = Image::load_from_path(
                &NUHXBOARD_PATH
                    .join("keyboards")
                    .join(images_path)
                    .join(image),
            ) {
                self.set_background_image(i);
            } else {
                ErrorPopup::pop("Failed to load background image.")
            }
        }
    }
}

#[derive(Clone, Copy)]
enum StyleSource<'a, 'b> {
    Global,
    Local {
        category: &'a PathSegment,
        layout: &'b PathSegment,
    },
}

async fn load_style(source: StyleSource<'_, '_>, style: &PathSegment) -> Result<Style> {
    let parent = match source {
        StyleSource::Local { category, layout } => format!("{category}/{layout}"),
        StyleSource::Global => "global".into(),
    };

    let string = async_fs::read_to_string(
        NUHXBOARD_PATH
            .join("keyboards")
            .join(parent)
            .join(format!("{style}.style")),
    )
    .await?;
    Ok(serde_json::from_str(&string)?)
}

impl ErrorPopup {
    fn with_message(message: impl Display) -> Result<Self, slint::PlatformError> {
        let out = Self::new()?;
        out.set_message(message.to_shared_string());
        Ok(out)
    }

    #[track_caller]
    fn pop(message: impl Display) {
        eprintln!("Error at {}", std::panic::Location::caller());
        let _ = Self::with_message(message).and_then(|window| window.show());
    }
}

async fn load_layout(category: &PathSegment, layout: &PathSegment) -> Result<Layout> {
    let string = async_fs::read_to_string(
        NUHXBOARD_PATH
            .join("keyboards")
            .join(category)
            .join(layout)
            .join("keyboard.json"),
    )
    .await?;
    Ok(serde_json::from_str(&string)?)
}

async fn load_category(category: &PathSegment) -> Result<Vec<SharedString>> {
    Ok(
        async_fs::read_dir(NUHXBOARD_PATH.join("keyboards").join(category))
            .await?
            .map_ok(|e| e.path().file_name().unwrap().display().to_shared_string())
            .try_filter(|name| std::future::ready(name != "global"))
            .try_collect()
            .await?,
    )
}

async fn load_settings() -> Result<Settings> {
    let path = NUHXBOARD_PATH.join("settings.json");
    let mut file = match async_fs::File::open(&path).await {
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                async_fs::write(path, serde_json::to_string(&Settings::default())?).await?;
                return Ok(Settings::default());
            } else {
                return Err(e.into());
            }
        }
        Ok(f) => f,
    };

    let mut string = String::default();
    file.read_to_string(&mut string).await?;

    Ok(serde_json::from_str(&string)?)
}

async fn get_keyboards() -> Result<()> {
    let keyboards_path = NUHXBOARD_PATH.join("keyboards");
    if exists(&keyboards_path).await? {
        return Ok(());
    }

    let uri: Uri =
        "https://raw.githubusercontent.com/justDeeevin/NuhxBoard/refs/heads/v2/keyboards.tar.gz"
            .parse()
            .unwrap();
    let host = uri.host().unwrap();

    let tcp = TcpStream::connect(format!("{host}:{}", uri.port_u16().unwrap_or(443))).await?;
    let connector = TlsConnector::from(Arc::new(
        ClientConfig::builder()
            .with_platform_verifier()?
            .with_no_client_auth(),
    ));
    let stream = connector
        .connect(ServerName::try_from(host.to_owned())?, tcp)
        .await?;

    let (mut sender, conn) = http1::handshake(FuturesIo::new(stream)).await?;
    slint::spawn_local(conn)?;

    let req = Request::builder()
        .header(hyper::header::HOST, uri.authority().unwrap().as_str())
        .uri(uri)
        .body(Empty::<Bytes>::default())?;

    let res = sender.send_request(req).await?;
    ensure!(res.status().is_success(), "Download returned error code");

    let body =
        BodyDataStream::new(res.into_body().map_err(std::io::Error::other)).into_async_read();
    let mut entries = Archive::new(GzipDecoder::new(body)).entries()?;

    while let Some(r) = entries.next().await {
        let entry = r?;
        let path = keyboards_path.join::<&Path>(entry.path()?.as_ref().as_ref());
        match entry.header().entry_type() {
            EntryType::Directory => async_fs::create_dir_all(path).await,
            EntryType::Symlink => {
                let link_name = entry.link_name()?.unwrap();
                async_fs::unix::symlink::<&Path, _>(link_name.as_ref().as_ref(), path).await
            }
            _ => {
                if let Some(parent) = path.parent()
                    && !exists(parent).await?
                {
                    async_fs::create_dir_all(parent).await?;
                }
                let mut file = async_fs::File::create(path).await?;
                copy(entry, &mut file).await.map(|_| ())
            }
        }?;
    }

    Ok(())
}

async fn exists(path: impl AsRef<Path>) -> std::io::Result<bool> {
    match async_fs::metadata(path).await {
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                Ok(false)
            } else {
                Err(e)
            }
        }
        Ok(_) => Ok(true),
    }
}
