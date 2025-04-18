use crate::prelude::HyperResponse;

pub struct Response {
    builder: hyper::http::response::Builder,
    body: Option<http_body_util::Full<hyper::body::Bytes>>,
}

impl Response {
    pub fn new(body: impl Into<http_body_util::Full<hyper::body::Bytes>>) -> HyperResponse {
        hyper::Response::new(body.into())
    }

    #[allow(non_snake_case)]
    pub fn Ok() -> Self {
        Self {
            builder: hyper::Response::builder().status(200),
            body: None,
        }
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

    pub fn body(mut self, body: impl Into<http_body_util::Full<hyper::body::Bytes>>) -> Self {
        self.body = Some(body.into());
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
