use async_stream::stream;
use iced::advanced::subscription::Recipe;
use rdevin::Event;
use std::hash::Hash;

pub struct RdevinSubscriber;

impl Recipe for RdevinSubscriber {
    type Output = Event;

    fn hash(&self, state: &mut iced::advanced::subscription::Hasher) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced::advanced::subscription::EventStream,
    ) -> iced::advanced::graphics::futures::BoxStream<Self::Output> {
        let (tx, rx) = async_channel::unbounded();

        std::thread::spawn(|| {
            rdevin::listen(move |e| tx.send_blocking(e).unwrap()).unwrap();
        });

        Box::pin(stream! {
            while let Ok(e) = rx.recv().await {
                yield e;
            }
        })
    }
}
