use futures::future::Shared;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

pub type ValueFuture<R> = Shared<Pin<Box<dyn Future<Output = R>>>>;

#[derive(Clone)]
pub enum ValueStatus<R> {
    Loading(ValueFuture<R>),
    Loaded(R),
}

impl<R> Future for ValueStatus<R>
where
    R: Clone + Unpin,
{
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match &mut *self {
            Self::Loaded(res) => res.clone().into(),
            Self::Loading(res) => Pin::new(res).poll(cx),
        }
    }
}
