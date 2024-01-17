// use regex::Regex;
// use std::io::{self, BufRead};
// use std::process::Command;
//
// fn main() -> io::Result<()> {
//     let mut child = Command::new("xinput")
//         .arg("test-xi2")
//         .arg("--root")
//         .stdout(std::process::Stdio::piped())
//         .spawn()?;
//
//     let reader = io::BufReader::new(child.stdout.take().unwrap());
//
//     let mut key_down = false;
//     let mut key_up = false;
//
//     for line in reader.lines() {
//         let line = line?;
//
//         if line.contains("EVENT type 2") {
//             key_down = true;
//         } else if line.contains("EVENT type 3") {
//             key_up = true;
//         } else if line.contains("detail:") {
//             let re = Regex::new(r"detail:\s*(\d+)").unwrap();
//
//             if key_down {
//                 let captures = re.captures(&line).unwrap();
//                 let keycode: u32 = captures.get(1).unwrap().as_str().parse().unwrap();
//                 println!("Key down: {}", keycode)
//             }
//             if key_up {
//                 let captures = re.captures(&line).unwrap();
//                 let keycode: u32 = captures.get(1).unwrap().as_str().parse().unwrap();
//                 println!("Key up: {}", keycode)
//             }
//
//             key_down = false;
//             key_up = false;
//         }
//     }
//
//     Ok(())
// }
#![allow(unused)]
use iced::{Application, Command};
use serde::Deserialize;
use std::{fs::File, io::prelude::*};

#[derive(Deserialize, Default, Debug)]
struct Config {
    version: String,
    width: u32,
    height: u32,
    elements: Vec<BoardElement>,
}

#[derive(Deserialize, Debug)]
enum BoardElement {
    KeyboardKey(KeyboardKeyDefinition),
    MouseKey(MouseKeyDefinition),
    MouseScroll(MouseScrollDefinition),
    MouseSpeedIndicator(MouseSpeedIndicatorDefinition),
}

#[derive(Deserialize, Debug)]
struct KeyboardKeyDefinition {
    id: u32,
    boundaries: Vec<Point>,
    text_position: Point,
    keycodes: Vec<u32>,
    text: String,
    shift_text: String,
    change_on_caps: bool,
}

#[derive(Deserialize, Debug)]
struct MouseKeyDefinition {
    id: u32,
    boundaries: Vec<Point>,
    text_position: Point,
    keycodes: Vec<u32>,
    text: String,
}

#[derive(Deserialize, Debug)]
struct MouseScrollDefinition {
    id: u32,
    boundaries: Vec<Point>,
    text_position: Point,
    keycodes: Vec<u32>,
    text: String,
}

#[derive(Deserialize, Debug)]
struct MouseSpeedIndicatorDefinition {
    id: u32,
    location: Point,
    radius: u32,
}

#[derive(Deserialize, Debug)]
struct Point {
    x: u32,
    y: u32,
}

struct NuhxBoard {
    config: Config,
}

#[derive(Debug)]
enum Message {
    KeyDown(u32),
    KeyUp(u32),
}

#[derive(Default)]
struct Flags {
    config: Config,
}

impl Application for NuhxBoard {
    type Flags = Flags;
    type Theme = iced::Theme;
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new(flags: Flags) -> (Self, Command<Message>) {
        (
            Self {
                config: flags.config,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("NuhxBoard")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        "Hello, World!".into()
    }
}

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("No config file specified");
    }
    let mut config_file = match File::open(&args[1]) {
        Err(why) => panic!(
            "Error opening config file (given path: {}): {}",
            args[1], why
        ),
        Ok(file) => file,
    };
    let mut config_string = String::new();
    if let Err(why) = config_file.read_to_string(&mut config_string) {
        panic!("Error reading config file: {}", why)
    };
    let config: Config = match serde_json::from_str(&config_string) {
        Err(why) => panic!("Error parsing config file: {}", why),
        Ok(config) => config,
    };
    dbg!(&config);
    let flags = Flags { config };
    let settings = iced::Settings {
        window: iced::window::Settings {
            size: (flags.config.width, flags.config.height),
            resizable: true,
            ..iced::window::Settings::default()
        },
        flags,
        ..iced::Settings::default()
    };
    NuhxBoard::run(settings)
}
