use hyper::{http, StatusCode};
use std::fmt::{self, Debug, Display};

pub enum Internal {
    Http(http::Error),
    Hyper(hyper::Error),
    Json(serde_json::Error),
    Status(StatusCode),
}

pub struct Error(Internal);

impl Error {
    pub(crate) fn from_http(err: http::Error) -> Self {
        Self(Internal::Http(err))
    }

    pub(crate) fn from_hyper(err: hyper::Error) -> Self {
        Self(Internal::Hyper(err))
    }

    pub(crate) fn from_json(err: serde_json::Error) -> Self {
        Self(Internal::Json(err))
    }

    pub(crate) fn from_status(err: StatusCode) -> Self {
        Self(Internal::Status(err))
    }

    pub fn status(&self) -> Option<StatusCode> {
        if let Internal::Status(code) = self.0 {
            Some(code)
        } else {
            None
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Internal::Hyper(err) => Debug::fmt(err, f),
            Internal::Http(err) => Debug::fmt(err, f),
            Internal::Json(err) => Debug::fmt(err, f),
            Internal::Status(err) => Debug::fmt(err, f),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Internal::Hyper(err) => Display::fmt(err, f),
            Internal::Http(err) => Display::fmt(err, f),
            Internal::Json(err) => Display::fmt(err, f),
            Internal::Status(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            Internal::Http(err) => Some(err),
            Internal::Hyper(err) => Some(err),
            Internal::Json(err) => Some(err),
            Internal::Status(_) => None,
        }
    }
}
