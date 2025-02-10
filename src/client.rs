#![allow(unused)]

use std::{
    io::{Read, Write},
    net::TcpListener,
};

use crate::{bytes::Bytes, request::Request};

pub struct Glacier {
    listener: TcpListener,
    request_buf: Bytes,
    // route_func:Vec<>
}

impl Glacier {
    pub fn bind(port: u16) -> Self {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(addr).unwrap();

        Glacier {
            listener,
            request_buf: Bytes::with_capacity(512),
        }
    }

    pub fn run(&mut self) {
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap();

            let mut buf = [0; 512]; // 读取固定大小的缓冲区
            let request_buf = &mut self.request_buf;

            while let Ok(len @ 1..) = stream.read(&mut buf) {
                request_buf.push_slice(&buf[..len]);
                if request_buf.is_end() {
                    println!("{}", request_buf.to_string());
                    break;
                }
            }

            if !request_buf.is_empty() {
                let request = Request::new(&request_buf.to_string());
                // println!("{:#?}", request);
            } else {
                println!("{:#?}", "断开连接");
            }

            // 构造 HTTP 响应
            let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, world!";
            stream.write_all(response.as_bytes());
            stream.flush();

            self.request_buf.clear();
        }
    }
}
