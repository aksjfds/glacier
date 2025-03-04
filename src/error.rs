use std::{error::Error as StdError, fmt::Debug, str::Utf8Error};

use tokio::time::error::Elapsed;

//
//
//
//
//
impl Debug for GlacierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let None = self.source {
            f.debug_struct("GlacierError")
                .field("kind", &self.kind)
                .field("description", &self.description)
                .finish()
        } else {
            write!(f, "{:?}", self.source.as_ref().unwrap())
        }
    }
}

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;
pub struct GlacierError {
    kind: Kind,
    source: Option<BoxError>,
    description: &'static str,
}

#[derive(Debug)]
pub enum Kind {
    BuildReq,
    IOErr,
    UTF8Error,
    TimeOutErr,
    EofErr,
    Option,
}

impl GlacierError {
    fn new(kind: Kind, description: &'static str, source: Option<BoxError>) -> Self {
        GlacierError {
            kind,
            description,
            source,
        }
    }

    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    pub fn description(&self) -> &str {
        self.description
    }
}

/* --------------------------------- // 错误工厂 -------------------------------- */
impl GlacierError {
    pub(crate) fn build_req(description: &'static str) -> GlacierError {
        GlacierError::new(Kind::BuildReq, description, None)
    }

    pub(crate) fn build_eof(description: &'static str) -> GlacierError {
        GlacierError::new(Kind::EofErr, description, None)
    }

    pub(crate) fn build_option(description: &'static str) -> GlacierError {
        GlacierError::new(Kind::Option, description, None)
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
