// #![allow(unused)]

use std::future::Future;

use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use crate::Result;
use crate::{bytes::Bytes, request::Request, response::Response};

type Routes<T> = fn(Request, Response) -> T;

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

    pub async fn run(&self) -> Result<()> {
        let routes = self.routes;

        loop {
            let (stream, _) = self.listener.accept().await?;

            tokio::spawn(Glacier::handle_connection(stream, routes));
        }
    }

    async fn handle_connection(mut stream: TcpStream, routes: Routes<T>) {
        let mut buf = Bytes::with_capacity(1024);

        // 获取完整 request
        loop {
            match stream.read(buf.get_free_space()).await {
                Err(e) => {
                    eprintln!("Error reading from stream: {}", e);
                }
                Ok(0) => {
                    // stream.shutdown(std::net::Shutdown::Both);
                }
                Ok(len) => buf.modify_len(len),
            }

            if buf.is_end() {
                break;
            }
        }

        // 整理
        let req = match Request::parse(&buf) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Error when parsing request: {}", e);
                Response::bad_request(stream).await;
                return;
            }
        };

        let res = Response::hello(stream);
        let func = routes(req, res);
        func.await;
    }
}
