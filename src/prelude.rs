pub use crate::config::GlacierBuilder;
pub use crate::response::HttpResponse;
pub use crate::{CONTAIN_PATH, DIR_PATH, FILES_BUF, IP};
pub use glacier_macro::{glacier, main};
pub use h2::RecvStream;
pub use http::header::*;
pub use http::Request;

pub const TEXT_PLAIN: HeaderValue = HeaderValue::from_static("text/plain; charset=UTF-8");
pub const TEXT_HTML: HeaderValue = HeaderValue::from_static("text/html; charset=UTF-8");
pub const APPLICATION_JSON: HeaderValue =
    HeaderValue::from_static("application/json; charset=UTF-8");
