use std::future::Future;
use tokio::net::TcpListener;

use crate::prelude::{GlacierStream, Kind, OneRequest, Result, Routes};

//
//
//
//
//
//
//

pub struct Glacier<T> {
    pub(crate) listener: TcpListener,
    pub(crate) routes: Routes<T>,
}

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
                    Ok(stream) => {
                        let stream = (stream.0, stream.1.ip());
                        let stream = GlacierStream::new(stream);

                        tokio::spawn(Glacier::handle_connection(stream, routes));
                    }
                    Err(e) => tracing::info!(e = e.to_string(), "error connection!"),
                };
            }
        };

        let _ = tokio::spawn(srv).await;

        Ok(())
    }

    #[tracing::instrument(skip(routes), "")]
    async fn handle_connection(mut stream: GlacierStream, routes: Routes<T>) -> Result<()> {
        // let span = tracing::info_span!("", ip = stream.addr.to_string());
        // let _guard = span.enter();
        tracing::info!("new connection!");

        loop {
            let mut one_req = match stream.to_req().await {
                Ok(one_req) => one_req,
                Err(e) => {
                    match e.kind() {
                        Kind::EofErr => {}
                        Kind::Option => {}
                        Kind::TimeOutErr => tracing::debug!("connection timeout"),
                        _ => tracing::debug!(description = e.description()),
                    }
                    return Ok(());
                }
            };

            one_req = match routes(one_req).await {
                Ok(one_req) => one_req,
                Err(_) => return Ok(()),
            };
            stream = one_req.to_stream();
        }
    }
}
