use iced::{
    futures::{channel::mpsc, StreamExt},
    subscription, Subscription,
};
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

pub fn bind() -> Subscription<Event> {
    struct Keys;

    subscription::unfold(
        std::any::TypeId::of::<Keys>(),
        State::Starting,
        |state| async move {
            match state {
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
                    (Event::Ready, State::Ready(rx))
                }
                State::Ready(mut rx) => {
                    let received = rx.next().await;
                    match received {
                        Some(key) => (Event::KeyReceived(key), State::Ready(rx)),
                        None => (Event::None, State::Ready(rx)),
                    }
                }
            }
        },
    )
}
