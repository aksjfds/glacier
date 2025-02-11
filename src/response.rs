#![allow(unused)]

use std::{io::Write, net::TcpStream};

pub struct Response {
    stream: TcpStream,
    body: String,
}

impl Response {
    pub fn hello(stream: TcpStream) -> Self {
        let body = String::from("HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, world!");
        Response { stream, body }
    }

    pub fn not_found(mut stream: TcpStream) {
        let response = b"HTTP/1.1 404 Not Found\r\n\
                 Content-Type: text/html; charset=UTF-8\r\n\
                 Content-Length: 113\r\n\
                 \r\n\
                 <html><body><h1>404 - Not Found</h1>\
                 <p>The page you are looking for might have been removed, had its name changed, or is temporarily unavailable.</p></body></html>";
        stream.write_all(response).unwrap();
        stream.flush().unwrap();
    }

    pub fn respond(&mut self) {
        self.stream.write_all(self.body.as_bytes()).unwrap();
        self.stream.flush().unwrap();
    }
}
