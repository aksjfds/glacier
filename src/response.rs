use http_body_util::Full;
use hyper::body::Bytes;

use crate::prelude::HyperResponse;

pub struct Response {
    builder: hyper::http::response::Builder,
    body: Option<Full<Bytes>>,
}

impl Response {
    pub fn new() -> Self {
        Self {
            builder: hyper::Response::builder(),
            body: None,
        }
    }

    #[allow(non_snake_case)]
    pub fn Ok() -> Self {
        Self {
            builder: hyper::Response::builder().status(200),
            body: None,
        }
    }

    pub fn status<T>(mut self, code: T) -> Self
    where
        T: TryInto<hyper::StatusCode>,
        <T as TryInto<hyper::StatusCode>>::Error: Into<hyper::http::Error>,
    {
        self.builder = self.builder.status(code);
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<hyper::header::HeaderName>,
        <K as TryInto<hyper::header::HeaderName>>::Error: Into<hyper::http::Error>,
        V: TryInto<hyper::header::HeaderValue>,
        <V as TryInto<hyper::header::HeaderValue>>::Error: Into<hyper::http::Error>,
    {
        self.builder = self.builder.header(key, value);
        self
    }

    pub fn body(mut self, body: impl Into<Full<Bytes>>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn json(mut self, data: impl serde::Serialize) -> Self {
        use hyper::header::CONTENT_TYPE;

        self.builder = self
            .builder
            .header(CONTENT_TYPE, "application/json; charset=UTF-8");
        self.body = serde_json::to_vec(&data).map(Into::into).ok();

        self
    }
}

impl TryFrom<Response> for HyperResponse {
    type Error = hyper::http::Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        match value.body {
            Some(body) => value.builder.body(body),
            None => {
                let body = "".into();
                value.builder.body(body)
            }
        }
    }
}

impl<T> From<T> for Response
where
    T: serde::Serialize,
{
    fn from(value: T) -> Self {
        Response::Ok().json(value)
    }
}
