#![allow(unused)]

use std::fs::File;
use std::future::Future;
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::{TcpListener, TcpStream};

use crate::{prelude::*, FILES_BUF};
///
///
///
///
///
///
static R: AtomicU64 = AtomicU64::new(0);
type Routes<T> = fn(OneRequest) -> T;

pub struct Glacier<T> {
    listener: TcpListener,
    routes: Routes<T>,
}

impl<T> Glacier<T>
where
    T: Future<Output = Result<OneRequest>> + Send + Sync + 'static,
{
    /* ---------------------------------- // 绑定 --------------------------------- */
    pub async fn bind(port: u16, routes: Routes<T>) -> Result<Self> {
        let addr = ("0.0.0.0", port);
        let listener = TcpListener::bind(addr).await?;

        Ok(Glacier { listener, routes })
    }

    pub fn register_dir(self, dir_path: &str) -> Self {
        let entries = std::fs::read_dir(&dir_path[1..]).unwrap();
        for entry in entries {
            let entry = entry.unwrap();

            let file_path = entry.path().to_string_lossy().to_string();
            let mut f = File::open(entry.path()).unwrap();

            let mut buf = String::with_capacity(1024);
            f.read_to_string(&mut buf).unwrap();

            FILES_BUF.insert(file_path, buf);
        }

        self
    }

    /* -------------------------------- // 映射文件夹 -------------------------------- */

    /* ---------------------------------- // 运行 --------------------------------- */
    pub async fn run(self) -> Result<()> {
        let routes = self.routes;
        let listener = self.listener;

        let srv = async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();

                // R.fetch_add(1, Ordering::Relaxed);
                // println!("{:#?}", R.load(Ordering::Relaxed));

                tokio::spawn(Glacier::handle_connection(stream, routes));
            }
        };

        tokio::spawn(srv).await.unwrap();

        Ok(())
    }

    /* --------------------------------- // 处理连接 -------------------------------- */
    async fn handle_connection(stream: TcpStream, routes: Routes<T>) -> Result<()> {
        let mut glacier_stream = GlacierStream::new(stream);
        loop {
            let mut one_req = match glacier_stream.to_req().await {
                Ok(one_req) => one_req,
                Err(e) => {
                    if !matches!(e.kind(), Kind::EofErr) {
                        println!("{:#?}", e);
                    }
                    return Ok(());
                }
            };

            /* ----------------------------------- // ----------------------------------- */
            one_req = match routes(one_req).await {
                Ok(one_req) => one_req,
                Err(e) => {
                    println!("{:#?}", e);
                    return Ok(());
                }
            };
            glacier_stream = one_req.to_stream();
        }
    }
}
//
//
//
//
//
//
/* --------------------------------- // test -------------------------------- */
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
