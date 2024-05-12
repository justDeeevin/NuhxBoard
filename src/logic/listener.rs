use iced::{
    futures::{channel::mpsc, StreamExt},
    subscription, Subscription,
};
use rdev::grab;

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
                        grab(move |event| {
                            if let Err(e) = tx.unbounded_send(event.clone()) {
                                if !e.is_disconnected() {
                                    panic!("{}", e);
                                }
                            }
                            Some(event)
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
