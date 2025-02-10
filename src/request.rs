#![allow(unused)]

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
    uri: &'a str,
    version: &'a str,
}

#[derive(Debug)]
pub struct RequestHeader<'a> {
    key: &'a str,
    value: &'a str,
}

#[derive(Debug)]
pub struct RequestBody;

impl From<&str> for RequestMethod {
    fn from(value: &str) -> Self {
        match value {
            "GET" => RequestMethod::Get,
            "POST" => RequestMethod::Post,
            _ => todo!(),
        }
    }
}

impl<'a> From<&'a str> for RequestLine<'a> {
    fn from(value: &'a str) -> Self {
        // GET /favicon.ico HTTP/1.1
        let mut value = value.split(" ");
        let method = value.next().unwrap();
        let uri = value.next().unwrap();
        let version = value.next().unwrap();

        RequestLine {
            method: method.into(),
            uri,
            version,
        }
    }
}

impl<'a> From<&'a str> for RequestHeader<'a> {
    fn from(value: &'a str) -> Self {
        // Host: localhost:3000
        let mut value = value.split(": ");

        let (key, value) = match (value.next(), value.next()) {
            (Some(key), Some(value)) => (key, value),
            _ => ("", ""),
        };

        RequestHeader { key, value }
    }
}

impl<'a> From<&'a String> for Request<'a> {
    fn from(value: &'a String) -> Self {
        let mut value = value.split("\r\n");
        let request_line = value.next().unwrap();
        let request_line = request_line.into();

        let mut headers = Vec::new();
        for header in value {
            headers.push(header.into());
        }
        headers.pop();

        Request {
            request_line,
            headers,
            body: None,
        }
    }
}

impl<'a> Request<'a> {
    pub fn new(value: &'a String) -> Self {
        value.into()
    }
}

#[test]
fn test() {
    let a: RequestLine = "GET /favicon.ico HTTP/1.1".into();
    println!("{:#?}", a);

    let b: RequestHeader = "Host: localhost:3000".into();
    println!("{:#?}", b);
}
