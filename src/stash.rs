mod error;
mod futures;
mod http;

pub use error::Error;
pub use hyper::StatusCode;

use crate::{de, ser, Auth};
use futures::ids_chunk::IdsChunkFuture;
use futures_core::{Future, Stream};
use std::{
    fmt::{self, Debug},
    hint::unreachable_unchecked,
    mem,
    pin::Pin,
    task::{self, Poll},
};

#[derive(Clone)]
#[repr(transparent)]
pub struct Stash(http::Client);

impl Debug for Stash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stash")
            .field("auth", &self.0.auth())
            .field("hyper", &self.0.hyper())
            .finish()
    }
}

impl Stash {
    pub fn new(auth: Auth) -> Self {
        Stash(http::Client::new(auth.into_header_value()))
    }

    pub fn auth(&self) -> &Auth {
        unsafe { mem::transmute(self.0.auth()) }
    }

    pub async fn insert(&self, value: &str) -> Result<String, Error> {
        let resp = self
            .0
            .gists()
            .into_request(
                hyper::Method::POST,
                unsafe { serde_json::to_vec(&ser::Files::new(value, 0)).unwrap_unchecked() }.into(),
            )
            .map_err(Error::from_http)?
            .await
            .map_err(Error::from_hyper)?;
        if resp.status() == hyper::StatusCode::CREATED {
            serde_json::from_slice::<de::Id>(
                &futures::body::SliceFuture::from(resp.into_body())
                    .await
                    .map_err(Error::from_hyper)?,
            )
            .map(String::from)
            .map_err(Error::from_json)
        } else {
            Err(Error::from_status(resp.status()))
        }
    }

    pub async fn get(&self, id: &str) -> Result<String, Error> {
        let resp = self
            .0
            .gist(id)
            .map_err(Error::from_http)?
            .into_request(hyper::Method::GET, hyper::Body::empty())
            .map_err(Error::from_http)?
            .await
            .map_err(Error::from_hyper)?;
        if resp.status() == hyper::StatusCode::OK {
            serde_json::from_slice::<de::Files>(
                &futures::body::SliceFuture::from(resp.into_body())
                    .await
                    .map_err(Error::from_hyper)?,
            )
            .map(String::from)
            .map_err(Error::from_json)
        } else {
            Err(Error::from_status(resp.status()))
        }
    }

    pub async fn set(&self, id: &str, value: &str) -> Result<(), Error> {
        let client = self.0.gist(id).map_err(Error::from_http)?;
        let current_len = {
            let resp = client
                .request(hyper::Method::GET, hyper::Body::empty())
                .map_err(Error::from_http)?
                .await
                .map_err(Error::from_hyper)?;
            if resp.status() == hyper::StatusCode::OK {
                serde_json::from_slice::<de::FilesLen>(
                    &futures::body::SliceFuture::from(resp.into_body())
                        .await
                        .map_err(Error::from_hyper)?,
                )
                .map_err(Error::from_json)?
                .into()
            } else {
                return Err(Error::from_status(resp.status()));
            }
        };
        let resp = client
            .into_request(
                hyper::Method::PATCH,
                unsafe {
                    serde_json::to_vec(&ser::Files::new(value, current_len)).unwrap_unchecked()
                }
                .into(),
            )
            .map_err(Error::from_http)?
            .await
            .map_err(Error::from_hyper)?;
        if resp.status() == hyper::StatusCode::OK {
            Ok(())
        } else {
            Err(Error::from_status(resp.status()))
        }
    }

    pub async fn remove(&self, id: &str) -> Result<(), Error> {
        let resp = self
            .0
            .gist(id)
            .map_err(Error::from_http)?
            .into_request(hyper::Method::DELETE, hyper::Body::empty())
            .map_err(Error::from_http)?
            .await
            .map_err(Error::from_hyper)?;
        if resp.status() == hyper::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(Error::from_status(resp.status()))
        }
    }

    fn ids_chunk_future(&self, index: usize) -> Result<IdsChunkFuture<IDS_CHUNK_SIZE>, Error> {
        self.0
            .gists_page(IDS_CHUNK_SIZE, index)
            .request(hyper::Method::GET, hyper::Body::empty())
            .map(IdsChunkFuture::new)
            .map_err(Error::from_http)
    }

    pub fn ids(&self) -> Ids {
        Ids(match self.ids_chunk_future(1) {
            Ok(fut) => IdsInternal::NotExhausted {
                stash: self,
                next_index: 2,
                current_future: fut,
            },
            Err(err) => IdsInternal::Err(err),
        })
    }
}

const IDS_CHUNK_SIZE: usize = 100;

#[repr(transparent)]
pub struct IdsChunk(std::array::IntoIter<String, IDS_CHUNK_SIZE>);

impl IdsChunk {
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn as_slice(&self) -> &[String] {
        self.0.as_slice()
    }
}

impl IntoIterator for IdsChunk {
    type Item = String;

    type IntoIter = std::array::IntoIter<String, IDS_CHUNK_SIZE>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0
    }
}

enum IdsInternal<'a> {
    Exhausted,
    Err(Error),
    NotExhausted {
        stash: &'a Stash,
        next_index: usize,
        current_future: IdsChunkFuture<IDS_CHUNK_SIZE>,
    },
}

pub struct Ids<'a>(IdsInternal<'a>);

impl<'a> Stream for Ids<'a> {
    type Item = Result<IdsChunk, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let state = &mut self.get_mut().0;
        match state {
            IdsInternal::Exhausted => Poll::Ready(None),
            IdsInternal::Err(_) => match mem::replace(state, IdsInternal::Exhausted) {
                IdsInternal::Err(err) => Poll::Ready(Some(Err(err))),
                _ => unsafe { unreachable_unchecked() },
            },
            IdsInternal::NotExhausted {
                stash,
                next_index,
                current_future,
            } => match Future::poll(Pin::new(current_future), cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(err)) => {
                    *state = IdsInternal::Exhausted;
                    Poll::Ready(Some(Err(err)))
                }
                Poll::Ready(Ok(chunk)) => {
                    if chunk.len() < IDS_CHUNK_SIZE {
                        *state = IdsInternal::Exhausted;
                        if chunk.len() == 0 {
                            return Poll::Ready(None);
                        }
                    } else {
                        match stash.ids_chunk_future(*next_index) {
                            Ok(fut) => {
                                *current_future = fut;
                                *next_index += 1;
                            }
                            Err(err) => *state = IdsInternal::Err(err),
                        }
                    }
                    Poll::Ready(Some(Ok(IdsChunk(chunk))))
                }
            },
        }
    }
}
