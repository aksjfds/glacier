#![allow(unused)]

use std::{collections::HashMap, io::Write};

use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::Result;

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

    pub async fn from(mut res: Response) -> Response {
        res.response_line = String::from("HTTP/1.1 404 Not Found\r\n");
        res.body = String::from("<html><body><h1>404 - Not Found</h1>\
                 <p>The page you are looking for might have been removed, had its name changed, or is temporarily unavailable.</p></body></html>");

        res
    }

    pub async fn not_found(mut stream: TcpStream) {
        let response = b"HTTP/1.1 404 Not Found\r\n\
                 Content-Type: text/html; charset=UTF-8\r\n\
                 Content-Length: 113\r\n\
                 \r\n\
                 <html><body><h1>404 - Not Found</h1>\
                 <p>The page you are looking for might have been removed, had its name changed, or is temporarily unavailable.</p></body></html>";

        if let Ok(_) = stream.write_all(response).await {}
        if let Ok(_) = stream.flush().await {}
    }

    pub async fn bad_request(mut stream: TcpStream) {
        println!("{:#?}", 1);
        let response = b"HTTP/1.1 400 Bad Request\r\n\r\n";
        stream.write_all(response).await;
        stream.flush().await;
        // if let Ok(_) = stream.write_all(response) {}
        // if let Ok(_) = stream.flush() {}
    }

    pub async fn send_file(&mut self, mut f: File) -> Result<()> {
        // 发送 HTTP 响应头部
        let info =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/plain\r\n\r\n";

        self.stream.write_all(info).await?;

        // 1 MB buffer
        let mut buf = vec![0; 1024 * 1024]; // 使用 Vec 来处理不同长度的数据
        loop {
            match f.read(&mut buf).await {
                Ok(0) => break, // EOF
                Ok(len) => {
                    // 发送块的大小
                    let chunk_size = format!("{:X}\r\n", len); // 转换为十六进制
                    self.stream.write_all(chunk_size.as_bytes()).await?;
                    // 发送文件内容块
                    self.stream.write_all(&buf[..len]).await?;
                    self.stream.write_all(b"\r\n").await?; // 块结束
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    break; // 读取失败时退出循环
                }
            }
        }
        
        // 发送最后的零长度块，表示结束
        self.stream.write_all(b"0\r\n\r\n").await?;
        Ok(())
    }

    pub async fn respond(mut self) {
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

        self.stream.write_all(res.as_bytes()).await;
        self.stream.flush().await;
    }
}
