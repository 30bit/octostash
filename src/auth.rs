use hyper::{header::InvalidHeaderValue, http::HeaderValue};
use std::fmt::{self, Debug, Display};

#[repr(transparent)]
pub struct Error(InvalidHeaderValue);

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Auth(HeaderValue);

impl Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Auth {
    pub fn new(personal_access_token: &str) -> Result<Self, Error> {
        HeaderValue::from_str(&format!("Bearer {personal_access_token}"))
            .map(Self)
            .map_err(Error)
    }

    pub(crate) fn into_header_value(self) -> HeaderValue {
        self.0
    }
}

impl TryFrom<&str> for Auth {
    type Error = Error;

    #[inline]
    fn try_from(personal_access_token: &str) -> Result<Self, Self::Error> {
        Self::new(personal_access_token)
    }
}
