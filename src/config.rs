use bytes::Bytes;
use serde::Deserialize;
use std::{io::Read, str::FromStr};

use crate::prelude::{Glacier, Routes, DIR_PATH, FILES_BUF};
//
//
//
//
//
//
fn default_host() -> String {
    String::from("0.0.0.0")
}

fn default_port() -> u16 {
    3000
}

#[derive(Debug, Deserialize)]
struct Server {
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_port")]
    port: u16,
}

fn default_assets() -> String {
    String::new()
}

#[derive(Debug, Deserialize)]
struct Resources {
    #[serde(default = "default_assets")]
    assets: String,
}

fn default_logging_level() -> String {
    String::from("info")
}

#[derive(Debug, Deserialize)]
struct Logging {
    #[serde(default = "default_logging_level")]
    level: String,
    file_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GlacierConfig {
    server: Server,
    resources: Resources,
    logging: Option<Logging>,
}

impl GlacierConfig {
    fn parse_config(config_path: &str) -> GlacierConfig {
        let content = std::fs::read_to_string(config_path).unwrap();
        let config: GlacierConfig = toml::from_str(&content).unwrap();

        config
    }
}

pub struct GlacierBuilder<T> {
    routes: Option<Routes<T>>,
    addr: Option<(String, u16)>,
}

impl<T> GlacierBuilder<T> {
    pub fn new() -> Self {
        GlacierBuilder {
            routes: None,
            addr: None,
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
    pub fn bind(mut self, addr: impl IntoAddr) -> Self {
        self.addr = Some(addr.into_addr());
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

    /// 从配置文件中加载服务器配置
    /// # Examples
    /// ```
    /// let glacier = GlacierBuilder::from_config("config.toml")
    ///     .server(routes)
    ///     .build()
    ///     .await;
    /// glacier.run().await.unwrap();
    /// ```
    pub fn from_config(config_path: &str) -> Self {
        let config = GlacierConfig::parse_config(config_path);

        // Logging
        let logging = config.logging;
        if let Some(logging) = logging {
            let level = tracing::Level::from_str(&logging.level).unwrap();
            let file_path = logging.file_path;
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
        }

        // Server
        let server = config.server;
        let addr = (server.host, server.port);

        // Resources
        let resources = config.resources;
        let assets_path = resources.assets;
        GlacierBuilder::<T>::new().register_dir(Box::leak(assets_path.into_boxed_str()));

        GlacierBuilder {
            routes: None,
            addr: Some(addr),
        }
    }

    pub async fn build(self) -> Glacier<T> {
        let addr = self.addr.unwrap();
        let routes = self.routes.unwrap();

        let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();

        tracing::info!("start server: http://{}:{}/", addr.0, addr.1);

        Glacier { listener, routes }
    }
}

pub trait IntoAddr {
    fn into_addr(self) -> (String, u16);
}

impl IntoAddr for u16 {
    fn into_addr(self) -> (String, u16) {
        (String::from("0.0.0.0"), self)
    }
}

impl IntoAddr for (&str, u16) {
    fn into_addr(self) -> (String, u16) {
        (String::from(self.0), self.1)
    }
}
