#![allow(unused)]

use crate::prelude::*;
use std::{sync::atomic::AtomicU64, time::Duration};

use bytes::BytesMut;
use tokio::{io::AsyncReadExt, net::TcpStream, time::timeout};

use super::request::{RequestHeader, RequestLine};

///
///
///
///

/* ---------------------------- // GlacierStream ---------------------------- */
pub struct GlacierStream {
    pub(crate) stream: TcpStream,
    pub(crate) buf: BytesMut,
}

static R: AtomicU64 = AtomicU64::new(0);

impl GlacierStream {
    pub fn new(stream: TcpStream) -> Self {
        GlacierStream {
            stream,
            buf: BytesMut::with_capacity(1024),
        }
    }

    // 读取 请求行 和 请求头 的全部数据
    pub async fn read(&mut self) -> Result<Vec<usize>> {
        /* --------------------------------- // 准备工作 -------------------------------- */
        let mut buf = &mut self.buf;
        let mut pos = Vec::with_capacity(10);
        buf.clear();
        pos.push(0);
        // 读取数据到buf, 然后标记buf上的位置
        loop {
            let len = timeout(Duration::from_secs(10), self.stream.read_buf(&mut buf)).await??;
            if 0 == len {
                // R.fetch_add(1, Ordering::Relaxed);
                // println!("close: {:#?}", R.load(Ordering::Relaxed));
                Err(GlacierError::build_eof("Connection close"))?
            }

            for i in (buf.len() - len..buf.len()).step_by(2) {
                match buf[i] {
                    b'\r' => {
                        if let Some(_x @ b'\n') = buf.get(i + 1) {
                            pos.push(i + 2);
                        }
                    }
                    b'\n' => {
                        if buf[i - 1] == b'\r' {
                            pos.push(i + 1);
                        }
                    }
                    _ => {}
                }
            }

            // 判断是否到了 请求体
            let len = pos.len();
            if len >= 2 && pos[len - 1] - pos[len - 2] == 2 {
                pos.pop();
                break;
            }
        }

        Ok(pos)
    }

    pub async fn to_req(mut self) -> Result<OneRequest> {
        let pos = self.read().await?;
        let mut lines = pos
            .as_slice()
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, pos_temp)| [pos[i - 1], *pos_temp]);

        // 请求行处理
        let request_line = lines.next().expect("lines为空");
        let request_line_pos = RequestLine::parse(&self.buf, request_line)?;

        // 请求头处理
        let request_headers_pos: Vec<[usize; 3]> = lines
            .map(|line| RequestHeader::parse(&self.buf, line))
            .collect::<Result<Vec<_>>>()
            .unwrap();

        Ok(OneRequest {
            stream: self.stream,
            buf: self.buf,
            request_line_pos,
            request_headers_pos,
        })
    }
}
