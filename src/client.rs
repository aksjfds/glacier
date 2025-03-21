use std::{future::Future, net::IpAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;

use crate::Result;
use crate::{prelude::HttpResponse, Routes};
//
//
//
//
//
//
//

// pub struct Glacier<T> {
pub struct Glacier<T> {
    pub(crate) listener: TcpListener,
    pub(crate) routes: Routes<T>,
    pub(crate) acceptor: TlsAcceptor,
}

impl<T> Glacier<T>
where
    T: Future<Output = HttpResponse> + Send + 'static,
{
    /// 开始运行代码
    /// # Examples
    /// ```
    /// let glacier = Glacier::bind(3000, routes).await.unwrap();
    /// glacier.run().await.unwrap();
    /// ```
    pub async fn run(self) -> Result<()> {
        let listener = self.listener;
        let acceptor = self.acceptor;
        let routes = self.routes;
        let srv = async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let acceptor = acceptor.clone();
                        let stream = match acceptor.accept(stream).await {
                            Ok(stream) => stream,
                            Err(_) => continue,
                        };

                        tokio::spawn(async move {
                            Glacier::handle_connection(stream, routes, addr.ip())
                                .await
                                .unwrap();
                        });
                    }
                    Err(e) => tracing::info!(e = e.to_string(), "error connection!"),
                };
            }
        };

        let _ = tokio::spawn(srv).await;

        Ok(())
    }

    async fn handle_connection(
        stream: tokio_rustls::server::TlsStream<TcpStream>,
        routes: Routes<T>,
        _addr: IpAddr,
    ) -> Result<()> {
        let mut h2_conn = h2::server::handshake(stream).await.unwrap();

        while let Some(Ok((req, mut responder))) = h2_conn.accept().await {
            let res = routes(req).await;

            match res.data {
                Some(data) => responder
                    .send_response(res.builder.body(()).unwrap(), false)
                    .unwrap()
                    .send_data(data, true)
                    .unwrap(),

                None => {
                    responder
                        .send_response(res.builder.body(()).unwrap(), true)
                        .unwrap();
                }
            }
        }

        Ok(())
    }
}
