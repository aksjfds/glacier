#![allow(unused)]

use glacier::prelude::*;
use std::str::from_utf8;

#[glacier(GET, "/", [ip_middle(2000, 3)])]
async fn basic(mut req: OneRequest) {
    let res = ResponseBuilder::new(128)
        .status(200)
        .header("Connection", "close")
        .content_type(ContentType::Plain)
        .body("你好，世界".as_bytes())
        .build();
    req.respond(res).await.unwrap();
}

#[glacier(POST, "/hello")]
async fn hello(mut req: OneRequest) {
    let body = req.body().await.unwrap();
    println!("{:#?}", from_utf8(body));

    req.respond_hello().await;
}

#[main]
async fn main() -> Result<()> {
    println!("{:#?}", "http://localhost:3000");
    let glacier = GlacierBuilder::from_config("config.toml")
        .server(routes)
        .build()
        .await;

    glacier.run().await.unwrap();

    Ok(())
}

#[test]
fn test() -> Result<()> {
    use std::io::Write;
    use std::net::TcpStream;

    // 连接到 localhost:3000
    let mut stream = TcpStream::connect("127.0.0.1:3000")?;

    // 构造 POST 请求
    let json_data = r#"{"name": "Rust", "message": "Hello from Rust!"}"#;
    let request = format!(
        "POST /hello HTTP/1.1\r\n\
         Host: localhost\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        json_data.len(),
        json_data
    );

    // 发送请求
    stream.write_all(request.as_bytes())?;
    stream.flush()?;

    Ok(())
}
