#![allow(unused)]

use std::ops::Add;
use std::sync::Mutex;
use std::{collections::HashMap, sync::LazyLock};

use glacier::{bytes::Bytes, client::Glacier, request::Request};
use glacier_macro::{glacier, main};

struct Routes<T>
where
    T: Fn(Request) -> (),
{
    routes: Vec<T>,
}

impl<T> Routes<T>
where
    T: Fn(Request) -> (),
{
    pub fn new(routes: Vec<T>) -> Self {
        Routes { routes }
    }
}

#[glacier(GET, "/hello")]
fn hello(req: Request) {
    println!("{:#?}", 1);
}

#[glacier(GET, "/byebye")]
fn byebye(req: Request) {}

// static XXX: LazyLock<u8> = LazyLock::new(|| {
//     let lock = unsafe { GLACIER_GET.lock() };
//     let mut lock = lock.unwrap();
//     lock.insert("/hello", hello);
//     1
// });

#[main]
fn main() {
    // Glacier::bind(3000);

    unsafe {
        let lock = GLACIER_GET.lock().unwrap();
        // let func = lock.get("/hello").unwrap();
        // func(Request::new(&"value".to_string()));
    }

    for i in 1..1 {
        println!("{:#?}", i);
    }
}
