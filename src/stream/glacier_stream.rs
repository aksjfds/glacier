#![allow(unused)]

use std::{
    alloc::{alloc, Layout},
    pin::Pin,
    ptr,
    str::{from_utf8, from_utf8_unchecked},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use bytes::{BufMut, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

use crate::{error::GlacierError, Result};

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
        unsafe { from_utf8_unchecked(uri) }
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

    pub async fn respond(&mut self) {
        let res =
            "HTTP/1.1 200 OK\r\nContent-Length: 13\r\nConnection: keep-alive\r\n\r\nHello, world!";
        self.stream.write_all(res.as_bytes()).await;
    }
}
