use crate::{client::Glacier, Routes};

//
//
//
//
//
//

pub struct GlacierBuilder<T> {
    routes: Option<Routes<T>>,
    addr: (String, u16),
}

impl<T> GlacierBuilder<T> {
    pub fn bind(addr: impl IntoAddr) -> Self {
        GlacierBuilder {
            routes: None,
            addr: addr.into_addr(),
            // acceptor: None,
        }
    }

    pub fn server(mut self, routes: Routes<T>) -> Self {
        self.routes = Some(routes);
        self
    }

    pub async fn build(self) -> Glacier<T> {
        let routes = self.routes.unwrap();

        let addr = self.addr;
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        tracing::info!("start server: https://localhost:{}/", addr.1);

        Glacier { listener, routes }
    }
}

pub trait IntoAddr {
    fn into_addr(self) -> (String, u16);
}

impl IntoAddr for u16 {
    fn into_addr(self) -> (String, u16) {
        (String::from("127.0.0.1"), self)
    }
}

impl IntoAddr for (&str, u16) {
    fn into_addr(self) -> (String, u16) {
        (String::from(self.0), self.1)
    }
}
