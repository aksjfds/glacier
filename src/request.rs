#![allow(unused)]

use crate::Result;
use crate::{bytes::Bytes, error::GlacierError};

#[derive(Debug)]
pub struct Request<'a> {
    pub request_line: RequestLine<'a>,
    pub headers: Vec<RequestHeader<'a>>,
    pub body: Option<RequestBody>,
}

#[derive(Debug)]
pub enum RequestMethod {
    Get,
    Post,
}

#[derive(Debug)]
pub struct RequestLine<'a> {
    method: RequestMethod,
    pub uri: &'a str,
    version: &'a str,
}

#[derive(Debug)]
pub struct RequestHeader<'a> {
    key: &'a str,
    value: &'a str,
}

#[derive(Debug)]
pub struct RequestBody;

impl TryFrom<&str> for RequestMethod {
    type Error = GlacierError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "GET" => Ok(RequestMethod::Get),
            "POST" => Ok(RequestMethod::Post),
            _ => Err(GlacierError::FromRequest(String::from("解析请求方法出错"))),
        }
    }
}

impl<'a> TryFrom<&'a str> for RequestLine<'a> {
    type Error = GlacierError;

    fn try_from(value: &'a str) -> Result<Self> {
        // GET /favicon.ico HTTP/1.1
        let mut value = value.split(" ");

        if let [Some(method), Some(uri), Some(version)] = [value.next(), value.next(), value.next()]
        {
            Ok(RequestLine {
                method: method.try_into()?,
                uri,
                version,
            })
        } else {
            Err(GlacierError::FromRequest(String::from("解析请求行出错")))
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for RequestLine<'a> {
    type Error = GlacierError;

    fn try_from(value: &'a [u8]) -> Result<Self> {
        let value = unsafe { std::str::from_utf8_unchecked(value) };
        value.try_into()
    }
}

impl<'a> TryFrom<&'a str> for RequestHeader<'a> {
    type Error = GlacierError;

    fn try_from(value: &'a str) -> Result<Self> {
        // Host: localhost:3000
        let mut value = value.split(": ");

        if let [Some(key), Some(value)] = [value.next(), value.next()] {
            Ok(RequestHeader { key, value })
        } else {
            Err(GlacierError::FromRequest(String::from("解析请求头出错")))
        }
    }
}

impl<'a> TryFrom<&'a Bytes> for Request<'a> {
    type Error = GlacierError;

    fn try_from(value: &'a Bytes) -> Result<Self> {
        let value = value.as_str();
        let mut value = value.split("\r\n");

        // 解析 request-line 一般不会出错
        let request_line = value.next().unwrap();
        let request_line = request_line.try_into()?;

        // 解析 请求头
        let mut headers = Vec::new();
        for header in value {
            if let Ok(header) = header.try_into() {
                headers.push(header);
            }
            // headers.push(header.try_into()?);
        }
        // headers.pop();

        Ok(Request {
            request_line,
            headers,
            body: None,
        })
    }
}

impl<'a> Request<'a> {
    pub fn parse(value: &'a Bytes) -> Result<Self> {
        value.try_into()
    }

    pub fn path(&self) -> &str {
        self.request_line.uri
    }
}

#[test]
fn test() -> Result<()> {
    let a: RequestLine = "GET /favicon.ico HTTP/1.1".try_into()?;
    println!("{:#?}", a);

    let b: RequestHeader = "Host: localhost:3000".try_into()?;
    println!("{:#?}", b);

    Ok(())
}
