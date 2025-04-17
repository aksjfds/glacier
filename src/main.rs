#![allow(unused)]

use glacier::prelude::*;
use http::Method;
use std::{
    collections::HashSet, fmt::Debug, fs::File, future::Future, slice::from_raw_parts,
    sync::LazyLock, thread,
};

fn rate_limit(mut req: HttpRequest, limit: usize) -> Result<HttpRequest, u16> {
    if req.counter() < limit {
        Ok(req)
    } else {
        Err(0)
    }
}

async fn middle(mut req: HttpRequest) -> Result<HttpRequest, u16> {
    Ok(req)
}

async fn hello(mut req: HttpRequest) -> Result<HttpResponse, u16> {
    Ok(HttpResponse::Ok())
}

async fn router(mut req: HttpRequest) -> HttpResponse {
    let res = match req.req.uri().path() {
        // "/user" => Ok(req).filter(middle).map(hello),
        "/user" => {
            let a = req
                .filter_with(|req| rate_limit(req, 10))
                .unwrap()
                .async_filter(|req| middle(req))
                .await
                .unwrap()
                .async_apply(|x| hello(x))
                .await;
            a
        }
        _ => todo!(),
    };

    match res {
        Ok(res) => res,
        Err(_) => HttpResponse::Ok(),
    }
}

#[tokio::main]
async fn main() {
    let glacier = GlacierBuilder::bind(("0.0.0.0", 443))
        .server(router)
        .build()
        .await;

    glacier.run().await.unwrap();

    thread::park();
}
