#![allow(unused)]

use bytes::Buf;
use std::future::Future;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use crate::my_future::{MyFuture, MyFutureTasks, PollStream};
use crate::prelude::*;
use crate::stream::glacier_stream::{GlacierStream, OneRequest};
///
///
///
///
///
///
type Routes<T> = (fn(OneRequest) -> T, fn(&str) -> bool);

pub struct Glacier<T> {
    listener: TcpListener,
    routes: Routes<T>,
}
unsafe impl<T> Sync for Glacier<T> {}
unsafe impl<T> Send for Glacier<T> {}

impl<T> Glacier<T>
where
    T: Future<Output = ()> + Send + Sync + 'static,
{
    pub async fn bind(port: u16, routes: Routes<T>) -> Result<Glacier<T>> {
        let addr = ("127.0.0.1", port);
        let listener = TcpListener::bind(addr).await?;

        Ok(Glacier { listener, routes })
    }

    pub async fn run(self) -> Result<()> {
        let routes = self.routes;
        let listener = self.listener;

        let mut poll_stream = PollStream::with_capacity(64, listener);

        loop {
            let streams = poll_stream.poll_some().await;

            let streams: Vec<_> = streams
                .into_iter()
                .filter_map(|item| match item {
                    Ok((stream, _)) => {
                        Some(MyFuture::new(Glacier::handle_connection(stream, routes)))
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        None
                    }
                })
                .collect();

            let tasks = MyFutureTasks::new(streams);
            tokio::spawn(tasks);
        }
    }

    async fn handle_connection(mut stream: TcpStream, routes: Routes<T>) -> Result<()> {
        let (route, match_route) = routes;

        let mut glacier_stream = GlacierStream::new(stream);
        glacier_stream.init().await.unwrap();

        let one_req = glacier_stream.build_request().unwrap();

        /* --------------------------------- // 判断路径 -------------------------------- */
        if !match_route(one_req.path()) {
            println!("路径不存在: {:#?}", one_req.path());
            return Ok(());
        }

        /* ----------------------------------- // ----------------------------------- */
        let func = route(one_req);
        func.await;

        Ok(())
    }
}

#[test]
fn test1() {
    use std::io::Write;

    let mut a = std::net::TcpStream::connect("www.localhost:3000").unwrap();
    a.write_all(b"GET / HTTP/1.1\r\n1:11").unwrap();
}

#[tokio::test]
async fn test2() {
    use futures::future::join_all;
    use tokio::io::AsyncWriteExt;

    let tasks: Vec<_> = (0..1000)
        .map(|_i| async {
            let mut stream = TcpStream::connect("www.localhost:3000").await.unwrap();
            stream.write_all(b"").await.unwrap();
        })
        .collect();

    join_all(tasks).await;
}
