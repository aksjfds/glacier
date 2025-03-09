use std::{
    error::Error as StdError,
    fmt::{self, Debug},
    str::Utf8Error,
};

use tokio::time::error::Elapsed;

//
//
//
//
//
impl Debug for GlacierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlacierError::BoxErr(err) => f
                .debug_struct("GlacierError")
                .field("kind", &"BoxErr")
                .field("description", &err.to_string())
                .finish(),
            GlacierError::NotOkErr(err_info) => f
                .debug_struct("GlacierError")
                .field("kind", &err_info.kind)
                .field("description", &err_info.description)
                .finish(),
            GlacierError::OkErr(kind) => {
                f.debug_struct("GlacierError").field("kind", kind).finish()
            }
            GlacierError::Option => f
                .debug_struct("GlacierError")
                .field("kind", &"Option")
                .finish(),
        }
    }
}

impl Debug for ErrInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErrInfo")
            .field("kind", &self.kind)
            .field("description", &self.description)
            .finish()
    }
}

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;
pub enum GlacierError {
    BoxErr(BoxError),
    NotOkErr(ErrInfo),
    OkErr(Kind),
    Option,
}

pub struct ErrInfo {
    kind: Kind,
    description: String,
}

#[derive(Debug)]
pub enum Kind {
    InRequest,
    InServer,
    IOErr,
    UTF8Error,
    TimeOutErr,
    EofErr,
}

/* --------------------------------- // 错误工厂 -------------------------------- */
pub(crate) trait IntoDes {
    fn to_des(self) -> String;
}
impl IntoDes for String {
    fn to_des(self) -> String {
        self
    }
}

impl IntoDes for &str {
    fn to_des(self) -> String {
        String::from(self)
    }
}

impl GlacierError {
    pub(crate) fn not_ok_err(kind: Kind, description: impl IntoDes) -> GlacierError {
        GlacierError::NotOkErr(ErrInfo {
            kind,
            description: description.to_des(),
        })
    }
}

/* --------------------------------- // From -------------------------------- */
impl From<Utf8Error> for GlacierError {
    fn from(value: Utf8Error) -> Self {
        GlacierError::BoxErr(Box::new(value))
    }
}

impl From<std::io::Error> for GlacierError {
    fn from(value: std::io::Error) -> Self {
        GlacierError::BoxErr(Box::new(value))
    }
}

impl From<Elapsed> for GlacierError {
    fn from(value: Elapsed) -> Self {
        GlacierError::BoxErr(Box::new(value))
    }
}

#[cfg(feature = "tls")]
impl From<rustls::pki_types::pem::Error> for GlacierError {
    fn from(value: rustls::pki_types::pem::Error) -> Self {
        GlacierError::BoxErr(Box::new(value))
    }
}
#[cfg(feature = "tls")]
impl From<rustls::Error> for GlacierError {
    fn from(value: rustls::Error) -> Self {
        GlacierError::BoxErr(Box::new(value))
    }
}
