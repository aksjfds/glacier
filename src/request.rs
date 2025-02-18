#![allow(unused)]

use tokio::io::AsyncBufReadExt;

use crate::error::GlacierError;
use crate::Result;

#[derive(Debug)]
pub struct Request {
    pub request_line: RequestLine,
    pub headers: Vec<RequestHeader>,
    pub body: Option<RequestBody>,
}

#[derive(Debug)]
pub enum RequestMethod {
    Get,
    Post,
}

#[derive(Debug)]
pub struct RequestLine {
    method: RequestMethod,
    pub uri: String,
    version: String,
}

#[derive(Debug)]
pub struct RequestHeader {
    key: String,
    value: String,
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

impl<'a> TryFrom<&'a str> for RequestLine {
    type Error = GlacierError;

    fn try_from(value: &'a str) -> Result<Self> {
        // GET /favicon.ico HTTP/1.1
        let mut value = value.split(" ");

        if let [Some(method), Some(uri), Some(version)] = [value.next(), value.next(), value.next()]
        {
            Ok(RequestLine {
                method: method.try_into()?,
                uri: String::from(uri),
                version: String::from(version),
            })
        } else {
            Err(GlacierError::FromRequest(String::from("解析请求行出错")))
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for RequestLine {
    type Error = GlacierError;

    fn try_from(value: &'a [u8]) -> Result<Self> {
        let value = unsafe { std::str::from_utf8_unchecked(value) };
        value.try_into()
    }
}

impl<'a> TryFrom<&'a str> for RequestHeader {
    type Error = GlacierError;

    fn try_from(value: &'a str) -> Result<Self> {
        // Host: localhost:3000
        let pos = value.find(":").unwrap();
        let (key, value) = (&value[..pos], &value[pos + 2..]);

        Ok(RequestHeader {
            key: String::from(key),
            value: String::from(value),
        })
    }
}

impl TryFrom<String> for RequestHeader {
    type Error = GlacierError;

    fn try_from(value: String) -> Result<Self> {
        let pos = value.find(":").unwrap();
        let (key, value) = (&value[..pos], &value[pos + 2..]);

        Ok(RequestHeader {
            key: String::from(key),
            value: String::from(value),
        })
    }
}

impl<'a> TryFrom<Vec<u8>> for Request {
    type Error = GlacierError;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        let value = unsafe { String::from_utf8_unchecked(value) };
        let mut lines = value.lines();

        // 解析 request-line 一般不会出错
        let request_line = lines.next().unwrap();
        let request_line = request_line.try_into()?;

        // 解析 请求头
        let mut headers = Vec::new();
        while let Some(line) = lines.next() {
            if line.is_empty() {
                break;
            }

            headers.push(RequestHeader::try_from(line)?);
        }

        Ok(Request {
            request_line,
            headers,
            body: None,
        })
    }
}

impl<'a> Request {
    pub fn parse(value: Vec<u8>) -> Result<Self> {
        value.try_into()
    }

    pub fn path(&self) -> &str {
        self.request_line.uri.as_str()
    }

    pub async fn parse_from_vec(vec: Vec<u8>) -> Result<Self> {
        let mut lines = vec.lines();

        let request_line = match lines.next_line().await? {
            Some(line) => RequestLine::try_from(line.as_str())?,
            None => {
                return Err(GlacierError::FromRequest(String::from(
                    "lines is already None",
                )))
            }
        };

        let mut headers = Vec::new();
        while let Some(line) = lines.next_line().await? {
            if line.is_empty() {
                break;
            }
            headers.push(RequestHeader::try_from(line)?);
        }

        // TODO 处理请求体
        // let body=

        Ok(Request {
            request_line,
            headers,
            body: None,
        })
    }
}

impl RequestLine {
    pub fn uri(&self) -> &str {
        self.uri.as_str()
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

#[tokio::test]
async fn test2() -> Result<()> {
    let a = b"xxx\r\nxxxxd\r\nddddddd\r\naaaaaaa\r\n";
    let a = a.to_vec();

    let mut a = a.lines();
    println!("{:#?}", a.next_line().await.unwrap().unwrap());

    let a = a.into_inner();
    unsafe {
        println!("{:#?}", String::from_utf8_unchecked(a.to_vec()));
    }

    Ok(())
}
