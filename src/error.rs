#![allow(unused)]

use std::{
    error::Error as StdError,
    fmt::{self, Debug},
};

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
impl GlacierError {
    pub(crate) fn new(kind: Kind, description: impl Into<String>) -> GlacierError {
        GlacierError::NotOkErr(ErrInfo {
            kind,
            description: description.into(),
        })
    }
}

/* --------------------------------- // From -------------------------------- */
impl From<std::str::Utf8Error> for GlacierError {
    fn from(value: std::str::Utf8Error) -> Self {
        GlacierError::BoxErr(Box::new(value))
    }
}

impl From<std::io::Error> for GlacierError {
    fn from(value: std::io::Error) -> Self {
        GlacierError::BoxErr(Box::new(value))
    }
}

impl From<tokio::time::error::Elapsed> for GlacierError {
    fn from(value: tokio::time::error::Elapsed) -> Self {
        GlacierError::BoxErr(Box::new(value))
    }
}
