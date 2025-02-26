#![allow(unused)]

use std::{
    alloc::{alloc, Layout},
    future::Future,
    io::IoSlice,
    pin::Pin,
    ptr,
    str::{from_utf8, from_utf8_unchecked},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use bytes::{BufMut, BytesMut};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

use crate::{error::GlacierError, Result, FILES_BUF};

use super::request::{RequestHeader, RequestLine};
///
///
///
///

/* ---------------------------- // GlacierStream ---------------------------- */
pub struct GlacierStream {
    stream: TcpStream,
    buf: BytesMut,
}

static R: AtomicU64 = AtomicU64::new(0);

impl GlacierStream {
    pub fn new(stream: TcpStream) -> Self {
        GlacierStream {
            stream,
            buf: BytesMut::with_capacity(1024),
        }
    }

    // 读取 请求行 和 请求头 的全部数据
    pub async fn read(&mut self) -> Result<Vec<usize>> {
        /* --------------------------------- // 准备工作 -------------------------------- */
        let mut buf = &mut self.buf;
        buf.clear();
        let mut pos = Vec::with_capacity(10);
        pos.push(0);

        // 读取数据到buf, 然后标记buf上的位置
        loop {
            let mut len =
                timeout(Duration::from_secs(10), self.stream.read_buf(&mut buf)).await??;
            if 0 == len {
                // R.fetch_add(1, Ordering::Relaxed);
                // println!("close: {:#?}", R.load(Ordering::Relaxed));
                Err(GlacierError::build_eof("Connection close"))?
            }

            for i in (buf.len() - len..buf.len()).step_by(2) {
                match buf[i] {
                    b'\r' => {
                        if let Some(x @ b'\n') = buf.get(i + 1) {
                            pos.push(i + 2);
                        }
                    }
                    b'\n' => {
                        if buf[i - 1] == b'\r' {
                            pos.push(i + 1);
                        }
                    }
                    _ => {}
                }
            }

            // 判断是否到了 请求体
            let len = pos.len();
            if len >= 2 && pos[len - 1] - pos[len - 2] == 2 {
                pos.pop();
                break;
            }
        }

        Ok(pos)
    }

    pub async fn to_req(mut self) -> Result<OneRequest> {
        let pos = self.read().await?;
        let mut lines = pos
            .as_slice()
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, pos_temp)| [pos[i - 1], *pos_temp]);

        // 请求行处理
        let request_line = lines.next().expect("lines为空");
        let request_line_pos = RequestLine::parse(&self.buf, request_line)?;

        // // 请求头处理
        let request_headers_pos: Vec<[usize; 3]> = lines
            .map(|line| RequestHeader::parse(&self.buf, line))
            .collect::<Result<Vec<_>>>()
            .unwrap();

        Ok(OneRequest {
            stream: self.stream,
            buf: self.buf,
            request_line_pos,
            request_headers_pos,
        })
    }
}

// /* ------------------------------ // OneRequest ----------------------------- */
pub struct OneRequest {
    pub stream: TcpStream,
    pub buf: BytesMut,
    pub request_line_pos: [usize; 4],
    pub request_headers_pos: Vec<[usize; 3]>,
}

impl OneRequest {
    pub fn request_line(&self) -> &str {
        let request_line = &self.buf[self.request_line_pos[0]..self.request_line_pos[3] - 1];

        unsafe { from_utf8_unchecked(request_line) }
    }

    pub fn path(&self) -> &str {
        let uri = &self.buf[self.request_line_pos[1]..self.request_line_pos[2] - 1];
        let uri = unsafe { from_utf8_unchecked(uri) };
        match uri.rfind("/") {
            Some(0) => "/",
            Some(pos) => &uri[..pos],
            None => uri,
        }
    }

    pub fn last_path(&self) -> &str {
        let uri = &self.buf[self.request_line_pos[1]..self.request_line_pos[2] - 1];
        let uri = unsafe { from_utf8_unchecked(uri) };
        if let Some(pos) = uri.rfind("/") {
            &uri[pos..]
        } else {
            uri
        }
    }

    pub fn query_header(&self, query_key: &str) -> Option<&str> {
        let headers = &self.request_headers_pos;
        for header in headers {
            let key = unsafe { from_utf8_unchecked(&self.buf[header[0]..header[1]]) };
            if query_key == key {
                let value = unsafe { from_utf8_unchecked(&self.buf[header[1] + 2..header[2] - 2]) };

                return Some(value);
            }
        }

        None
    }

    pub fn to_stream(mut self) -> GlacierStream {
        GlacierStream {
            stream: self.stream,
            buf: self.buf,
        }
    }

    // pub async fn respond(&mut self, mut data: impl IntoData) -> Result<()> {
    //     let res = format!(
    //         "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n",
    //         data.len().await,
    //     );
    //     self.stream.write_all(res.as_bytes()).await;

    //     let mut buf = [0; 1024];
    //     loop {
    //         match data.read(&mut buf).await {
    //             Ok(0) => break,
    //             Ok(len) => {
    //                 self.stream.write(&buf[..len]).await?;
    //             }
    //             Err(e) => {
    //                 println!("{:#?}", e);
    //                 return Ok(());
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    pub async fn respond(&mut self, file_path: String) -> Result<()> {
        // 获取缓存中的文件内容
        let buf = FILES_BUF
            .get(&file_path)
            .ok_or_else(|| GlacierError::build_req("没找到文件"))?;

        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n",
            buf.len()
        );

        // 使用缓冲写入合并 header 和 body
        let bufs = &[
            IoSlice::new(header.as_bytes()),
            IoSlice::new(buf.as_bytes()),
        ];
        let mut writer = &mut self.stream;
        writer.write_vectored(bufs).await?;
        writer.flush().await?;

        Ok(())
    }

    pub async fn respond_hello(&mut self) {
        let res =
            "HTTP/1.1 200 OK\r\nContent-Length: 13\r\nConnection: keep-alive\r\n\r\nHello, world!";
        self.stream.write_all(res.as_bytes()).await;
    }
}

pub trait IntoData: AsyncRead + Unpin {
    fn len(&self) -> impl Future<Output = u64>;
}

impl IntoData for File {
    async fn len(&self) -> u64 {
        self.metadata().await.unwrap().len()
    }
}
