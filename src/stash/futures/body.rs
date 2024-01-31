use hyper::{
    body::{Bytes, HttpBody},
    Body,
};
use std::{
    future::Future,
    mem,
    ops::Deref,
    pin::Pin,
    task::{self, Poll},
};

enum SliceInternal {
    Empty,
    Bytes(Bytes),
    Vec(Vec<u8>),
}

impl SliceInternal {
    fn take(&mut self) -> Self {
        mem::replace(self, Self::Empty)
    }
}

pub struct Slice(SliceInternal);

impl Deref for Slice {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            SliceInternal::Empty => &[],
            SliceInternal::Bytes(bytes) => bytes,
            SliceInternal::Vec(vec) => vec,
        }
    }
}

pub struct SliceFuture {
    body: Body,
    buf: SliceInternal,
}

impl From<Body> for SliceFuture {
    fn from(body: Body) -> Self {
        Self {
            body,
            buf: SliceInternal::Empty,
        }
    }
}

impl Future for SliceFuture {
    type Output = hyper::Result<Slice>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match HttpBody::poll_data(Pin::new(&mut self.body), cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(Ok(Slice(self.buf.take()))),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Err(err)),
            Poll::Ready(Some(Ok(next))) => {
                self.buf = match self.buf.take() {
                    SliceInternal::Empty => SliceInternal::Bytes(next),
                    SliceInternal::Bytes(prev) => {
                        let mut vec = Vec::with_capacity(
                            prev.len()
                                + next.len()
                                + (self.body.size_hint().lower() as usize).min(1024 * 16),
                        );
                        vec.extend_from_slice(&prev);
                        vec.extend_from_slice(&next);
                        SliceInternal::Vec(vec)
                    }
                    SliceInternal::Vec(mut vec) => {
                        vec.extend_from_slice(&next);
                        SliceInternal::Vec(vec)
                    }
                };
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
