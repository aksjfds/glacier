use std::{
    convert::Infallible,
    env,
    error::Error,
};

use bytes::Bytes;

use http::{
    Request,
    Response,
    StatusCode,
};

use http_body_util::Full;

use hyper::{
    body::Incoming,
    server::conn::http1,
    service::service_fn,
};

use hyper_util::rt::TokioIo;

use tokio::net::{
    TcpListener,
    TcpStream,
};

type BoxError = Box<dyn Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "10000".to_string());

    let address = format!("0.0.0.0:{port}");

    let listener = TcpListener::bind(&address).await?;

    println!("Glacier listening on {address}");

    loop {
        let (stream, peer_addr) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream).await {
                eprintln!(
                    "[{peer_addr}] connection error: {error}"
                );
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
) -> Result<(), BoxError> {
    let io = TokioIo::new(stream);

    /*
     * Render Edge -> Glacier 使用 HTTP/1.1。
     *
     * keep_alive 保持开启，因为 Render 会复用
     * 到实例的 HTTP/1.1 connection。
     */
    let mut builder = http1::Builder::new();

    builder.keep_alive(true);

    builder
        .serve_connection(
            io,
            service_fn(handle_request),
        )
        .await?;

    Ok(())
}

async fn handle_request(
    request: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    println!(
        "{} {} {:?}",
        request.method(),
        request.uri(),
        request.version(),
    );

    let response = match request.uri().path() {
        "/" => {
            html_response(
                StatusCode::OK,
                r#"<!doctype html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <title>Glacier</title>
</head>

<body>
    <h1>Glacier</h1>
    <p>Running on Render.</p>
    <p>Browser -> Render: HTTP/2</p>
    <p>Render -> Glacier: HTTP/1.1</p>
</body>
</html>"#,
            )
        }

        "/health" => {
            text_response(
                StatusCode::OK,
                "OK",
            )
        }

        _ => {
            text_response(
                StatusCode::NOT_FOUND,
                "404 Not Found",
            )
        }
    };

    Ok(response)
}

fn html_response(
    status: StatusCode,
    body: &'static str,
) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header(
            "content-type",
            "text/html; charset=utf-8",
        )
        .header(
            "server",
            "glacier",
        )
        .body(
            Full::new(
                Bytes::from_static(body.as_bytes()),
            ),
        )
        .unwrap()
}

fn text_response(
    status: StatusCode,
    body: &'static str,
) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header(
            "content-type",
            "text/plain; charset=utf-8",
        )
        .header(
            "server",
            "glacier",
        )
        .body(
            Full::new(
                Bytes::from_static(body.as_bytes()),
            ),
        )
        .unwrap()
}