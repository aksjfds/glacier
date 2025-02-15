#![allow(unused)]

use std::{collections::HashMap, io::Write, net::TcpStream};

pub struct Response {
    response_line: String,
    response_headers: HashMap<String, String>,

    pub body: String,
    stream: TcpStream,
}

impl Response {
    pub fn hello(stream: TcpStream) -> Self {
        let response_line = String::from("HTTP/1.1 200 OK\r\n");
        let response_headers = HashMap::new();
        let body = String::from("Hello, world!");
        Response {
            stream,
            body,
            response_line,
            response_headers,
        }
    }

    pub fn not_found(mut stream: TcpStream) {
        let response = b"HTTP/1.1 404 Not Found\r\n\
                 Content-Type: text/html; charset=UTF-8\r\n\
                 Content-Length: 113\r\n\
                 \r\n\
                 <html><body><h1>404 - Not Found</h1>\
                 <p>The page you are looking for might have been removed, had its name changed, or is temporarily unavailable.</p></body></html>";

        if let Ok(_) = stream.write_all(response) {}
        if let Ok(_) = stream.flush() {}
    }

    pub fn bad_request(mut stream: TcpStream) {
        println!("{:#?}", 1);
        let response = b"HTTP/1.1 400 Bad Request\r\n\r\n";
        stream.write_all(response).unwrap();
        stream.flush().unwrap();
        // if let Ok(_) = stream.write_all(response) {}
        // if let Ok(_) = stream.flush() {}
    }

    pub fn respond(&mut self) {
        let mut res = String::new();

        // 响应行
        res.push_str(self.response_line.as_str());

        // 响应头
        self.response_headers.insert(
            String::from("Content-Length"),
            String::from(format!("{}", self.body.len())),
        );
        let headers: Vec<_> = self
            .response_headers
            .iter()
            .map(|(key, value)| format!("{}: {}\r\n", key, value))
            .collect();
        let headers = headers.join("\r\n");
        res.push_str(headers.as_str());

        // 响应体
        res.push_str("\r\n");
        res.push_str(self.body.as_str());

        self.stream.write_all(res.as_bytes()).unwrap();
        self.stream.flush().unwrap();
    }
}
