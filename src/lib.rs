pub mod bytes;
pub mod client;
pub mod error;
pub mod request;
pub mod response;
//
//
//
use request::Request;
use response::Response;
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
pub static GLACIER_GET: LazyLock<Mutex<HashMap<&str, fn(Request<'_>, Response)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
//
//
//
pub type Result<T> = core::result::Result<T, error::GlacierError>;
//
//
//
#[test]
pub fn test() {}
