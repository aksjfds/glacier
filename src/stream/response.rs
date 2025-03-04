use bytes::{BufMut, BytesMut};

#[derive(Debug)]
pub struct Response {
    pub buf: BytesMut,
}

pub enum ContentType {
    Plain,
    Html,
    Json,
}

pub struct ResponseBuilder {
    buf: BytesMut,
}

impl ResponseBuilder {
    /// 创建一个响应构造器
    /// # Args
    /// - `capacity` - 响应的预期大小
    /// # Exanples
    /// ```
    ///
    /// let res = ResponseBuilder::new(128)
    ///     .status(200)
    ///     .header("Connection", "close")
    ///     .content_type(ContentType::Plain)
    ///     .body("Hello, World!")
    ///     .build();
    /// req.respond(res).await.unwrap();
    ///
    /// ```
    pub fn new(capacity: usize) -> Self {
        ResponseBuilder {
            buf: BytesMut::with_capacity(capacity),
        }
    }

    /// 设置响应代码
    /// # Examples
    /// ```
    ///
    /// let res = ResponseBuilder::new(128)
    ///     .status(200)
    ///     .header("Connection", "close")
    ///     .content_type(ContentType::Plain)
    ///     .body("Hello, World!")
    ///     .build();
    /// req.respond(res).await.unwrap();
    ///
    /// ```
    pub fn status(mut self, status: u16) -> Self {
        match status {
            200 => self.buf.put(&b"HTTP/1.1 200 OK\r\n"[..]),
            404 => self.buf.put(&b"HTTP/1.1 404 Not Found\r\n"[..]),
            _ => {}
        }
        self
    }

    /// 设置响应头
    /// # Examples
    /// ```
    ///
    /// let res = ResponseBuilder::new(128)
    ///     .status(200)
    ///     .header("Connection", "close")
    ///     .content_type(ContentType::Plain)
    ///     .body("Hello, World!")
    ///     .build();
    /// req.respond(res).await.unwrap();
    ///
    /// ```
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.buf.put(key.as_bytes());
        self.buf.put(&b": "[..]);
        self.buf.put(value.as_bytes());
        self.buf.put(&b"\r\n"[..]);

        self
    }

    /// 设置响应格式，默认utf8
    /// # Examples
    /// ```
    ///
    /// let res = ResponseBuilder::new(128)
    ///     .status(200)
    ///     .header("Connection", "close")
    ///     .content_type(ContentType::Plain)
    ///     .body("Hello, World!")
    ///     .build();
    /// req.respond(res).await.unwrap();
    ///
    /// ```
    pub fn content_type(self, t: ContentType) -> Self {
        match t {
            ContentType::Plain => self.header("Content-Type", "text/plain; charset=UTF-8"),
            ContentType::Html => self.header("Content-Type", "text/html; charset=UTF-8"),
            ContentType::Json => self.header("Content-Type", "application/json; charset=UTF-8"),
        }
    }

    /// 设置响应体
    /// # Examples
    /// ```
    ///
    /// let res = ResponseBuilder::new(128)
    ///     .status(200)
    ///     .header("Connection", "close")
    ///     .content_type(ContentType::Plain)
    ///     .body("Hello, World!")
    ///     .build();
    /// req.respond(res).await.unwrap();
    ///
    /// ```
    pub fn body(mut self, body: &[u8]) -> Self {
        self.buf.put_slice(b"Content-Length: ");
        self.buf.put(body.len().to_string().as_bytes());
        self.buf.put_slice(b"\r\n\r\n");
        self.buf.put_slice(body);

        self
    }

    /// 构造响应
    /// # Examples
    /// ```
    ///
    /// let res = ResponseBuilder::new(128)
    ///     .status(200)
    ///     .header("Connection", "close")
    ///     .content_type(ContentType::Plain)
    ///     .body("Hello, World!")
    ///     .build();
    /// req.respond(res).await.unwrap();
    ///
    /// ```
    pub fn build(self) -> Response {
        Response { buf: self.buf }
    }
}
