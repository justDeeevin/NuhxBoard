use async_stream::stream;
use blocking::unblock;
use iced::advanced::subscription::Recipe;
use rdev::Event;
use std::hash::Hash;

pub struct RdevSubscriber;

impl Recipe for RdevSubscriber {
    type Output = Event;

    fn hash(&self, state: &mut iced::advanced::subscription::Hasher) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced::advanced::subscription::EventStream,
    ) -> iced::advanced::graphics::futures::BoxStream<Self::Output> {
        let (tx, rx) = async_channel::unbounded();

        drop(smol::spawn(unblock(|| {
            rdev::listen(move |e| tx.send_blocking(e).unwrap()).unwrap();
        })));

        Box::pin(stream! {
            while let Ok(e) = rx.recv().await {
                yield e;
            }
        })
    }
}
