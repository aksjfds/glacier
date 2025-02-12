#![allow(unused)]

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::{io::Read, net::TcpListener};

use crate::request::RequestLine;
use crate::{bytes::Bytes, request::Request, response::Response, GLACIER_GET};
use crate::{error, Result};

pub struct Glacier {
    listener: TcpListener,
    req_buf: RefCell<Bytes>,
    routes: HashMap<&'static str, fn(Request, Response)>,
}

impl Glacier {
    pub fn bind(port: u16) -> Result<Glacier> {
        let routes = GLACIER_GET.lock().unwrap().clone();

        let addr = ("127.0.0.1", port);
        let listener = TcpListener::bind(addr)?;

        Ok(Glacier {
            listener,
            req_buf: RefCell::new(Bytes::with_capacity(32)),
            routes,
        })
    }

    fn buf(&self) -> RefMut<'_, Bytes> {
        self.req_buf.borrow_mut()
    }

    pub fn run(&self) {
        for stream in self.listener.incoming() {
            match stream {
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
                Ok(mut stream) => {
                    let mut buf = self.buf();

                    // 获取 request_line
                    let path = 'block2: {
                        while let Ok(len @ 1..) = stream.read(buf.get_free_space()) {
                            buf.modify_len(len);

                            if let Some(line) = buf.parse_line() {
                                let line = RequestLine::try_from(line).unwrap();
                                break 'block2 line.uri;
                            }
                        }

                        ""
                    };

                    // 获取路由
                    if let Some(func) = self.routes.get(path) {
                        // 获取完整 request
                        while let Ok(len @ 1..) = stream.read(buf.get_free_space()) {
                            buf.modify_len(len);
                            if buf.is_end() {
                                break;
                            }
                        }

                        // 组织
                        let temp = buf.to_string();
                        let req = Request::parse(&temp).unwrap();
                        let res = Response::hello(stream);
                        func(req, res);
                    } else {
                        Response::not_found(stream);
                    }
                }
            }

            self.buf().clear();
        }
    }
}
