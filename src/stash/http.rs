use hyper::{
    body::Bytes,
    client::HttpConnector,
    header::{self, HeaderName, HeaderValue},
    http::{self, uri},
    Uri,
};
use hyper_tls::HttpsConnector;

pub type Hyper = hyper::Client<HttpsConnector<HttpConnector>>;

#[derive(Clone)]
pub struct Client {
    auth: HeaderValue,
    hyper: Hyper,
}

impl Client {
    pub fn new(auth: HeaderValue) -> Self {
        let mut https = HttpsConnector::new();
        https.https_only(true);
        Self {
            auth,
            hyper: hyper::Client::builder().http2_only(true).build(https),
        }
    }

    pub fn auth(&self) -> &HeaderValue {
        &self.auth
    }

    pub fn hyper(&self) -> &Hyper {
        &self.hyper
    }

    pub fn gists(&self) -> ClientForUri {
        ClientForUri {
            client: self,
            uri: unsafe {
                Uri::builder()
                    .scheme(uri::Scheme::HTTPS)
                    .authority("api.github.com")
                    .path_and_query("/gists")
                    .build()
                    .unwrap_unchecked()
            },
        }
    }

    pub fn gists_page(&self, per_page: usize, page: usize) -> ClientForUri {
        ClientForUri {
            client: self,
            uri: unsafe {
                Uri::builder()
                    .scheme(uri::Scheme::HTTPS)
                    .authority("api.github.com")
                    .path_and_query(format!("/gists?per_page={per_page}&page={page}"))
                    .build()
                    .unwrap_unchecked()
            },
        }
    }

    pub fn gist(&self, id: &str) -> Result<ClientForUri, http::Error> {
        Uri::builder()
            .scheme(uri::Scheme::HTTPS)
            .authority("api.github.com")
            .path_and_query(format!("/gists/{id}"))
            .build()
            .map(|uri| ClientForUri { client: self, uri })
    }
}

fn append_headers(
    request_builder: http::request::Builder,
    auth: HeaderValue,
) -> http::request::Builder {
    request_builder
        .header(header::USER_AGENT, unsafe {
            HeaderValue::from_maybe_shared_unchecked(Bytes::from_static(b"octostash"))
        })
        .header(header::AUTHORIZATION, auth)
        .header(header::ACCEPT, unsafe {
            HeaderValue::from_maybe_shared_unchecked(Bytes::from_static(
                b"application/vnd.github+json",
            ))
        })
        .header(
            unsafe { HeaderName::from_bytes(b"X-GitHub-Api-Version").unwrap_unchecked() },
            unsafe { HeaderValue::from_maybe_shared_unchecked(b"2022-11-28") },
        )
}

pub struct ClientForUri<'a> {
    client: &'a Client,
    uri: Uri,
}

impl<'a> ClientForUri<'a> {
    pub fn request(
        &self,
        method: hyper::Method,
        body: hyper::Body,
    ) -> Result<hyper::client::ResponseFuture, http::Error> {
        append_headers(
            hyper::Request::builder().uri(&self.uri),
            self.client.auth.clone(),
        )
        .method(method)
        .body(body)
        .map(|request| self.client.hyper.request(request))
    }

    pub fn into_request(
        self,
        method: hyper::Method,
        body: hyper::Body,
    ) -> Result<hyper::client::ResponseFuture, http::Error> {
        append_headers(
            hyper::Request::builder().uri(self.uri),
            self.client.auth.clone(),
        )
        .method(method)
        .body(body)
        .map(|request| self.client.hyper.request(request))
    }
}
