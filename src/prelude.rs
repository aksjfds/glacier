use std::sync::atomic::{AtomicU64, AtomicU8};

pub use crate::config::GlacierBuilder;
pub use crate::request::HttpRequest;
pub use crate::request_ext::*;
pub use crate::result_ext::*;
pub use crate::response::HttpResponse;
pub use glacier_macro::glacier;
pub use http::header::*;

pub const TEXT_PLAIN: HeaderValue = HeaderValue::from_static("text/plain; charset=UTF-8");
pub const TEXT_HTML: HeaderValue = HeaderValue::from_static("text/html; charset=UTF-8");
pub const APPLICATION_JSON: HeaderValue =
    HeaderValue::from_static("application/json; charset=UTF-8");
pub const APPLICATION_JS: HeaderValue =
    // HeaderValue::from_static("application/javascript; charset=UTF-8");
    HeaderValue::from_static("application/javascript");

pub struct GlobalVal {
    pub error_count: AtomicU8,
    pub last_time: AtomicU64,
}

impl GlobalVal {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let last_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        GlobalVal {
            error_count: AtomicU8::new(0),
            last_time: AtomicU64::new(last_time),
        }
    }

    pub fn log(&self, visit_limit: u64, error_limit: u8) -> u8 {
        use std::sync::atomic::Ordering;
        use std::time::{SystemTime, UNIX_EPOCH};

        let current = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let last_time = self.last_time.swap(current, Ordering::Relaxed);
        let sub = match current > last_time {
            true => current - last_time,
            false => last_time - current,
        };
        if sub < visit_limit {
            let error_count = self.error_count.fetch_add(1, Ordering::Relaxed);

            if error_count > error_limit {
                return error_count;
            }
        }

        0
    }
}
