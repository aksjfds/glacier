use bytes::Bytes;
use http::{header::CONTENT_TYPE, HeaderName, HeaderValue};
use serde::Serialize;

use crate::prelude::{APPLICATION_JSON, TEXT_HTML, TEXT_PLAIN};

// 标志
pub struct HttpResponse {
    pub(crate) builder: http::response::Builder,
    pub data: Option<Bytes>,
}

impl HttpResponse {
    #[allow(non_snake_case)]
    pub fn Ok() -> Self {
        HttpResponse {
            builder: http::Response::builder(),
            data: None,
        }
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        <K as TryInto<HeaderName>>::Error: Into<http::Error>,
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.header(key, value);
        self
    }

    pub fn json(mut self, x: impl Serialize) -> Self {
        self.builder = self.builder.header(CONTENT_TYPE, APPLICATION_JSON);
        self.data = serde_json::to_vec(&x).map(Into::into).ok();

        self
    }

    pub fn easy_json(mut self, data: impl AsRef<[u8]> + Send + 'static) -> Self {
        self.builder = self.builder.header(CONTENT_TYPE, APPLICATION_JSON);
        self.data = Some(Bytes::from_owner(data));

        self
    }

    pub fn plain(mut self, data: impl AsRef<[u8]> + Send + 'static) -> Self {
        self.builder = self.builder.header(CONTENT_TYPE, TEXT_PLAIN);
        self.data = Some(Bytes::from_owner(data));

        self
    }

    pub fn html(mut self, data: impl AsRef<[u8]> + Send + 'static) -> Self {
        self.builder = self.builder.header(CONTENT_TYPE, TEXT_HTML);

        self.data = Some(Bytes::from_owner(data));

        self
    }

    pub fn body<V>(mut self, content_type: V, data: impl Into<Bytes>) -> Self
    where
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.header(CONTENT_TYPE, content_type);

        self.data = Some(data.into());
        self
    }

    pub fn easy_body(mut self, data: impl Into<Bytes>) -> Self {
        self.data = Some(data.into());
        self
    }
}
