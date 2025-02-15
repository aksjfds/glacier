// #![allow(unused)]

use std::{fs::File, io::Read, thread};

use glacier::prelude::*;

#[glacier(GET, "/")]
fn hello(_req: Request, mut res: Response) {
    println!("{:#?}", "hello");

    let mut f = File::open("../index.html").unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data).unwrap();
    res.body = data;

    res.respond();
}

#[main]
fn main() {
    let glacier = Glacier::bind(3000, routes).unwrap();

    let handle = thread::spawn(move || glacier.run());

    println!("{:#?}", "http://localhost:3000");

    handle.join().unwrap();
}

#[test]
fn test() {}
