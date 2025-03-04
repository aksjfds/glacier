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
                let stream = listener.accept().await.unwrap();
                let stream = (stream.0, stream.1.ip());
                let stream = GlacierStream::new(stream);

                tokio::spawn(Glacier::handle_connection(stream, routes));
            }
        };

        tokio::spawn(srv).await.unwrap();

        Ok(())
    }

    async fn handle_connection(mut glacier_stream: GlacierStream, routes: Routes<T>) -> Result<()> {
        loop {
            let mut one_req = match glacier_stream.to_req().await {
                Ok(one_req) => one_req,
                Err(e) => {
                    if !matches!(e.kind(), Kind::EofErr) {
                        println!("{:#?}", e);
                    }
                    return Ok(());
                }
            };

            one_req = match routes(one_req).await {
                Ok(one_req) => one_req,
                Err(e) => {
                    println!("{:#?}", e);
                    return Ok(());
                }
            };
            glacier_stream = one_req.to_stream();
        }
    }
}
