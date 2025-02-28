#![allow(unused)]

use glacier::prelude::*;
use serde::Deserialize;

#[glacier(GET, "/")]
async fn basic(mut req: OneRequest) {
    println!("{:#?}", req.method());
    println!("{:#?}", req.version());
    req.respond_hello().await;
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    a: i32,
    b: i32,
}

#[glacier(GET, "/hello")]
async fn hello(mut req: OneRequest) {
    let params: QueryParams = req.get_params().unwrap();

    println!("{:#?}", params);

    req.respond_hello().await;

    // println!("{:#?}", req.headers.last().unwrap());
}

#[glacier(Static, "/public")]
async fn public(req: OneRequest) {}

#[main]
async fn main() -> Result<()> {
    let glacier = Glacier::bind(3000, routes).await.unwrap();
    println!("{:#?}", "http://localhost:3000");

    glacier.register_dir("/public").run().await.unwrap();

    Ok(())
}
