use crate::prelude::{GlacierError, OneRequest, Result, IP};
use std::time::SystemTime;
///
///
///
///
///

/// ip限制中间件，限制某ip连续两次访问的最小间隔，超过最小间隔次数过多则抛出错误
/// # Args
/// - `min_interval` - 连续两次访问的最小间隔
/// - `times` - 可允许超过最小间隔次数
pub async fn ip_middle(req: OneRequest, min_interval: u128, times: usize) -> Result<OneRequest> {
    let mut ip_entry = IP.entry(req.addr).or_insert((SystemTime::now(), 0));
    let (last_time, count) = ip_entry.value_mut();

    if *count > times {
        Err(GlacierError::build_option("ip访问次数过快"))?
    }

    let new_time = SystemTime::now();
    let time_interval = new_time.duration_since(*last_time).unwrap().as_millis();

    if time_interval < min_interval {
        *count += 1;
    } else {
        *last_time = new_time;
    }

    Ok(req)
}
