use std::{
    env,
    error::Error,
    net::SocketAddr,
};

use bytes::Bytes;

use h2::{
    RecvStream,
    server::{
        self,
        SendResponse,
    },
};

use http::{
    Request,
    Response,
    StatusCode,
    Version,
};

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

    println!("Glacier HTTP/2 server");
    println!("Listening on {address}");
    println!("HTTP/1.x is not supported");

    loop {
        let (stream, peer_addr) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(error) =
                handle_connection(stream, peer_addr).await
            {
                eprintln!(
                    "[{peer_addr}] rejected: {error}"
                );
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
) -> Result<(), BoxError> {
    /*
     * 严格 HTTP/2。
     *
     * h2::server::handshake 会等待合法的
     * HTTP/2 connection preface：
     *
     * PRI * HTTP/2.0\r\n
     * \r\n
     * SM\r\n
     * \r\n
     *
     * HTTP/1.0 / HTTP/1.1 客户端不会发送这个 preface，
     * 因而握手失败，连接被关闭。
     */
    let mut connection =
        server::handshake(stream).await?;

    println!(
        "[{peer_addr}] HTTP/2 connection established"
    );

    while let Some(result) =
        connection.accept().await
    {
        match result {
            Ok((request, respond)) => {
                tokio::spawn(async move {
                    if let Err(error) =
                        handle_request(
                            request,
                            respond,
                        )
                        .await
                    {
                        eprintln!(
                            "stream error: {error}"
                        );
                    }
                });
            }

            Err(error) => {
                eprintln!(
                    "[{peer_addr}] HTTP/2 connection error: {error}"
                );

                break;
            }
        }
    }

    println!(
        "[{peer_addr}] connection closed"
    );

    Ok(())
}

async fn handle_request(
    request: Request<RecvStream>,
    mut respond: SendResponse<Bytes>,
) -> Result<(), BoxError> {
    /*
     * 理论上进入这里的一定已经是 HTTP/2。
     * 再做一次显式断言，保证框架内部约束。
     */
    if request.version() != Version::HTTP_2 {
        respond.send_reset(
            h2::Reason::PROTOCOL_ERROR,
        );

        return Ok(());
    }

    let stream_id = respond.stream_id();

    let (parts, mut body) =
        request.into_parts();

    println!(
        "[{:?}] {} {} {:?}",
        stream_id,
        parts.method,
        parts.uri,
        parts.version,
    );

    /*
     * 消费请求 DATA。
     *
     * 收到数据并处理后，把 flow-control
     * capacity 返还给 HTTP/2 connection。
     */
    while let Some(result) =
        body.data().await
    {
        let chunk = result?;

        body.flow_control()
            .release_capacity(chunk.len())?;
    }

    let html = format!(
        r#"<!doctype html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <title>Glacier HTTP/2</title>
</head>
<body>
    <h1>Glacier</h1>
    <p>Protocol: HTTP/2</p>
    <p>Version: {:?}</p>
    <p>Stream: {:?}</p>
    <p>Method: {}</p>
    <p>URI: {}</p>
</body>
</html>
"#,
        parts.version,
        stream_id,
        parts.method,
        parts.uri,
    );

    let response_body =
        Bytes::from(html);

    let response =
        Response::builder()
            .status(StatusCode::OK)
            .version(Version::HTTP_2)
            .header(
                "content-type",
                "text/html; charset=utf-8",
            )
            .header(
                "content-length",
                response_body.len().to_string(),
            )
            .header(
                "server",
                "glacier",
            )
            .body(())?;

    /*
     * HEADERS
     */
    let mut send_stream =
        respond.send_response(
            response,
            false,
        )?;

    /*
     * DATA + END_STREAM
     */
    send_stream.send_data(
        response_body,
        true,
    )?;

    Ok(())
}