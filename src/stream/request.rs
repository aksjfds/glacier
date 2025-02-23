#![allow(unused)]

use std::str::from_utf8;

use bytes::{BufMut, BytesMut};

use crate::error::GlacierError;
use crate::Result;
/* ----------------------------------- RequestMethod ----------------------------------- */
#[derive(Debug, Clone)]
pub enum RequestMethod {
    Get,
    Post,
}

/* ----------------------------------- RequestLine ----------------------------------- */
#[derive(Debug)]
pub struct RequestLine {
    pub(crate) method: RequestMethod,
    pub(crate) uri: String,
    pub(crate) version: String,
}

impl RequestLine {
    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }

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
            Err(GlacierError::FromRequest("解析请求行出错"))
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
        if let [Some(key), Some(value)] = [split.next(), split.next()] {
            Ok([line[0], line[0] + key.len(), line[1]])
        } else {
            Err(GlacierError::FromRequest("请求头格式错误"))
        }
    }
}
