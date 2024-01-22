use iced::advanced::subscription::Recipe;
use regex::Regex;
use std::{
    hash::Hash,
    io::{self, prelude::*},
    process::Command,
    task::Poll,
};

pub struct InputSubscription;

impl Recipe for InputSubscription {
    type Output = crate::Message;

    fn hash(&self, state: &mut iced::advanced::Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced::advanced::subscription::EventStream,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        let mut child = Command::new("xinput")
            .arg("test-xi2")
            .arg("--root")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let reader = io::BufReader::new(child.stdout.take().unwrap());
        std::boxed::Box::pin(InputStream { reader })
    }
}

struct InputStream {
    reader: io::BufReader<std::process::ChildStdout>,
}

enum InputEvent {
    MouseButtonPress,
    MouseButtonRelease,
    Motion,
    KeyPress,
    KeyRelease,
}

impl futures::stream::Stream for InputStream {
    type Item = crate::Message;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut line = String::new();
        let reader = &mut self.get_mut().reader;
        reader.read_line(&mut line).unwrap();
        if line.is_empty() {
            return Poll::Ready(Some(crate::Message::Dummy));
        }
        let event_type_re = Regex::new(r"EVENT type ([0-9]+) ").unwrap();
        let event_type = match event_type_re.captures(&line) {
            Some(caps) => match caps.get(1).unwrap().as_str() {
                "2" => InputEvent::KeyPress,
                "3" => InputEvent::KeyRelease,
                "15" => InputEvent::MouseButtonPress,
                "16" => InputEvent::MouseButtonRelease,
                "17" => InputEvent::Motion,
                _ => return Poll::Ready(Some(crate::Message::Dummy)),
            },
            None => return Poll::Ready(Some(crate::Message::Dummy)),
        };
        match event_type {
            InputEvent::KeyPress | InputEvent::KeyRelease => {
                for _ in 0..2 {
                    reader.read_line(&mut line).unwrap();
                }
                let keycode_re = Regex::new(r"detail: ([0-9]+)").unwrap();
                let keycode = keycode_re
                    .captures(&line)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse::<u32>()
                    .unwrap();

                for _ in 0..5 {
                    reader.read_line(&mut line).unwrap();
                }
                let modifiers_re = Regex::new(r"modifiers: locked (?:0x)?([a-f0-9]+)").unwrap();
                let modifiers = u8::from_str_radix(
                    modifiers_re
                        .captures(&line)
                        .unwrap()
                        .get(1)
                        .unwrap()
                        .as_str(),
                    16,
                )
                .unwrap();
                let caps = modifiers & 0b10 != 0;
                match event_type {
                    InputEvent::KeyPress => {
                        Poll::Ready(Some(crate::Message::KeyPress { keycode, caps }))
                    }
                    InputEvent::KeyRelease => {
                        Poll::Ready(Some(crate::Message::KeyRelease { keycode, caps }))
                    }
                    _ => unreachable!(),
                }
            }
            InputEvent::MouseButtonPress | InputEvent::MouseButtonRelease => {
                for _ in 0..2 {
                    reader.read_line(&mut line).unwrap();
                }
                let button_code_re = Regex::new(r"detail: ([0-9]+)").unwrap();
                let button_code = button_code_re
                    .captures(&line)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse::<u32>()
                    .unwrap();

                match event_type {
                    InputEvent::MouseButtonPress => {
                        if button_code == 4 || button_code == 5 {
                            dbg!();
                            return Poll::Ready(Some(crate::Message::ScrollButtonPress(
                                button_code - 4,
                            )));
                        }
                        Poll::Ready(Some(crate::Message::MouseButtonPress(button_code)))
                    }
                    InputEvent::MouseButtonRelease => {
                        // See main.rs:36
                        if button_code == 4 || button_code == 5 {
                            return Poll::Ready(Some(crate::Message::Dummy));
                        }
                        Poll::Ready(Some(crate::Message::MouseButtonRelease(button_code)))
                    }
                    _ => unreachable!(),
                }
            }
            InputEvent::Motion => {
                for _ in 0..5 {
                    reader.read_line(&mut line).unwrap();
                }
                let scroll_check_re = Regex::new(r"3: -?[0-9]+").unwrap();
                if scroll_check_re.captures(&line).is_some() {
                    return Poll::Ready(Some(crate::Message::Dummy));
                }
                let x_vel_re = Regex::new(r"0: (-?[0-9]+)").unwrap();
                let x_vel = x_vel_re
                    .captures(&line)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse::<f32>()
                    .unwrap();
                reader.read_line(&mut line).unwrap();
                let y_vel_re = Regex::new(r"1: (-?[0-9]+)").unwrap();
                let y_vel = y_vel_re
                    .captures(&line)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse::<f32>()
                    .unwrap();

                Poll::Ready(Some(crate::Message::Motion { x: x_vel, y: y_vel }))
            }
        }
    }
}
