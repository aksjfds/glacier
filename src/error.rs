#![allow(unused)]

use std::{error::Error as StdError, fmt::Debug, str::Utf8Error};

use tokio::time::error::Elapsed;

//
//
//
//
//
impl Debug for GlacierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlacierError::Detail(inner_error) => {
                if let None = inner_error.source {
                    f.debug_struct("GlacierError")
                        .field("kind", &inner_error.kind)
                        .field("description", &inner_error.description)
                        .finish()
                } else {
                    write!(f, "{:?}", inner_error.source.as_ref().unwrap())
                }
            }
            _ => Ok(()),
        }
    }
}

pub enum GlacierError {
    Detail(InnerError),
    Option,
    IOErr,
    UTF8Error,
    EofErr,
    TimeOutErr,
}

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;
pub struct InnerError {
    kind: Kind,
    source: Option<BoxError>,
    description: &'static str,
}

#[derive(Debug)]
pub enum Kind {
    InRequest,
    InServer,
    IOErr,
    UTF8Error,
    TimeOutErr,
    EofErr,
    Option,
}

impl GlacierError {
    fn new(kind: Kind, description: &'static str, source: Option<BoxError>) -> Self {
        let e = InnerError {
            kind,
            description,
            source,
        };
        GlacierError::Detail(e)
    }

    pub fn kind(&self) -> &Kind {
        match self {
            GlacierError::Option => &Kind::Option,
            GlacierError::Detail(inner_error) => &inner_error.kind,
            GlacierError::IOErr => &Kind::IOErr,
            GlacierError::UTF8Error => &Kind::UTF8Error,
            GlacierError::EofErr => &Kind::EofErr,
            GlacierError::TimeOutErr => &Kind::TimeOutErr,
        }
    }

    pub fn description(&self) -> String {
        match self {
            GlacierError::Detail(inner_error) => match &inner_error.source {
                Some(source) => source.to_string(),
                None => String::from(""),
            },
            _ => String::from(""),
        }
    }
}

/* --------------------------------- // 错误工厂 -------------------------------- */
impl GlacierError {
    pub(crate) fn build_req(description: &'static str) -> GlacierError {
        GlacierError::new(Kind::InRequest, description, None)
    }

    pub(crate) fn build_eof(description: &'static str) -> GlacierError {
        GlacierError::new(Kind::EofErr, description, None)
    }

    pub(crate) fn build_option(description: &'static str) -> GlacierError {
        GlacierError::new(Kind::Option, description, None)
    }

    pub(crate) fn build_server(description: &'static str) -> GlacierError {
        GlacierError::new(Kind::InServer, description, None)
    }
}

/* --------------------------------- // From -------------------------------- */
impl From<Utf8Error> for GlacierError {
    fn from(value: Utf8Error) -> Self {
        GlacierError::new(Kind::UTF8Error, "", Some(Box::from(value)))
    }
}

impl From<std::io::Error> for GlacierError {
    fn from(value: std::io::Error) -> Self {
        GlacierError::new(Kind::IOErr, "", Some(Box::from(value)))
    }
}

impl From<Elapsed> for GlacierError {
    fn from(value: Elapsed) -> Self {
        GlacierError::new(Kind::TimeOutErr, "", Some(Box::from(value)))
    }
}
