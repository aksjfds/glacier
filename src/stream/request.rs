use bytes::BytesMut;
use serde::Deserialize;
use std::str::from_utf8_unchecked;
use std::{io::IoSlice, net::IpAddr};
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::prelude::{GlacierError, GlacierStream, Response, Result, FILES_BUF};

// /* ------------------------------ // OneRequest ----------------------------- */
pub struct OneRequest {
    pub(crate) stream: TcpStream,
    pub(crate) addr: IpAddr,
    pub(crate) buf: BytesMut,
    pub(crate) request_line_pos: [usize; 4],
    pub(crate) request_headers_pos: Vec<[usize; 3]>,
}

impl OneRequest {
    /// 请求行
    pub fn request_line(&self) -> &str {
        let request_line = unsafe {
            self.buf
                .get_unchecked(self.request_line_pos[0]..self.request_line_pos[3] - 1)
        };

        unsafe { from_utf8_unchecked(request_line) }
    }

    /// 请求方法
    pub fn method(&self) -> &str {
        let method = unsafe {
            self.buf
                .get_unchecked(self.request_line_pos[0]..self.request_line_pos[1] - 1)
        };
        unsafe { from_utf8_unchecked(method) }
    }

    /// 修改请求方法
    /// # Examples
    /// ```
    /// if req.method() == "POST" {
    ///     req.modify_method("GET")
    /// }
    /// ```
    pub fn modify_method(&mut self, data: &str) {
        let method = unsafe {
            &mut self
                .buf
                .get_unchecked_mut(self.request_line_pos[0]..self.request_line_pos[1] - 1)
        };

        method.copy_from_slice(data.as_bytes());
    }

    /// 解析请求路径用于路由函数
    /// # Examples
    /// ```
    /// "/public/index.html" -> "/public/index.html"
    /// "/public" -> "/public"
    /// "/public?a=1&b=2" -> "/public"
    /// ```
    pub fn path_for_routes(&self) -> &str {
        let uri = unsafe {
            self.buf
                .get_unchecked(self.request_line_pos[1]..self.request_line_pos[2] - 1)
        };
        let uri = unsafe { from_utf8_unchecked(uri) };
        if let Some(pos) = uri.find("?") {
            &uri[..pos]
        } else {
            uri
        }
    }

    /// 请求路径
    pub fn path(&self) -> &str {
        let uri = unsafe {
            self.buf
                .get_unchecked(self.request_line_pos[1]..self.request_line_pos[2] - 1)
        };
        unsafe { from_utf8_unchecked(uri) }
    }

    /// 修改请求路径
    pub fn modify_path(&mut self, data: &str) {
        let path = unsafe {
            &mut self
                .buf
                .get_unchecked_mut(self.request_line_pos[1]..self.request_line_pos[2] - 1)
        };

        path.copy_from_slice(data.as_bytes());
    }

    /// http协议版本
    pub fn version(&self) -> &str {
        let version = unsafe {
            self.buf
                .get_unchecked(self.request_line_pos[2]..self.request_line_pos[3] - 1)
        };
        unsafe { from_utf8_unchecked(version) }
    }

    /// 修改http协议版本
    pub fn modify_version(&mut self, data: &str) {
        let version = unsafe {
            &mut self
                .buf
                .get_unchecked_mut(self.request_line_pos[2]..self.request_line_pos[3] - 1)
        };

        version.copy_from_slice(data.as_bytes());
    }

    /// 查找请求头
    /// Examples
    /// ```
    /// let header_value = req.query_header("Host").unwrap();
    /// ```
    pub fn query_header(&self, query_key: &str) -> Option<&str> {
        let query_key = query_key.as_bytes();
        let headers = &self.request_headers_pos;
        for header in headers {
            let key = unsafe { self.buf.get_unchecked(header[0]..header[1]) };
            if query_key == key {
                unsafe {
                    let value = self.buf.get_unchecked(header[1] + 2..header[2] - 2);
                    let value = from_utf8_unchecked(value);
                    return Some(value);
                }
            }
        }

        None
    }

    /// 修改请求头
    /// # Examples
    /// ```
    /// req.modify_header("Host", "new_value");
    /// ```
    pub fn modify_header(&mut self, query_key: &str, new_value: &str) {
        let query_key = query_key.as_bytes();
        let headers = &self.request_headers_pos;
        for header in headers {
            let key = unsafe { self.buf.get_unchecked(header[0]..header[1]) };
            if query_key == key {
                unsafe {
                    let value = self.buf.get_unchecked_mut(header[1] + 2..header[2] - 2);

                    value.copy_from_slice(new_value.as_bytes());
                }
            }
        }
    }

    /// 获取请求参数
    /// # Examples
    /// ```
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Args {
    ///     a: i32,
    ///     b: i32,
    /// }
    ///
    /// let params: Args = req.get_params().unwrap();
    ///
    /// ```
    pub fn get_params<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        let uri = unsafe {
            self.buf
                .get_unchecked(self.request_line_pos[1]..self.request_line_pos[2] - 1)
        };
        let uri = unsafe { from_utf8_unchecked(uri) };

        if let Some(pos) = uri.find("?") {
            let params = &uri[pos + 1..];
            serde_qs::from_str::<T>(params).ok()
        } else {
            None
        }
    }

    /// 获取请求体
    pub async fn body(&mut self) -> Option<&[u8]> {
        if self.method() == "GET" {
            return None;
        }

        while let Ok(_len @ 1..) = self.stream.try_read_buf(&mut self.buf) {}

        let pos_1 = self.request_headers_pos[self.request_headers_pos.len() - 1][2];

        unsafe { Some(self.buf.get_unchecked(pos_1 + 2..)) }
    }

    pub(crate) fn to_stream(self) -> GlacierStream {
        GlacierStream {
            stream: self.stream,
            addr: self.addr,
            buf: self.buf,
        }
    }

    /// 发生响应
    /// # Examples
    /// ```
    /// let res = ResponseBuilder::new()
    ///     .status(200)
    ///     .header("Keep-Alive", "close")
    ///     .body("Hello, World!")
    ///     .build();
    /// req.respond(res).await.unwrap();
    /// ```
    pub async fn respond(&mut self, mut res: Response) -> Result<()> {
        self.stream.write_all_buf(&mut res.buf).await.unwrap();
        self.stream.flush().await.unwrap();

        Ok(())
    }

    /// 发送放在缓存中的静态资源
    /// # Examples
    /// ```
    /// if let Err(e) = req.respond_buf("public/index.html").await {
    ///     req.respond_404().await?;
    /// }
    /// ```
    pub async fn respond_buf(&mut self, file_path: String) -> Result<()> {
        // 获取缓存中的文件内容
        let buf = FILES_BUF
            .get(&file_path)
            .ok_or_else(|| GlacierError::build_option("无此缓存"))?;

        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n",
            buf.len()
        );

        let bufs = [IoSlice::new(header.as_bytes()), IoSlice::new(&buf)];
        let writer = &mut self.stream;
        writer.write_vectored(&bufs).await?;
        writer.flush().await?;

        Ok(())
    }

    /// 发送默认响应：`Hello, world!`
    pub async fn respond_hello(&mut self) -> Result<()> {
        let res =
            "HTTP/1.1 200 OK\r\nContent-Length: 13\r\nConnection: keep-alive\r\n\r\nHello, world!";
        self.stream.write_all(res.as_bytes()).await?;
        self.stream.flush().await?;

        Ok(())
    }

    /// 发送404响应，先从缓存中查找是否存在 `public/404.html`，
    /// 不存在则返回字符串：`404 Not Found`
    pub async fn respond_404(&mut self) -> Result<()> {
        let file_buf = FILES_BUF.get("public/404.html");

        if let Some(file_buf) = file_buf.as_deref() {
            let header = format!(
                "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                file_buf.len()
            );

            // 使用缓冲写入合并 header 和 body
            let writer = &mut self.stream;
            let bufs = [IoSlice::new(header.as_bytes()), IoSlice::new(file_buf)];
            writer.write_vectored(&bufs).await?;
            writer.flush().await?;
        } else {
            let res =
            "HTTP/1.1 404 Not Found\r\nContent-Length: 13\r\nConnection: close\r\n\r\n404 Not Found";
            self.stream.write_all(res.as_bytes()).await?;
            self.stream.flush().await?;
        }

        Ok(())
    }
}

/* ----------------------------------- RequestLine ----------------------------------- */
#[derive(Debug)]
pub struct RequestLine;

impl RequestLine {
    pub(super) fn parse(buf: &BytesMut, pos: [usize; 2]) -> Result<[usize; 4]> {
        // GET /favicon.ico HTTP/1.1\r\n

        let request_line = unsafe { buf.get_unchecked(pos[0]..pos[1]) };

        let mut first_space = 0;
        let mut second_space = 0;
        for i in 0..request_line.len() {
            if b' ' == request_line[i] {
                if first_space == 0 {
                    first_space = i;
                } else {
                    second_space = i;
                    break;
                }
            }
        }
        if second_space == 0 {
            println!("{:#?}", request_line);
            Err(GlacierError::build_req("解析请求行出错"))?
        }

        Ok([0, first_space + 1, second_space + 1, pos[1] - 1])
    }
}

/* ----------------------------------- RequestHeader ----------------------------------- */
#[derive(Debug)]
pub struct RequestHeader;

impl RequestHeader {
    pub(super) fn parse(buf: &BytesMut, line: [usize; 2]) -> Result<[usize; 3]> {
        let header = unsafe { buf.get_unchecked(line[0]..line[1] - 2) };

        for i in 0..header.len() {
            if b':' == header[i] {
                return Ok([line[0], line[0] + i, line[1]]);
            }
        }

        Err(GlacierError::build_req("请求头格式错误"))
    }
}
