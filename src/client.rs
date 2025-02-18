use std::future::Future;

use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use crate::my_future::{MyFuture, MyFutureTasks, PollStream};
use crate::Result;
use crate::{request::Request, response::Response};

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

    async fn handle_connection(mut stream: TcpStream, routes: Routes<T>) {
        let mut buf = Vec::with_capacity(1024);

        // 获取完整 request
        loop {
            match stream.read_buf(&mut buf).await {
                Err(e) => {
                    eprintln!("Error reading from stream: {}", e);
                }
                Ok(0) => {
                    if let Ok(0) = stream.read_buf(&mut buf).await {
                        return;
                    }
                }
                _ => {}
            }

            if buf.ends_with(b"\r\n") {
                break;
            }
        }

        // unsafe {
        //     println!("{:#?}", String::from_utf8_unchecked(buf.clone()));
        // }

        // 整理
        let req = match Request::parse(buf.to_vec()) {
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

#[test]
fn test() {
    use std::io::Write;

    let mut a = std::net::TcpStream::connect("www.localhost:3000").unwrap();
    a.write_all(b"").unwrap();
}
