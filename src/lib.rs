mod client;
mod config;
mod error;
mod middles;
pub mod prelude;
mod request;
mod response;
mod request_ext;
//
//
//
//
//
//
pub type Result<T> = core::result::Result<T, error::GlacierError>;
pub type Routes<T> = fn(request::HttpRequest) -> T;
