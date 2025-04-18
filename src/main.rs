use glacier::handler::HandleReq;
use glacier::prelude::{HyperRequest, HyperResponse};
use glacier::request::Request;
use glacier::response::Response;

pub struct Hello;
impl HandleReq<String> for Hello {
    async fn async_handle(self, _req: Request) -> Result<Response, String> {
        let res = Response::Ok().header("key", "value").body("Hello, World!");

        Ok(res)
    }
}

async fn router(req: HyperRequest) -> Result<HyperResponse, String> {
    let req = Request::new(req);

    let res = match req.uri().path() {
        _ => req.async_map(Hello).await,
    };

    res.map(|res| res.header("global_key", "global_value"))?
        .try_into()
        .map_err(|_e| String::new())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use glacier::io::TokioIo;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    // 设置服务器监听地址
    let addr = SocketAddr::from(([0, 0, 0, 0], 443));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(router))
                .await
            {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }

    #[allow(unreachable_code)]
    Ok(())
}
