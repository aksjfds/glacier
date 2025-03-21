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
    acceptor: Option<tokio_rustls::TlsAcceptor>,
}

impl<T> GlacierBuilder<T> {
    pub fn bind(addr: impl IntoAddr) -> Self {
        GlacierBuilder {
            routes: None,
            addr: addr.into_addr(),
            acceptor: None,
        }
    }

    pub fn server(mut self, routes: Routes<T>) -> Self {
        self.routes = Some(routes);
        self
    }

    pub fn tls(mut self, cert_path: &str, key_path: &str) -> Self {
        use std::{fs::File, io::BufReader, sync::Arc};

        use rustls::pki_types::{pem::PemObject, PrivateKeyDer};
        use rustls_pemfile::certs;
        use tokio_rustls::TlsAcceptor;

        let mut cert_file = BufReader::new(File::open(cert_path).unwrap());
        let key_file = BufReader::new(File::open(key_path).unwrap());

        let cert_chain: Vec<_> = certs(&mut cert_file)
            .collect::<core::result::Result<_, _>>()
            .unwrap();
        let key = PrivateKeyDer::from_pem_reader(key_file).unwrap();

        let mut config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, key)
            .unwrap();

        config.alpn_protocols = vec![b"h2".to_vec()];
        let acceptor = TlsAcceptor::from(Arc::new(config));
        self.acceptor = Some(acceptor);

        self
    }

    pub async fn build(self) -> Glacier<T> {
        let acceptor = self.acceptor.unwrap();
        let routes = self.routes.unwrap();

        let addr = self.addr;
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        tracing::info!("start server: https://localhost:{}/", addr.1);

        Glacier {
            listener,
            routes,
            acceptor,
        }
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
