use super::{super::Error, body};
use hyper::client::ResponseFuture;
use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
};

enum IdsChunkFutureInternal<const CHUNK_SIZE: usize> {
    Reqwest(ResponseFuture),
    Body(body::SliceFuture),
}

pub struct IdsChunkFuture<const CHUNK_SIZE: usize>(IdsChunkFutureInternal<CHUNK_SIZE>);

impl<const CHUNK_SIZE: usize> IdsChunkFuture<CHUNK_SIZE> {
    pub fn new(request: ResponseFuture) -> Self {
        Self(IdsChunkFutureInternal::Reqwest(request))
    }
}

impl<const CHUNK_SIZE: usize> Future for IdsChunkFuture<CHUNK_SIZE> {
    type Output = Result<std::array::IntoIter<String, CHUNK_SIZE>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let state = &mut self.get_mut().0;
        match state {
            IdsChunkFutureInternal::Reqwest(request) => match Future::poll(Pin::new(request), cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(err)) => Poll::Ready(Err(Error::from_hyper(err))),
                Poll::Ready(Ok(resp)) if resp.status() != hyper::StatusCode::OK => {
                    Poll::Ready(Err(Error::from_status(resp.status())))
                }
                Poll::Ready(Ok(resp)) => {
                    *state = IdsChunkFutureInternal::Body(resp.into_body().into());
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
            IdsChunkFutureInternal::Body(fut) => match Future::poll(Pin::new(fut), cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(err)) => Poll::Ready(Err(Error::from_hyper(err))),
                Poll::Ready(Ok(slice)) => Poll::Ready(
                    serde_json::from_slice::<crate::de::IdArray<CHUNK_SIZE>>(&slice)
                        .map(Into::into)
                        .map_err(Error::from_json),
                ),
            },
        }
    }
}
