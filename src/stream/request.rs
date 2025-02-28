use std::{
    io::IoSlice,
    str::{from_utf8, from_utf8_unchecked},
};

use bytes::BytesMut;
use serde::Deserialize;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::prelude::*;

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

    pub fn method(&self) -> &str {
        let method = &self.buf[self.request_line_pos[0]..self.request_line_pos[1] - 1];
        unsafe { from_utf8_unchecked(method) }
    }

    pub fn path(&self) -> &str {
        let uri = &self.buf[self.request_line_pos[1]..self.request_line_pos[2] - 1];
        let uri = unsafe { from_utf8_unchecked(uri) };
        if let Some(pos) = uri.find("?") {
            &uri[..pos]
        } else {
            uri
        }
    }

    pub fn version(&self) -> &str {
        let version = &self.buf[self.request_line_pos[1]..self.request_line_pos[2] - 1];
        unsafe { from_utf8_unchecked(version) }
    }

    pub fn query_header(&self, query_key: &str) -> Option<&str> {
        let query_key = query_key.as_bytes();
        let headers = &self.request_headers_pos;
        for header in headers {
            let key = &self.buf[header[0]..header[1]];
            if query_key == key {
                let value = unsafe { from_utf8_unchecked(&self.buf[header[1] + 2..header[2] - 2]) };

                return Some(value);
            }
        }

        None
    }

    pub fn get_params<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        let uri = &self.buf[self.request_line_pos[1]..self.request_line_pos[2] - 1];
        let uri = unsafe { from_utf8_unchecked(uri) };

        if let Some(pos) = uri.find("?") {
            let params = &uri[pos + 1..];
            serde_qs::from_str::<T>(params).ok()
        } else {
            None
        }
    }

    pub fn to_stream(self) -> GlacierStream {
        GlacierStream {
            stream: self.stream,
            buf: self.buf,
        }
    }

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
        let writer = &mut self.stream;
        writer.write_vectored(bufs).await?;
        writer.flush().await?;

        Ok(())
    }

    pub async fn respond_hello(&mut self) {
        let res =
            "HTTP/1.1 200 OK\r\nContent-Length: 13\r\nConnection: keep-alive\r\n\r\nHello, world!";
        self.stream.write_all(res.as_bytes()).await.unwrap();
        self.stream.flush().await.unwrap();
    }
}

/* ----------------------------------- RequestMethod ----------------------------------- */
#[derive(Debug, Clone)]
pub enum RequestMethod {
    Get,
    Post,
}

/* ----------------------------------- RequestLine ----------------------------------- */
#[derive(Debug)]
pub struct RequestLine;

impl RequestLine {
    pub(super) fn parse(buf: &BytesMut, pos: [usize; 2]) -> Result<[usize; 4]> {
        // GET /favicon.ico HTTP/1.1\r\n

        let request_line = from_utf8(&buf[pos[0]..pos[1]][..])?;

        let mut split = request_line.split(" ");

        if let [Some(method), Some(uri), Some(version)] = [split.next(), split.next(), split.next()]
        {
            Ok([
                0,
                method.len() + 1,
                method.len() + uri.len() + 2,
                method.len() + uri.len() + version.len() + 1,
            ])
        } else {
            println!("{:#?}", request_line);
            Err(GlacierError::build_req("解析请求行出错"))
        }
    }
}

/* ----------------------------------- RequestHeader ----------------------------------- */
#[derive(Debug)]
pub struct RequestHeader;

impl RequestHeader {
    pub fn parse(buf: &BytesMut, line: [usize; 2]) -> Result<[usize; 3]> {
        let header = std::str::from_utf8(&buf[line[0]..line[1] - 2])?;

        let mut split = header.split(": ");
        if let [Some(key), Some(_)] = [split.next(), split.next()] {
            Ok([line[0], line[0] + key.len(), line[1]])
        } else {
            Err(GlacierError::build_req("请求头格式错误"))
        }
    }
}
