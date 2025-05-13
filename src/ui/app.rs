use super::keyboard::Keyboard;
use crate::{message::*, nuhxboard::*};
use iced::{
    widget::{image::Handle, Image, Stack},
    window, Color, Theme,
};
use iced_multi_window::Window;
use std::sync::Arc;

static IMAGE: &[u8] = include_bytes!("../../media/NuhxBoard.png");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Main;
impl Window<NuhxBoard, Theme, Message> for Main {
    fn settings(&self) -> window::Settings {
        let icon_image = image::load_from_memory(IMAGE).unwrap();
        let icon = window::icon::from_rgba(icon_image.to_rgba8().to_vec(), 256, 256).unwrap();

        window::Settings {
            size: DEFAULT_WINDOW_SIZE,
            // resizable: false,
            icon: Some(icon),
            ..window::Settings::default()
        }
    }

    fn view<'a>(&self, app: &'a NuhxBoard) -> iced::Element<'a, Message, Theme> {
        let keyboard = Keyboard::new(app.layout.width, app.layout.height, app);

        if app.style.background_image_file_name.is_some() {
            let image = Image::new(Handle::from_path(
                KEYBOARDS_PATH.parent().unwrap().join("background.png"),
            ));
            return Stack::with_children(vec![image.into(), keyboard.into()]).into();
        }
        keyboard.into()
    }

    fn theme(&self, app: &NuhxBoard) -> Theme {
        let red = app.style.background_color.red / 255.0;
        let green = app.style.background_color.green / 255.0;
        let blue = app.style.background_color.blue / 255.0;
        let palette = iced::theme::Palette {
            background: Color::from_rgb(red, green, blue),
            ..iced::theme::Palette::DARK
        };
        Theme::Custom(Arc::new(iced::theme::Custom::new("Custom".into(), palette)))
    }

    fn title(&self, app: &NuhxBoard) -> String {
        app.settings.window_title.clone()
    }
}
