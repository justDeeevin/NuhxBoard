#![allow(clippy::await_holding_refcell_ref)]

use anyhow::{Result, ensure};
use async_compression::futures::bufread::GzipDecoder;
use async_net::TcpStream;
use async_tar::{Archive, EntryType};
use futures_rustls::{TlsConnector, pki_types::ServerName, rustls::ClientConfig};
use futures_util::{AsyncReadExt, StreamExt, TryFutureExt, TryStreamExt, io::copy};
use http_body_util::{BodyDataStream, BodyExt, Empty};
use hyper::{Request, Uri, body::Bytes, client::conn::http1};
use json::{
    PathSegment,
    layout::{CommonElement, Layout},
    settings::Settings,
    style::Style,
};
use rustls_platform_verifier::BuilderVerifierExt;
use slint::{
    Image, LogicalPosition, SharedString, ToSharedString, Weak, language::StandardListViewItem,
};
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
    let settings = Rc::new(RefCell::new(Settings::default()));
    let categories = Rc::new(RefCell::new(Vec::default()));
    let layouts = Rc::new(RefCell::new(Vec::default()));
    let local_styles = Rc::new(RefCell::new(Vec::default()));
    let global_styles = Rc::new(RefCell::new(Vec::default()));

    let app = Main::new().unwrap();

    app.set_version(env!("CARGO_PKG_VERSION").to_shared_string());

    let weak = app.as_weak();
    let settings_clone = settings.clone();
    app.on_open_settings(move || {
        let Ok(popup) = SettingsPopup::new() else {
            return;
        };

        popup.set_settings((&*settings_clone.borrow()).into());

        let app = weak.clone();
        let settings = settings_clone.clone();
        popup.on_submit(move |user_settings| {
            app.unwrap().set_t(user_settings.window_title.clone());
            if let Err(es) = settings.borrow_mut().apply(user_settings) {
                for e in es {
                    ErrorPopup::pop(format!("Bad integer value: {e}"));
                }
            }

            // TODO: save settings?
        });

        let _ = popup.show();
    });

    let settings_clone = settings.clone();
    let categories_clone = categories.clone();
    let layouts_clone = layouts.clone();
    let local_styles_clone = local_styles.clone();
    let global_styles_clone = global_styles.clone();
    let weak = app.as_weak();
    app.on_open_load(move || {
        let Ok(popup) = LoadKeyboard::new() else {
            return;
        };

        let settings = settings_clone.clone();
        let layouts = layouts_clone.clone();
        let weak_popup = popup.as_weak();
        let app = weak.clone();
        popup.on_category_changed(move |category| {
            settings.borrow_mut().category = Some(PathSegment::new(category.clone()).unwrap());
            app.unwrap().set_loading(true);

            let layouts = layouts.clone();
            let popup = weak_popup.clone();
            let app = app.clone();
            slint::spawn_local(async move {
                match load_category(&PathSegment::new(category).unwrap()).await {
                    Ok(options) => {
                        let items: Vec<_> = options.into_iter().map(Into::into).collect();
                        popup.unwrap().set_layouts(items.as_slice().into());
                        *layouts.borrow_mut() = items;
                    }
                    Err(e) => ErrorPopup::pop(format!("Failed to load category: {e}")),
                }
                app.unwrap().set_loading(false);
            })
            .unwrap();
        });

        let local_styles = local_styles_clone.clone();
        let settings = settings_clone.clone();
        let app = weak.clone();
        let weak_popup = popup.as_weak();
        popup.on_layout_changed(move |layout| {
            settings.borrow_mut().layout = Some(PathSegment::new(layout.clone()).unwrap());
            app.unwrap().set_loading(true);

            let local_styles = local_styles.clone();
            let settings = settings.clone();
            let app = app.clone();
            let popup = weak_popup.clone();
            slint::spawn_local(async move {
                match load_layout(
                    settings.borrow().category.as_ref().unwrap(),
                    &PathSegment::new(layout).unwrap(),
                )
                .await
                {
                    Ok((layout, options)) => {
                        app.unwrap().apply_layout(&layout);
                        let items: Vec<_> = options.into_iter().map(Into::into).collect();
                        popup.unwrap().set_local_styles(items.as_slice().into());
                        *local_styles.borrow_mut() = items;
                    }
                    Err(e) => ErrorPopup::pop(format!("Failed to load layout: {e}")),
                }
                app.unwrap().set_loading(false);
            })
            .unwrap();
        });

        let settings = settings_clone.clone();
        let app = weak.clone();
        popup.on_style_changed(move |kind, name| {
            settings.borrow_mut().global_style = kind == StyleKind::Global;
            settings.borrow_mut().style = Some(PathSegment::new(name.clone()).unwrap());
            app.unwrap().set_loading(true);

            let settings = settings.clone();
            let app = app.clone();
            slint::spawn_local(async move {
                let settings = settings.borrow();
                let source = match kind {
                    StyleKind::Global => StyleSource::Global,
                    StyleKind::Local => StyleSource::Local {
                        category: settings.category.as_ref().unwrap(),
                        layout: settings.layout.as_ref().unwrap(),
                    },
                };
                let name = PathSegment::new(name).unwrap();
                match load_style(source, &name).await {
                    Ok(style) => app.unwrap().apply_style(source, &name, &style),
                    Err(e) => ErrorPopup::pop(format!("Failed to load style: {e}")),
                }
                app.unwrap().set_loading(false);
            })
            .unwrap();
        });

        popup.set_global_styles(global_styles_clone.borrow().as_slice().into());

        popup.set_categories(categories_clone.borrow().as_slice().into());
        if let Some(category) = &settings_clone.borrow().category {
            popup.set_category(category.to_shared_string());
        }

        let mut layout_index = -1;
        popup.set_layouts(
            layouts_clone
                .borrow()
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    if settings_clone
                        .borrow()
                        .layout
                        .as_ref()
                        .map(ToSharedString::to_shared_string)
                        == Some(name.text.clone())
                    {
                        layout_index = index as i32;
                    }
                    name.clone()
                })
                .collect::<Vec<_>>()
                .as_slice()
                .into(),
        );
        popup.set_layout(layout_index);

        let mut style_index = -1;
        popup.set_local_styles(
            local_styles_clone
                .borrow()
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    if settings_clone.borrow().global_style
                        && settings_clone
                            .borrow()
                            .style
                            .as_ref()
                            .map(ToSharedString::to_shared_string)
                            == Some(name.text.clone())
                    {
                        style_index = index as i32;
                    }
                    name.clone()
                })
                .collect::<Vec<_>>()
                .as_slice()
                .into(),
        );
        popup.set_local_style(style_index);

        let _ = popup.show();
    });

    let weak = app.as_weak();
    let settings_clone = settings.clone();
    let categories_clone = categories.clone();
    let layouts_clone = layouts.clone();
    let local_styles_clone = local_styles.clone();
    let global_styles_clone = global_styles.clone();
    slint::spawn_local(async move {
        setup(
            weak.clone(),
            settings_clone,
            categories_clone,
            layouts_clone,
            local_styles_clone,
            global_styles_clone,
        )
        .await;
        weak.unwrap().set_loading(false);
    })
    .unwrap();

    app.run().unwrap();
}

async fn setup(
    app: Weak<Main>,
    settings: Rc<RefCell<Settings>>,
    categories: Rc<RefCell<Vec<SharedString>>>,
    layouts: Rc<RefCell<Vec<StandardListViewItem>>>,
    local_styles: Rc<RefCell<Vec<StandardListViewItem>>>,
    global_styles: Rc<RefCell<Vec<StandardListViewItem>>>,
) {
    let res = get_keyboards().await;
    if let Err(e) = res {
        ErrorPopup::pop(format!("Failed to download stock keyboards: {e}"));
    }
    match load_settings().await {
        Ok(s) => *settings.borrow_mut() = s,
        Err(e) => ErrorPopup::pop(format!("Failed to load settings: {e}")),
    }
    app.unwrap()
        .set_t(settings.borrow().window_title.to_shared_string());
    match async_fs::read_dir(NUHXBOARD_PATH.join("keyboards"))
        .and_then(|entries| {
            entries
                .map_ok(|e| e.file_name().display().to_shared_string())
                .try_collect::<Vec<_>>()
        })
        .await
    {
        Ok(options) => *categories.borrow_mut() = options,
        Err(e) => ErrorPopup::pop(format!("Failed to get categories: {e}")),
    }
    match async_fs::read_dir(NUHXBOARD_PATH.join("keyboards/global"))
        .and_then(|entries| {
            entries
                .try_filter_map(|entry| {
                    std::future::ready(Ok(entry
                        .file_name()
                        .display()
                        .to_string()
                        .strip_suffix(".style")
                        .map(Into::into)))
                })
                .try_collect::<Vec<_>>()
        })
        .await
    {
        Ok(options) => *global_styles.borrow_mut() = options,
        Err(e) => ErrorPopup::pop(format!("Failed to get global styles: {e}")),
    }

    let Some(category) = &settings.borrow().category else {
        return;
    };
    match load_category(category).await {
        Ok(options) => *layouts.borrow_mut() = options.into_iter().map(Into::into).collect(),
        Err(e) => {
            ErrorPopup::pop(format!("Failed to load category: {e}"));
            return;
        }
    }
    let Some(layout) = &settings.borrow().layout else {
        return;
    };
    match load_layout(category, layout).await {
        Ok((layout, options)) => {
            app.unwrap().apply_layout(&layout);
            *local_styles.borrow_mut() = options.into_iter().map(Into::into).collect();
        }
        Err(e) => ErrorPopup::pop(format!("Failed to load layout: {e}")),
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
        Err(e) => ErrorPopup::pop(format!("Failed to load style: {e}")),
    }
}

impl Main {
    fn apply_layout(&self, layout: &Layout) {
        self.set_w(layout.width);
        self.set_h(layout.height);

        let mut keys = Vec::default();
        let mut speedometers = Vec::new();

        for element in &layout.elements {
            match <&CommonElement>::try_from(&element.kind) {
                Ok(def) => {
                    let mut commands = format!("M {} {}", def.boundaries[0].x, def.boundaries[0].y)
                        .to_shared_string();
                    for point in &def.boundaries[1..] {
                        commands += &format!("L {} {}", point.x, point.y);
                    }
                    keys.push(Key {
                        commands,
                        text: def.text.to_shared_string(),
                        text_position: LogicalPosition {
                            x: def.text_position.x,
                            y: def.text_position.y,
                        },
                        ..Default::default()
                    });
                }
                Err(def) => speedometers.push(Speedometer {
                    position: LogicalPosition {
                        x: def.location.x,
                        y: def.location.y,
                    },
                    radius: def.radius,
                }),
            }
        }

        self.set_keys(keys.as_slice().into());
        self.set_speedometers(speedometers.as_slice().into());
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

async fn load_layout(
    category: &PathSegment,
    layout: &PathSegment,
) -> Result<(Layout, Vec<SharedString>)> {
    let string = async_fs::read_to_string(
        NUHXBOARD_PATH
            .join("keyboards")
            .join(category)
            .join(layout)
            .join("keyboard.json"),
    )
    .await?;
    let mut styles = Vec::default();
    match async_fs::read_dir(NUHXBOARD_PATH.join("keyboards").join(category).join(layout))
        .and_then(|entries| {
            entries
                .try_filter_map(|entry| {
                    std::future::ready(Ok(entry
                        .file_name()
                        .display()
                        .to_string()
                        .strip_suffix(".style")
                        .map(ToSharedString::to_shared_string)))
                })
                .try_collect::<Vec<_>>()
        })
        .await
    {
        Ok(options) => styles.extend(options),
        Err(e) => ErrorPopup::pop(format!("Failed to get layouts: {e}")),
    }
    Ok((serde_json::from_str(&string)?, styles))
}

async fn load_category(category: &PathSegment) -> Result<Vec<SharedString>> {
    Ok(
        async_fs::read_dir(NUHXBOARD_PATH.join("keyboards").join(category))
            .await?
            .map_ok(|e| e.file_name().display().to_shared_string())
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
    let conn = slint::spawn_local(conn)?;

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

    conn.abort();

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
