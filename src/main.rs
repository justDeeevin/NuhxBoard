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
mod config;
mod style;
use clap::Parser;
use colors_transform::{AlphaColor, Color as ColorTrait, Rgb};
use config::*;
use iced::{
    mouse,
    widget::{
        canvas,
        canvas::{Cache, Geometry, Path},
        container,
    },
    Application, Color, Command, Length, Point, Rectangle, Renderer, Theme,
};
use serde::Deserialize;
use std::{fs::File, io::prelude::*};

struct NuhxBoard {
    config: Config,
    canvas: Cache,
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
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new(flags: Flags) -> (Self, Command<Message>) {
        (
            Self {
                config: flags.config,
                canvas: Cache::default(),
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
        let canvas = canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill);

        container(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<Message> canvas::Program<Message, Renderer> for NuhxBoard {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let canvas = self.canvas.draw(renderer, bounds.size(), |frame| {
            for element in &self.config.elements {
                match element {
                    BoardElement::KeyboardKey(def) => {
                        let mut boundaries_iter = def.boundaries.iter();
                        let key = Path::new(|builder| {
                            builder.move_to((*boundaries_iter.next().unwrap()).clone().into());
                            for boundary in boundaries_iter {
                                builder.line_to((*boundary).clone().into());
                            }
                            builder.close()
                        });
                        let fill_color = Rgb::from_hex_str("#000000").unwrap();
                        frame.fill(
                            &key,
                            Color::from_rgba(
                                fill_color.get_red(),
                                fill_color.get_green(),
                                fill_color.get_blue(),
                                fill_color.get_alpha(),
                            ),
                        );
                    }
                    _ => unimplemented!(),
                }
            }
        });
        vec![canvas]
    }
}

#[derive(Parser, Debug)]
#[command(author = "justDeeevin", version = "0.1.0")]
struct Args {
    #[arg(short, long)]
    config_path: String,

    #[arg(short, long)]
    style_path: String,
}

fn main() -> iced::Result {
    let args = Args::parse();

    let mut config_file = match File::open(&args.config_path) {
        Err(why) => panic!(
            "Error opening config file (given path: {}): {}",
            args.config_path, why
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

    let icon = iced::window::icon::from_file(std::path::Path::new("NuhxBoard.png")).unwrap();

    let flags = Flags { config };
    let settings = iced::Settings {
        window: iced::window::Settings {
            size: (flags.config.width, flags.config.height),
            resizable: true,
            icon: Some(icon),
            ..iced::window::Settings::default()
        },
        flags,
        ..iced::Settings::default()
    };
    NuhxBoard::run(settings)
}
