#![allow(unused)]

use std::{
    alloc::{alloc, Layout},
    pin::Pin,
    ptr,
    str::{from_utf8, from_utf8_unchecked},
};

use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
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
    pub buf: BytesMut,
    pos: Vec<usize>,
}

impl GlacierStream {
    pub fn new(stream: TcpStream) -> Self {
        GlacierStream {
            stream,
            buf: BytesMut::with_capacity(1024),
            pos: Vec::with_capacity(10),
        }
    }

    // 读取 请求行 和 请求头 的全部数据
    pub async fn init(&mut self) -> Result<()> {
        self.pos.push(0);
        let buf = &mut self.buf;

        loop {
            let len = self.stream.read_buf(buf).await?;
            if 0 == len {
                Err(GlacierError::Option("EOF when init_request_line"))?
            }

            for i in (buf.len() - len..buf.len()).step_by(2) {
                match buf[i] {
                    b'\r' => {
                        if let Some(x @ b'\n') = buf.get(i + 1) {
                            self.pos.push(i + 2);
                        }
                    }
                    b'\n' => {
                        if buf[i - 1] == b'\r' {
                            self.pos.push(i + 1);
                        }
                    }
                    _ => {}
                }
            }

            // 判断是否到了 请求体
            let len = self.pos.len();
            if len >= 2 && self.pos[len - 1] - self.pos[len - 2] == 2 {
                self.pos.pop();
                break;
            }
        }

        Ok(())
    }

    pub fn build_request<'a>(self) -> Result<OneRequest> {
        let mut lines = self
            .pos
            .as_slice()
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, pos)| [self.pos[i - 1], *pos]);

        // 请求行处理
        let request_line = lines.next().expect("lines为空");
        let request_line_pos = RequestLine::parse(&self.buf, request_line)?;

        // // 请求头处理
        let request_headers_pos: Vec<[usize; 3]> = lines
            .map(|line| RequestHeader::parse(&self.buf, line))
            .collect::<Result<Vec<_>>>()
            .unwrap();

        Ok(OneRequest {
            glacier_stream: self,
            request_line_pos,
            request_headers_pos,
        })
    }
}

// /* ------------------------------ // OneRequest ----------------------------- */
pub struct OneRequest {
    pub glacier_stream: GlacierStream,
    pub request_line_pos: [usize; 4],
    pub request_headers_pos: Vec<[usize; 3]>,
}

impl OneRequest {
    pub fn request_line(&self) -> &str {
        let request_line =
            &self.glacier_stream.buf[self.request_line_pos[0]..self.request_line_pos[3] - 1];

        unsafe { from_utf8_unchecked(request_line) }
    }

    pub fn path(&self) -> &str {
        let uri = &self.glacier_stream.buf[self.request_line_pos[1]..self.request_line_pos[2] - 1];
        unsafe { from_utf8_unchecked(uri) }
    }

    pub fn query_header(&self, query_key: &str) -> Option<&str> {
        let headers = &self.request_headers_pos;
        for header in headers {
            let key =
                unsafe { from_utf8_unchecked(&self.glacier_stream.buf[header[0]..header[1]]) };
            if query_key == key {
                let value = unsafe {
                    from_utf8_unchecked(&self.glacier_stream.buf[header[1] + 2..header[2] - 2])
                };

                return Some(value);
            }
        }

        None
    }

    pub async fn respond(mut self) {
        let res = b"\
                HTTP/1.1 200 OK\r\n\
                Connection: close\r\n\
                Content-Length: 13\r\n\r\n\
                Hello, World!";
        self.glacier_stream.stream.write_all(res).await;
    }
}
