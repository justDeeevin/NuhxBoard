#![allow(clippy::await_holding_refcell_ref)]

// TODO:
// - clamda for async blocks?

use async_fn::*;
use clamda::lambda;
use nuhxboard_json::{PathSegment, settings::Settings};
use nuhxboard_ui::*;
use slint::{ToSharedString, language::StandardListViewItem};
use std::{cell::RefCell, fmt::Display, rc::Rc};

mod async_fn;

fn main() {
    // TODO: save settings?
    let settings = Rc::new(RefCell::new(Settings::default()));
    let categories = Rc::new(RefCell::new(Vec::default()));
    let layouts = Rc::new(RefCell::new(Vec::default()));
    let local_styles = Rc::new(RefCell::new(Vec::default()));
    let global_styles = Rc::new(RefCell::new(Vec::default()));
    let style = Rc::new(RefCell::new(None));

    let app = Main::new().unwrap();
    let weak = app.as_weak();

    app.set_version(env!("CARGO_PKG_VERSION").to_shared_string());

    app.on_open_settings(lambda!(
        [clone] || {
            let Ok(popup) = SettingsPopup::new() else {
                return;
            };
            let weak_clone = weak.clone();

            popup.set_settings((&*settings.borrow()).into());

            popup.on_submit(lambda!(
                [clone] | user_settings | {
                    weak_clone
                        .unwrap()
                        .set_t(user_settings.window_title.clone());
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
        [clone, clone settings] || {
            let Ok(popup) = LoadKeyboard::new() else {
                return;
            };

            let weak_popup = popup.as_weak();
            let weak_clone = weak.clone();
            let settings = settings.clone();
            popup.on_category_changed(lambda!(
                [clone] | category | {
                    settings.borrow_mut().category =
                        Some(PathSegment::new(category.clone()).unwrap());
                    weak_clone.unwrap().set_loading(true);

                    let layouts_clone = layouts.clone();
                    let popup = weak_popup.clone();
                    let app = weak_clone.clone();
                    slint::spawn_local(async move {
                        match load_category(&PathSegment::new(category).unwrap()).await {
                            Ok(options) => {
                                let items: Vec<_> = options.into_iter().map(Into::into).collect();
                                popup.unwrap().set_layouts(items.as_slice().into());
                                *layouts_clone.borrow_mut() = items;
                            }
                            Err(e) => ErrorPopup::pop(format!("Failed to load category: {e}")),
                        }
                        app.unwrap().set_loading(false);
                    })
                    .unwrap();
                }
            ));

            let style_clone = style.clone();
            popup.on_layout_changed(lambda!(
                [clone] | layout | {
                    settings.borrow_mut().layout = Some(PathSegment::new(layout.clone()).unwrap());
                    weak_clone.unwrap().set_loading(true);
                    if !settings.borrow().global_style {
                        weak_popup.unwrap().set_local_style(-1);
                    }

                    let local_styles_clone = local_styles.clone();
                    let settings = settings.clone();
                    let app = weak.clone();
                    let popup = weak_popup.clone();
                    let style = style_clone.clone();
                    slint::spawn_local(async move {
                        match load_layout(
                            settings.borrow().category.as_ref().unwrap(),
                            &PathSegment::new(layout.clone()).unwrap(),
                        )
                        .await
                        {
                            Ok((layout, options)) => {
                                app.unwrap().apply_layout(&layout);
                                let items: Vec<_> = options.into_iter().map(Into::into).collect();
                                popup.unwrap().set_local_styles(items.as_slice().into());
                                *local_styles_clone.borrow_mut() = items;
                            }
                            Err(e) => ErrorPopup::pop(format!("Failed to load layout: {e}")),
                        }
                        app.unwrap().set_loading(false);

                        if let Some(style) = &*style.borrow()
                            && settings.borrow().global_style
                        {
                            app.unwrap().apply_style(
                                StyleSource::Global,
                                settings.borrow().style.as_ref().unwrap(),
                                style,
                            );
                        }
                    })
                    .unwrap();
                }
            ));

            let style_clone = style.clone();
            popup.on_style_changed(lambda!(
                [clone] | kind,
                name | {
                    settings.borrow_mut().global_style = kind == StyleKind::Global;
                    settings.borrow_mut().style = Some(PathSegment::new(name.clone()).unwrap());
                    weak_clone.unwrap().set_loading(true);

                    let settings = settings.clone();
                    let app = weak.clone();
                    let style = style_clone.clone();
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
                            Ok(new_style) => {
                                app.unwrap().apply_style(source, &name, &new_style);
                                *style.borrow_mut() = Some(new_style);
                            }
                            Err(e) => ErrorPopup::pop(format!("Failed to load style: {e}")),
                        }
                        app.unwrap().set_loading(false);
                    })
                    .unwrap();
                }
            ));

            fn is_item(seg: &impl Display, item: &StandardListViewItem) -> bool {
                seg.to_shared_string() == item.text
            }

            popup.set_global_styles(global_styles.borrow().as_slice().into());
            {
                let settings = settings.borrow();
                if settings.global_style
                    && let Some(style) = &settings.style
                    && let Some(index) =
                        global_styles
                            .borrow()
                            .iter()
                            .enumerate()
                            .find_map(|(index, item)| {
                                if is_item(style, item) {
                                    Some(index)
                                } else {
                                    Option::None
                                }
                            })
                {
                    popup.set_global_style(index as i32);
                }
            }

            popup.set_categories(categories.borrow().as_slice().into());
            if let Some(category) = &settings.borrow().category {
                popup.set_category(category.to_shared_string());
            }

            let mut layout_index = -1;
            popup.set_layouts(
                layouts
                    .borrow()
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
                local_styles
                    .borrow()
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

    let weak = app.as_weak();
    let settings_clone = settings.clone();
    let categories_clone = categories.clone();
    let layouts_clone = layouts.clone();
    let local_styles_clone = local_styles.clone();
    let global_styles_clone = global_styles.clone();
    let style_clone = style.clone();
    slint::spawn_local(async move {
        setup(
            weak.clone(),
            settings_clone,
            categories_clone,
            layouts_clone,
            local_styles_clone,
            global_styles_clone,
            style_clone,
        )
        .await;
        weak.unwrap().set_loading(false);
    })
    .unwrap();

    app.run().unwrap();
}
