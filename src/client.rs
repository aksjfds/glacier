// #![allow(unused)]

use std::sync::Mutex;
use std::{io::Read, net::TcpListener};

use crate::request::RequestLine;
use crate::route::Routes;
use crate::Result;
use crate::{bytes::Bytes, request::Request, response::Response};

pub struct Glacier {
    listener: TcpListener,
    req_buf: Mutex<Bytes>,
    routes: Routes,
}

impl Glacier {
    pub fn bind(port: u16, routes: Routes) -> Result<Glacier> {
        let addr = ("127.0.0.1", port);
        let listener = TcpListener::bind(addr)?;

        Ok(Glacier {
            listener,
            req_buf: Mutex::new(Bytes::with_capacity(32)),
            routes,
        })
    }

    pub fn run(&self) {
        'run: for stream in self.listener.incoming() {
            match stream {
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
                Ok(mut stream) => {
                    let mut buf = self.req_buf.lock().unwrap();

                    // 获取 request_line
                    let path = loop {
                        match stream.read(buf.get_32_space()) {
                            Err(e) => {
                                eprintln!("Error reading from stream: {}", e);
                                continue 'run;
                            }
                            Ok(0) => {
                                // stream.shutdown(std::net::Shutdown::Both);
                                continue 'run;
                            }
                            Ok(len) => buf.modify_len(len),
                        }

                        if let Some(line) = buf.first_line() {
                            match RequestLine::try_from(line) {
                                Ok(line) => break line.uri,
                                Err(e) => {
                                    eprintln!("Error when parsing request-line: {}", e);
                                    continue 'run;
                                }
                            }
                        }
                    };

                    // 获取路由
                    let func = match self.routes.query(path) {
                        Some(func) => func,
                        None => {
                            Response::not_found(stream);
                            continue 'run;
                        }
                    };

                    // 获取完整 request
                    loop {
                        if buf.is_end() {
                            break;
                        }

                        match stream.read(buf.get_free_space()) {
                            Err(e) => {
                                eprintln!("Error reading from stream: {}", e);
                                continue 'run;
                            }
                            Ok(0) => {
                                // stream.shutdown(std::net::Shutdown::Both);
                                continue 'run;
                            }
                            Ok(len) => buf.modify_len(len),
                        }
                    }

                    // 组织
                    let req = match Request::parse(&buf) {
                        Ok(req) => req,
                        Err(e) => {
                            eprintln!("Error when parsing request: {}", e);
                            Response::bad_request(stream);
                            continue 'run;
                        }
                    };
                    let res = Response::hello(stream);
                    func(req, res);
                }
            }
            self.req_buf.lock().unwrap().clear();
        }
    }
}
