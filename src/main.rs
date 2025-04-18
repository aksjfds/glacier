use glacier::handler::HandleReq;
use glacier::prelude::{HyperRequest, HyperResponse};
use glacier::request::Request;
use glacier::response::Response;

pub struct Hello;
impl HandleReq<String> for Hello {
    async fn async_handle(self, _req: Request) -> Result<Response, String> {
        let res = Response::Ok().header("key", "value").json("Hello, World!");

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
    use std::net::SocketAddr;

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    glacier::client::Glacier::bind(addr)
        .serve(router)
        .await
        .unwrap();

    Ok(())
}
