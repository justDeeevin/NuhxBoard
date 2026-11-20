// TODO:
// - clamda for async blocks?

use antmon::{Button, ButtonState, Event, key::convert::from_windows_vkcode};
use async_fn::*;
use clamda::lambda;
use futures_util::{Stream, StreamExt};
use nuhxboard_json::{
    PathSegment,
    layout::{CommonElement, ElementKind, Layout},
    settings::Settings,
    style::{ElementStyle, KeyStyle, Style},
};
use nuhxboard_ui::*;
use slint::{
    CloseRequestResponse, Image, JoinHandle, Model, SharedString, ToSharedString, Weak,
    language::StandardListViewItem,
};
use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

mod async_fn;
mod cli;

#[derive(Default)]
struct LoadPopupMemory {
    categories: Vec<SharedString>,
    layouts: Vec<StandardListViewItem>,
    local_styles: Vec<StandardListViewItem>,
    global_styles: Vec<StandardListViewItem>,
}

#[derive(Default)]
struct IdMaps {
    keys: HashMap<antmon::Key, Vec<u32>>,
    buttons: HashMap<Button, Vec<u32>>,
    up: Vec<u32>,
    right: Vec<u32>,
    down: Vec<u32>,
    left: Vec<u32>,
}

fn main() {
    let args = cli::parse();

    // TODO: save settings
    let settings = Rc::new(RefCell::new(Settings::default()));
    let style = Rc::new(RefCell::new(None::<Style>));

    let prev_cursor = Rc::new(RefCell::new(None));

    let memory = Rc::new(RefCell::new(LoadPopupMemory::default()));

    let app = Main::new().unwrap();
    let weak = app.as_weak();

    let key_indices = Rc::new(RefCell::new(HashMap::default()));
    let id_maps = Rc::new(RefCell::new(IdMaps::default()));

    let key_indices_clone = key_indices.clone();
    let id_maps_clone = id_maps.clone();
    let weak_clone = weak.clone();
    let style_clone = style.clone();
    let settings_clone = settings.clone();
    let prev_cursor_clone = prev_cursor.clone();
    slint::spawn_local(async move {
        let mut release_up_task = None::<JoinHandle<_>>;
        let mut release_right_task = None::<JoinHandle<_>>;
        let mut release_down_task = None::<JoinHandle<_>>;
        let mut release_left_task = None::<JoinHandle<_>>;

        let mut stream: Box<dyn Stream<Item = _> + Unpin> = if args.x11 {
            Box::new(antmon::linux::x11::events::<byor::runtime::Smol>().unwrap())
        } else {
            Box::new(antmon::events::<byor::runtime::Smol>().await.unwrap())
        };
        while let Some(Ok(event)) = stream.next().await {
            if let Some(style) = style_clone.borrow().as_ref() {
                match event {
                    Event::Scroll { dx, dy, .. } => {
                        if dx > 0.0 {
                            update_key(
                                ButtonState::Pressed,
                                &id_maps_clone.borrow().right,
                                key_indices_clone.clone(),
                                style_clone.borrow().as_ref().unwrap(),
                                settings_clone.clone(),
                                weak_clone.clone(),
                            );

                            if let Some(task) = release_right_task.replace(
                                slint::spawn_local(release_task(
                                    Direction::Right,
                                    id_maps_clone.clone(),
                                    key_indices_clone.clone(),
                                    style_clone.clone(),
                                    settings_clone.clone(),
                                    weak_clone.clone(),
                                ))
                                .unwrap(),
                            ) {
                                task.abort();
                            }
                        }
                        if dx < 0.0 {
                            update_key(
                                ButtonState::Pressed,
                                &id_maps_clone.borrow().left,
                                key_indices_clone.clone(),
                                style_clone.borrow().as_ref().unwrap(),
                                settings_clone.clone(),
                                weak_clone.clone(),
                            );

                            if let Some(task) = release_left_task.replace(
                                slint::spawn_local(release_task(
                                    Direction::Left,
                                    id_maps_clone.clone(),
                                    key_indices_clone.clone(),
                                    style_clone.clone(),
                                    settings_clone.clone(),
                                    weak_clone.clone(),
                                ))
                                .unwrap(),
                            ) {
                                task.abort();
                            }
                        }
                        if dy > 0.0 {
                            update_key(
                                ButtonState::Pressed,
                                &id_maps_clone.borrow().down,
                                key_indices_clone.clone(),
                                style_clone.borrow().as_ref().unwrap(),
                                settings_clone.clone(),
                                weak_clone.clone(),
                            );

                            if let Some(task) = release_down_task.replace(
                                slint::spawn_local(release_task(
                                    Direction::Down,
                                    id_maps_clone.clone(),
                                    key_indices_clone.clone(),
                                    style_clone.clone(),
                                    settings_clone.clone(),
                                    weak_clone.clone(),
                                ))
                                .unwrap(),
                            ) {
                                task.abort();
                            }
                        }
                        if dy < 0.0 {
                            update_key(
                                ButtonState::Pressed,
                                &id_maps_clone.borrow().up,
                                key_indices_clone.clone(),
                                style_clone.borrow().as_ref().unwrap(),
                                settings_clone.clone(),
                                weak_clone.clone(),
                            );

                            if let Some(task) = release_up_task.replace(
                                slint::spawn_local(release_task(
                                    Direction::Up,
                                    id_maps_clone.clone(),
                                    key_indices_clone.clone(),
                                    style_clone.clone(),
                                    settings_clone.clone(),
                                    weak_clone.clone(),
                                ))
                                .unwrap(),
                            ) {
                                task.abort();
                            }
                        }
                    }
                    Event::Motion { x, y } => {
                        if let Some((prev_x, prev_y)) = &*prev_cursor_clone.borrow() {
                            weak_clone.unwrap().set_dx(x - prev_x);
                            weak_clone.unwrap().set_dy(y - prev_y);
                        }
                        *prev_cursor_clone.borrow_mut() = Some((x, y));
                    }
                    Event::Button { button, state } => {
                        if let Some(ids) = id_maps_clone.borrow().buttons.get(&button) {
                            update_key(
                                state,
                                ids,
                                key_indices_clone.clone(),
                                style,
                                settings_clone.clone(),
                                weak_clone.clone(),
                            );
                        }
                    }
                    Event::Key { key, state, .. } => {
                        if let Some(ids) = id_maps_clone.borrow().keys.get(&key) {
                            if key == antmon::Key::LeftShift || key == antmon::Key::RightShift {
                                weak_clone.unwrap().set_shift(state == ButtonState::Pressed);
                            }
                            if key == antmon::Key::CapsLock && state == ButtonState::Pressed {
                                weak_clone
                                    .unwrap()
                                    .set_caps(!weak_clone.unwrap().get_caps());
                            }
                            update_key(
                                state,
                                ids,
                                key_indices_clone.clone(),
                                style,
                                settings_clone.clone(),
                                weak_clone.clone(),
                            )
                        }
                    }
                }
            }
        }
    })
    .unwrap();

    app.set_version(env!("CARGO_PKG_VERSION").to_shared_string());

    app.on_open_settings(lambda!(
        [clone, clone weak] || {
            let Ok(popup) = SettingsPopup::new() else {
                return;
            };
            popup.set_settings((&*settings.borrow()).into());

            popup.on_submit(lambda!(
                [clone] | user_settings | {
                    weak.unwrap().set_t(user_settings.window_title.clone());
                    if let Err(es) = user_settings.apply_to(&mut settings.borrow_mut()) {
                        for e in es {
                            ErrorPopup::pop(format!("Bad integer value: {e}"));
                        }
                    }
                }
            ));

            let _ = popup.show();
        }
    ));

    app.on_open_load(lambda!(
        [clone memory, clone settings, clone weak, clone key_indices, clone id_maps, clone style] || {
            let Ok(popup) = LoadKeyboard::new() else {
                return;
            };
            let weak_popup = popup.as_weak();

            popup.on_category_changed(lambda!(
                [clone] | category | {
                    settings.borrow_mut().category =
                        Some(PathSegment::new(category.clone()).unwrap());
                    weak.unwrap().set_loading(true);

                    let memory_clone = memory.clone();
                    let popup = weak_popup.clone();
                    let weak_clone = weak.clone();
                    slint::spawn_local(async move {
                        match load_category(&PathSegment::new(category).unwrap()).await {
                            Ok(options) => {
                                let items: Vec<_> = options.into_iter().map(Into::into).collect();
                                popup.unwrap().set_layouts(items.as_slice().into());
                                memory_clone.borrow_mut().layouts = items;
                            }
                            Err(e) => ErrorPopup::pop(format!("Failed to load category: {e}")),
                        }
                        weak_clone.unwrap().set_loading(false);
                    })
                    .unwrap();
                }
            ));

            popup.on_layout_changed(lambda!(
                [clone] | layout | {
                    settings.borrow_mut().layout =
                        Some(PathSegment::new(layout.clone()).unwrap());
                    weak.unwrap().set_loading(true);
                    if !settings.borrow().global_style {
                        weak_popup.unwrap().set_local_style(-1);
                    }

                    let memory_clone = memory.clone();
                    let settings_clone = settings.clone();
                    let weak_clone = weak.clone();
                    let popup = weak_popup.clone();
                    let style_clone = style.clone();
                    let key_indices_clone = key_indices.clone();
                    let id_maps_clone = id_maps.clone();
                    #[allow(
                        clippy::await_holding_refcell_ref,
                        reason = "false positive (see #6353)"
                    )]
                    slint::spawn_local(async move {
                        match load_layout(
                            settings_clone.borrow().category.as_ref().unwrap(),
                            &PathSegment::new(layout.clone()).unwrap(),
                        )
                        .await
                        {
                            Ok((layout, options)) => {
                                let key_indices = weak_clone.unwrap().apply_layout(&layout);
                                *key_indices_clone.borrow_mut() = key_indices;
                                id_maps_clone.borrow_mut().keys.clear();
                                id_maps_clone.borrow_mut().buttons.clear();
                                for element in &layout.elements {
                                    let Ok(def) = <&CommonElement>::try_from(&element.kind) else {
                                        continue;
                                    };
                                    if matches!(element.kind, ElementKind::MouseKey(_)) {
                                        for button in &def.key_codes {
                                        id_maps_clone.borrow_mut().
                                            buttons
                                                .entry(mouse_button_code_convert(*button))
                                                .and_modify(|ids| ids.push(element.id))
                                                .or_insert_with(|| vec![element.id]);
                                        }
                                    }
                                    for key in &def.key_codes {
                                    id_maps_clone.borrow_mut().
                                        keys
                                            // TODO: this is missing some stuff?
                                            // There's a chance that NohBoard is using chrome
                                            // keycodes instead of vkcodes
                                            .entry(from_windows_vkcode(*key as i32))
                                            .and_modify(|ids| ids.push(element.id))
                                            .or_insert_with(|| vec![element.id]);
                                    }
                                }
                                let items: Vec<_> = options.into_iter().map(Into::into).collect();
                                popup.unwrap().set_local_styles(items.as_slice().into());
                                memory_clone.borrow_mut().local_styles = items;
                            }
                            Err(e) => ErrorPopup::pop(format!("Failed to load layout: {e}")),
                        }
                        weak_clone.unwrap().set_loading(false);

                        if let Some(style) = &*style_clone.borrow()
                            && settings_clone.borrow().global_style
                        {
                            weak_clone.unwrap().apply_style(
                                StyleSource::Global,
                                settings_clone.borrow().style.as_ref().unwrap(),
                                style,
                            );
                        }
                    })
                    .unwrap();
                }
            ));

            popup.on_style_changed(lambda!(
                [clone] | kind,
                name | {
                    settings.borrow_mut().global_style = kind == StyleKind::Global;
                    settings.borrow_mut().style = Some(PathSegment::new(name.clone()).unwrap());
                    weak.unwrap().set_loading(true);

                    let settings_clone = settings.clone();
                    let weak_clone = weak.clone();
                    let style_clone = style.clone();
                    #[allow(
                        clippy::await_holding_refcell_ref,
                        reason = "false positive (see #6353)"
                    )]
                    slint::spawn_local(async move {
                        let settings = settings_clone.borrow();
                        let source = match kind {
                            StyleKind::Global => StyleSource::Global,
                            StyleKind::Local => StyleSource::Local {
                                category: settings.category.clone().unwrap(),
                                layout: settings.layout.clone().unwrap(),
                            },
                        };
                        let name = PathSegment::new(name).unwrap();
                        drop(settings);
                        match load_style(source.clone(), name.clone()).await {
                            Ok(new_style) => {
                                weak_clone.unwrap().apply_style(source, &name, &new_style);
                                *style_clone.borrow_mut() = Some(new_style);
                            }
                            Err(e) => ErrorPopup::pop(format!("Failed to load style: {e}")),
                        }
                        weak_clone.unwrap().set_loading(false);
                    })
                    .unwrap();
                }
            ));

            fn is_item(seg: &impl Display, item: &StandardListViewItem) -> bool {
                seg.to_shared_string() == item.text
            }

            popup.set_global_styles(memory.borrow().global_styles.as_slice().into());
            {
                let settings = settings.borrow();
                if settings.global_style
                    && let Some(style) = &settings.style
                    && let Some(index) = memory.borrow().global_styles.iter().enumerate().find_map(
                        |(index, item)| {
                            if is_item(style, item) {
                                Some(index)
                            } else {
                                Option::None
                            }
                        },
                    )
                {
                    popup.set_global_style(index as i32);
                }
            }

            popup.set_categories(memory.borrow().categories.as_slice().into());
            if let Some(category) = &settings.borrow().category {
                popup.set_category(category.to_shared_string());
            }

            let mut layout_index = -1;
            popup.set_layouts(
                memory
                    .borrow()
                    .layouts
                    .iter()
                    .enumerate()
                    .map(|(index, name)| {
                        if settings
                            .borrow()
                            .layout
                            .as_ref()
                            .is_some_and(|layout| is_item(layout, name))
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
                memory
                    .borrow()
                    .local_styles
                    .iter()
                    .enumerate()
                    .map(|(index, name)| {
                        if !settings.borrow().global_style
                            && settings
                                .borrow()
                                .style
                                .as_ref()
                                .is_some_and(|style| is_item(style, name))
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
        }
    ));

    let weak_clone = weak.clone();
    let settings_clone = settings.clone();
    let style_clone = style.clone();
    let key_indices_clone = key_indices.clone();
    let id_maps_clone = id_maps.clone();
    let memory_clone = memory.clone();
    slint::spawn_local(async move {
        setup(
            weak_clone.clone(),
            settings_clone,
            memory_clone,
            style_clone,
            key_indices_clone,
            id_maps_clone,
        )
        .await;
        weak_clone.unwrap().set_loading(false);
    })
    .unwrap();

    app.window().on_close_requested(lambda!(
        [clone] || {
            let weak_clone = weak.clone();
            let settings_clone = settings.clone();

            slint::spawn_local(async move {
                let Ok(string) = serde_json::to_string(&*settings_clone.borrow()) else {
                    ErrorPopup::pop("Failed to save settings");
                    return;
                };
                if async_fs::write(NUHXBOARD_PATH.join("settings.json"), &string)
                    .await
                    .is_err()
                {
                    ErrorPopup::pop("Failed to save settings");
                    return;
                }
                if weak_clone.unwrap().window().hide().is_err() {
                    slint::quit_event_loop().unwrap();
                }
            })
            .unwrap();

            CloseRequestResponse::KeepWindowShown
        }
    ));

    app.run().unwrap();
}

fn mouse_button_code_convert(code: u32) -> Button {
    match code {
        0 => Button::Left,
        1 => Button::Right,
        2 => Button::Middle,
        3 => Button::Back,
        4 => Button::Forward,
        _ => Button::Other(code as u8),
    }
}

fn update_key(
    state: ButtonState,
    ids: &[u32],
    key_indices: Rc<RefCell<HashMap<u32, usize>>>,
    style: &Style,
    settings: Rc<RefCell<Settings>>,
    app: Weak<Main>,
) {
    for id in ids {
        let images_path = if settings.borrow().global_style {
            format!("global/{}", settings.borrow().style.as_ref().unwrap())
        } else {
            format!("{}/images", settings.borrow().category.as_ref().unwrap())
        };
        let images_path = NUHXBOARD_PATH.join("keyboards").join(images_path);

        let style = match style.element_styles.get(id) {
            Some(ElementStyle::KeyStyle(KeyStyle {
                pressed: Some(style),
                ..
            })) if state == ButtonState::Pressed => style,
            Some(ElementStyle::KeyStyle(KeyStyle {
                loose: Some(style), ..
            })) => style,
            _ => {
                if state == ButtonState::Pressed {
                    &style.default_key_style.pressed
                } else {
                    &style.default_key_style.loose
                }
            }
        };
        let index = *key_indices.borrow().get(id).unwrap();
        let old_key = app.unwrap().get_keys().row_data(index).unwrap();
        let background_image = style
            .background_image_file_name
            .as_ref()
            .and_then(
                |name| match Image::load_from_path(&images_path.join(name)) {
                    Ok(i) => Some(i),
                    Err(e) => {
                        ErrorPopup::pop(format!("Failed to load image: {e}"));
                        None
                    }
                },
            )
            .unwrap_or_default();
        app.unwrap().get_keys().set_row_data(
            index,
            Key {
                background: style.background.into(),
                text_color: style.text.into(),
                font_family: style.font.font_family.to_shared_string(),
                font_size: style.font.size,
                outline_color: style.outline.into(),
                outline_width: if style.show_outline {
                    style.outline_width
                } else {
                    0.0
                },
                background_image,
                ..old_key
            },
        );
    }
}

fn update_layout_state(layout: &Layout, id_maps: Rc<RefCell<IdMaps>>) {
    for element in &layout.elements {
        let Ok(def) = <&CommonElement>::try_from(&element.kind) else {
            continue;
        };
        match element.kind {
            ElementKind::MouseKey(_) => {
                for button in &def.key_codes {
                    id_maps
                        .borrow_mut()
                        .buttons
                        .entry(mouse_button_code_convert(*button))
                        .and_modify(|ids| ids.push(element.id))
                        .or_insert_with(|| vec![element.id]);
                }
            }
            ElementKind::MouseScroll(_) => {
                for code in &def.key_codes {
                    match code {
                        0 => id_maps.borrow_mut().up.push(element.id),
                        1 => id_maps.borrow_mut().down.push(element.id),
                        2 => id_maps.borrow_mut().right.push(element.id),
                        3 => id_maps.borrow_mut().left.push(element.id),
                        _ => ErrorPopup::pop(format!(
                            "Unrecognized scroll code {code} for element {}",
                            element.id
                        )),
                    }
                }
            }
            _ => {
                for key in &def.key_codes {
                    id_maps
                        .borrow_mut()
                        .keys
                        .entry(from_windows_vkcode(*key as i32))
                        .and_modify(|ids| ids.push(element.id))
                        .or_insert_with(|| vec![element.id]);
                }
            }
        }
    }
}
