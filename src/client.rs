use crate::io::TokioIo;
use crate::prelude::{HyperRequest, HyperResponse};
use hyper::{server::conn::http1, service::service_fn};
use tokio::net::TcpListener;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub struct Glacier {
    addr: std::net::SocketAddr,
}

impl Glacier {
    pub fn bind(addr: std::net::SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn serve<T, E>(
        self,
        router: impl Fn(HyperRequest) -> T + Send + 'static + Copy,
    ) -> Result<(), BoxError>
    where
        T: Future<Output = Result<HyperResponse, E>> + Send,
        E: Into<BoxError>,
        E: Send + Sync + 'static,
    {
        let listener = TcpListener::bind(self.addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(router))
                    .await
                {
                    tracing::debug!("Error serving connection: {}", err);
                }
            });
        }
    }
}
