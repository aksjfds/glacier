#![allow(unused)]

use std::{collections::HashMap, sync::LazyLock};

use dashmap::DashMap;
use tokio::sync::RwLock;

pub mod client;
pub mod error;
pub mod my_future;
pub mod prelude;
pub mod stream;
//
//
//
// use request::Request;
// use response::Response;
// use std::{
//     collections::HashMap,
//     sync::{LazyLock, Mutex},
// };
// pub static GLACIER_GET: LazyLock<Mutex<HashMap<&str, fn(Request<'_>, Response)>>> =
//     LazyLock::new(|| Mutex::new(HashMap::new()));
//
//
//
pub type Result<T> = core::result::Result<T, error::GlacierError>;
pub static FILES_BUF: LazyLock<DashMap<String, String>> = LazyLock::new(|| DashMap::new());
////
///
///
///
///
///

//
//
//
#[test]
pub fn test() {}
