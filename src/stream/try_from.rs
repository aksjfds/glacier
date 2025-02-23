use super::request::{RequestLine, RequestMethod};
use crate::error::GlacierError;
use crate::Result;

/* -------------------------------- // Method ------------------------------- */
impl TryFrom<&str> for RequestMethod {
    type Error = GlacierError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "GET" => Ok(RequestMethod::Get),
            "POST" => Ok(RequestMethod::Post),
            _ => Err(GlacierError::FromRequest("解析请求方法出错")),
        }
    }
}

/* ----------------------------- // RequestLine ----------------------------- */
impl TryFrom<&String> for RequestLine {
    type Error = GlacierError;

    fn try_from(value: &String) -> Result<Self> {
        // GET /favicon.ico HTTP/1.1
        let mut split = value.split(" ");

        if let [Some(method), Some(uri), Some(version)] = [split.next(), split.next(), split.next()]
        {
            Ok(RequestLine {
                method: method.try_into()?,
                uri: String::from(uri),
                version: String::from(version),
            })
        } else {
            Err(GlacierError::FromRequest("解析请求行出错"))
        }
    }
}

impl<'a> TryFrom<&'a str> for RequestLine {
    type Error = GlacierError;

    fn try_from(value: &'a str) -> Result<Self> {
        // GET /favicon.ico HTTP/1.1
        let mut split = value.split(" ");

        if let [Some(method), Some(uri), Some(version)] = [split.next(), split.next(), split.next()]
        {
            Ok(RequestLine {
                method: method.try_into()?,
                uri: String::from(uri),
                version: String::from(version),
            })
        } else {
            println!("{:#?}", value);
            Err(GlacierError::FromRequest("解析请求行出错"))
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

/* ------------------------------- // Request ------------------------------- */
