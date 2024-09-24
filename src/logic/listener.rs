use futures::{
    channel::mpsc,
    sink::SinkExt,
    stream::{Stream, StreamExt},
};
use iced::stream;
#[cfg(feature = "grab")]
use rdev::grab;
use rdev::listen;

enum State {
    Starting,
    Ready(mpsc::UnboundedReceiver<rdev::Event>),
}

#[derive(Debug, Clone)]
pub enum Event {
    Ready,
    KeyReceived(rdev::Event),
    None,
}

pub fn bind() -> impl Stream<Item = Event> {
    stream::channel(100, |mut output| async move {
        let mut state = State::Starting;

        loop {
            match &mut state {
                State::Starting => {
                    let (tx, rx) = mpsc::unbounded();
                    std::thread::spawn(move || {
                        #[cfg(feature = "grab")]
                        if cfg!(target_os = "linux")
                            && std::env::var("XDG_SESSION_TYPE").unwrap() == "wayland"
                        {
                            println!("Wayland detected, using grab");
                            grab(move |event| {
                                if let Err(e) = tx.unbounded_send(event.clone()) {
                                    if !e.is_disconnected() {
                                        panic!("{}", e);
                                    }
                                }
                                Some(event)
                            })
                            .unwrap();
                        } else {
                            listen(move |event| {
                                if let Err(e) = tx.unbounded_send(event.clone()) {
                                    if !e.is_disconnected() {
                                        panic!("{}", e);
                                    }
                                }
                            })
                            .unwrap();
                        }
                        #[cfg(not(feature = "grab"))]
                        listen(move |event| {
                            if let Err(e) = tx.unbounded_send(event.clone()) {
                                if !e.is_disconnected() {
                                    panic!("{}", e);
                                }
                            }
                        })
                        .unwrap();
                    });
                    output.send(Event::Ready).await.unwrap();
                    state = State::Ready(rx);
                }
                State::Ready(ref mut rx) => {
                    let Some(key) = rx.next().await else {
                        continue;
                    };
                    output.send(Event::KeyReceived(key)).await.unwrap();
                }
            }
        }
    })
}
