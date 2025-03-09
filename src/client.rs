use bytes::BytesMut;
use std::{future::Future, net::IpAddr, time::Duration};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_rustls::TlsAcceptor;

use crate::stream::request::{ReqInfo, RequestHeader, RequestLine};
use crate::{
    error::Kind,
    prelude::{GlacierError, OneRequest, Result, Routes},
};

//
//
//
//
//
//
//

#[cfg(not(feature = "tls"))]
pub struct Glacier<T> {
    pub(crate) listener: TcpListener,
    pub(crate) routes: Routes<T>,
}

#[cfg(not(feature = "tls"))]
impl<T> Glacier<T>
where
    T: Future<Output = Result<OneRequest>> + Send + Sync + 'static,
{
    /// 开始运行代码
    /// # Examples
    /// ```
    /// let glacier = Glacier::bind(3000, routes).await.unwrap();
    /// glacier.run().await.unwrap();
    /// ```
    pub async fn run(self) -> Result<()> {
        let routes = self.routes;
        let listener = self.listener;

        let srv = async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        tokio::spawn(async move {
                            Glacier::handle_connection(stream, routes, addr.ip())
                                .await
                                .unwrap();
                        });
                    }
                    Err(e) => tracing::info!(e = e.to_string(), "error connection!"),
                };
            }
        };

        let _ = tokio::spawn(srv).await;

        Ok(())
    }

    async fn handle_connection(
        mut stream: TcpStream,
        routes: Routes<T>,
        addr: IpAddr,
    ) -> Result<()> {
        tracing::info!("new connection!");
        let mut buf = BytesMut::with_capacity(1024);
        loop {
            let req_info = match read_stream(&mut stream, &mut buf).await {
                Ok(req_info) => req_info,
                Err(e) => {
                    match e {
                        GlacierError::OkErr(_kind @ Kind::EofErr) => {}
                        GlacierError::Option => {}
                        _ => tracing::debug!("{:#?}", e),
                    }
                    return Ok(());
                }
            };
            let mut one_req = OneRequest::new(stream, buf, req_info, addr);

            one_req = match routes(one_req).await {
                Ok(one_req) => one_req,
                Err(_) => return Ok(()),
            };
            stream = one_req.stream;
            buf = one_req.buf;
            buf.clear();
        }
    }
}

#[cfg(not(feature = "tls"))]
async fn read_stream(stream: &mut TcpStream, buf: &mut BytesMut) -> Result<ReqInfo> {
    /* --------------------------------- // 准备工作 -------------------------------- */
    let mut pos = Vec::with_capacity(10);
    pos.push(0);
    buf.clear();

    /* ------------------------ // 读取数据到buf, 然后标记buf上的位置 ------------------------ */
    loop {
        let read_task: _ = stream.read_buf(buf);
        let read_task: _ = timeout(Duration::from_secs(10), read_task);

        let len = match read_task.await {
            Ok(Ok(0)) => Err(GlacierError::OkErr(Kind::EofErr))?,
            Ok(Ok(len @ 1..)) => len,
            Ok(Err(e)) => Err(e)?,
            _ => Err(GlacierError::OkErr(Kind::TimeOutErr))?,
        };

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

    let mut lines = pos
        .as_slice()
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, pos_temp)| [pos[i - 1], *pos_temp]);

    // 请求行处理
    let request_line = lines.next().ok_or_else(|| {
        tracing::debug!("lines is empty");
        GlacierError::Option
    })?;
    let line_pos = RequestLine::parse(&buf, request_line)?;

    // 请求头处理
    let headers_pos: Vec<[usize; 3]> = lines
        .map(|line| RequestHeader::parse(&buf, line))
        .collect::<Result<Vec<_>>>()?;

    Ok(ReqInfo {
        line_pos,
        headers_pos,
    })
}

#[cfg(feature = "tls")]
pub struct Glacier<T> {
    pub(crate) listener: TcpListener,
    pub(crate) routes: Routes<T>,
    pub(crate) acceptor: TlsAcceptor,
}

#[cfg(feature = "tls")]
impl<T> Glacier<T>
where
    T: Future<Output = Result<OneRequest>> + Send + Sync + 'static,
{
    /// 开始运行代码
    /// # Examples
    /// ```
    /// let glacier = Glacier::bind(3000, routes).await.unwrap();
    /// glacier.run().await.unwrap();
    /// ```
    pub async fn run(self) -> Result<()> {
        let routes = self.routes;
        let listener = self.listener;
        let acceptor = self.acceptor;

        let srv = async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let acceptor = acceptor.clone();
                        let stream = match acceptor.accept(stream).await {
                            Ok(stream) => stream,
                            Err(_) => continue,
                        };

                        tokio::spawn(async move {
                            Glacier::handle_connection(stream, routes, addr.ip())
                                .await
                                .unwrap();
                        });
                    }
                    Err(e) => tracing::info!(e = e.to_string(), "error connection!"),
                };
            }
        };

        let _ = tokio::spawn(srv).await;

        Ok(())
    }

    async fn handle_connection(
        mut stream: tokio_rustls::server::TlsStream<TcpStream>,
        routes: Routes<T>,
        addr: IpAddr,
    ) -> Result<()> {
        tracing::info!("new connection!");
        let mut buf = BytesMut::with_capacity(1024);
        loop {
            let req_info = match read_stream(&mut stream, &mut buf).await {
                Ok(req_info) => req_info,
                Err(e) => {
                    match e {
                        GlacierError::OkErr(_kind @ Kind::EofErr) => {}
                        GlacierError::Option => {}
                        _ => tracing::debug!("{:#?}", e),
                    }
                    return Ok(());
                }
            };
            let mut one_req = OneRequest::new(stream, buf, req_info, addr);

            one_req = match routes(one_req).await {
                Ok(one_req) => one_req,
                Err(_) => return Ok(()),
            };
            stream = one_req.stream;
            buf = one_req.buf;
            buf.clear();
        }
    }
}

#[cfg(feature = "tls")]
async fn read_stream(
    stream: &mut tokio_rustls::server::TlsStream<TcpStream>,
    buf: &mut BytesMut,
) -> Result<ReqInfo> {
    /* --------------------------------- // 准备工作 -------------------------------- */

    let mut pos = Vec::with_capacity(10);
    pos.push(0);
    buf.clear();

    /* ------------------------ // 读取数据到buf, 然后标记buf上的位置 ------------------------ */
    loop {
        let read_task: _ = stream.read_buf(buf);
        let read_task: _ = timeout(Duration::from_secs(10), read_task);

        let len = match read_task.await {
            Ok(Ok(0)) => Err(GlacierError::OkErr(Kind::EofErr))?,
            Ok(Ok(len @ 1..)) => len,
            Ok(Err(e)) => Err(e)?,
            _ => Err(GlacierError::OkErr(Kind::TimeOutErr))?,
        };

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

    let mut lines = pos
        .as_slice()
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, pos_temp)| [pos[i - 1], *pos_temp]);

    // 请求行处理
    let request_line = lines.next().ok_or_else(|| {
        tracing::debug!("lines is empty");
        GlacierError::Option
    })?;
    let line_pos = RequestLine::parse(&buf, request_line)?;

    // 请求头处理
    let headers_pos: Vec<[usize; 3]> = lines
        .map(|line| RequestHeader::parse(&buf, line))
        .collect::<Result<Vec<_>>>()?;

    Ok(ReqInfo {
        line_pos,
        headers_pos,
    })
}
