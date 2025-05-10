# `iced-multi-window`

Utilities for managing many windows in an `iced` application.

## Goals

Working with multiple windows in iced can become quite painful quite quickly. If you want to introduce a window type with unique behavior, you may have to make additions in more than five places accross your codebase. Oversights are easy, and most of the mistakes you can make aren't caught by the compiler. This library seeks to ease this experince by making defining and working with multiple windows simpler, more intuitive, and harder to screw up.

## Usage

The first step is to define the windows that will appear in your app. This is done by creating a corresponding struct and implementing the `Window` trait for it. This trait will describe the logic behind that window's content, title, and theme, as well as defining its spawn-time settings.

Next, add a `WindowManager` to your app's state. It keeps track of all of the `Id`s and corresponding `Window`s that are currently open. It also provides `view`, `theme`, and `title` methods that return the proper output for the specified `Id`.

You have to manually inform the `WindowManager` when a window is closed. This can be done by subscribing to `iced::window::close_events()` and passing the `Id` of each closed window to `WindowManager::was_closed()`.
