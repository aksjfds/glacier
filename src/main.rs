#![allow(unused)]

use std::{io::Write, mem::transmute, ptr::read, time};
use tokio::{fs::File as AsyncFile, net::TcpStream};

use glacier::{prelude::*, stream::glacier_stream::OneRequest};

#[glacier(GET, "/")]
async fn basic(mut req: OneRequest) {
    // println!("{:#?}", "hello");

    // println!("{:#?}", req.query_header("Host"));
    req.respond().await;
}

#[glacier(POST, "/post")]
async fn basic_post(req: OneRequest) {
    // println!("{:#?}", "hello");

    // println!("{:#?}", req.headers.last().unwrap());
}

#[main]
async fn main() {
    let glacier = Glacier::bind(3000, (routes, match_route)).await.unwrap();

    println!("{:#?}", "http://localhost:3000");
    glacier.run().await.unwrap();
}
