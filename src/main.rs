#![allow(unused)]

use std::arch::x86_64;
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, Read};
use std::net::TcpStream;
use std::ops::Add;
use std::slice::from_raw_parts;
use std::sync::LazyLock;
use std::time;

use glacier::response::Response;
use glacier::{client::Glacier, request::Request};
use glacier_macro::{glacier, main};

#[glacier(GET, "/")]
fn default(req: Request, mut res: Response) {
    println!("{:#?}", "hello");
    res.respond();
}

#[main]
fn main() {
    let glacier = Glacier::bind(3000).unwrap();

    println!("{:#?}", "http://localhost:3000");
    glacier.run();
}
