use bytes::Bytes;
use dashmap::DashMap;
use std::{net::IpAddr, sync::LazyLock, time::SystemTime};

pub mod client;
pub mod config;
pub mod error;
pub mod middles;
pub mod prelude;
pub mod response;
//
//
//
//
//
//
pub type Result<T> = core::result::Result<T, error::GlacierError>;
pub type Routes<T> = fn(http::Request<h2::RecvStream>) -> T;

/// 静态资源路径
pub static mut DIR_PATH: &'static str = "";

/// 静态资源缓存
pub static FILES_BUF: LazyLock<DashMap<String, Bytes>> = LazyLock::new(|| DashMap::new());

/// 访问者ip，用来记录上一次的访问时间戳
/// # Args
/// - `ip` - 访问者ip
/// - `systemtime` - 上一次访问时间戳
/// - `count` - 异常（访问间隔过短）次数
pub static IP: LazyLock<DashMap<IpAddr, (SystemTime, usize)>> = LazyLock::new(|| DashMap::new());

/// 路由（待定）
pub static mut CONTAIN_PATH: fn(&str) -> bool = {
    fn temp(_x: &str) -> bool {
        false
    }
    temp
};
//
//
//
//
