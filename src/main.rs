#![allow(unused)]

use std::cell::RefCell;
use std::ops::Add;
use std::sync::LazyLock;

use glacier::response::Response;
use glacier::{client::Glacier, request::Request};
use glacier_macro::{glacier, main};

#[glacier(GET, "/hello")]
fn hello(req: Request, mut res: Response) {
    println!("{:#?}", 1);
    res.respond();
}

#[glacier(GET, "/byebye")]
fn byebye(req: Request, mut res: Response) {
    res.respond();
}

#[glacier(GET, "/")]
fn default(req: Request, mut res: Response) {
    println!("{:#?}", req);
    res.respond();
}

#[main]
fn main() {
    let mut client = Glacier::bind(3000).unwrap();
    println!("{:#?}", "http://localhost:3000");
    client.run();
}

#[test]
fn test() {
    let b = RefCell::new(1);
    // let a = b.borrow();
    let mut c = b.borrow_mut();

    *c = 10;
}
