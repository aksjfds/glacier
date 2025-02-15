// #![allow(unused)]

use std::future::Future;

use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::Result;
use crate::{bytes::Bytes, request::Request, response::Response};

type Routes<T> = fn(Request, Response) -> T;

pub struct Glacier<T> {
    listener: TcpListener,
    req_buf: Mutex<Bytes>,
    routes: Routes<T>,
}

impl<T> Glacier<T>
where
    T: Future<Output = ()>,
{
    pub async fn bind(port: u16, routes: Routes<T>) -> Result<Glacier<T>> {
        let addr = ("127.0.0.1", port);
        let listener = TcpListener::bind(addr).await?;

        Ok(Glacier {
            listener,
            req_buf: Mutex::new(Bytes::with_capacity(32)),
            routes,
        })
    }

    pub async fn run(&self) {
        'run: loop {
            match self.listener.accept().await {
                Err(_) => todo!(),
                Ok((mut stream, _addr)) => {
                    let mut buf = self.req_buf.lock().await;

                    // 获取完整 request
                    loop {
                        if buf.is_end() {
                            break;
                        }

                        match stream.read(buf.get_free_space()).await {
                            Err(e) => {
                                eprintln!("Error reading from stream: {}", e);
                                continue 'run;
                            }
                            Ok(0) => {
                                // stream.shutdown(std::net::Shutdown::Both);
                                continue 'run;
                            }
                            Ok(len) => buf.modify_len(len),
                        }
                    }

                    // 整理
                    let req = match Request::parse(&buf) {
                        Ok(req) => req,
                        Err(e) => {
                            eprintln!("Error when parsing request: {}", e);
                            Response::bad_request(stream).await;
                            continue 'run;
                        }
                    };

                    let res = Response::hello(stream);
                    let func = (self.routes)(req, res);
                    func.await;
                }
            }
            self.req_buf.lock().await.clear();
        }
    }
}
