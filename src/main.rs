#![allow(unused)]

use glacier::prelude::*;
use std::{fmt::Debug, fs::File, slice::from_raw_parts, thread};

async fn handle_error<E: Debug>(res: std::result::Result<HttpResponse, E>) -> HttpResponse {
    res.unwrap()
}

async fn handle_404(mut req: Request<RecvStream>) -> std::result::Result<HttpResponse, ()> {
    Ok(HttpResponse::Ok().body(TEXT_PLAIN, "404"))
}

#[glacier(POST, "/")]
async fn hello(mut req: Request<RecvStream>) -> std::result::Result<HttpResponse, ()> {
    let data = req.body_mut().data().await;

    let res = HttpResponse::Ok().body(TEXT_PLAIN, "Hello, World!");

    Ok(res)
}

// TODO 改成这种形式：
// async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//     match req.uri().path() {
//         "/" => [ip_middle, home],
//         "/user" => [user],
//         _ => [404],
//     }
// }

#[main]
async fn main() {
    let glacier = GlacierBuilder::bind(443)
        // .tls()
        // .log("debug", None)
        // .register_dir("/public")
        .server(routes)
        .build()
        .await;

    glacier.run().await.unwrap();
}
