#![allow(unused)]

use std::{io::Read, sync::RwLock};

use bytes::BytesMut;
use glacier::prelude::*;

#[glacier(GET, "/")]
async fn basic(mut req: OneRequest) {
    req.respond_hello().await;
}

#[glacier(POST, "/hello")]
async fn hello(mut req: OneRequest) {
    println!("{:#?}", req.last_path());
    req.respond_hello().await;

    // println!("{:#?}", req.headers.last().unwrap());
}

#[glacier(Static, "/public")]
async fn public(mut req: OneRequest) {}

#[main]
async fn main() {
    let glacier = Glacier::bind(3000, routes).await.unwrap();
    println!("{:#?}", "http://localhost:3000");
    glacier
        .register_dir("/public")
        .unwrap()
        .run()
        .await
        .unwrap();
}
