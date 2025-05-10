//! # `iced-multi-window`
//!
//! Utilities for managing many windows in an `iced` application.
//!
//! ## Goals
//!
//! Working with multiple windows in iced can become quite painful quite quickly. If you want to introduce a window type with unique behavior, you may have to make additions in more than five places accross your codebase. Oversights are easy, and most of the mistakes you can make aren't caught by the compiler. This library seeks to ease this experince by making defining and working with multiple windows simpler, more intuitive, and harder to screw up.
//!
//! ## Usage
//!
//! The first step is to define the windows that will appear in your app. This is done by creating a corresponding struct and implementing the `Window` trait for it. This trait will describe the logic behind that window's content, title, and theme, as well as defining its spawn-time settings.
//!
//! Next, add a `WindowManager` to your app's state. It keeps track of all of the `Id`s and corresponding `Window`s that are currently open. It also provides `view`, `theme`, and `title` methods that return the proper output for the specified `Id`.
//!
//! You have to manually inform the `WindowManager` when a window is closed. This can be done by subscribing to `iced::window::close_events()` and passing the `Id` of each closed window to `WindowManager::was_closed()`.

use dyn_clone::DynClone;
use iced::{
    window::{self, Id},
    Element, Task,
};
use std::{any::type_name, collections::HashMap};

#[allow(private_bounds)]
pub trait Window<App, Theme, Message, Renderer = iced::Renderer>:
    Send + std::fmt::Debug + DynClone
{
    fn view<'a>(&'a self, app: &'a App) -> iced::Element<'a, Message, Theme, Renderer>;
    fn title(&self, app: &App) -> String;
    fn theme(&self, app: &App) -> Theme;
    fn settings(&self) -> window::Settings;
    /// The unique identifier for this window. This includes any internal data.
    fn id(&self) -> String {
        let data = format!("{:?}", self);
        let data = if let Some(i) = data.find(" {") {
            data[i..].to_string()
        } else {
            format!("::{}", data)
        };

        format!("{}{}", type_name::<Self>(), data)
    }
    /// An identifier for this window's "class". Whereas `id` is used to identify individual windows, `class` is used to identify a window's type.
    fn class(&self) -> &'static str {
        type_name::<Self>()
    }
}

dyn_clone::clone_trait_object!(<App, Theme, Message, Renderer> Window<App, Theme, Message, Renderer>);

impl<App, Theme, Message, Renderer, T: Window<App, Theme, Message, Renderer>> PartialEq<T>
    for Box<dyn Window<App, Theme, Message, Renderer>>
{
    fn eq(&self, other: &T) -> bool {
        self.id() == other.id()
    }
}

impl<App, Theme, Message, Renderer> Window<App, Theme, Message, Renderer>
    for Box<dyn Window<App, Theme, Message, Renderer>>
{
    fn view<'a>(&'a self, app: &'a App) -> iced::Element<'a, Message, Theme, Renderer> {
        self.as_ref().view(app)
    }

    fn title(&self, app: &App) -> String {
        self.as_ref().title(app)
    }

    fn theme(&self, app: &App) -> Theme {
        self.as_ref().theme(app)
    }

    fn settings(&self) -> window::Settings {
        self.as_ref().settings()
    }

    fn id(&self) -> String {
        self.as_ref().id()
    }

    fn class(&self) -> &'static str {
        self.as_ref().class()
    }
}

pub struct WindowManager<App, Theme, Message, Renderer = iced::Renderer> {
    windows: HashMap<Id, Box<dyn Window<App, Theme, Message, Renderer>>>,
}

impl<App, Theme, Message, Renderer> WindowManager<App, Theme, Message, Renderer> {
    /// Returns the window associated with the given Id, panicking if it doesn't exist.
    fn get(&self, id: Id) -> &dyn Window<App, Theme, Message, Renderer> {
        self.windows
            .get(&id)
            .expect("No window found with given Id")
            .as_ref()
    }

    pub fn view<'a>(&'a self, app: &'a App, id: Id) -> Element<'a, Message, Theme, Renderer> {
        self.get(id).view(app)
    }

    pub fn title(&self, app: &App, id: Id) -> String {
        self.get(id).title(app)
    }

    pub fn theme(&self, app: &App, id: Id) -> Theme {
        self.get(id).theme(app)
    }

    pub fn open(
        &mut self,
        window: Box<dyn Window<App, Theme, Message, Renderer>>,
    ) -> (Id, Task<Id>) {
        let (id, task) = window::open(window.settings());
        self.windows.insert(id, window);
        (id, task)
    }

    pub fn close_all(&mut self) -> Task<Id> {
        let mut tasks = Vec::new();
        for id in self.windows.keys() {
            tasks.push(window::close(*id));
        }
        Task::batch(tasks)
    }

    pub fn close_all_of(
        &mut self,
        window: Box<dyn Window<App, Theme, Message, Renderer>>,
    ) -> Task<Id> {
        let mut tasks = Vec::new();
        for (id, w) in self.windows.iter() {
            if *w == window {
                tasks.push(window::close(*id));
            }
        }

        Task::batch(tasks)
    }

    /// Checks for any open instances of the given window.
    pub fn any_of(&self, window: &impl Window<App, Theme, Message, Renderer>) -> bool {
        self.windows.values().any(|w| w == window)
    }

    /// Updates internal state to reflect that the given window Id  was closed.
    pub fn was_closed(&mut self, id: Id) {
        self.windows.remove(&id);
    }

    /// Returns all instances of the given window and their associated Ids.
    #[allow(clippy::type_complexity)]
    pub fn instances_of(
        &self,
        window: &impl Window<App, Theme, Message, Renderer>,
    ) -> Vec<(&Id, &Box<dyn Window<App, Theme, Message, Renderer>>)> {
        self.windows.iter().filter(|(_, w)| *w == window).collect()
    }

    pub fn empty(&self) -> bool {
        self.windows.is_empty()
    }
}

impl<App, Theme, Message, Renderer> Default for WindowManager<App, Theme, Message, Renderer> {
    fn default() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }
}
