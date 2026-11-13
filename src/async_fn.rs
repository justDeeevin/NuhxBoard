use anyhow::{Result, ensure};
use async_compression::futures::bufread::GzipDecoder;
use async_net::TcpStream;
use async_tar::{Archive, EntryType};
use futures_rustls::{TlsConnector, pki_types::ServerName, rustls::ClientConfig};
use futures_util::{AsyncReadExt, StreamExt, TryFutureExt, TryStreamExt, io::copy};
use http_body_util::{BodyDataStream, BodyExt, Empty};
use hyper::{Request, Uri, body::Bytes, client::conn::http1};
use nuhxboard_json::{PathSegment, layout::Layout, settings::Settings, style::Style};
use nuhxboard_ui::{ErrorPopup, Main, NUHXBOARD_PATH, StyleSource};
use rustls_platform_verifier::BuilderVerifierExt;
use slint::{SharedString, ToSharedString, Weak, language::StandardListViewItem};
use smol_hyper::rt::FuturesIo;
use std::{cell::RefCell, io::ErrorKind, path::Path, rc::Rc, sync::Arc};

pub async fn setup(
    app: Weak<Main>,
    settings: Rc<RefCell<Settings>>,
    categories: Rc<RefCell<Vec<SharedString>>>,
    layouts: Rc<RefCell<Vec<StandardListViewItem>>>,
    local_styles: Rc<RefCell<Vec<StandardListViewItem>>>,
    global_styles: Rc<RefCell<Vec<StandardListViewItem>>>,
    style: Rc<RefCell<Option<Style>>>,
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
                .try_filter(|name| std::future::ready(name != "global"))
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
        Ok(new_style) => {
            app.unwrap().apply_style(source, style_name, &new_style);
            *style.borrow_mut() = Some(new_style);
        }
        Err(e) => ErrorPopup::pop(format!("Failed to load style: {e}")),
    }
}

pub async fn load_style(source: StyleSource<'_, '_>, style: &PathSegment) -> Result<Style> {
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

pub async fn load_layout(
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

pub async fn load_category(category: &PathSegment) -> Result<Vec<SharedString>> {
    Ok(
        async_fs::read_dir(NUHXBOARD_PATH.join("keyboards").join(category))
            .await?
            .map_ok(|e| e.file_name().display().to_shared_string())
            .try_filter(|name| std::future::ready(name != "images"))
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
