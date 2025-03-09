use bytes::Bytes;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{io::Read, net::SocketAddrV4, str::FromStr};
use tokio::net::TcpListener;

use crate::{
    error::{GlacierError, Kind},
    prelude::{Glacier, Result, Routes, DIR_PATH, FILES_BUF},
};
//
//
//
//
//
//

pub struct GlacierBuilder<T> {
    routes: Option<Routes<T>>,
    addr: Option<(String, u16)>,
    reuse_port: bool,
    #[cfg(feature = "tls")]
    acceptor: Option<tokio_rustls::TlsAcceptor>,
}

impl<T> GlacierBuilder<T> {
    pub fn new() -> Self {
        GlacierBuilder {
            routes: None,
            addr: None,
            acceptor: None,
            reuse_port: false,
        }
    }

    /// 绑定端口
    /// # Args
    /// - `addr` - 服务器地址: `u16`或者 (`&str`, `u16`)
    ///
    /// # Examples
    /// ```
    /// let glacier = GlacierBuilder::new().bind(3000);
    /// ```
    pub fn bind(mut self, addr: impl IntoAddr, reuse_port: bool) -> Self {
        self.addr = Some(addr.into_addr());
        self.reuse_port = reuse_port;
        self
    }

    /// 绑定路由函数
    /// # Args
    /// - `routes` - 路由函数，由宏生成的异步函数
    ///
    /// # Examples
    /// ```
    /// async fn routes(mut req: OneRequest) -> Result<OneRequest> {
    ///     let path = req.path();
    ///     match path {
    ///         "/" => {
    ///             req.respond_hello().await;
    ///         }
    ///         ...
    ///     }
    ///     Ok(req)
    /// }
    /// ```
    pub fn server(mut self, routes: Routes<T>) -> Self {
        self.routes = Some(routes);
        self
    }

    /// 将指定目录的文件加载到缓存, 尽量不要加载大文件.
    ///
    /// # Args
    /// - `dir_path` - 文件夹路径
    ///
    /// # Examples
    /// ```
    /// let glacier = GlacierBuilder::new()
    ///     .bind(3000)
    ///     .register_dir("/public")
    ///     .serve(routes)
    ///     .build().await;
    /// glacier.run().await.unwrap();
    /// ```
    pub fn register_dir(self, dir_path: &'static str) -> Self {
        tracing::info!(dir_path, "loading static resources");

        unsafe { DIR_PATH = dir_path };

        let entries = std::fs::read_dir(&dir_path[1..]).unwrap();
        for entry in entries {
            let entry = entry.unwrap();

            let file_path = entry.path().to_string_lossy().to_string();
            let mut f = std::fs::File::open(entry.path()).unwrap();

            let mut buf = Vec::with_capacity(1024);
            f.read_to_end(&mut buf).unwrap();

            FILES_BUF.insert(file_path, Bytes::from(buf));
        }

        self
    }

    /// 开启日志记录
    /// # Args
    /// - `max-level` - 日志最高级别，设置error,则不会记录info级别的日志
    /// - `file_path` - 传入None表示输出到终端
    /// # Examples
    /// ```
    ///
    /// let glacier = GlacierBuilder::new()
    ///     .bind(3000)
    ///     .register_dir("/public")
    ///     .start_log("info", Some("my_log.log"))
    ///     .serve(routes)
    ///     .build().await;
    /// glacier.run().await.unwrap();
    ///
    /// ```
    pub fn start_log(self, max_level: &str, file_path: Option<&str>) -> Self {
        let level = tracing::Level::from_str(max_level).unwrap();

        let file = file_path.map(|file_path| {
            let options: _ = std::fs::OpenOptions::new().append(true).open(&file_path);
            match options {
                Ok(f) => f,
                Err(_) => std::fs::File::create(&file_path).unwrap(),
            }
        });

        match file {
            Some(file) => tracing_subscriber::fmt()
                .with_max_level(level)
                .with_writer(file)
                .with_ansi(false)
                .with_target(false)
                .init(),
            None => tracing_subscriber::fmt()
                .with_max_level(level)
                .with_target(false)
                .init(),
        }

        self
    }

    #[cfg(feature = "tls")]
    pub fn open_tls(mut self) -> crate::Result<Self> {
        use std::{fs::File, io::BufReader, sync::Arc};

        use rustls::pki_types::{pem::PemObject, PrivateKeyDer};
        use rustls_pemfile::certs;
        use tokio_rustls::TlsAcceptor;

        let mut cert_file =
            BufReader::new(File::open("/home/aksjfds/codes/http3_server/cert.pem")?);
        let key_file = BufReader::new(File::open("/home/aksjfds/codes/http3_server/key.pem")?);

        let cert_chain: Vec<_> =
            certs(&mut cert_file).collect::<core::result::Result<Vec<_>, _>>()?;
        let key = PrivateKeyDer::from_pem_reader(key_file)?;

        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, key)?;

        let acceptor = TlsAcceptor::from(Arc::new(config));
        self.acceptor = Some(acceptor);

        Ok(self)
    }

    pub async fn build(self) -> Result<Glacier<T>> {
        let routes = self.routes.unwrap();
        let (ip, port) = self.addr.unwrap();
        let addr = SocketAddrV4::new(ip.parse().unwrap(), port);

        let listener = match self.reuse_port {
            true => {
                let addr = SockAddr::from(addr);
                let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
                socket.set_reuse_port(true)?;
                socket.set_nonblocking(true)?;
                socket.bind(&addr).map_err(|e| {
                    let description = format!("error when bind: {}", e);
                    GlacierError::not_ok_err(Kind::IOErr, description)
                })?;
                socket.listen(128)?;
                let listener = socket.into();
                TcpListener::from_std(listener)?
            }
            false => tokio::net::TcpListener::bind(addr).await?,
        };

        match self.acceptor.as_ref() {
            Some(_) => tracing::info!("start server: https://{}:{}/", ip, port),
            None => tracing::info!("start server: http://{}:{}/", ip, port),
        };

        let acceptor = self.acceptor.unwrap();
        Ok(Glacier {
            listener,
            routes,
            acceptor,
        })
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
